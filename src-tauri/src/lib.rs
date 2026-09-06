use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{self, Sender};
use tauri::{AppHandle, Manager, State, RunEvent};
use walkdir::WalkDir;
use tokio::sync::oneshot;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
pub mod db;
pub mod audio;
pub mod providers;
pub mod queue;
pub mod telemetry;
pub mod feature_flags;
pub mod sandbox;
pub mod logger;

use providers::{ProviderManager, TrackResult};
use audio::AudioCommand;
use db::{DbRequest, TrackData};
use queue::QueueState;

#[derive(Serialize, Deserialize, Clone)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub cover_art_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalTrack {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album_id: Option<i64>,
    pub track_number: Option<i64>,
    pub file_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

pub struct AppState {
    pub audio_tx: std::sync::Mutex<Sender<AudioCommand>>,
    pub db_tx: std::sync::mpsc::Sender<DbRequest>,
    pub provider_manager: Arc<ProviderManager>,
    pub recommendation_compiler: Arc<providers::recommendations::RecommendationCompiler>,
    pub in_flight_recommendation_cancel: Arc<std::sync::Mutex<Option<tokio_util::sync::CancellationToken>>>,
    pub audio_thread: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,
    pub db_thread: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,
    pub queue: std::sync::Mutex<QueueState>,
    pub reqwest_client: reqwest::Client,
    pub sandbox_manager: sandbox::sandbox_vm::SandboxManager,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub file_path: String,
    pub status: String,
    pub error_message: Option<String>,
    pub checksum: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub settings_schema: Option<String>,
    pub priority: i32,
    pub icon: Option<String>,
    pub settings: Option<String>,
}

#[tauri::command]
async fn sync_providers(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut providers = Vec::new();
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let providers_dir = app_data_dir.join("providers");
    
    if !providers_dir.exists() {
        let _ = std::fs::create_dir_all(&providers_dir);
    }
    
    // We parse the Lua metadata using regex to avoid spinning up a full VM for every file
    let re_id = regex::Regex::new(r#"id\s*=\s*"([^"]+)""#).unwrap();
    let re_name = regex::Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap();
    let re_author = regex::Regex::new(r#"author\s*=\s*"([^"]+)""#).unwrap();
    let re_version = regex::Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap();
    let re_homepage = regex::Regex::new(r#"homepage\s*=\s*"([^"]+)""#).unwrap();
    let re_settings = regex::Regex::new(r#"settings_schema\s*=\s*"([^"]+)""#).unwrap();
    let re_priority = regex::Regex::new(r#"priority\s*=\s*([0-9]+)"#).unwrap();
    let re_icon = regex::Regex::new(r#"icon\s*=\s*"([^"]+)""#).unwrap();
    // Simplified capabilities parser - expects a simple lua array of strings
    let re_capabilities = regex::Regex::new(r#"capabilities\s*=\s*\{([^}]+)\}"#).unwrap();

    if let Ok(entries) = std::fs::read_dir(&providers_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            
            let ext = path.extension().and_then(|e| e.to_str());
            
            if ext == Some("lua") {
                let fallback_id = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                let checksum = Some(hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>());
                
                let id = re_id.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()).unwrap_or(fallback_id);
                let name = re_name.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()).unwrap_or(id.clone());
                let author = re_author.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()).unwrap_or_else(|| "Unknown".to_string());
                let version = re_version.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()).unwrap_or_else(|| "0.0.0".to_string());
                let homepage = re_homepage.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
                let settings_schema = re_settings.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
                let priority = re_priority.captures(&content).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or(0);
                let icon = re_icon.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
                
                let capabilities = re_capabilities.captures(&content).and_then(|c| c.get(1)).map(|m| {
                    m.as_str().split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                });
                
                providers.push(ProviderInfo {
                    id,
                    name,
                    author,
                    version,
                    file_path: path.to_string_lossy().into_owned(),
                    status: "enabled".to_string(), // Default when inserting, preserved on update by SQL
                    error_message: None,
                    checksum,
                    capabilities,
                    homepage,
                    settings_schema,
                    priority,
                    icon,
                    settings: None,
                });
            } else if ext == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = json["id"].as_str().unwrap_or_default().to_string();
                        let name = json["name"].as_str().unwrap_or(&id).to_string();
                        let author = json["author"].as_str().unwrap_or("Unknown").to_string();
                        let version = json["version"].as_str().unwrap_or("0.0.0").to_string();
                        
                        let main_file = json["main"].as_str().unwrap_or_default();
                        let mut wasm_path = path.clone();
                        wasm_path.set_file_name(main_file);
                        
                        let capabilities = json["capabilities"]
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>());
                            
                        let homepage = json["homepage"].as_str().map(|s| s.to_string());
                        let settings_schema = json["settings_schema"].as_str().map(|s| s.to_string());
                        let priority = json["priority"].as_i64().unwrap_or(0) as i32;
                        let icon = json["icon"].as_str().map(|s| s.to_string());
                        
                        providers.push(ProviderInfo {
                            id,
                            name,
                            author,
                            version,
                            file_path: wasm_path.to_string_lossy().into_owned(),
                            status: "enabled".to_string(),
                            error_message: None,
                            checksum: None,
                            capabilities,
                            homepage,
                            settings_schema,
                            priority,
                            icon,
                            settings: None,
                        });
                    }
                }
            }
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::SyncProviders { providers, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())??;
    
    Ok(())
}

