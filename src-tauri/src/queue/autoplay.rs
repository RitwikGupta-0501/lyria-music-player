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
        // Collect canonical keys from the active queue to avoid 2-track reciprocal ping-pong loops
        let recent_canonical_keys: std::collections::HashSet<String> = {
            if let Ok(q) = state.queue.lock() {
                q.tracks.iter().map(|t| {
                    format!("{}::{}", t.artist.as_deref().unwrap_or("").trim().to_lowercase(), t.title.trim().to_lowercase())
                }).collect()
            } else {
                std::collections::HashSet::new()
            }
        };

        // First pass: Filter out current seed track and any tracks already in recent queue history
        let fresh_candidates: Vec<_> = radio_stream.tracks
            .iter()
            .filter(|t| {
                let cand_key = format!("{}::{}", t.artist.trim().to_lowercase(), t.title.trim().to_lowercase());
                cand_key != canonical_key && !recent_canonical_keys.contains(&cand_key)
            })
            .collect();

        // Second pass fallback: If all candidates are in recent queue history, relax history check but preserve seed exclusion
        let candidate_tracks: Vec<_> = if !fresh_candidates.is_empty() {
            fresh_candidates
        } else {
            radio_stream.tracks
                .iter()
                .filter(|t| {
                    let cand_key = format!("{}::{}", t.artist.trim().to_lowercase(), t.title.trim().to_lowercase());
                    cand_key != canonical_key
                })
                .collect()
        };

        if !candidate_tracks.is_empty() {
            let selected = candidate_tracks[0];
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

            // Correctly prioritize the provider_id attached to the track; do NOT treat ISRC code as provider name!
            let provider_id = selected.provider_id.as_deref().unwrap_or(&fallback_provider).to_string();
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


#[cfg(test)]
mod tests {
    use crate::providers::TrackResult;

    #[test]
    fn test_candidate_filtering_and_provider_attribution() {
        let canonical_key = "artist a::track 1".to_string();
        let mut recent_keys = std::collections::HashSet::new();
        recent_keys.insert("artist b::track 2".to_string());

        let t1 = TrackResult {
            id: "1".into(),
            title: "Track 1".into(),
            artist: "Artist A".into(),
            album: None,
            cover_art_url: None,
            stream_url: None,
            quality_hint: None,
            duration_ms: Some(180000),
            isrc: Some("USRC12345678".into()),
            plays: None,
            provider_id: Some("youtube-wasm".into()),
        };

        let t2 = TrackResult {
            id: "2".into(),
            title: "Track 2".into(),
            artist: "Artist B".into(),
            album: None,
            cover_art_url: None,
            stream_url: None,
            quality_hint: None,
            duration_ms: Some(200000),
            isrc: Some("USRC87654321".into()),
            plays: None,
            provider_id: Some("youtube-wasm".into()),
        };

        let t3 = TrackResult {
            id: "3".into(),
            title: "Track 3".into(),
            artist: "Artist C".into(),
            album: None,
            cover_art_url: None,
            stream_url: None,
            quality_hint: None,
            duration_ms: Some(210000),
            isrc: Some("USRC99999999".into()),
            plays: None,
            provider_id: Some("soundcloud-wasm".into()),
        };

        let all_tracks = [t1, t2, t3];

        let fresh: Vec<_> = all_tracks
            .iter()
            .filter(|t| {
                let key = format!("{}::{}", t.artist.trim().to_lowercase(), t.title.trim().to_lowercase());
                key != canonical_key && !recent_keys.contains(&key)
            })
            .collect();

        // Should filter out t1 (seed) and t2 (in recent queue history)
        assert_eq!(fresh.len(), 1);
        let selected = fresh[0];
        assert_eq!(selected.title, "Track 3");
        // Ensure provider_id comes from track.provider_id, not isrc
        assert_eq!(selected.provider_id.as_deref(), Some("soundcloud-wasm"));
        assert_ne!(selected.provider_id.as_deref(), selected.isrc.as_deref());
    }
}
