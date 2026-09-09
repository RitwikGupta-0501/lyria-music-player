use crate::queue::{QueueTrack, TrackSourceInfo};
use crate::AppState;
use crate::providers::CanonicalSeedV1;
use std::time::Duration;

/// Resolves the next track in the Dual-Mode Infinite Core Loop:
/// Waterfall logic:
/// 1. Query remote federated WASM radio stream (if extensions available)
/// 2. Seamlessly fall back to local Markov random walk over SQLite library
/// 3. Return None if both local library and remote stream are exhausted
pub async fn resolve_autoplay_next_track(
    state: &AppState,
    seed: &QueueTrack,
) -> Result<Option<QueueTrack>, String> {
    let seed_artist_str = seed.artist.as_deref().unwrap_or("").trim().to_string();
    let seed_title_str = seed.title.trim().to_string();

    let (seed_duration_ms, seed_native_id, seed_provider_id) = match &seed.source {
        TrackSourceInfo::Remote { provider_id, remote_track_id, duration_ms, .. } => {
            (*duration_ms, Some(remote_track_id.clone()), Some(provider_id.clone()))
        }
        TrackSourceInfo::Local { .. } => {
            (None, None, None)
        }
    };

    let canonical_key = format!("{}::{}", seed_artist_str.to_lowercase(), seed_title_str.to_lowercase());
    let canonical_seed = CanonicalSeedV1 {
        abi_version: 1,
        canonical_key: canonical_key.clone(),
        title: seed_title_str,
        artist: seed_artist_str.clone(),
        album: None,
        isrc: None,
        duration_ms: seed_duration_ms,
        native_id: seed_native_id,
        provider_id: seed_provider_id,
    };

    // ─────────────────────────────────────────────────────────────────────────────
    // Tier 1: Remote Federated WASM Radio Stream
    // ─────────────────────────────────────────────────────────────────────────────
    let remote_attempt = tokio::time::timeout(
        Duration::from_millis(3500),
        state.recommendation_compiler.compile_federated_radio(&canonical_seed)
    ).await;

    if let Ok(Ok(radio_stream)) = remote_attempt {
        let candidate_tracks: Vec<_> = radio_stream.tracks
            .into_iter()
            .filter(|t| {
                let cand_key = format!("{}::{}", t.artist.trim().to_lowercase(), t.title.trim().to_lowercase());
                cand_key != canonical_key
            })
            .collect();

        if !candidate_tracks.is_empty() {
            let selected = &candidate_tracks[0];
            let fallback_provider = {
                let (tx, rx) = tokio::sync::oneshot::channel();
                if state.db_tx.send(crate::db::DbRequest::GetSetting {
                    key: "default_remote_provider".to_string(),
                    resp: tx,
                }).is_ok() {
                    rx.await.ok().and_then(|r| r.ok()).flatten().filter(|p| p != "local" && !p.trim().is_empty())
                } else {
                    None
                }
            }.unwrap_or_else(|| "youtube-wasm".to_string());
            let provider_id = selected.isrc.as_deref().unwrap_or(&fallback_provider).to_string();
            let next_queue_track = QueueTrack {
                instance_id: uuid::Uuid::new_v4().to_string(),
                title: selected.title.clone(),
                artist: Some(selected.artist.clone()),
                track_number: None,
                source: TrackSourceInfo::Remote {
                    provider_id,
                    remote_track_id: selected.id.clone(),
                    stream_url: selected.stream_url.clone(),
                    quality_hint: selected.quality_hint.clone(),
                    cover_art_url: selected.cover_art_url.clone(),
                    duration_ms: selected.duration_ms,
                },
            };
            return Ok(Some(next_queue_track));
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Tier 2: Local Markov Random Walk over SQLite Library
    // ─────────────────────────────────────────────────────────────────────────────
    let (seed_track_id, seed_file_path, seed_album_id) = match &seed.source {
        TrackSourceInfo::Local { track_id, file_path, album_id } => {
            (Some(*track_id), Some(file_path.clone()), *album_id)
        }
        _ => (None, None, None),
    };

    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetMarkovAutoplayCandidate {
        seed_artist: seed.artist.clone(),
        seed_album_id,
        seed_track_id,
        seed_file_path,
        seed_duration_ms,
        resp: resp_tx,
    }).map_err(|e| format!("Failed to send DB request: {}", e))?;

    let local_res = resp_rx.await.map_err(|e| format!("DB response dropped: {}", e))??;

    if let Some(local_track) = local_res {
        let next_queue_track = QueueTrack {
            instance_id: uuid::Uuid::new_v4().to_string(),
            title: local_track.title,
            artist: local_track.artist,
            track_number: local_track.track_number,
            source: TrackSourceInfo::Local {
                track_id: local_track.id,
                file_path: local_track.file_path,
                album_id: local_track.album_id,
            },
        };
        return Ok(Some(next_queue_track));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Tier 3: Graceful stop when both remote & local are empty
    // ─────────────────────────────────────────────────────────────────────────────
    Ok(None)
}