#[tauri::command]
async fn get_providers(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetProviders { resp: tx }).map_err(|e| e.to_string())?;
    let providers = rx.await.map_err(|e| e.to_string())??;
    
    state.provider_manager.sync_registry(providers.clone());
    
    Ok(providers)
}

pub async fn warmup_all_eligible_providers(state: &AppState) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    if let Err(e) = state.db_tx.send(crate::db::DbRequest::GetProviders { resp: tx }) {
        tracing::warn!("Failed to request providers for warmup: {}", e);
        return;
    }
    
    let providers = match rx.await {
        Ok(Ok(p)) => p,
        _ => return,
    };
    
    state.provider_manager.sync_registry(providers);
    let warmup_ids = state.provider_manager.get_warmup_eligible_providers();
    
    if warmup_ids.is_empty() {
        return;
    }
    
    tracing::info!("Starting background prewarm for {} provider(s): {:?}", warmup_ids.len(), warmup_ids);
    for id in warmup_ids {
        if let Err(e) = state.provider_manager.warmup_provider(&id).await {
            tracing::warn!("Failed to warmup provider '{}': {}", id, e);
        }
    }
    tracing::info!("Background prewarm for all eligible providers completed successfully");
}


#[tauri::command]
async fn delete_provider(
    app: AppHandle,
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::DeleteProvider {
        provider_id: provider_id.clone(),
        resp: tx,
    }).map_err(|e| e.to_string())?;

    let file_path_opt = rx.await.map_err(|e| e.to_string())??;

    if let Some(file_path) = file_path_opt {
        let path = std::path::PathBuf::from(&file_path);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        let json_manifest = path.with_extension("json");
        if json_manifest.exists() {
            let _ = std::fs::remove_file(&json_manifest);
        }
    }

    state.provider_manager.remove_provider(&provider_id);

    let _ = sync_providers(app, state).await;
    Ok(())
}

#[tauri::command]
async fn toggle_provider(
    provider_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::ToggleProvider { 
        provider_id, 
        enabled, 
        resp: tx 
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())??;
    Ok(())
}

