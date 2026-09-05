use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::db::canonical::{
    clean_artist_for_display, compute_dedup_confidence, make_canonical_key, CandidateTrack, FederatedTrack,
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

    pub fn get_local_shelves(&self, mood: Option<&str>) -> Result<HomeLocalShelves, String> {
        let conn = self.open_read_conn()?;

        let quick_picks_raw = if crate::feature_flags::FEATURE_FLAGS.is_enabled(crate::feature_flags::FeatureFlag::HomeQuickPicks) {
            queries::get_canonical_quick_picks(&conn, mood, 20).unwrap_or_default()
        } else {
            Vec::new()
        };

        let keep_listening_raw = queries::get_canonical_keep_listening(&conn, mood, 20).unwrap_or_default();

        let jump_back_in = if crate::feature_flags::FEATURE_FLAGS.is_enabled(crate::feature_flags::FeatureFlag::HomeJumpBackIn) {
            queries::get_incomplete_playback_sessions(&conn, 8).unwrap_or_default()
        } else {
            Vec::new()
        };

        let heavy_rotation = if crate::feature_flags::FEATURE_FLAGS.is_enabled(crate::feature_flags::FeatureFlag::HomeHeavyRotation) {
            queries::get_heavy_rotation_7d(&conn, 20).unwrap_or_default()
        } else {
            crate::db::queries::HeavyRotationShelf { artists: Vec::new(), albums: Vec::new() }
        };

        let forgotten_favorites_raw = if crate::feature_flags::FEATURE_FLAGS.is_enabled(crate::feature_flags::FeatureFlag::HomeForgottenFavorites) {
            queries::get_canonical_forgotten_favorites(&conn, mood, 20).unwrap_or_default()
        } else {
            Vec::new()
        };

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

    pub fn compile_algorithmic_radios(&self, mood: Option<&str>) -> Result<Vec<RadioMixCard>, String> {
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

            let display_name = clean_artist_for_display(&a.artist);
            let seed_key = make_canonical_key("", &display_name);
            cards.push(RadioMixCard {
                id: format!("radio-artist-{}", display_name.to_lowercase().replace(' ', "-")),
                title: format!("{} Mix", display_name),
                subtitle: format!("Inspired by your plays of {}", display_name),
                category: "artist".to_string(),
                covers,
                gradient_start: palette.0.to_string(),
                gradient_end: palette.1.to_string(),
                seed: CanonicalSeedV1 {
                    abi_version: PROVIDER_ABI_VERSION,
                    canonical_key: seed_key,
                    title: String::new(),
                    artist: display_name,
                    album: None,
                    isrc: None,
                    duration_ms: None,
                    native_id: None,
                    provider_id: None,
                },
            });
        }

        // 2. Circadian / Temporal Mood Mixes (Time of Day & Contextual Mood Engine)
        let hour = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => ((d.as_secs() / 3600) % 24) as u32,
            Err(_) => 12,
        };

        let top_artists = queries::get_heavy_rotation_7d(&conn, 4).unwrap_or_default();
        let a1_raw = top_artists.artists.first().map(|a| a.artist.as_str()).unwrap_or("OneRepublic");
        let a2_raw = top_artists.artists.get(1).map(|a| a.artist.as_str()).unwrap_or(a1_raw);
        let a3_raw = top_artists.artists.get(2).map(|a| a.artist.as_str()).unwrap_or(a2_raw);

        let a1_clean = clean_artist_for_display(a1_raw);
        let a2_clean = clean_artist_for_display(a2_raw);
        let a3_clean = clean_artist_for_display(a3_raw);
        let a1 = a1_clean.as_str();
        let a2 = a2_clean.as_str();
        let a3 = a3_clean.as_str();

        let (m1_title, m1_sub, m1_artist, m1_query, m1_start, m1_end, m2_title, m2_sub, m2_artist, m2_query, m2_start, m2_end) = match mood.map(|m| m.trim().to_lowercase()).as_deref() {
            Some("deep focus") | Some("focus") => (
                format!("{} & Focus Flow", a2),
                "Crisp, minimal rhythms to power deep concentration".to_string(),
                a2,
                format!("{} focus instrumental songs", a2),
                "#1B5E20", "#002700",
                format!("{} & Ambient Pulse", a1),
                "Steady rhythmic textures to sustain unbroken focus".to_string(),
                a1,
                format!("{} ambient chill songs", a1),
                "#004D40", "#001E18",
            ),
            Some("relax & chill") | Some("relax") | Some("chill") => (
                format!("{} & Twilight Acoustics", a2),
                "Warm acoustic chords and gentle indie resonance".to_string(),
                a2,
                format!("{} acoustic chill songs", a2),
                "#BF360C", "#3E2723",
                format!("{} & Mellow Soul", a3),
                "Smooth downtempo grooves and mellow vocal warmth".to_string(),
                a3,
                format!("{} neo soul chill songs", a3),
                "#4A148C", "#12005E",
            ),
            Some("energy & drive") | Some("energy") | Some("drive") => (
                format!("{} & High-Energy Drive", a1),
                "Uplifting anthems, dynamic pop, and high-velocity rhythm".to_string(),
                a1,
                format!("{} upbeat energy songs", a1),
                "#E65100", "#3E2723",
                format!("{} & Workout Momentum", a3),
                "Driving tempo and powerful basslines for peak performance".to_string(),
                a3,
                format!("{} high tempo songs", a3),
                "#C2185B", "#311B92",
            ),
            Some("late night drift") | Some("late night") | Some("night") => (
                format!("{} & Late Night Drift", a3),
                "Nocturnal basslines and atmospheric ambient rhythm".to_string(),
                a3,
                format!("{} night chill songs", a3),
                "#1A237E", "#000051",
                format!("{} & Midnight Acoustics", a2),
                "Minimal, introspective acoustic warmth for the quiet night".to_string(),
                a2,
                format!("{} acoustic night songs", a2),
                "#880E4F", "#311B92",
            ),
            Some("commute") => (
                format!("{} & Highway Momentum", a1),
                "Anthemic favorites and melodic pulse for the road".to_string(),
                a1,
                format!("{} road trip drive songs", a1),
                "#004D40", "#001E18",
                format!("{} & Transit Flow", a2),
                "Steady rhythms to make the journey seamless".to_string(),
                a2,
                format!("{} commute upbeat songs", a2),
                "#311B92", "#12005E",
            ),
            _ => match hour {
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
            },
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
                title: m1_query.to_string(),
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
                title: m2_query.to_string(),
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
        self.ensure_providers_synced();
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

    pub fn compute_adjacent_horizons(&self) -> Result<Vec<AdjacentHorizonPayload>, String> {
        let conn = self.open_read_conn()?;
        let top_artists = queries::get_heavy_rotation_7d(&conn, 5).unwrap_or_default();
        let dominant_artist = top_artists.artists.first().map(|a| a.artist.clone()).unwrap_or_else(|| "Your Library".to_string());

        struct HorizonDef {
            genre: &'static str,
            description: &'static str,
            accent: &'static str,
            preview_tracks: &'static [(&'static str, &'static str, &'static str, u64, Option<&'static str>)],
        }

        let definitions = [
            HorizonDef {
                genre: "Modern Japanese Jazz & Fusion",
                description: "Take a detour from standard arrangements into intricate polyrhythmic brass, electric piano solos, and Tokyo city fusion.",
                accent: "#D4A86E",
                preview_tracks: &[
                    ("Midnight Rendezvous", "Casiopea", "Mint Jams", 227000, None),
                    ("Early Summer", "Ryo Fukui", "Scenery", 254000, None),
                    ("Truth", "T-Square", "Truth", 298000, None),
                ],
            },
            HorizonDef {
                genre: "Analog Synthwave & Cyberpunk",
                description: "Step into lush analog sawtooth waves, gated reverb drums, and cinematic neon retro-futurism.",
                accent: "#FF8C38",
                preview_tracks: &[
                    ("Nightcall", "Kavinsky", "OutRun", 259000, Some("https://i.ytimg.com/vi/MV_3Dpw-BRY/hqdefault.jpg")),
                    ("Resonance", "HOME", "Odyssey", 212000, Some("https://i.ytimg.com/vi/8GW6sLrK40k/hqdefault.jpg")),
                    ("Days of Thunder", "The Midnight", "Days of Thunder", 328000, None),
                ],
            },
            HorizonDef {
                genre: "Neo-Classical & Cinematic Strings",
                description: "Clear your auditory palette with minimalist acoustic piano motifs and contemplative, breathing string quartets.",
                accent: "#38BDF8",
                preview_tracks: &[
                    ("Divenire", "Ludovico Einaudi", "Divenire", 402000, None),
                    ("On The Nature of Daylight", "Max Richter", "The Blue Notebooks", 371000, None),
                    ("Written on the Sky", "Max Richter", "The Blue Notebooks", 99000, None),
                ],
            },
            HorizonDef {
                genre: "Afro-Cuban Jazz & Global Funk",
                description: "Bridge rhythmic grooves into organic percussion, montuno piano riffs, brass polyrhythms, and vintage soul.",
                accent: "#E76F51",
                preview_tracks: &[
                    ("Water No Get Enemy", "Fela Kuti", "Expensive Shit", 590000, Some("https://i.ytimg.com/vi/IQBC5URoF0s/hqdefault.jpg")),
                    ("Afrodisia", "Mongo Santamaria", "Afro-Roots", 242000, None),
                    ("Chameleon", "Herbie Hancock", "Head Hunters", 941000, Some("https://i.ytimg.com/vi/UbkqE4fpvdI/hqdefault.jpg")),
                ],
            },
            HorizonDef {
                genre: "Nordic Ambient & Downtempo",
                description: "Immerse in glacial harmonic textures, subdued acoustic strums, and expansive Scandinavian soundscapes.",
                accent: "#A78BFA",
                preview_tracks: &[
                    ("A Walk", "Tycho", "Dive", 317000, Some("https://i.ytimg.com/vi/mehLx_Fjv_c/hqdefault.jpg")),
                    ("Cirrus", "Bonobo", "The North Borders", 352000, None),
                    ("Svefn-g-englar", "Sigur Rós", "Ágætis byrjun", 604000, None),
                ],
            },
        ];

        let mut horizons = Vec::new();
        let start_idx = (dominant_artist.len()) % definitions.len();

        for i in 0..4 {
            let def_idx = (start_idx + i) % definitions.len();
            let def = &definitions[def_idx];

            let seed_key = make_canonical_key(def.genre, "Adjacent Horizon");
            let seed = CanonicalSeedV1 {
                abi_version: PROVIDER_ABI_VERSION,
                canonical_key: seed_key,
                title: def.genre.to_string(),
                artist: "Adjacent Horizon".to_string(),
                album: None,
                isrc: None,
                duration_ms: None,
                native_id: None,
                provider_id: None,
            };

            let preview_tracks: Vec<FederatedTrack> = def.preview_tracks.iter().map(|(t, a, alb, dur, art)| {
                let key = make_canonical_key(t, a);
                FederatedTrack {
                    canonical_key: key.clone(),
                    title: t.to_string(),
                    artist: a.to_string(),
                    album: Some(alb.to_string()),
                    isrc: None,
                    cover_art_url: art.map(|s| s.to_string()),
                    duration_ms: Some(*dur),
                    play_count: 0,
                    seed_provenance: Some(def.genre.to_string()),
                    sources: vec![
                        TrackSourceInfo::Remote {
                            provider_id: "youtube-wasm".to_string(),
                            remote_track_id: key,
                            stream_url: None,
                            quality_hint: None,
                            cover_art_url: art.map(|s| s.to_string()),
                            duration_ms: Some(*dur),
                        }
                    ],
                    liked: false,
                }
            }).collect();

            horizons.push(AdjacentHorizonPayload {
                dominant_genre_or_artist: dominant_artist.clone(),
                suggested_genre: def.genre.to_string(),
                tagline: format!("Beyond {}. Explore {}.", dominant_artist, def.genre),
                description: def.description.to_string(),
                accent_color: def.accent.to_string(),
                seed,
                preview_tracks,
            });
        }

        Ok(horizons)
    }

    pub fn compute_adjacent_horizon(&self) -> Result<Option<AdjacentHorizonPayload>, String> {
        let horizons = self.compute_adjacent_horizons()?;
        Ok(horizons.into_iter().next())
    }

    pub fn get_daily_discover_seeds(&self, mood: Option<&str>) -> Result<Vec<CanonicalSeedV1>, String> {
        let conn = self.open_read_conn()?;
        let raw_seeds = queries::get_canonical_discover_seeds(&conn, mood, 25).unwrap_or_default();

        // Distinct artist diversity for seeds: maximum 1 seed per artist
        // Balance selection: prioritize up to 3 recent listens and up to 2 favorite tracks
        let mut seen_artists = std::collections::HashSet::new();
        let mut recent_candidates = Vec::new();
        let mut favorite_candidates = Vec::new();

        for s in raw_seeds {
            let artist_norm = s.artist.trim().to_lowercase();
            if seen_artists.insert(artist_norm) {
                let is_recent = s.last_played_at.is_some();
                let is_favorite = s.liked || s.play_count >= 2;

                let key = make_canonical_key(&s.title, &s.artist);
                let seed = CanonicalSeedV1 {
                    abi_version: PROVIDER_ABI_VERSION,
                    canonical_key: key,
                    title: s.title,
                    artist: s.artist,
                    album: s.album,
                    isrc: None,
                    duration_ms: s.duration_ms,
                    native_id: if s.last_provider_id != "local" { Some(s.last_source_id) } else { None },
                    provider_id: Some(s.last_provider_id),
                };

                if is_recent {
                    recent_candidates.push(seed.clone());
                }
                if is_favorite {
                    favorite_candidates.push(seed);
                } else if !is_recent {
                    recent_candidates.push(seed);
                }
            }
        }

        let mut diverse_seeds = Vec::new();
        let mut picked_keys = std::collections::HashSet::new();

        // 1. Pick up to 6 recent seeds
        for s in &recent_candidates {
            if diverse_seeds.len() >= 6 {
                break;
            }
            if picked_keys.insert(s.canonical_key.clone()) {
                diverse_seeds.push(s.clone());
            }
        }

        // 2. Pick up to 4 favorite seeds
        for s in &favorite_candidates {
            if diverse_seeds.len() >= 10 {
                break;
            }
            if picked_keys.insert(s.canonical_key.clone()) {
                diverse_seeds.push(s.clone());
            }
        }

        // 3. Fallback: fill remaining slots up to 10 from all candidate pools
        for s in recent_candidates.into_iter().chain(favorite_candidates) {
            if diverse_seeds.len() >= 10 {
                break;
            }
            if picked_keys.insert(s.canonical_key.clone()) {
                diverse_seeds.push(s);
            }
        }

        Ok(diverse_seeds)
    }

    pub fn get_valid_cache(&self, seed_canonical_key: &str, provider_id: &str, shelf_type: &str) -> Option<Vec<TrackResult>> {
        let conn = self.open_read_conn().ok()?;
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM recommendation_cache
             WHERE seed_canonical_key = ?1 AND provider_id = ?2 AND shelf_type = ?3
               AND (datetime(fetched_at, '+' || ttl_seconds || ' seconds') > datetime('now') OR ttl_seconds = 0)"
        ).ok()?;

        let json: String = stmt.query_row(rusqlite::params![seed_canonical_key, provider_id, shelf_type], |row| {
            row.get(0)
        }).ok()?;

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

    fn ensure_providers_synced(&self) {
        if let Some(ref pm) = self.provider_manager {
            let is_empty = pm.providers.read().map(|p| p.is_empty()).unwrap_or(true);
            if is_empty {
                if let Ok(conn) = self.open_read_conn() {
                    if let Ok(providers) = crate::db::queries::get_providers(&conn) {
                        pm.sync_registry(providers);
                    }
                }
            }
        }
    }

    pub async fn get_federated_daily_discover(
        &self,
        seeds: Vec<CanonicalSeedV1>,
        cancel_token: CancellationToken,
    ) -> FederatedShelfResult {
        self.ensure_providers_synced();
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
                    res = tokio::time::timeout(Duration::from_secs(8), async {
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
        let liked_keys: std::collections::HashSet<String> = if let Ok(conn) = self.open_read_conn() {
            let stmt = conn.prepare("SELECT canonical_key FROM song_telemetry WHERE liked = 1").ok();
            if let Some(mut s) = stmt {
                s.query_map([], |row| row.get::<_, String>(0))
                    .ok()
                    .map(|iter| iter.flatten().collect())
                    .unwrap_or_default()
            } else {
                std::collections::HashSet::new()
            }
        } else {
            std::collections::HashSet::new()
        };

        let mut federated: Vec<FederatedTrack> = Vec::new();
        let mut artist_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for (provider_id, track, provenance) in tracks {
            let artist_norm = track.artist.trim().to_lowercase();
            let count = artist_counts.entry(artist_norm.clone()).or_insert(0);
            if *count >= 3 {
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
                let is_liked = liked_keys.contains(&key);
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
                    liked: is_liked,
                });
            }
        }

        federated
    }

    fn canonical_songs_to_federated(&self, songs: Vec<CanonicalSong>) -> Vec<FederatedTrack> {
        let artwork_dir = self.db_path.parent().and_then(|p| p.parent()).map(|p| p.join("artwork"));

        songs.into_iter().map(|s| {
            let mut resolved_cover = s.cover_art_url.clone();
            if resolved_cover.is_none() {
                if let Some(local_id) = s.local_track_id {
                    if let Some(ref art_dir) = artwork_dir {
                        let candidate_path = art_dir.join(format!("{}.jpg", local_id));
                        if candidate_path.exists() {
                            resolved_cover = Some(candidate_path.to_string_lossy().to_string());
                        }
                    }
                }
            }

            let source = if let Some(local_id) = s.local_track_id {
                if let Some(ref fp) = s.local_file_path {
                    if !fp.is_empty() {
                        TrackSourceInfo::Local {
                            track_id: local_id,
                            file_path: fp.clone(),
                            album_id: None,
                        }
                    } else {
                        TrackSourceInfo::Remote {
                            provider_id: if s.last_provider_id == "local" || s.last_provider_id.is_empty() { "youtube-wasm".to_string() } else { s.last_provider_id },
                            remote_track_id: s.last_source_id,
                            stream_url: None,
                            quality_hint: None,
                            cover_art_url: resolved_cover.clone(),
                            duration_ms: s.duration_ms,
                        }
                    }
                } else {
                    TrackSourceInfo::Remote {
                        provider_id: if s.last_provider_id == "local" || s.last_provider_id.is_empty() { "youtube-wasm".to_string() } else { s.last_provider_id },
                        remote_track_id: s.last_source_id,
                        stream_url: None,
                        quality_hint: None,
                        cover_art_url: resolved_cover.clone(),
                        duration_ms: s.duration_ms,
                    }
                }
            } else if s.last_provider_id == "local" || s.last_provider_id.is_empty() {
                if let Some(ref fp) = s.local_file_path {
                    if !fp.is_empty() {
                        TrackSourceInfo::Local {
                            track_id: 0,
                            file_path: fp.clone(),
                            album_id: None,
                        }
                    } else {
                        TrackSourceInfo::Remote {
                            provider_id: "youtube-wasm".to_string(),
                            remote_track_id: s.last_source_id,
                            stream_url: None,
                            quality_hint: None,
                            cover_art_url: resolved_cover.clone(),
                            duration_ms: s.duration_ms,
                        }
                    }
                } else {
                    TrackSourceInfo::Remote {
                        provider_id: "youtube-wasm".to_string(),
                        remote_track_id: s.last_source_id,
                        stream_url: None,
                        quality_hint: None,
                        cover_art_url: resolved_cover.clone(),
                        duration_ms: s.duration_ms,
                    }
                }
            } else {
                TrackSourceInfo::Remote {
                    provider_id: s.last_provider_id,
                    remote_track_id: s.last_source_id,
                    stream_url: None,
                    quality_hint: None,
                    cover_art_url: resolved_cover.clone(),
                    duration_ms: s.duration_ms,
                }
            };

            FederatedTrack {
                canonical_key: s.canonical_key,
                title: s.title,
                artist: s.artist,
                album: s.album,
                isrc: None,
                cover_art_url: resolved_cover,
                duration_ms: s.duration_ms,
                play_count: s.play_count as u64,
                seed_provenance: None,
                sources: vec![source],
                liked: s.liked,
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
        let shelves = compiler.get_local_shelves(None).unwrap();
        assert_eq!(shelves.quick_picks.len(), 1);
        assert_eq!(shelves.quick_picks[0].title, "Everything in Its Right Place");
        assert_eq!(shelves.heavy_rotation.artists.len(), 1);
        assert_eq!(shelves.heavy_rotation.artists[0].artist, "Radiohead");

        // 2. Test compile_algorithmic_radios
        let radios = compiler.compile_algorithmic_radios(None).unwrap();
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

    #[test]
    fn test_daily_discover_seeds_and_cache_ttl() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ttl.db");
        {
            let conn = crate::db::schema::init_db(&db_path).unwrap();
            queries::record_playback_event(&conn, "Track Old", "Old Artist", None, None, "local", "1", Some(200000)).unwrap();
            queries::record_playback_event(&conn, "Track New", "New Artist", None, None, "local", "2", Some(180000)).unwrap();
        }

        let compiler = RecommendationCompiler::new_for_testing(db_path);

        // 1. Verify seeds pick the distinct artists
        let seeds = compiler.get_daily_discover_seeds(None).unwrap();
        assert_eq!(seeds.len(), 2);

        // 2. Test cache save and TTL retrieval
        let track = TrackResult {
            id: "rec-1".into(),
            title: "Recommended 1".into(),
            artist: "Rec Artist".into(),
            album: None,
            isrc: None,
            cover_art_url: None,
            stream_url: None,
            quality_hint: None,
            duration_ms: None,
            plays: None,
        };

        // Cache valid with 3600s TTL
        compiler.save_cache("seed::1", "test-provider", "daily_discover", std::slice::from_ref(&track), 3600);
        let cached = compiler.get_valid_cache("seed::1", "test-provider", "daily_discover");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        // Expired cache with negative/0 relative time simulation (insert directly with expired fetched_at)
        let write_conn = compiler.open_write_conn().unwrap();
        write_conn.execute(
            "UPDATE recommendation_cache SET fetched_at = datetime('now', '-7200 seconds') WHERE seed_canonical_key = 'seed::1'",
            [],
        ).unwrap();

        let expired = compiler.get_valid_cache("seed::1", "test-provider", "daily_discover");
        assert!(expired.is_none(), "Expired cache should return None");
    }
}
