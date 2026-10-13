use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::db::queries::{DownloadItemRecord, DownloadRecord};
use crate::db::{DbRequest, TrackData};
use crate::download::models::{
    AlbumTrackDownloadInput, DownloadQualityPreference, DownloadQueryInput, DownloadSourceResult,
};
use crate::download::resolver::DownloadResolver;
use crate::download::storage::{
    build_temp_file_path, build_track_destination_path, get_default_downloads_dir,
};
use crate::providers::ProviderManager;

const PROGRESS_THROTTLE_MS: u128 = 250; // Max 4Hz progress telemetry

#[derive(serde::Serialize, Clone, Debug)]
pub struct DownloadProgressPayload {
    pub item_id: String,
    pub download_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct DownloadStatusPayload {
    pub item_id: String,
    pub download_id: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct DownloadManager {
    app_handle: AppHandle,
    reqwest_client: reqwest::Client,
    db_tx: Sender<DbRequest>,
    provider_manager: Arc<ProviderManager>,
    resolver: Arc<DownloadResolver>,
    concurrency_semaphore: Arc<Semaphore>,
    cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    is_worker_active: Arc<AtomicBool>,
}

impl DownloadManager {
    pub fn new(
        app_handle: AppHandle,
        reqwest_client: reqwest::Client,
        db_tx: Sender<DbRequest>,
        provider_manager: Arc<ProviderManager>,
        resolver: Arc<DownloadResolver>,
        max_concurrent_downloads: usize,
    ) -> Self {
        Self {
            app_handle,
            reqwest_client,
            db_tx,
            provider_manager,
            resolver,
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
            cancellation_tokens: Arc::new(Mutex::new(HashMap::new())),
            is_worker_active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Enqueues and begins a single track download
    #[allow(clippy::too_many_arguments)]
    pub async fn enqueue_track_download(
        &self,
        title: String,
        artist: String,
        album: Option<String>,
        track_number: Option<u32>,
        _duration_ms: Option<u64>,
        native_id: Option<String>,
        preferred_provider: Option<String>,
    ) -> Result<String, String> {
        let download_id = uuid::Uuid::new_v4().to_string();
        let item_id = uuid::Uuid::new_v4().to_string();

        let base_dir = get_default_downloads_dir(&self.app_handle)?;
        let dest_path = build_track_destination_path(
            &base_dir,
            &artist,
            album.as_deref(),
            track_number,
            &title,
            "flac", // temporary extension before resolution
        );
        let temp_path = build_temp_file_path(&dest_path);

        let download = DownloadRecord {
            id: download_id.clone(),
            entity_type: "track".to_string(),
            entity_id: native_id.clone(),
            title: title.clone(),
            artist: artist.clone(),
            album: album.clone(),
            total_items: 1,
            completed_items: 0,
            status: "queued".to_string(),
            created_at: None,
            updated_at: None,
        };

        let item = DownloadItemRecord {
            id: item_id.clone(),
            download_id: download_id.clone(),
            track_title: title,
            artist,
            album,
            track_number,
            provider_id: preferred_provider.clone(),
            download_url: None,
            dest_file_path: Some(dest_path.to_string_lossy().to_string()),
            temp_file_path: Some(temp_path.to_string_lossy().to_string()),
            total_bytes: 0,
            downloaded_bytes: 0,
            status: "queued".to_string(),
            error_message: None,
            retry_count: 0,
            created_at: None,
            updated_at: None,
            native_id,
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::EnqueueDownloadJob {
                download,
                items: vec![item],
                resp: resp_tx,
            })
            .map_err(|e| format!("Failed to dispatch DB enqueue: {}", e))?;

        resp_rx
            .await
            .map_err(|e| format!("DB actor dropped response: {}", e))??;

        self.trigger_processing_loop(preferred_provider);

        Ok(download_id)
    }

    /// Enqueues and begins a batch album download
    pub async fn enqueue_album_download(
        &self,
        album_title: String,
        artist: String,
        tracks: Vec<AlbumTrackDownloadInput>,
        preferred_provider: Option<String>,
    ) -> Result<String, String> {
        if tracks.is_empty() {
            return Err("Cannot enqueue empty album download".to_string());
        }

        let download_id = uuid::Uuid::new_v4().to_string();
        let base_dir = get_default_downloads_dir(&self.app_handle)?;

        let mut items = Vec::with_capacity(tracks.len());
        for t in &tracks {
            let item_id = uuid::Uuid::new_v4().to_string();
            let dest_path = build_track_destination_path(
                &base_dir,
                &artist,
                Some(&album_title),
                t.track_number,
                &t.title,
                "flac",
            );
            let temp_path = build_temp_file_path(&dest_path);

            items.push(DownloadItemRecord {
                id: item_id,
                download_id: download_id.clone(),
                track_title: t.title.clone(),
                artist: artist.clone(),
                album: Some(album_title.clone()),
                track_number: t.track_number,
                provider_id: preferred_provider.clone(),
                download_url: None,
                dest_file_path: Some(dest_path.to_string_lossy().to_string()),
                temp_file_path: Some(temp_path.to_string_lossy().to_string()),
                total_bytes: 0,
                downloaded_bytes: 0,
                status: "queued".to_string(),
                error_message: None,
                retry_count: 0,
                created_at: None,
                updated_at: None,
                native_id: t.native_id.clone(),
            });
        }

        let download = DownloadRecord {
            id: download_id.clone(),
            entity_type: "album".to_string(),
            entity_id: None,
            title: album_title,
            artist,
            album: None,
            total_items: items.len() as u32,
            completed_items: 0,
            status: "queued".to_string(),
            created_at: None,
            updated_at: None,
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::EnqueueDownloadJob {
                download,
                items,
                resp: resp_tx,
            })
            .map_err(|e| format!("Failed to dispatch DB enqueue: {}", e))?;

        resp_rx
            .await
            .map_err(|e| format!("DB actor dropped response: {}", e))??;

        self.trigger_processing_loop(preferred_provider);

        Ok(download_id)
    }

    /// Triggers the background queue consumer if not already actively running
    pub fn trigger_processing_loop(&self, preferred_provider: Option<String>) {
        if self.is_worker_active.swap(true, Ordering::SeqCst) {
            return; // Worker already in progress
        }

        let this = self.clone();
        tokio::spawn(async move {
            this.process_queued_items(preferred_provider).await;
            this.is_worker_active.store(false, Ordering::SeqCst);
        });
    }

    async fn process_queued_items(&self, preferred_provider: Option<String>) {
        loop {
            // 1. Fetch active/queued items from DB
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            if self
                .db_tx
                .send(DbRequest::GetActiveDownloadItems { resp: resp_tx })
                .is_err()
            {
                break;
            }

            let items = match resp_rx.await {
                Ok(Ok(items)) => items,
                _ => break,
            };

            let queued: Vec<DownloadItemRecord> = items
                .into_iter()
                .filter(|i| i.status == "queued")
                .collect();

            if queued.is_empty() {
                break; // Nothing left to download
            }

            for item in queued {
                let permit = match self.concurrency_semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                let this = self.clone();
                let pref = preferred_provider.clone();
                tokio::spawn(async move {
                    this.execute_item_download(item, pref).await;
                    drop(permit);
                });
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn execute_item_download(
        &self,
        item: DownloadItemRecord,
        preferred_provider: Option<String>,
    ) {
        let cancel_token = CancellationToken::new();
        {
            let mut guard = self.cancellation_tokens.lock().await;
            guard.insert(item.id.clone(), cancel_token.clone());
        }

        // Notify UI: resolving
        self.emit_item_status(&item.id, &item.download_id, "resolving", None);
        let _ = self.set_item_status(&item.id, "resolving", None).await;

        // 1. Resolve download stream candidate via extensions
        let query = DownloadQueryInput {
            title: item.track_title.clone(),
            artist: item.artist.clone(),
            album: item.album.clone(),
            duration_ms: None,
            isrc: None,
            native_id: item.native_id.clone(),
            quality_preference: DownloadQualityPreference::Lossless,
        };

        let candidates = self.provider_manager.get_download_providers();
        let resolution_result = {
            let pm = self.provider_manager.clone();
            self.resolver
                .resolve_with_fallback(
                    candidates,
                    preferred_provider.as_deref(),
                    &query,
                    |pid, q| {
                        let pm_clone = pm.clone();
                        async move {
                            pm_clone
                                .resolve_download(&pid, &q)
                                .await
                                .map_err(|e| e.to_string())
                        }
                    },
                )
                .await
        };

        let (winning_provider, source) = match resolution_result {
            Ok(res) => res,
            Err(err) => {
                let err_msg = err.to_string();
                tracing::error!("Download resolution failed for '{}': {}", item.track_title, err_msg);
                self.emit_item_status(&item.id, &item.download_id, "failed", Some(&err_msg));
                let _ = self.set_item_status(&item.id, "failed", Some(&err_msg)).await;
                self.cleanup_token(&item.id).await;
                return;
            }
        };

        if cancel_token.is_cancelled() {
            self.cleanup_token(&item.id).await;
            return;
        }

        // 2. Adjust destination filename with resolved codec extension
        let base_dir = match get_default_downloads_dir(&self.app_handle) {
            Ok(b) => b,
            Err(e) => {
                self.emit_item_status(&item.id, &item.download_id, "failed", Some(&e));
                let _ = self.set_item_status(&item.id, "failed", Some(&e)).await;
                self.cleanup_token(&item.id).await;
                return;
            }
        };

        let dest_path = build_track_destination_path(
            &base_dir,
            &item.artist,
            item.album.as_deref(),
            item.track_number,
            &item.track_title,
            source.codec.extension(),
        );
        let temp_path = build_temp_file_path(&dest_path);

        // Ensure parent folder exists
        if let Some(parent) = dest_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        // 3. Resumable streaming fetch
        self.emit_item_status(&item.id, &item.download_id, "downloading", None);
        let _ = self.set_item_status(&item.id, "downloading", None).await;

        if let Err(e) = self
            .stream_to_file(&item, &source, &temp_path, &cancel_token)
            .await
        {
            if cancel_token.is_cancelled() {
                tracing::info!("Download item '{}' paused or cancelled", item.id);
            } else {
                tracing::error!("Stream download failed for '{}': {}", item.track_title, e);
                self.emit_item_status(&item.id, &item.download_id, "failed", Some(&e));
                let _ = self.set_item_status(&item.id, "failed", Some(&e)).await;
            }
            self.cleanup_token(&item.id).await;
            return;
        }

        // 4. Atomic finalize: Rename .part -> target file
        if let Err(e) = tokio::fs::rename(&temp_path, &dest_path).await {
            let err_msg = format!("Failed to finalize audio file: {}", e);
            self.emit_item_status(&item.id, &item.download_id, "failed", Some(&err_msg));
            let _ = self.set_item_status(&item.id, "failed", Some(&err_msg)).await;
            self.cleanup_token(&item.id).await;
            return;
        }

        // 5. Ingest into SQLite library
        let track_data = TrackData {
            title: item.track_title.clone(),
            artist: Some(item.artist.clone()),
            album: item.album.clone(),
            genre: None,
            track_number: item.track_number.map(|n| n as i64),
            file_path: dest_path.to_string_lossy().to_string(),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.db_tx.send(DbRequest::CompleteDownloadItem {
            item_id: item.id.clone(),
            local_path: dest_path.to_string_lossy().to_string(),
            track_data,
            resp: resp_tx,
        });

        match resp_rx.await {
            Ok(Ok(_track_id)) => {
                tracing::info!(
                    "Successfully downloaded and indexed '{}' via provider '{}'",
                    item.track_title,
                    winning_provider
                );
                self.emit_item_status(&item.id, &item.download_id, "completed", None);
            }
            Ok(Err(e)) => {
                tracing::warn!("Download completed but library indexing returned: {}", e);
            }
            Err(_) => (),
        }

        self.cleanup_token(&item.id).await;
    }

    async fn stream_to_file(
        &self,
        item: &DownloadItemRecord,
        source: &DownloadSourceResult,
        temp_path: &Path,
        cancel_token: &CancellationToken,
    ) -> Result<(), String> {
        let resume_offset = if temp_path.exists() {
            tokio::fs::metadata(temp_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let mut req_builder = self.reqwest_client.get(&source.download_url);
        if let Some(ref headers) = source.headers {
            for (k, v) in headers {
                req_builder = req_builder.header(k, v);
            }
        }

        if resume_offset > 0 {
            req_builder = req_builder.header("Range", format!("bytes={}-", resume_offset));
        }

        let mut resp = req_builder
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let (mut file, mut current_bytes, total_bytes) = if status == StatusCode::PARTIAL_CONTENT {
            let total = resp
                .content_length()
                .map(|len| resume_offset + len)
                .or(source.content_length_bytes)
                .unwrap_or(0);
            let f = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(temp_path)
                .await
                .map_err(|e| format!("Failed to open temp file for append: {}", e))?;
            (f, resume_offset, total)
        } else if status.is_success() {
            let total = resp
                .content_length()
                .or(source.content_length_bytes)
                .unwrap_or(0);
            let f = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(temp_path)
                .await
                .map_err(|e| format!("Failed to open temp file: {}", e))?;
            (f, 0, total)
        } else {
            return Err(format!("Unexpected server response: {}", status));
        };

        let mut last_progress_time = Instant::now();
        let mut last_progress_bytes = current_bytes;

        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| format!("Failed reading stream chunk: {}", e))?
        {
            if cancel_token.is_cancelled() {
                let _ = file.flush().await;
                return Err("Download cancelled by user".to_string());
            }

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed writing chunk to disk: {}", e))?;

            current_bytes += chunk.len() as u64;

            // Throttle UI progress telemetry to <= 4Hz (every 250ms)
            let elapsed = last_progress_time.elapsed().as_millis();
            if elapsed >= PROGRESS_THROTTLE_MS {
                let bytes_diff = current_bytes.saturating_sub(last_progress_bytes);
                let speed_bps = (bytes_diff as f64 / (elapsed as f64 / 1000.0)) as u64;

                let _ = self.app_handle.emit(
                    "download://progress",
                    DownloadProgressPayload {
                        item_id: item.id.clone(),
                        download_id: item.download_id.clone(),
                        downloaded_bytes: current_bytes,
                        total_bytes,
                        speed_bytes_per_sec: speed_bps,
                    },
                );

                // Fire-and-forget DB progress update
                let _ = self.db_tx.send(DbRequest::UpdateDownloadItemProgress {
                    item_id: item.id.clone(),
                    downloaded_bytes: current_bytes,
                    total_bytes,
                });

                last_progress_time = Instant::now();
                last_progress_bytes = current_bytes;
            }
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed flushing file buffer: {}", e))?;
        file.sync_all()
            .await
            .map_err(|e| format!("Failed syncing file to disk: {}", e))?;

        Ok(())
    }

    pub async fn pause_download(&self, download_id: &str) -> Result<(), String> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::GetDownloadQueue { resp: resp_tx })
            .map_err(|e| e.to_string())?;

        if let Ok(Ok(queue)) = resp_rx.await {
            if let Some(job) = queue.into_iter().find(|j| j.download.id == download_id) {
                let mut tokens = self.cancellation_tokens.lock().await;
                for item in job.items {
                    if let Some(token) = tokens.remove(&item.id) {
                        token.cancel();
                    }
                }
            }
        }

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::SetDownloadJobStatus {
                download_id: download_id.to_string(),
                status: "paused".to_string(),
                resp: resp_tx,
            })
            .map_err(|e| e.to_string())?;

        resp_rx.await.map_err(|e| e.to_string())?
    }

    pub async fn resume_download(
        &self,
        download_id: &str,
        preferred_provider: Option<String>,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::SetDownloadJobStatus {
                download_id: download_id.to_string(),
                status: "queued".to_string(),
                resp: resp_tx,
            })
            .map_err(|e| e.to_string())?;

        resp_rx.await.map_err(|e| e.to_string())??;

        self.trigger_processing_loop(preferred_provider);
        Ok(())
    }

    pub async fn cancel_download(&self, download_id: &str) -> Result<(), String> {
        let _ = self.pause_download(download_id).await;

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::DeleteDownloadJob {
                download_id: download_id.to_string(),
                resp: resp_tx,
            })
            .map_err(|e| e.to_string())?;

        resp_rx.await.map_err(|e| e.to_string())?
    }

    fn emit_item_status(
        &self,
        item_id: &str,
        download_id: &str,
        status: &str,
        error: Option<&str>,
    ) {
        let _ = self.app_handle.emit(
            "download://item-status",
            DownloadStatusPayload {
                item_id: item_id.to_string(),
                download_id: download_id.to_string(),
                status: status.to_string(),
                error: error.map(|e| e.to_string()),
            },
        );
    }

    async fn set_item_status(
        &self,
        item_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), String> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.db_tx
            .send(DbRequest::SetDownloadItemStatus {
                item_id: item_id.to_string(),
                status: status.to_string(),
                error_message: error.map(|e| e.to_string()),
                resp: resp_tx,
            })
            .map_err(|e| e.to_string())?;

        resp_rx.await.map_err(|e| e.to_string())?
    }

    async fn cleanup_token(&self, item_id: &str) {
        let mut guard = self.cancellation_tokens.lock().await;
        guard.remove(item_id);
    }
}