#[tauri::command]
async fn save_provider_settings(
    provider_id: String,
    settings_json: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::SaveProviderSettings {
        provider_id,
        settings_json,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    
    rx.await.map_err(|e| e.to_string())??;
    Ok(())
}


#[tauri::command]
async fn browse_provider_album(
    state: State<'_, AppState>,
    provider_id: String,
    album_id: String,
) -> Result<crate::providers::AlbumDetailResult, String> {
    state.provider_manager.browse_album(&provider_id, &album_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn browse_provider_artist(
    state: State<'_, AppState>,
    provider_id: String,
    artist_id: String,
) -> Result<crate::providers::ArtistDetailResult, String> {
    let res = state.provider_manager.browse_artist(&provider_id, &artist_id).await.map_err(|e| e.to_string())?;

    // Asynchronously persist artist portrait & canonical name to SQLite cache
    let (tx, _rx) = oneshot::channel();
    let _ = state.db_tx.send(crate::db::DbRequest::UpsertArtistMetadata {
        id: artist_id.clone(),
        name: res.name.clone(),
        avatar_url: res.avatar_url.clone(),
        bio: res.bio.clone(),
        provider_id: provider_id.clone(),
        resp: tx,
    });

    Ok(res)
}

#[tauri::command]
async fn search_provider_categorized(
    state: State<'_, AppState>,
    provider_id: String,
    query: String,
    filter: Option<String>,
) -> Result<crate::providers::CategorizedSearchResult, String> {
    let input = crate::providers::SearchQueryInput { query, filter };
    let results = state.provider_manager
        .search_categorized(&provider_id, &input)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[tauri::command]
async fn search_provider(state: State<'_, AppState>, provider_id: String, query: String) -> Result<Vec<TrackResult>, String> {
    let results = state.provider_manager
        .search(&provider_id, &query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeFeedPayload {
    pub quick_picks: Vec<crate::db::queries::CanonicalSong>,
    pub keep_listening: Vec<crate::db::queries::CanonicalSong>,
    pub forgotten_favorites: Vec<crate::db::queries::CanonicalSong>,
    pub discover_seeds: Vec<crate::db::queries::CanonicalSong>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedUrlPayload {
    pub provider_id: String,
    pub provider_name: String,
    pub track: crate::providers::TrackResult,
}

#[tauri::command]
async fn resolve_stream_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<Option<ResolvedUrlPayload>, String> {
    match state.provider_manager.resolve_url(&url).await {
        Ok(Some((provider_id, provider_name, track))) => {
            Ok(Some(ResolvedUrlPayload {
                provider_id,
                provider_name,
                track,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}


#[tauri::command]
async fn get_home_local_shelves(
    state: State<'_, AppState>,
    mood: Option<String>,
) -> Result<providers::recommendations::HomeLocalShelves, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageHome) {
        return Ok(providers::recommendations::HomeLocalShelves {
            quick_picks: Vec::new(),
            keep_listening: Vec::new(),
            jump_back_in: Vec::new(),
            heavy_rotation: crate::db::queries::HeavyRotationShelf { artists: Vec::new(), albums: Vec::new() },
            forgotten_favorites: Vec::new(),
            cold_start_seeds: Vec::new(),
        });
    }
    let mood_filter = if mood.as_deref() == Some("All") { None } else { mood.as_deref() };
    state.recommendation_compiler.get_local_shelves(mood_filter)
}

#[tauri::command]
async fn get_home_remote_shelves(
    state: State<'_, AppState>,
    mood: Option<String>,
) -> Result<providers::recommendations::FederatedShelfResult, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageHome)
        || !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::HomeDailyDiscover)
    {
        return Ok(providers::recommendations::FederatedShelfResult::empty());
    }
    let new_token = tokio_util::sync::CancellationToken::new();
    {
        let mut lock = state.in_flight_recommendation_cancel.lock().unwrap();
        if let Some(old_token) = lock.take() {
            old_token.cancel();
        }
        *lock = Some(new_token.clone());
    }

    let mood_filter = if mood.as_deref() == Some("All") { None } else { mood.as_deref() };
    let seeds = state.recommendation_compiler.get_daily_discover_seeds(mood_filter)?;
    Ok(state.recommendation_compiler.get_federated_daily_discover(seeds, new_token).await)
}


#[tauri::command]
async fn get_home_radios(
    state: State<'_, AppState>,
    mood: Option<String>,
) -> Result<Vec<providers::recommendations::RadioMixCard>, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageHome)
        || !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::HomeRadioMix)
    {
        return Ok(Vec::new());
    }
    let mood_filter = if mood.as_deref() == Some("All") { None } else { mood.as_deref() };
    state.recommendation_compiler.compile_algorithmic_radios(mood_filter)
}

#[tauri::command]
async fn get_home_adjacent_horizons(
    state: State<'_, AppState>,
) -> Result<Vec<providers::recommendations::AdjacentHorizonPayload>, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageHome)
        || !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::HomeAdjacentHorizons)
    {
        return Ok(Vec::new());
    }
    state.recommendation_compiler.compute_adjacent_horizons()
}

#[tauri::command]
async fn get_home_adjacent_horizon(
    state: State<'_, AppState>,
) -> Result<Option<providers::recommendations::AdjacentHorizonPayload>, String> {
    state.recommendation_compiler.compute_adjacent_horizon()
}

#[tauri::command]
async fn resolve_autoplay_next_track(
    state: State<'_, AppState>,
    seed: queue::QueueTrack,
) -> Result<Option<queue::QueueTrack>, String> {
    queue::autoplay::resolve_autoplay_next_track(&state, &seed).await
}

#[tauri::command]
async fn get_radio_stream(
    state: State<'_, AppState>,
    #[allow(unused_variables)] provider_id: String,
    seed: providers::CanonicalSeedV1,
) -> Result<providers::RadioStreamResultV1, String> {
    state.recommendation_compiler.compile_federated_radio(&seed).await
}

#[tauri::command]
async fn get_home_feed(state: State<'_, AppState>, mood: Option<String>) -> Result<HomeFeedPayload, String> {
    let mood_filter = if mood.as_deref() == Some("All") { None } else { mood };

    let (tx_qp, rx_qp) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetCanonicalQuickPicks { mood: mood_filter.clone(), limit: 20, resp: tx_qp }).map_err(|e| e.to_string())?;
    let qp = rx_qp.await.map_err(|e| e.to_string())??;

    let (tx_kl, rx_kl) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetCanonicalKeepListening { mood: mood_filter.clone(), limit: 20, resp: tx_kl }).map_err(|e| e.to_string())?;
    let kl = rx_kl.await.map_err(|e| e.to_string())??;

    let (tx_ff, rx_ff) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetCanonicalForgottenFavorites { mood: mood_filter.clone(), limit: 20, resp: tx_ff }).map_err(|e| e.to_string())?;
    let ff = rx_ff.await.map_err(|e| e.to_string())??;

    let (tx_ds, rx_ds) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetCanonicalDiscoverSeeds { mood: mood_filter, limit: 5, resp: tx_ds }).map_err(|e| e.to_string())?;
    let ds = rx_ds.await.map_err(|e| e.to_string())??;

    Ok(HomeFeedPayload {
        quick_picks: qp,
        keep_listening: kl,
        forgotten_favorites: ff,
        discover_seeds: ds,
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn record_track_play(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    album: Option<String>,
    cover_art_url: Option<String>,
    provider_id: String,
    source_id: String,
    duration_ms: Option<u64>,
) -> Result<(), String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::RecordPlaybackEvent {
        title,
        artist,
        album,
        cover_art_url,
        provider_id: provider_id.clone(),
        source_id: source_id.clone(),
        duration_ms,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())??;

    if provider_id != "local" && !source_id.is_empty() {
        let p_mgr = state.provider_manager.clone();
        let p_id = provider_id.clone();
        let s_id = source_id.clone();
        let dur = duration_ms.unwrap_or(0);
        tokio::spawn(async move {
            let event = crate::providers::PlaybackTelemetryEventV1 {
                native_track_id: s_id,
                duration_ms: dur,
                total_track_duration_ms: dur,
                completed: true,
            };
            let _ = p_mgr.notify_playback_completed(&p_id, &event).await;
        });
    }

    Ok(())
}

#[tauri::command]
async fn get_liked_songs(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::queries::CanonicalSong>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetCanonicalLikedSongs {
        limit: limit.unwrap_or(500),
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn toggle_track_like(
    state: State<'_, AppState>,
    canonical_key: String,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    cover_art_url: Option<String>,
    provider_id: Option<String>,
    source_id: Option<String>,
    local_track_id: Option<i64>,
    duration_ms: Option<u64>,
) -> Result<bool, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::ToggleCanonicalLike {
        canonical_key,
        title,
        artist,
        album,
        cover_art_url,
        provider_id,
        source_id,
        local_track_id,
        duration_ms,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_extension_metrics(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::queries::ExtensionMetric>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetExtensionMetrics { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_explore_feed(state: State<'_, AppState>) -> Result<Vec<crate::providers::AggregatedModule>, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageExplore) {
        return Ok(Vec::new());
    }
    Ok(state.provider_manager.get_all_explore_modules().await)
}

#[tauri::command]
async fn get_provider_modules(state: State<'_, AppState>, provider_id: String) -> Result<Vec<providers::ProviderModule>, String> {
    let results = state.provider_manager
        .get_modules(&provider_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[tauri::command]
async fn fetch_provider_module(
    state: State<'_, AppState>, 
    provider_id: String, 
    module_id: String,
    force_refresh: Option<bool>,
) -> Result<providers::ModuleData, String> {
    if !feature_flags::FEATURE_FLAGS.is_enabled(feature_flags::FeatureFlag::PageExplore) {
        return Ok(providers::ModuleData { items: Vec::new() });
    }
    let is_force = force_refresh.unwrap_or(false);

    if !is_force {
        let (tx, rx) = oneshot::channel();
        if state.db_tx.send(DbRequest::GetFeedCache {
            provider_id: provider_id.clone(),
            module_id: module_id.clone(),
            resp: tx,
        }).is_ok() {
            if let Ok(Ok(Some(json_str))) = rx.await {
                if let Ok(data) = serde_json::from_str::<providers::ModuleData>(&json_str) {
                    if !data.items.is_empty() {
                        return Ok(data);
                    }
                }
            }
        }
    }

    let results = state.provider_manager
        .fetch_module(&provider_id, &module_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(json_str) = serde_json::to_string(&results) {
        let (tx, _rx) = oneshot::channel();
        let _ = state.db_tx.send(DbRequest::SetFeedCache {
            provider_id,
            module_id,
            payload_json: json_str,
            ttl_seconds: 86400,
            resp: tx,
        });
    }

    Ok(results)
}

#[tauri::command]
async fn get_provider_config(
    state: State<'_, AppState>,
    provider_id: String,
    key: String,
) -> Result<Option<String>, String> {
    let secret_store = providers::secrets::ProviderSecretStore::new();
    Ok(secret_store.get(&provider_id, &key, &state.db_tx))
}

#[tauri::command]
async fn set_provider_config(
    state: State<'_, AppState>,
    provider_id: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let secret_store = providers::secrets::ProviderSecretStore::new();
    secret_store.set(&provider_id, &key, &value, &state.db_tx)
}

#[tauri::command]
async fn search_library(state: State<'_, AppState>, query: String, limit: u32) -> Result<Vec<LocalTrack>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(DbRequest::SearchLibrary { query, limit, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn fuzzy_match_tracks(local_track: LocalTrack, remote_tracks: Vec<providers::TrackResult>) -> Vec<providers::TrackResult> {
    let mut matches = Vec::new();
    let local_title = local_track.title.to_lowercase();
    let local_artist = local_track.artist.unwrap_or_default().to_lowercase();

    for remote in remote_tracks {
        let remote_title = remote.title.to_lowercase();
        let remote_artist = remote.artist.to_lowercase();
        
        let title_sim = strsim::jaro_winkler(&local_title, &remote_title);
        let artist_sim = strsim::jaro_winkler(&local_artist, &remote_artist);
        
        if title_sim > 0.85 && artist_sim > 0.85 {
            matches.push(remote);
        }
    }
    matches
}

#[tauri::command]
async fn scan_local_directory(state: State<'_, AppState>, path: String) -> Result<usize, String> {
    let path_clone = path.clone();
    
    let tracks = tokio::task::spawn_blocking(move || {
        let mut found_tracks = Vec::new();
        let cleaner = regex::Regex::new(r"(?i)\s*(?:\[[^\]]*\]|\([^\)]*\))").unwrap();
        
        for entry in WalkDir::new(path_clone).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if ext == "mp3" || ext == "flac" || ext == "wav" || ext == "m4a" || ext == "ogg" {
                        let mut title = entry_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        let mut artist = None;
                        let mut album = None;
                        let mut track_number = None;
                        if let Ok(file) = std::fs::File::open(entry_path) {
                            let mss = MediaSourceStream::new(Box::new(file), Default::default());
                            let mut hint = Hint::new();
                            hint.with_extension(&ext);

                            if let Ok(mut probed) = symphonia::default::get_probe().format(
                                &hint,
                                mss,
                                &FormatOptions::default(),
                                &MetadataOptions::default(),
                            ) {
                                // Try format metadata first, then global metadata
                                let mut meta_opt = probed.format.metadata().current().cloned();
                                if meta_opt.is_none() {
                                    meta_opt = probed.metadata.get().as_ref().and_then(|m| m.current()).cloned();
                                }
                                
                                if let Some(metadata) = meta_opt {
                                    for tag in metadata.tags() {
                                        match tag.std_key {
                                            Some(StandardTagKey::TrackTitle) => {
                                                let t = cleaner.replace_all(&tag.value.to_string(), "").trim().to_string();
                                                title = if t.is_empty() { tag.value.to_string() } else { t };
                                            },
                                            Some(StandardTagKey::Artist) => artist = Some(tag.value.to_string()),
                                            Some(StandardTagKey::Album) => {
                                                let a = cleaner.replace_all(&tag.value.to_string(), "").trim().to_string();
                                                album = Some(if a.is_empty() { tag.value.to_string() } else { a });
                                            },
                                            Some(StandardTagKey::TrackNumber) => {
                                                let val = tag.value.to_string();
                                                if let Ok(num) = val.parse::<i64>() {
                                                    track_number = Some(num);
                                                } else if let Some(num_str) = val.split('/').next() {
                                                    if let Ok(num) = num_str.parse::<i64>() {
                                                        track_number = Some(num);
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        
                        found_tracks.push(TrackData {
                            title, artist, album, track_number, file_path: entry_path.to_string_lossy().to_string()
                        });
                    }
                }
            }
        }
        found_tracks
    }).await.map_err(|e| e.to_string())?;

    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::InsertTracks { tracks, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_local_tracks(state: State<'_, AppState>, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetLocalTracks { limit, offset, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}


#[tauri::command]
async fn get_recent_albums(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<crate::Album>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetRecentAlbums { limit, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn resolve_track(
    state: State<'_, AppState>,
    provider_id: String,
    track_id: String,
) -> Result<crate::providers::ResolvedTrack, String> {
    state.provider_manager.resolve(&provider_id, &track_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_albums(state: State<'_, AppState>, limit: u32, offset: u32) -> Result<Vec<Album>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetAlbums { limit, offset, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn toggle_save_album(
    id: String,
    title: String,
    artist: Option<String>,
    cover_art_url: Option<String>,
    provider_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let (tx, rx) = oneshot::channel();
    let p_id = provider_id.unwrap_or_else(|| "local".to_string());
    state.db_tx.send(crate::db::DbRequest::ToggleSavedAlbum {
        id,
        title,
        artist,
        cover_art_url,
        provider_id: p_id,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_saved_albums(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::queries::SavedAlbum>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetSavedAlbums { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn toggle_save_playlist(
    id: String,
    title: String,
    author: Option<String>,
    cover_art_url: Option<String>,
    provider_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let (tx, rx) = oneshot::channel();
    let p_id = provider_id.unwrap_or_else(|| "youtube-wasm".to_string());
    state.db_tx.send(crate::db::DbRequest::ToggleSavedPlaylist {
        id,
        title,
        author,
        cover_art_url,
        provider_id: p_id,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_saved_playlists(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::queries::SavedPlaylist>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetSavedPlaylists { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_artist_metadata(
    state: State<'_, AppState>,
    id: String,
    name: String,
    avatar_url: Option<String>,
    bio: Option<String>,
    provider_id: Option<String>,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let p_id = provider_id.unwrap_or_else(|| "youtube-wasm".to_string());
    state.db_tx.send(crate::db::DbRequest::UpsertArtistMetadata {
        id,
        name,
        avatar_url,
        bio,
        provider_id: p_id,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_artist_metadata(
    state: State<'_, AppState>,
    query: String,
) -> Result<Option<crate::db::queries::ArtistMetadata>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::GetArtistMetadata {
        query,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_album_tracks(state: State<'_, AppState>, album_id: i64, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetAlbumTracks { album_id, limit, offset, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_playlists(state: State<'_, AppState>, limit: u32, offset: u32) -> Result<Vec<Playlist>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetPlaylists { limit, offset, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_queue_as_playlist(
    state: State<'_, AppState>,
    name: String,
    tracks: Vec<crate::queue::QueueTrack>,
) -> Result<i64, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::SaveQueueAsPlaylist { name, tracks, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn create_playlist(state: State<'_, AppState>, name: String) -> Result<i64, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::CreatePlaylist { name, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn add_to_playlist(state: State<'_, AppState>, playlist_id: i64, track_id: i64) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::AddToPlaylist { playlist_id, track_id, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_playlist_tracks(state: State<'_, AppState>, playlist_id: i64, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetPlaylistTracks { playlist_id, limit, offset, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn remove_from_playlist(state: State<'_, AppState>, playlist_id: i64, track_id: i64) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::RemoveFromPlaylist { playlist_id, track_id, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_playlist(state: State<'_, AppState>, playlist_id: i64) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::DeletePlaylist { playlist_id, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn rename_playlist(state: State<'_, AppState>, playlist_id: i64, new_name: String) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::RenamePlaylist { playlist_id, new_name, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn reorder_playlist_track(state: State<'_, AppState>, playlist_id: i64, from_pos: i64, to_pos: i64) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::ReorderPlaylistTrack { playlist_id, from_pos, to_pos, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn clear_local_library(state: State<'_, AppState>) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::ClearLocalLibrary { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetSetting { key, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_all_settings(state: State<'_, AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::GetAllSettings { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::SetSetting { key, value, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn remove_track_by_path(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::RemoveTrackByPath { path, resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn factory_reset(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    state.db_tx.send(DbRequest::FactoryReset { resp: tx }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())??;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let providers_dir = app_data_dir.join("providers");
    let artwork_dir = app_data_dir.join("artwork");
    
    let _ = std::fs::remove_dir_all(&providers_dir);
    let _ = std::fs::remove_dir_all(&artwork_dir);

    if let Ok(tx) = state.audio_tx.lock() {
        let _ = tx.send(AudioCommand::Stop);
    }
    
    // Also clear the queue state explicitly
    if let Ok(mut q) = state.queue.lock() {
        let _ = q.clear();
    }

    Ok(())
}

#[tauri::command]
async fn extract_and_cache_artwork(app_handle: AppHandle, track_id: i64, file_path: String) -> Result<Option<String>, String> {
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let artwork_dir = app_data_dir.join("artwork");
    std::fs::create_dir_all(&artwork_dir).map_err(|e| e.to_string())?;

    let dest_path = artwork_dir.join(format!("{}.jpg", track_id));
    if dest_path.exists() {
        return Ok(Some(dest_path.to_string_lossy().to_string()));
    }

    let result = tokio::task::spawn_blocking(move || -> Result<Option<String>, String> {
        if let Ok(file) = std::fs::File::open(&file_path) {
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let mut hint = Hint::new();
            if let Some(ext) = std::path::Path::new(&file_path).extension().and_then(|e| e.to_str()) {
                hint.with_extension(ext);
            }

            if let Ok(mut probed) = symphonia::default::get_probe().format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            ) {
                let mut visual_data: Option<Vec<u8>> = None;
                
                if let Some(meta) = probed.format.metadata().current() {
                    if let Some(v) = meta.visuals().first() {
                        visual_data = Some(v.data.to_vec());
                    }
                }
                
                if visual_data.is_none() {
                    if let Some(global) = probed.metadata.get().as_ref() {
                        if let Some(meta) = global.current() {
                            if let Some(v) = meta.visuals().first() {
                                visual_data = Some(v.data.to_vec());
                            }
                        }
                    }
                }
                
                if let Some(data) = visual_data {
                    if std::fs::write(&dest_path, &data).is_ok() {
                        return Ok(Some(dest_path.to_string_lossy().to_string()));
                    }
                }
            }
        }
        Ok(None)
    }).await.map_err(|e| e.to_string())?;

    result
}

#[tauri::command]
fn validate_queue_reorder(from_index: u32, to_index: u32, queue_length: u32) -> Result<(), String> {
    if from_index >= queue_length {
        return Err(format!("Invalid source index: {}", from_index));
    }
    if to_index >= queue_length {
        return Err(format!("Invalid target index: {}", to_index));
    }
    if from_index == to_index {
        return Err("Source and target indices are the same".to_string());
    }
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// TELEMETRY & DIAGNOSTICS COMMANDS
// ════════════════════════════════════════════════════════════════════════════════

#[tauri::command]
fn get_error_log() -> Vec<telemetry::ErrorEvent> {
    telemetry::get_error_log()
}

#[tauri::command]
fn get_error_count() -> u64 {
    telemetry::error_count()
}

#[tauri::command]
fn clear_error_log() {
    telemetry::clear_error_log();
    tracing::info!("Error log cleared");
}

// ════════════════════════════════════════════════════════════════════════════════
// FEATURE FLAG COMMANDS
// ════════════════════════════════════════════════════════════════════════════════

#[tauri::command]
fn get_feature_flags() -> Vec<String> {
    feature_flags::FEATURE_FLAGS
        .get_enabled_flags()
        .iter()
        .map(|f| f.key().to_string())
        .collect()
}

#[tauri::command]
fn get_feature_flags_detail() -> Vec<feature_flags::FlagDetail> {
    feature_flags::FEATURE_FLAGS.get_all_details()
}

#[tauri::command]
fn is_feature_enabled(flag: String) -> bool {
    if let Some(f) = feature_flags::FeatureFlag::from_key(&flag) {
        feature_flags::FEATURE_FLAGS.is_enabled(f)
    } else {
        false
    }
}

#[tauri::command]
async fn set_feature_flag(
    state: State<'_, AppState>,
    flag: String,
    enabled: bool,
) -> Result<(), String> {
    let feature = feature_flags::FeatureFlag::from_key(&flag)
        .ok_or_else(|| format!("Unknown feature flag: {}", flag))?;

    feature_flags::FEATURE_FLAGS.set_flag(feature, enabled);

    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::SetFeatureFlag {
        key: feature.key().to_string(),
        enabled,
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn reset_feature_flags(
    state: State<'_, AppState>,
) -> Result<(), String> {
    feature_flags::FEATURE_FLAGS.reset_defaults();

    let (tx, rx) = oneshot::channel();
    state.db_tx.send(crate::db::DbRequest::ResetFeatureFlags {
        resp: tx,
    }).map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_feature_enabled(flag: String, enabled: bool) {
    if let Some(f) = feature_flags::FeatureFlag::from_key(&flag) {
        feature_flags::FEATURE_FLAGS.set_flag(f, enabled);
    }
}

#[tauri::command]
async fn sync_playback_state(state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(tx) = state.audio_tx.lock() {
        let _ = tx.send(AudioCommand::SyncState);
    }
    Ok(())
}


#[tauri::command]
async fn sandbox_callback(
    req_id: String,
    payload: Option<serde_json::Value>,
    error: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut pending = state.sandbox_manager.pending_requests.lock().await;
    if let Some(tx) = pending.remove(&req_id) {
        let res = if let Some(e) = error {
            Err(e)
        } else {
            Ok(payload.unwrap_or(serde_json::Value::Null))
        };
        let _ = tx.send(res);
    } else {
        tracing::warn!("sandbox_callback: Unknown req_id {}", req_id);
    }
    Ok(())
}

#[tauri::command]
fn open_in_file_explorer(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }
    
    // Convert to canonical absolute path to prevent traversal/symlink tricks
    let canonical_path = p.canonicalize().map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(canonical_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(canonical_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        // On Linux, xdg-open on a file opens the file (potentially executing it).
        // We must safely extract the parent directory to just open the folder.
        let target = if canonical_path.is_file() {
            canonical_path.parent().unwrap_or(&canonical_path).to_path_buf()
        } else {
            canonical_path
        };
        
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn clear_recommendations_cache(state: State<'_, AppState>) -> Result<(), String> {
    state.recommendation_compiler.clear_all_cache();
    state.provider_manager.clear_all_plugin_cache();
    tracing::info!("Cleared recommendation cache and reloaded providers");
    Ok(())
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Lyria starting up");

    let (audio_tx, audio_rx) = mpsc::channel();
    let (db_tx, db_rx) = mpsc::channel();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            logger::init_logging(&handle);
            
            // Setup shared reqwest client with strict redirect/SSRF policy and timeouts
            let redirect_policy = reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 10 {
                    return attempt.stop();
                }
                if crate::providers::check_url_allowed(attempt.url().as_str()).is_err() {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            });
            let reqwest_client = reqwest::Client::builder()
                .redirect(redirect_policy)
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build reqwest client");

            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let db_dir = app_data_dir.join("database");
            let providers_dir = app_data_dir.join("providers");
            
            std::fs::create_dir_all(&db_dir).expect("Failed to create database directory");
            std::fs::create_dir_all(&providers_dir).expect("Failed to create providers directory");
            
            let db_path = db_dir.join("echo_library.db");
            let conn = db::schema::init_db(&db_path).expect("Failed to initialize SQLite");

            // Recover queue state on startup (Phase 5)
            let recovered_queue = queue::recovery::recover_on_startup(&conn)
                .unwrap_or_else(|e| {
                    tracing::warn!("Queue recovery failed: {}, starting fresh", e);
                    telemetry::record_error("queue_recovery", &e);
                    QueueState::new()
                });

            tracing::info!("Queue initialized with {} tracks", recovered_queue.tracks.len());

            // Initialize Feature Flags from persistent SQLite table
            feature_flags::FEATURE_FLAGS.init_from_db(&conn);

            let db_thread_handle = db::start_db_thread(conn, db_rx);
            
            #[cfg(any(debug_assertions, feature = "sync-workspace-extensions"))]
            {
                // Copy / update extension bundles into providers_dir (Disabled by default)
                let mut source_dirs = Vec::new();
                if let Ok(resource_dir) = app.path().resource_dir() {
                    source_dirs.push(resource_dir.join("providers"));
                    source_dirs.push(resource_dir.join("extensions"));
                }
                // Dev workspace fallback
                source_dirs.push(std::path::PathBuf::from("../extensions"));
                source_dirs.push(std::path::PathBuf::from("extensions"));

                for src in source_dirs {
                    if src.exists() && src.is_dir() {
                        if let Ok(entries) = std::fs::read_dir(&src) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() {
                                    if let Some(name) = p.file_name() {
                                        let dest = providers_dir.join(name);
                                        let should_copy = if !dest.exists() {
                                            true
                                        } else if let (Ok(meta_src), Ok(meta_dest)) = (p.metadata(), dest.metadata()) {
                                            meta_src.len() != meta_dest.len() || meta_src.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH) > meta_dest.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                        } else {
                                            false
                                        };
                                        if should_copy {
                                            let _ = std::fs::copy(&p, &dest);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let provider_manager = Arc::new(ProviderManager::new(handle.clone(), reqwest_client.clone(), db_tx.clone()));
            let recommendation_compiler = Arc::new(providers::recommendations::RecommendationCompiler::new(provider_manager.clone(), db_path.clone()));
            let in_flight_recommendation_cancel = Arc::new(std::sync::Mutex::new(None));
            let audio_thread_handle = audio::start_audio_thread(audio_rx, handle.clone(), reqwest_client.clone(), tauri::async_runtime::handle(), db_tx.clone());

            app.manage(AppState {
                audio_tx: std::sync::Mutex::new(audio_tx),
                db_tx,
                provider_manager,
                recommendation_compiler,
                in_flight_recommendation_cancel,
                audio_thread: std::sync::Mutex::new(Some(audio_thread_handle)),
                db_thread: std::sync::Mutex::new(Some(db_thread_handle)),
                queue: std::sync::Mutex::new(recovered_queue),
                reqwest_client,
                sandbox_manager: sandbox::sandbox_vm::SandboxManager::new(),
            });

            // Automatic background startup provider discovery, sync, and warmup
            let handle_for_sync = handle.clone();
            tauri::async_runtime::spawn(async move {
                use tauri::Manager;
                let state = handle_for_sync.state::<AppState>();
                if let Err(e) = sync_providers(handle_for_sync.clone(), state.clone()).await {
                    tracing::warn!("Background startup sync_providers failed: {}", e);
                } else {
                    tracing::info!("Background startup sync_providers completed successfully");
                }
                
                warmup_all_eligible_providers(&state).await;
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::commands::load_audio,
            audio::commands::queue_next_audio,
            audio::commands::play_audio,
            audio::commands::pause_audio,
            audio::commands::stop_audio,
            audio::commands::seek_audio,
            audio::commands::set_volume,
            audio::commands::set_mute,
            get_providers,
            toggle_provider,
            delete_provider,
            save_provider_settings,
            sync_providers,
            get_provider_config,
            set_provider_config,
            search_provider,
            search_provider_categorized,
            browse_provider_album,
            browse_provider_artist,
            get_provider_modules,
            fetch_provider_module,
            get_explore_feed,
            resolve_stream_url,
            get_home_feed,
            get_home_local_shelves,
            get_home_remote_shelves,
            get_home_radios,
            get_home_adjacent_horizons,
            get_home_adjacent_horizon,
            get_radio_stream,
            resolve_autoplay_next_track,
            clear_recommendations_cache,
            record_track_play,
            toggle_track_like,
            get_liked_songs,
            get_extension_metrics,
            search_library,
            fuzzy_match_tracks,
            scan_local_directory,
            get_local_tracks,
            get_albums,
            get_recent_albums,
            toggle_save_album,
            get_saved_albums,
            toggle_save_playlist,
            get_saved_playlists,
            save_artist_metadata,
            get_artist_metadata,
            resolve_track,
            get_album_tracks,
            get_playlists,
            save_queue_as_playlist,
            create_playlist,
            add_to_playlist,
            get_playlist_tracks,
            remove_from_playlist,
            delete_playlist,
            rename_playlist,
            reorder_playlist_track,
            validate_queue_reorder,
            extract_and_cache_artwork,
            clear_local_library,
            get_setting,
            get_all_settings,
            set_setting,
            factory_reset,
            remove_track_by_path,
            // Queue commands (Phase 2)
            queue::commands::set_queue,
            queue::commands::add_to_queue,
            queue::commands::clear_queue,
            queue::commands::skip_forward,
            queue::commands::skip_backward,
            queue::commands::jump_to_position,
            queue::commands::jump_to_track,
            queue::commands::reorder_queue,
            queue::commands::set_repeat_mode,
            queue::commands::set_shuffle,
            queue::commands::get_queue,
            queue::commands::get_queue_length,
            queue::commands::get_current_track,
            queue::commands::reshuffle,
            queue::commands::get_next_track,
            // Telemetry commands (Phase 7)
            get_error_log,
            get_error_count,
            clear_error_log,
            // Feature flag commands (Phase 7)
            get_feature_flags,
            get_feature_flags_detail,
            is_feature_enabled,
            set_feature_enabled,
            set_feature_flag,
            reset_feature_flags,
            toggle_provider,
            delete_provider,
            sync_playback_state,
            open_in_file_explorer,
            sandbox_callback,
            // Debug logger commands (Phase 1)
            logger::open_debug_window,
            logger::get_debug_logs,
            logger::clear_debug_logs,
            logger::copy_debug_log_to_clipboard,
            logger::open_log_directory,
            logger::sandbox_log,
            logger::set_log_collection_enabled,
            logger::get_log_collection_enabled,
        ]);

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                let state: State<'_, AppState> = app.state();
                if let Ok(tx) = state.audio_tx.lock() {
                    let _ = tx.send(AudioCommand::Quit);
                }
                let _ = state.db_tx.send(DbRequest::Quit);
                
                if let Ok(mut lock) = state.audio_thread.lock() {
                    if let Some(handle) = lock.take() {
                        let _ = handle.join();
                    }
                };
                
                if let Ok(mut lock) = state.db_thread.lock() {
                    if let Some(handle) = lock.take() {
                        let _ = handle.join();
                    }
                };
            }
        });
}
