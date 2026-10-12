use tauri::State;

use crate::db::queries::DownloadJobSummary;
use crate::db::DbRequest;
use crate::download::models::{AlbumTrackDownloadInput, ProviderCandidate};
use crate::AppState;

#[tauri::command]
pub async fn download_track(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    album: Option<String>,
    track_number: Option<u32>,
    duration_ms: Option<u64>,
    preferred_provider: Option<String>,
) -> Result<String, String> {
    state
        .download_manager
        .enqueue_track_download(
            title,
            artist,
            album,
            track_number,
            duration_ms,
            preferred_provider,
        )
        .await
}

#[tauri::command]
pub async fn download_album(
    state: State<'_, AppState>,
    album_title: String,
    artist: String,
    tracks: Vec<AlbumTrackDownloadInput>,
    preferred_provider: Option<String>,
) -> Result<String, String> {
    state
        .download_manager
        .enqueue_album_download(album_title, artist, tracks, preferred_provider)
        .await
}

#[tauri::command]
pub async fn pause_download(
    state: State<'_, AppState>,
    download_id: String,
) -> Result<(), String> {
    state.download_manager.pause_download(&download_id).await
}

#[tauri::command]
pub async fn resume_download(
    state: State<'_, AppState>,
    download_id: String,
    preferred_provider: Option<String>,
) -> Result<(), String> {
    state
        .download_manager
        .resume_download(&download_id, preferred_provider)
        .await
}

#[tauri::command]
pub async fn cancel_download(
    state: State<'_, AppState>,
    download_id: String,
) -> Result<(), String> {
    state.download_manager.cancel_download(&download_id).await
}

#[tauri::command]
pub async fn get_download_queue(
    state: State<'_, AppState>,
) -> Result<Vec<DownloadJobSummary>, String> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state
        .db_tx
        .send(DbRequest::GetDownloadQueue { resp: resp_tx })
        .map_err(|e| e.to_string())?;
    resp_rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_download_providers(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderCandidate>, String> {
    Ok(state.provider_manager.get_download_providers())
}

#[tauri::command]
pub async fn set_preferred_download_provider(
    state: State<'_, AppState>,
    provider_id: Option<String>,
) -> Result<(), String> {
    let val = provider_id.unwrap_or_default();
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state
        .db_tx
        .send(DbRequest::SetSetting {
            key: "preferred_download_provider".to_string(),
            value: val,
            resp: resp_tx,
        })
        .map_err(|e| e.to_string())?;
    resp_rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_preferred_download_provider(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state
        .db_tx
        .send(DbRequest::GetSetting {
            key: "preferred_download_provider".to_string(),
            resp: resp_tx,
        })
        .map_err(|e| e.to_string())?;
    let res = resp_rx.await.map_err(|e| e.to_string())??;
    match res {
        Some(s) if !s.trim().is_empty() => Ok(Some(s)),
        _ => Ok(None),
    }
}
