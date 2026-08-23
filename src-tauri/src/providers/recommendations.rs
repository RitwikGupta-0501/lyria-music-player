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
use crate::providers::{CanonicalSeedV1, ProviderManager, TrackResult, RadioStreamResultV1, PROVIDER_ABI_VERSION};
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
    pub jump_back_in: Vec<queries::IncompleteSessionItem>,
    pub heavy_rotation: queries::HeavyRotationShelf,
    pub forgotten_favorites: Vec<FederatedTrack>,
    pub cold_start_seeds: Vec<queries::ColdStartSeedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RadioMixCard {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub category: String, // "artist" | "temporal_mood"
    pub covers: Vec<String>,
    pub gradient_start: String,
    pub gradient_end: String,
    pub seed: CanonicalSeedV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdjacentHorizonPayload {
    pub dominant_genre_or_artist: String,
    pub suggested_genre: String,
    pub tagline: String,
    pub description: String,
    pub accent_color: String,
    pub seed: CanonicalSeedV1,
    pub preview_tracks: Vec<FederatedTrack>,
}

pub struct RecommendationCompiler {
    provider_manager: Option<Arc<ProviderManager>>,
    db_path: PathBuf,
}

impl RecommendationCompiler {
    pub fn new(provider_manager: Arc<ProviderManager>, db_path: PathBuf) -> Self {
        Self {
            provider_manager: Some(provider_manager),
            db_path,
        }
    }

    pub fn new_for_testing(db_path: PathBuf) -> Self {
        Self {
            provider_manager: None,
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
        let jump_back_in = queries::get_incomplete_playback_sessions(&conn, 8).unwrap_or_default();
        let heavy_rotation = queries::get_heavy_rotation_7d(&conn, 6).unwrap_or_default();
        let forgotten_favorites_raw = queries::get_canonical_forgotten_favorites(&conn, 20).unwrap_or_default();
        let cold_start_seeds = queries::get_cold_start_local_artists(&conn, 6).unwrap_or_default();

        Ok(HomeLocalShelves {
            quick_picks: self.canonical_songs_to_federated(quick_picks_raw),
            keep_listening: self.canonical_songs_to_federated(keep_listening_raw),
            jump_back_in,
            heavy_rotation,
            forgotten_favorites: self.canonical_songs_to_federated(forgotten_favorites_raw),
            cold_start_seeds,
        })
    }

    pub fn compile_algorithmic_radios(&self) -> Result<Vec<RadioMixCard>, String> {
        let conn = self.open_read_conn()?;
        let mut cards = Vec::new();

        // 1. Artist Mixes from Heavy Rotation / Top Telemetry
        let hr = queries::get_heavy_rotation_7d(&conn, 3).unwrap_or_default();
        let artist_palettes = [
            ("#2A1E5C", "#0C091A"),
            ("#1E3C72", "#2A5298"),
            ("#4A148C", "#12005E"),
        ];

        for (idx, a) in hr.artists.into_iter().enumerate() {
            let palette = artist_palettes[idx % artist_palettes.len()];
            let covers = if let Some(url) = a.avatar_url {
                vec![url]
            } else {
                Vec::new()
            };

            let seed_key = make_canonical_key("", &a.artist);
            cards.push(RadioMixCard {
                id: format!("radio-artist-{}", a.artist.to_lowercase().replace(' ', "-")),
                title: format!("{} Mix", a.artist),
                subtitle: format!("Inspired by your plays of {}", a.artist),
                category: "artist".to_string(),
                covers,
                gradient_start: palette.0.to_string(),
                gradient_end: palette.1.to_string(),
                seed: CanonicalSeedV1 {
                    abi_version: PROVIDER_ABI_VERSION,
                    canonical_key: seed_key,
                    title: String::new(),
                    artist: a.artist,
                    album: None,
                    isrc: None,
                    duration_ms: None,
                    native_id: None,
                    provider_id: None,
                },
            });
        }

        // 2. Circadian / Temporal Mood Mixes (Time of Day Engine anchored in User Telemetry)
        let hour = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => ((d.as_secs() / 3600) % 24) as u32,
            Err(_) => 12,
        };

        let top_artists = queries::get_heavy_rotation_7d(&conn, 4).unwrap_or_default();
        let a1 = top_artists.artists.get(0).map(|a| a.artist.as_str()).unwrap_or("OneRepublic");
        let a2 = top_artists.artists.get(1).map(|a| a.artist.as_str()).unwrap_or(a1);
        let a3 = top_artists.artists.get(2).map(|a| a.artist.as_str()).unwrap_or(a2);

        let (m1_title, m1_sub, m1_artist, m1_query, m1_start, m1_end, m2_title, m2_sub, m2_artist, m2_query, m2_start, m2_end) = match hour {
            5..=11 => (
                format!("{} & Morning Momentum", a1),
                "Uplifting acoustic rhythms and energetic melodies for dawn".to_string(),
                a1,
                format!("{} upbeat songs", a1),
                "#E65100", "#3E2723",
                format!("{} & Focus Flow", a2),
                "Crisp, rhythmic clarity to start your morning workflow".to_string(),
                a2,
                format!("{} flow songs", a2),
                "#1B5E20", "#002700",
            ),
            12..=16 => (
                format!("{} & Afternoon Drive", a1),
                "Driving mid-tempo flow and dynamic melodies for peak hours".to_string(),
                a1,
                format!("{} dynamic songs", a1),
                "#004D40", "#001E18",
                format!("{} & Deep Work Pulse", a2),
                "Steady rhythmic textures to power through the afternoon".to_string(),
                a2,
                format!("{} focus songs", a2),
                "#311B92", "#12005E",
            ),
            17..=21 => (
                format!("{} & Twilight Harmonies", a2),
                "Warm acoustic resonance and reflective vocal chords for twilight".to_string(),
                a2,
                format!("{} acoustic chill songs", a2),
                "#BF360C", "#3E2723",
                format!("{} & Golden Hour Unwind", a3),
                "Smooth grooves and mellow sonic textures for sunset".to_string(),
                a3,
                format!("{} chill songs", a3),
                "#4A148C", "#12005E",
            ),
            _ => (
                format!("{} & Late Night Drift", a3),
                "Nocturnal basslines and atmospheric ambient rhythm for late hours".to_string(),
                a3,
                format!("{} night chill songs", a3),
                "#1A237E", "#000051",
                format!("{} & Midnight Acoustics", a2),
                "Minimal, introspective acoustic warmth for the quiet night".to_string(),
                a2,
                format!("{} acoustic night songs", a2),
                "#880E4F", "#311B92",
            ),
        };

        cards.push(RadioMixCard {
            id: "radio-mood-primary".to_string(),
            title: m1_title,
            subtitle: m1_sub,
            category: "temporal_mood".to_string(),
            covers: Vec::new(),
            gradient_start: m1_start.to_string(),
            gradient_end: m1_end.to_string(),
            seed: CanonicalSeedV1 {
                abi_version: PROVIDER_ABI_VERSION,
                canonical_key: make_canonical_key(m1_artist, &m1_query),
                title: m1_query,
                artist: m1_artist.to_string(),
                album: None,
                isrc: None,
                duration_ms: None,
                native_id: None,
                provider_id: None,
            },
        });

        cards.push(RadioMixCard {
            id: "radio-mood-secondary".to_string(),
            title: m2_title,
            subtitle: m2_sub,
            category: "temporal_mood".to_string(),
            covers: Vec::new(),
            gradient_start: m2_start.to_string(),
            gradient_end: m2_end.to_string(),
            seed: CanonicalSeedV1 {
                abi_version: PROVIDER_ABI_VERSION,
                canonical_key: make_canonical_key(m2_artist, &m2_query),
                title: m2_query,
                artist: m2_artist.to_string(),
                album: None,
                isrc: None,
                duration_ms: None,
                native_id: None,
                provider_id: None,
            },
        });

        Ok(cards)
    }

    pub async fn compile_federated_radio(&self, seed: &CanonicalSeedV1) -> Result<RadioStreamResultV1, String> {
        let p_mgr = match &self.provider_manager {
            Some(pm) => pm.clone(),
            None => return Err("ProviderManager not initialized".to_string()),
        };

        let candidate_providers = p_mgr.get_providers_with_capability("radio");
        let fallback_providers = if candidate_providers.is_empty() {
            p_mgr.get_providers_with_capability("related")
        } else {
            candidate_providers
        };

        if fallback_providers.is_empty() {
            return Err("No active radio or recommendation providers found".to_string());
        }

        let mut tasks = Vec::new();
        for (p_id, _) in fallback_providers {
            let p_mgr_clone = p_mgr.clone();
            let seed_clone = seed.clone();
            let p_id_clone = p_id.clone();
            tasks.push(tokio::spawn(async move {
                let res = tokio::time::timeout(
                    std::time::Duration::from_millis(4000),
                    p_mgr_clone.get_radio(&p_id_clone, &seed_clone)
                ).await;
                (p_id_clone, res)
            }));
        }

        let mut all_tracks = Vec::new();
        let mut seen_canonical = std::collections::HashSet::new();

        for task in tasks {
            if let Ok((_pid, Ok(Ok(radio_res)))) = task.await {
                for track in radio_res.tracks {
                    let c_key = format!("{}::{}", track.artist.trim().to_lowercase(), track.title.trim().to_lowercase());
                    if seen_canonical.insert(c_key) {
                        all_tracks.push(track);
                    }
                }
            }
        }

        Ok(RadioStreamResultV1 {
            tracks: all_tracks,
            continuation_token: None,
        })
    }

    pub fn compute_adjacent_horizon(&self) -> Result<Option<AdjacentHorizonPayload>, String> {
        let conn = self.open_read_conn()?;
        let top_artists = queries::get_heavy_rotation_7d(&conn, 1).unwrap_or_default();
        let dominant_artist = top_artists.artists.first().map(|a| a.artist.clone()).unwrap_or_else(|| "Your Library".to_string());

        // Contrast bridge mapping based on hash / rotation
        let contrasts = [
            ("Modern Japanese Jazz", "Take a detour from standard structures into intricate polyrhythmic brass and Tokyo city jazz fusion.", "#FFB703"),
            ("Analog Synthwave & Retrowave", "Step into lush analog sawtooth waves, gated drums, and neon retro-futurism.", "#FB8500"),
            ("Neo-Classical & Ambient Strings", "Clear your auditory palette with minimalist acoustic piano and cinematic string quartets.", "#219EBC"),
            ("Afro-Cuban Jazz & Global Funk", "Bridge rhythmic grooves into organic percussion, brass polyrhythms, and vintage soul.", "#E76F51"),
        ];

        let idx = (dominant_artist.len()) % contrasts.len();
        let (suggested_genre, description, accent) = contrasts[idx];

        let seed_key = make_canonical_key(suggested_genre, "Adjacent Horizon");
        let seed = CanonicalSeedV1 {
            abi_version: PROVIDER_ABI_VERSION,
            canonical_key: seed_key,
            title: suggested_genre.to_string(),
            artist: "Adjacent Horizon".to_string(),
            album: None,
            isrc: None,
            duration_ms: None,
            native_id: None,
            provider_id: None,
        };

        // Quick sample preview tracks from local database or fallback
        let preview_raw = queries::get_canonical_discover_seeds(&conn, 4).unwrap_or_default();
        let preview_tracks = self.canonical_songs_to_federated(preview_raw);

        Ok(Some(AdjacentHorizonPayload {
            dominant_genre_or_artist: dominant_artist.clone(),
            suggested_genre: suggested_genre.to_string(),
            tagline: format!("You've been listening to {}. Explore {}.", dominant_artist, suggested_genre),
            description: description.to_string(),
            accent_color: accent.to_string(),
            seed,
            preview_tracks,
        }))
    }

    pub fn get_daily_discover_seeds(&self) -> Result<Vec<CanonicalSeedV1>, String> {
        let conn = self.open_read_conn()?;
        let raw_seeds = queries::get_canonical_discover_seeds(&conn, 15).unwrap_or_default();

        // Distinct artist diversity for seeds: maximum 1 seed per artist
        let mut seen_artists = std::collections::HashSet::new();
        let mut diverse_seeds = Vec::new();

        for s in raw_seeds {
            let artist_norm = s.artist.trim().to_lowercase();
            if seen_artists.insert(artist_norm) {
                let key = make_canonical_key(&s.title, &s.artist);
                diverse_seeds.push(CanonicalSeedV1 {
                    abi_version: PROVIDER_ABI_VERSION,
                    canonical_key: key,
                    title: s.title,
                    artist: s.artist,
                    album: s.album,
                    isrc: None,
                    duration_ms: s.duration_ms,
                    native_id: if s.last_provider_id != "local" { Some(s.last_source_id) } else { None },
                    provider_id: Some(s.last_provider_id),
                });
            }
            if diverse_seeds.len() >= 5 {
                break;
            }
        }

        Ok(diverse_seeds)
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

    pub fn clear_all_cache(&self) {
        if let Ok(conn) = self.open_write_conn() {
            let _ = conn.execute("DELETE FROM recommendation_cache;", []);
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

        let active_providers = self.provider_manager.as_ref().map(|pm| pm.get_providers_with_capability("related")).unwrap_or_default();

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
            let p_mgr = match self.provider_manager.clone() {
                Some(pm) => pm,
                None => continue,
            };
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

        let mut artist_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for (provider_id, track, provenance) in tracks {
            let artist_norm = track.artist.trim().to_lowercase();
            let count = artist_counts.entry(artist_norm.clone()).or_insert(0);
            if *count >= 2 {
                continue; // Enforce artist diversity capping in Daily Discover
            }

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
                *count += 1;
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_compiler_shelves_and_radios() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_echo.db");
        {
            let conn = crate::db::schema::init_db(&db_path).unwrap();
            conn.execute("INSERT INTO albums (id, title, artist, cover_art_path) VALUES (1, 'Kid A', 'Radiohead', '/art/kida.jpg')", []).unwrap();
            conn.execute("INSERT INTO tracks (id, title, artist, album_id, track_number, file_path) VALUES (1, 'Everything in Its Right Place', 'Radiohead', 1, 1, '/music/kida1.flac')", []).unwrap();
            queries::record_playback_event(&conn, "Everything in Its Right Place", "Radiohead", Some("Kid A"), Some("/art/kida.jpg"), "local", "1", Some(251000)).unwrap();
        }

        let compiler = RecommendationCompiler::new_for_testing(db_path);

        // 1. Test get_local_shelves returns all expanded fields
        let shelves = compiler.get_local_shelves().unwrap();
        assert_eq!(shelves.quick_picks.len(), 1);
        assert_eq!(shelves.quick_picks[0].title, "Everything in Its Right Place");
        assert_eq!(shelves.heavy_rotation.artists.len(), 1);
        assert_eq!(shelves.heavy_rotation.artists[0].artist, "Radiohead");

        // 2. Test compile_algorithmic_radios
        let radios = compiler.compile_algorithmic_radios().unwrap();
        assert!(radios.len() >= 3); // At least 1 artist mix + 2 temporal mood mixes
        let artist_mix = radios.iter().find(|r| r.category == "artist").unwrap();
        assert_eq!(artist_mix.title, "Radiohead Mix");

        // 3. Test compute_adjacent_horizon
        let horizon = compiler.compute_adjacent_horizon().unwrap();
        assert!(horizon.is_some());
        let h = horizon.unwrap();
        assert_eq!(h.dominant_genre_or_artist, "Radiohead");
        assert!(!h.suggested_genre.is_empty());
    }
}
