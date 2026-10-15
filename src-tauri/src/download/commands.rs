use tauri::State;

use crate::db::queries::DownloadJobSummary;
use crate::db::DbRequest;
use crate::download::models::{AlbumTrackDownloadInput, ProviderCandidate};
use crate::AppState;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn download_track(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    album: Option<String>,
    track_number: Option<u32>,
    duration_ms: Option<u64>,
    native_id: Option<String>,
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
            native_id,
            preferred_provider,
        )
        .await
}

#[tauri::command]
pub async fn search_download_candidates(
    state: State<'_, AppState>,
    provider_id: String,
    query: String,
) -> Result<Vec<crate::download::models::DownloadCandidateItem>, String> {
    state
        .provider_manager
        .search_download_candidates(&provider_id, &query)
        .await
        .map_err(|e| e.to_string())
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
pub async fn retry_download(
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
pub async fn retry_download_item(
    state: State<'_, AppState>,
    item_id: String,
    preferred_provider: Option<String>,
) -> Result<(), String> {
    state
        .download_manager
        .retry_download_item(&item_id, preferred_provider)
        .await
}

#[tauri::command]
pub async fn dismiss_download_item(
    state: State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    state.download_manager.dismiss_download_item(&item_id).await
}

#[tauri::command]
pub async fn retry_all_failed_downloads(
    state: State<'_, AppState>,
    preferred_provider: Option<String>,
) -> Result<Vec<String>, String> {
    state
        .download_manager
        .retry_all_failed(preferred_provider)
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

#[tauri::command]
pub fn get_downloads_dir(app: tauri::AppHandle) -> Result<String, String> {
    crate::download::storage::get_default_downloads_dir(&app)
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_downloads_folder(app: tauri::AppHandle) -> Result<(), String> {
    let dir = crate::download::storage::get_default_downloads_dir(&app)?;
    tauri_plugin_opener::reveal_item_in_dir(&dir)
        .or_else(|_| tauri_plugin_opener::open_path(&dir, None::<&str>))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_completed_downloads(state: State<'_, AppState>) -> Result<(), String> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state
        .db_tx
        .send(DbRequest::ClearCompletedDownloads { resp: resp_tx })
        .map_err(|e| e.to_string())?;
    resp_rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_downloaded_tracks(state: State<'_, AppState>) -> Result<Vec<crate::LocalTrack>, String> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state
        .db_tx
        .send(DbRequest::GetDownloadedTracks { resp: resp_tx })
        .map_err(|e| e.to_string())?;
    resp_rx.await.map_err(|e| e.to_string())?
}
