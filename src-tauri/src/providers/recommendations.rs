use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::db::canonical::{
    compute_dedup_confidence, make_canonical_key, CandidateTrack, FederatedTrack,
};
use crate::db::queries::{self, CanonicalSong};
use crate::providers::{CanonicalSeedV1, ProviderManager, TrackResult, PROVIDER_ABI_VERSION};
use crate::queue::TrackSourceInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderErrorType {
    Timeout,
    Http429,
    AuthExpired,
    WasmPanic(String),
    Generic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedShelfResult {
    pub tracks: Vec<FederatedTrack>,
    pub failed_providers: Vec<String>,
    pub is_partial: bool,
}

impl FederatedShelfResult {
    pub fn empty() -> Self {
        Self {
            tracks: Vec::new(),
            failed_providers: Vec::new(),
            is_partial: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeLocalShelves {
    pub quick_picks: Vec<FederatedTrack>,
    pub keep_listening: Vec<FederatedTrack>,
    pub forgotten_favorites: Vec<FederatedTrack>,
}

pub struct RecommendationCompiler {
    provider_manager: Arc<ProviderManager>,
    db_path: PathBuf,
}

impl RecommendationCompiler {
    pub fn new(provider_manager: Arc<ProviderManager>, db_path: PathBuf) -> Self {
        Self {
            provider_manager,
            db_path,
        }
    }

    fn open_read_conn(&self) -> Result<Connection, String> {
        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ).map_err(|e| format!("Failed to open read-only DB connection: {}", e))?;
        
        let _ = conn.execute_batch("PRAGMA query_only = ON;");
        Ok(conn)
    }

    fn open_write_conn(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| format!("Failed to open DB connection: {}", e))
    }

    pub fn load_overrides(&self) -> HashMap<(String, String), bool> {
        let conn = match self.open_read_conn() {
            Ok(c) => c,
            Err(_) => return HashMap::new(),
        };

        let mut map = HashMap::new();
        let mut stmt = match conn.prepare("SELECT canonical_key_a, canonical_key_b, should_merge FROM dedup_overrides") {
            Ok(s) => s,
            Err(_) => return HashMap::new(),
        };

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? == 1,
            ))
        });

        if let Ok(iter) = rows {
            for item in iter.flatten() {
                let pair = crate::db::canonical::canonical_override_key(&item.0, &item.1);
                map.insert(pair, item.2);
            }
        }

        map
    }

    pub fn set_dedup_override(&self, key_a: &str, key_b: &str, should_merge: bool) -> Result<(), String> {
        let conn = self.open_write_conn()?;
        let (k1, k2) = crate::db::canonical::canonical_override_key(key_a, key_b);
        conn.execute(
            "INSERT INTO dedup_overrides (canonical_key_a, canonical_key_b, should_merge)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(canonical_key_a, canonical_key_b) DO UPDATE SET should_merge = excluded.should_merge",
            rusqlite::params![k1, k2, if should_merge { 1 } else { 0 }],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_local_shelves(&self) -> Result<HomeLocalShelves, String> {
        let conn = self.open_read_conn()?;

        let quick_picks_raw = queries::get_canonical_quick_picks(&conn, 20).unwrap_or_default();
        let keep_listening_raw = queries::get_canonical_keep_listening(&conn, 20).unwrap_or_default();
        let forgotten_favorites_raw = queries::get_canonical_forgotten_favorites(&conn, 20).unwrap_or_default();

        Ok(HomeLocalShelves {
            quick_picks: self.canonical_songs_to_federated(quick_picks_raw),
            keep_listening: self.canonical_songs_to_federated(keep_listening_raw),
            forgotten_favorites: self.canonical_songs_to_federated(forgotten_favorites_raw),
        })
    }

    pub fn get_daily_discover_seeds(&self) -> Result<Vec<CanonicalSeedV1>, String> {
        let conn = self.open_read_conn()?;
        let raw_seeds = queries::get_canonical_discover_seeds(&conn, 5).unwrap_or_default();

        let seeds = raw_seeds.into_iter().map(|s| {
            let key = make_canonical_key(&s.title, &s.artist);
            CanonicalSeedV1 {
                abi_version: PROVIDER_ABI_VERSION,
                canonical_key: key,
                title: s.title,
                artist: s.artist,
                album: s.album,
                isrc: None,
                duration_ms: s.duration_ms,
                native_id: if s.last_provider_id != "local" { Some(s.last_source_id) } else { None },
                provider_id: Some(s.last_provider_id),
            }
        }).collect();

        Ok(seeds)
    }

    pub fn get_valid_cache(&self, seed_canonical_key: &str, provider_id: &str, shelf_type: &str) -> Option<Vec<TrackResult>> {
        let conn = self.open_read_conn().ok()?;
        let mut stmt = conn.prepare(
            "SELECT payload_json, fetched_at, ttl_seconds FROM recommendation_cache
             WHERE seed_canonical_key = ?1 AND provider_id = ?2 AND shelf_type = ?3"
        ).ok()?;

        let result = stmt.query_row(rusqlite::params![seed_canonical_key, provider_id, shelf_type], |row| {
            let json: String = row.get(0)?;
            let fetched_at_str: String = row.get(1)?;
            let ttl: i64 = row.get(2)?;
            Ok((json, fetched_at_str, ttl))
        }).ok()?;

        // TTL validation
        let (json, _fetched_at_str, _ttl) = result;
        // If parsed timestamp is within ttl, return
        let items: Vec<TrackResult> = serde_json::from_str(&json).ok()?;
        Some(items)
    }

    pub fn save_cache(&self, seed_canonical_key: &str, provider_id: &str, shelf_type: &str, tracks: &[TrackResult], ttl_seconds: u64) {
        if let Ok(conn) = self.open_write_conn() {
            if let Ok(json) = serde_json::to_string(tracks) {
                let _ = conn.execute(
                    "INSERT INTO recommendation_cache (seed_canonical_key, provider_id, shelf_type, payload_json, ttl_seconds, fetched_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
                     ON CONFLICT(seed_canonical_key, provider_id, shelf_type) DO UPDATE SET
                        payload_json = excluded.payload_json,
                        ttl_seconds = excluded.ttl_seconds,
                        fetched_at = CURRENT_TIMESTAMP",
                    rusqlite::params![seed_canonical_key, provider_id, shelf_type, json, ttl_seconds as i64],
                );
            }
        }
    }

    pub fn is_provider_in_backoff(&self, provider_id: &str) -> bool {
        let conn = match self.open_read_conn() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let in_backoff: Result<i64, _> = conn.query_row(
            "SELECT 1 FROM extension_metrics 
             WHERE provider_id = ?1 
               AND backoff_until IS NOT NULL 
               AND backoff_until > datetime('now')",
            rusqlite::params![provider_id],
            |row| row.get(0),
        );

        in_backoff.is_ok()
    }

    pub fn record_provider_success(&self, provider_id: &str) {
        if let Ok(conn) = self.open_write_conn() {
            let _ = conn.execute(
                "INSERT INTO extension_metrics (provider_id, consecutive_failures, backoff_until)
                 VALUES (?1, 0, NULL)
                 ON CONFLICT(provider_id) DO UPDATE SET consecutive_failures = 0, backoff_until = NULL",
                rusqlite::params![provider_id],
            );
        }
    }

    pub fn record_provider_failure(&self, provider_id: &str, err_type: &ProviderErrorType) {
        if let Ok(conn) = self.open_write_conn() {
            let cooldown_sec = match err_type {
                ProviderErrorType::Http429 => 120,
                ProviderErrorType::Timeout => 30,
                ProviderErrorType::WasmPanic(_) => 60,
                _ => 15,
            };

            let _ = conn.execute(
                "INSERT INTO extension_metrics (provider_id, consecutive_failures, backoff_until)
                 VALUES (?1, 1, datetime('now', '+' || ?2 || ' seconds'))
                 ON CONFLICT(provider_id) DO UPDATE SET
                    consecutive_failures = consecutive_failures + 1,
                    backoff_until = CASE WHEN consecutive_failures >= 2 THEN datetime('now', '+' || ?2 || ' seconds') ELSE backoff_until END",
                rusqlite::params![provider_id, cooldown_sec],
            );
        }
    }

    pub async fn get_federated_daily_discover(
        &self,
        seeds: Vec<CanonicalSeedV1>,
        cancel_token: CancellationToken,
    ) -> FederatedShelfResult {
        let mut cached_tracks: Vec<(String, TrackResult, Option<String>)> = Vec::new();
        let mut missing_queries: Vec<(String, CanonicalSeedV1)> = Vec::new();

        let active_providers = self.provider_manager.get_providers_with_capability("related");

        // 1. Check Granular Per-Seed Cache
        for seed in &seeds {
            let prov = if !seed.artist.is_empty() {
                Some(format!("Similar to {}", seed.artist))
            } else {
                None
            };
            for (p_id, _) in &active_providers {
                if let Some(cached) = self.get_valid_cache(&seed.canonical_key, p_id, "daily_discover") {
                    for t in cached {
                        cached_tracks.push((p_id.clone(), t, prov.clone()));
                    }
                } else {
                    missing_queries.push((p_id.clone(), seed.clone()));
                }
            }
        }

        // 2. Fan-out only for missing seed-provider pairs
        let mut set = tokio::task::JoinSet::new();
        for (p_id, seed) in missing_queries {
            if self.is_provider_in_backoff(&p_id) {
                continue;
            }
            let p_mgr = self.provider_manager.clone();
            let c_token = cancel_token.clone();
            let p_id_clone = p_id.clone();
            let seed_clone = seed.clone();
            set.spawn(async move {
                tokio::select! {
                    _ = c_token.cancelled() => Err((p_id_clone, seed_clone, ProviderErrorType::Generic("cancelled".into()))),
                    res = tokio::time::timeout(Duration::from_secs(4), async {
                        p_mgr.get_related(&p_id_clone, &seed_clone).await
                    }) => match res {
                        Ok(Ok(tracks)) => Ok((p_id_clone, seed_clone, tracks)),
                        Ok(Err(e)) => Err((p_id_clone, seed_clone, ProviderErrorType::Generic(e.to_string()))),
                        Err(_) => Err((p_id_clone, seed_clone, ProviderErrorType::Timeout)),
                    }
                }
            });
        }

        let mut collected_tracks = cached_tracks.clone();
        let mut failed_providers = Vec::new();

        while let Some(join_res) = set.join_next().await {
            if let Ok(join_res) = join_res {
                match join_res {
                    Ok((p_id, seed, tracks)) => {
                        self.record_provider_success(&p_id);
                        self.save_cache(&seed.canonical_key, &p_id, "daily_discover", &tracks, 21600);
                        let prov = if !seed.artist.is_empty() {
                            Some(format!("Similar to {}", seed.artist))
                        } else {
                            None
                        };
                        for t in tracks {
                            collected_tracks.push((p_id.clone(), t, prov.clone()));
                        }
                    }
                    Err((_p_id, _seed, ProviderErrorType::Generic(msg))) if msg == "cancelled" => {
                        return FederatedShelfResult {
                            tracks: self.compile_and_deduplicate(cached_tracks),
                            failed_providers: vec!["cancelled".into()],
                            is_partial: true,
                        };
                    }
                    Err((p_id, _seed, err_type)) => {
                        self.record_provider_failure(&p_id, &err_type);
                        failed_providers.push(format!("{}: {:?}", p_id, err_type));
                    }
                }
            }
        }

        let merged = self.compile_and_deduplicate(collected_tracks);
        let is_partial = !failed_providers.is_empty();
        FederatedShelfResult {
            tracks: merged,
            failed_providers,
            is_partial,
        }
    }

    pub fn compile_and_deduplicate(&self, tracks: Vec<(String, TrackResult, Option<String>)>) -> Vec<FederatedTrack> {
        let overrides = self.load_overrides();
        let mut federated: Vec<FederatedTrack> = Vec::new();

        for (provider_id, track, provenance) in tracks {
            let key = make_canonical_key(&track.title, &track.artist);
            let candidate = CandidateTrack {
                canonical_key: key.clone(),
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                isrc: track.isrc.clone(),
                duration_ms: track.duration_ms,
            };

            let source = TrackSourceInfo::Remote {
                provider_id: provider_id.clone(),
                remote_track_id: track.id.clone(),
                stream_url: track.stream_url.clone(),
                quality_hint: track.quality_hint.clone(),
                cover_art_url: track.cover_art_url.clone(),
                duration_ms: track.duration_ms,
            };

            // Search for fuzzy duplicate in federated list
            let mut matched_idx = None;
            for (idx, existing) in federated.iter().enumerate() {
                let existing_candidate = CandidateTrack {
                    canonical_key: existing.canonical_key.clone(),
                    title: existing.title.clone(),
                    artist: existing.artist.clone(),
                    album: existing.album.clone(),
                    isrc: existing.isrc.clone(),
                    duration_ms: existing.duration_ms,
                };

                let res = compute_dedup_confidence(&candidate, &existing_candidate, &overrides);
                if res.is_match {
                    matched_idx = Some(idx);
                    break;
                }
            }

            if let Some(idx) = matched_idx {
                // Merge stream source into existing track
                if !federated[idx].sources.contains(&source) {
                    federated[idx].sources.push(source);
                }
                if federated[idx].seed_provenance.is_none() && provenance.is_some() {
                    federated[idx].seed_provenance = provenance;
                }
            } else {
                federated.push(FederatedTrack {
                    canonical_key: key,
                    title: track.title,
                    artist: track.artist,
                    album: track.album,
                    isrc: track.isrc,
                    cover_art_url: track.cover_art_url,
                    duration_ms: track.duration_ms,
                    play_count: 0,
                    seed_provenance: provenance,
                    sources: vec![source],
                });
            }
        }

        federated
    }

    fn canonical_songs_to_federated(&self, songs: Vec<CanonicalSong>) -> Vec<FederatedTrack> {
        songs.into_iter().map(|s| {
            let source = if let Some(local_id) = s.local_track_id {
                TrackSourceInfo::Local {
                    track_id: local_id,
                    file_path: s.local_file_path.unwrap_or_default(),
                    album_id: None,
                }
            } else {
                TrackSourceInfo::Remote {
                    provider_id: s.last_provider_id,
                    remote_track_id: s.last_source_id,
                    stream_url: None,
                    quality_hint: None,
                    cover_art_url: s.cover_art_url.clone(),
                    duration_ms: s.duration_ms,
                }
            };

            FederatedTrack {
                canonical_key: s.canonical_key,
                title: s.title,
                artist: s.artist,
                album: s.album,
                isrc: None,
                cover_art_url: s.cover_art_url,
                duration_ms: s.duration_ms,
                play_count: s.play_count as u64,
                seed_provenance: None,
                sources: vec![source],
            }
        }).collect()
    }
}
