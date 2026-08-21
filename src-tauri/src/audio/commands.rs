use tauri::State;
use crate::AppState;
use super::AudioCommand;

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AppState>,
    source: crate::queue::TrackSourceInfo,
    title: String,
    artist: Option<String>,
    album: Option<String>,
) -> Result<(), String> {
    match &source {
        crate::queue::TrackSourceInfo::Local { .. } => {
            tracing::info!("Played '{}' (Local)", title);
        }
        crate::queue::TrackSourceInfo::Remote { provider_id, .. } => {
            tracing::info!("Played '{}' in {}", title, provider_id);
        }
    }

    let (resolved_source, duration_hint) = match source {
        crate::queue::TrackSourceInfo::Local { file_path, .. } => {
            let pb = std::path::PathBuf::from(&file_path);
            if !pb.exists() {
                return Err("FILE_NOT_FOUND".to_string());
            }
            (crate::audio::TrackSource::Local(pb), None)
        }
        crate::queue::TrackSourceInfo::Remote { provider_id, remote_track_id, duration_ms, .. } => {
            let (final_url, headers, resolved_duration) = {
                let manager = state.provider_manager.lock().await;
                let resolved = manager.resolve(&provider_id, &remote_track_id).await.map_err(|e| {
                    tracing::error!("Failed to resolve track '{}' via '{}': {}", remote_track_id, provider_id, e);
                    e.to_string()
                })?;
                (resolved.stream_url, resolved.headers, resolved.duration_ms)
            };
            
            if final_url.is_empty() {
                tracing::error!("Failed to resolve a valid stream URL (provider returned empty string)");
                return Err("Failed to resolve a valid stream URL (provider returned empty string)".to_string());
            }
            
            crate::providers::check_url_allowed(&final_url).map_err(|e| {
                tracing::error!("URL not allowed: {}", e);
                e.to_string()
            })?;
            let parsed_url = url::Url::parse(&final_url).map_err(|e| {
                tracing::error!("Invalid URL parsed: {}", e);
                format!("Invalid URL: {}", e)
            })?;
            let final_duration = resolved_duration.or(duration_ms);
            (crate::audio::TrackSource::Remote(parsed_url, headers), final_duration)
        }
    };

    // We skip LoadAudioCache since it's just for local files.
    // In a real app we'd do a smarter history log here instead.

    let tx = state.audio_tx.lock().map_err(|e| {
        tracing::error!("Failed to lock audio tx: {}", e);
        e.to_string()
    })?;
    tx.send(AudioCommand::Load { source: resolved_source, title, artist, album, duration_hint }).map_err(|e| {
        tracing::error!("Failed to send AudioCommand::Load: {}", e);
        e.to_string()
    })?;
    
    Ok(())
}

#[tauri::command]
pub async fn queue_next_audio(
    state: State<'_, AppState>,
    source: crate::queue::TrackSourceInfo,
    title: String,
    artist: Option<String>,
    album: Option<String>,
) -> Result<(), String> {
    let (resolved_source, duration_hint) = match source {
        crate::queue::TrackSourceInfo::Local { file_path, .. } => {
            let pb = std::path::PathBuf::from(&file_path);
            if !pb.exists() {
                return Err("FILE_NOT_FOUND".to_string());
            }
            (crate::audio::TrackSource::Local(pb), None)
        }
        crate::queue::TrackSourceInfo::Remote { provider_id, remote_track_id, duration_ms, .. } => {
            let (final_url, headers, resolved_duration) = {
                let manager = state.provider_manager.lock().await;
                let resolved = manager.resolve(&provider_id, &remote_track_id).await.map_err(|e| {
                    tracing::error!("Failed to resolve track '{}' via '{}': {}", remote_track_id, provider_id, e);
                    e.to_string()
                })?;
                (resolved.stream_url, resolved.headers, resolved.duration_ms)
            };
            
            if final_url.is_empty() {
                tracing::error!("Failed to resolve a valid stream URL (provider returned empty string)");
                return Err("Failed to resolve a valid stream URL (provider returned empty string)".to_string());
            }
            
            crate::providers::check_url_allowed(&final_url).map_err(|e| {
                tracing::error!("URL not allowed: {}", e);
                e.to_string()
            })?;
            let parsed_url = url::Url::parse(&final_url).map_err(|e| {
                tracing::error!("Invalid URL parsed: {}", e);
                format!("Invalid URL: {}", e)
            })?;
            let final_duration = resolved_duration.or(duration_ms);
            (crate::audio::TrackSource::Remote(parsed_url, headers), final_duration)
        }
    };

    let tx = state.audio_tx.lock().map_err(|e| {
        tracing::error!("Failed to lock audio tx: {}", e);
        e.to_string()
    })?;
    tx.send(AudioCommand::QueueNext { source: resolved_source, title, artist, album, duration_hint }).map_err(|e| {
        tracing::error!("Failed to send AudioCommand::QueueNext: {}", e);
        e.to_string()
    })?;
    
    Ok(())
}

#[tauri::command]
pub async fn play_audio(state: State<'_, AppState>) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::Play).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn pause_audio(state: State<'_, AppState>) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::Pause).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn stop_audio(state: State<'_, AppState>) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::Stop).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn seek_audio(state: State<'_, AppState>, position: f64) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::Seek(position)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::SetVolume(volume)).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_mute(state: State<'_, AppState>, mute: bool) -> Result<(), String> {
    let tx = state.audio_tx.lock().map_err(|e| e.to_string())?;
    tx.send(AudioCommand::SetMute(mute)).map_err(|e| e.to_string())?;
    Ok(())
}
