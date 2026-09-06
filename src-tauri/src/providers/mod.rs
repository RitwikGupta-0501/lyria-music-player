use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use extism::{Manifest, Plugin, Wasm};
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
use std::sync::mpsc::Sender;
use crate::db::DbRequest;

pub mod errors;
pub mod secrets;
pub mod wasm_bridge;
pub mod recommendations;
#[cfg(test)]
pub mod tests;

use errors::SandboxError;

// ── Constants ────────────────────────────────────────────────────────────────
const SEARCH_TIMEOUT_SECS: u64 = 15;
const MODULE_FETCH_TIMEOUT_SECS: u64 = 15;

pub const PROVIDER_ABI_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalSeedV1 {
    pub abi_version: u32,
    pub canonical_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub isrc: Option<String>,
    pub duration_ms: Option<u64>,
    pub native_id: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RadioStreamResultV1 {
    pub tracks: Vec<TrackResult>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackTelemetryEventV1 {
    pub native_track_id: String,
    pub duration_ms: u64,
    pub total_track_duration_ms: u64,
    pub completed: bool,
}

// ── Public types ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderModule {
    pub id: String,
    pub name: String,
    pub layout: ModuleLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModuleLayout {
    #[default]
    Grid,
    List,
    Carousel,
    #[serde(rename = "chart4row")]
    Chart4Row,
    #[serde(rename = "genrecloud")]
    GenreCloud,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlbumItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: Option<String>,
    pub cover_art_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub item_count: Option<u32>,
    pub cover_art_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenreItem {
    pub id: String,
    pub title: String,
    pub endpoint_params: Option<String>,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtistItem {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub subscribers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopResultItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub item_type: String, // "artist", "album", "song"
    pub cover_art_url: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum SearchItem {
    TopResult(TopResultItem),
    Track(TrackResult),
    Album(AlbumItem),
    Artist(ArtistItem),
    Playlist(PlaylistItem),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchCategorySection {
    pub category: String, // "Top Result", "Songs", "Albums", "Artists", "Playlists"
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlbumDetailResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: Option<String>,
    pub description: Option<String>,
    pub cover_art_url: Option<String>,
    pub track_count: Option<u32>,
    pub tracks: Vec<TrackResult>,
    #[serde(default)]
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtistDetailResult {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub subscribers: Option<String>,
    pub bio: Option<String>,
    pub top_tracks: Vec<TrackResult>,
    pub albums: Vec<AlbumItem>,
    pub singles: Vec<AlbumItem>,
    #[serde(default)]
    pub videos: Vec<TrackResult>,
    #[serde(default)]
    pub featured_on: Vec<PlaylistItem>,
    #[serde(default)]
    pub similar_artists: Vec<ArtistItem>,
    #[serde(default)]
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CategorizedSearchResult {
    pub sections: Vec<SearchCategorySection>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchQueryInput {
    pub query: String,
    pub filter: Option<String>, // "all", "songs", "albums", "artists", "playlists"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorialSpotlight {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_art_url: Option<String>,
    pub description: Option<String>,
    pub release_year: Option<String>,
    pub track_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryShelf {
    pub title: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ModuleItem {
    Track(TrackResult),
    Album(AlbumItem),
    Playlist(PlaylistItem),
    Genre(GenreItem),
    Artist(ArtistItem),
    Spotlight(EditorialSpotlight),
    Shelf(CategoryShelf),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleData {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AggregatedModule {
    pub provider_id: String,
    pub provider_name: String,
    pub module: ProviderModule,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub cover_art_url: Option<String>,
    pub stream_url: Option<String>,
    pub quality_hint: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub isrc: Option<String>,
    #[serde(default)]
    pub plays: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedTrack {
    pub stream_url: String,
    pub quality_hint: Option<String>,
    pub duration_ms: Option<u64>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

pub struct ActiveProvider {
    pub id: String,
    pub name: String,
    pub script_path: std::path::PathBuf,
    pub config: std::collections::HashMap<String, String>,
    pub capabilities: Vec<String>,
}

pub struct ProviderManager {
    providers: std::sync::RwLock<std::collections::HashMap<String, ActiveProvider>>,
    app_handle: tauri::AppHandle,
    reqwest_client: reqwest::Client,
    db_tx: Sender<DbRequest>,
    pub search_semaphore: Arc<Semaphore>,
    pub explore_semaphore: Arc<Semaphore>,
    plugin_cache: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<std::sync::Mutex<Plugin>>>>>,
    search_epoch: Arc<AtomicU64>,
    active_search_canceller: Arc<std::sync::Mutex<Option<extism::CancelHandle>>>,
}

impl ProviderManager {
    pub fn new(app_handle: tauri::AppHandle, reqwest_client: reqwest::Client, db_tx: Sender<DbRequest>) -> Self {
        Self {
            providers: std::sync::RwLock::new(std::collections::HashMap::new()),
            app_handle,
            reqwest_client,
            db_tx,
            search_semaphore: Arc::new(Semaphore::new(4)),
            explore_semaphore: Arc::new(Semaphore::new(4)),
            plugin_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            search_epoch: Arc::new(AtomicU64::new(0)),
            active_search_canceller: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn sync_registry(&self, providers_info: Vec<crate::ProviderInfo>) {
        let mut new_registry = std::collections::HashMap::new();
        for info in providers_info {
            if info.status != "enabled" { continue; }
            let config = if let Some(settings) = info.settings {
                serde_json::from_str(&settings).unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };
            new_registry.insert(info.id.clone(), ActiveProvider {
                id: info.id,
                name: info.name,
                script_path: std::path::PathBuf::from(info.file_path),
                config,
                capabilities: info.capabilities.unwrap_or_default(),
            });
        }
        *self.providers.write().unwrap() = new_registry;
        // Clear cache so updated plugins reload
        self.plugin_cache.lock().unwrap().clear();
        let count = self.providers.read().unwrap().len();
        tracing::info!("Provider registry synced with {} enabled providers", count);
    }

    fn get_or_create_plugin(&self, provider_id: &str, timeout_secs: u64) -> Result<Arc<std::sync::Mutex<Plugin>>, SandboxError> {
        let mut cache = self.plugin_cache.lock().unwrap();
        if let Some(plugin) = cache.get(provider_id) {
            return Ok(plugin.clone());
        }

        let providers_guard = self.providers.read().unwrap();
        let provider = providers_guard.get(provider_id).ok_or_else(|| SandboxError::ScriptError {
            script: provider_id.to_string(),
            message: "provider not found or disabled in registry".to_string(),
        })?;

        let wasm_file = provider.script_path.clone();
        
        if !wasm_file.exists() {
            return Err(SandboxError::ScriptError {
                script: provider.name.clone(),
                message: format!("Wasm file not found at: {:?}", wasm_file),
            });
        }

        let manifest = Manifest::new([Wasm::file(wasm_file)])
            .with_memory_max(2048)
            .with_timeout(Duration::from_secs(timeout_secs));

        let reqwest_client = Arc::new(self.reqwest_client.clone());
        let db_tx = Arc::new(self.db_tx.clone());

        let has_telemetry = provider.capabilities.iter().any(|c| c.eq_ignore_ascii_case("telemetry_reporting"));

        let mut builder = extism::PluginBuilder::new(manifest)
            .with_wasi(true)
            .with_function(
                "host_log",
                [extism::ValType::I64],
                [],
                extism::UserData::new(()),
                wasm_bridge::host_log,
            )
            .with_function(
                "host_http_request",
                [extism::ValType::I64],
                [extism::ValType::I64],
                extism::UserData::new(reqwest_client.clone()),
                wasm_bridge::host_http_request,
            )
            .with_function(
                "host_storage_get",
                [extism::ValType::I64],
                [extism::ValType::I64],
                extism::UserData::new(db_tx.clone()),
                wasm_bridge::host_storage_get,
            )
            .with_function(
                "host_storage_set",
                [extism::ValType::I64],
                [],
                extism::UserData::new(db_tx),
                wasm_bridge::host_storage_set,
            )
            .with_function(
                "host_execute_webview_js",
                [extism::ValType::I64],
                [extism::ValType::I64],
                extism::UserData::new(Arc::new(self.app_handle.clone())),
                wasm_bridge::host_execute_webview_js,
            );

        if has_telemetry {
            builder = builder.with_function(
                "host_telemetry_request",
                [extism::ValType::I64],
                [extism::ValType::I64],
                extism::UserData::new(reqwest_client),
                wasm_bridge::host_telemetry_request,
            );
        } else {
            builder = builder.with_function(
                "host_telemetry_request",
                [extism::ValType::I64],
                [extism::ValType::I64],
                extism::UserData::new(()),
                wasm_bridge::host_telemetry_blocked,
            );
        }

        let plugin = builder.build()
            .map_err(|e| SandboxError::ScriptError {
                script: provider.name.clone(),
                message: e.to_string(),
            })?;
            
        let plugin_arc = Arc::new(std::sync::Mutex::new(plugin));
        cache.insert(provider_id.to_string(), plugin_arc.clone());
        Ok(plugin_arc)
    }

    pub async fn warmup_provider(&self, provider_id: &str) -> Result<(), SandboxError> {
        let plugin = self.get_or_create_plugin(provider_id, 30)?;
        let provider_name = self.providers.read().unwrap().get(provider_id).map(|p| p.name.clone()).unwrap_or_else(|| provider_id.to_string());
        
        spawn_blocking(move || {
            let mut plugin_guard = plugin.lock().unwrap();
            match plugin_guard.call::<&str, &str>("warmup", "{}") {
                Ok(res) => {
                    tracing::info!("Provider {} warmed up successfully: {}", provider_name, res);
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!("Provider {} warmup warning/skipped: {}", provider_name, e);
                    Err(SandboxError::ScriptError {
                        script: provider_name,
                        message: e.to_string(),
                    })
                }
            }
        })
        .await
        .map_err(|_| SandboxError::ExecutionTimeout {
            script: provider_id.to_string(),
            timeout_secs: 30,
        })?
    }

    pub fn get_warmup_eligible_providers(&self) -> Vec<String> {
        self.providers
            .read()
            .unwrap()
            .iter()
            .filter(|(_, p)| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case("warmup")))
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn remove_provider(&self, provider_id: &str) {
        self.providers.write().unwrap().remove(provider_id);
        self.invalidate_plugin_cache(provider_id);
    }

    pub fn invalidate_plugin_cache(&self, provider_id: &str) {
        let mut cache = self.plugin_cache.lock().unwrap();
        if cache.remove(provider_id).is_some() {
            tracing::warn!("Evicted failed plugin instance from cache for provider: {}", provider_id);
        }
    }

    pub fn clear_all_plugin_cache(&self) {
        let mut cache = self.plugin_cache.lock().unwrap();
        cache.clear();
        tracing::info!("Evicted all active plugin instances from cache");
    }

    pub async fn search_categorized(&self, provider_id: &str, input: &SearchQueryInput) -> Result<CategorizedSearchResult, SandboxError> {
        let provider_id = provider_id.to_string();
        let input_clone = input.clone();
        
        let my_epoch = self.search_epoch.fetch_add(1, Ordering::SeqCst) + 1;
        tracing::info!("Categorized search [epoch {}] for query '{}' (filter: {:?}) in {}", my_epoch, input.query, input.filter, provider_id);

        // Cancel any currently in-flight WASM execution from an older search epoch
        {
            let mut canceller = self.active_search_canceller.lock().unwrap();
            if let Some(handle) = canceller.take() {
                let _ = handle.cancel();
            }
        }
        
        let _permit = self.search_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Semaphore closed".into(),
        })?;

        // Fast-path bailout if superseded while acquiring semaphore
        if self.search_epoch.load(Ordering::Relaxed) > my_epoch {
            tracing::debug!("Search [epoch {}] superseded before semaphore acquisition, cancelling in 0ms", my_epoch);
            return Ok(CategorizedSearchResult::default());
        }

        let plugin_arc = self.get_or_create_plugin(&provider_id, SEARCH_TIMEOUT_SECS)?;
        let search_epoch = self.search_epoch.clone();
        let active_canceller = self.active_search_canceller.clone();
        
        let res_bytes = match spawn_blocking(move || {
            // 1. Pre-lock epoch check
            if search_epoch.load(Ordering::Relaxed) > my_epoch {
                tracing::debug!("Search [epoch {}] superseded before lock, cancelling in 0ms", my_epoch);
                return Ok(Vec::new());
            }

            let mut plugin = plugin_arc.lock().unwrap();

            // 2. Post-lock epoch check (after waiting for prior queued searches)
            if search_epoch.load(Ordering::Relaxed) > my_epoch {
                tracing::debug!("Search [epoch {}] superseded after acquiring lock, cancelling in 0ms", my_epoch);
                return Ok(Vec::new());
            }

            // 3. Register cancel handle for the active execution
            {
                let cancel_handle = plugin.cancel_handle();
                let mut guard = active_canceller.lock().unwrap();
                *guard = Some(cancel_handle);
            }

            let json_input = serde_json::to_vec(&input_clone).unwrap_or_default();
            let call_res = plugin.call::<&[u8], &[u8]>("search_categorized", &json_input).map(|res| res.to_vec());

            // Clear cancel handle
            {
                let mut guard = active_canceller.lock().unwrap();
                *guard = None;
            }

            call_res
        }).await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                // If cancelled by another thread, exit cleanly
                if self.search_epoch.load(Ordering::Relaxed) > my_epoch {
                    tracing::debug!("Search [epoch {}] cancelled mid-execution, bailing out cleanly", my_epoch);
                    return Ok(CategorizedSearchResult::default());
                }
                tracing::error!("search_categorized plugin call failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                tracing::error!("search_categorized spawn_blocking failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        if res_bytes.is_empty() {
            return Ok(CategorizedSearchResult::default());
        }

        let result: CategorizedSearchResult = serde_json::from_slice(&res_bytes).map_err(|e| {
            tracing::error!("Failed to parse categorized search results from provider '{}': {}", provider_id, e);
            SandboxError::ScriptError {
                script: provider_id,
                message: format!("Invalid categorized search JSON: {}", e),
            }
        })?;

        Ok(result)
    }

    pub async fn search(&self, provider_id: &str, query: &str) -> Result<Vec<TrackResult>, SandboxError> {
        let provider_id = provider_id.to_string();
        let query = query.to_string();
        
        tracing::info!("Searched query '{}' in {}", query, provider_id);
        
        let _permit = self.search_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, SEARCH_TIMEOUT_SECS)?;
        
        let query_for_search = query.clone();
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&query_for_search).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("search", &json_input).map(|res| res.to_vec())
        }).await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                tracing::error!("Search plugin call failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                tracing::error!("Search spawn_blocking failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        let mut results: Vec<TrackResult> = serde_json::from_slice(&res_bytes).unwrap_or_default();
        
        // Fallback: If flat search returned 0 items, use search_categorized to extract items (including Top Result hero card)
        if results.is_empty() {
            if let Ok(categorized) = self.search_categorized(&provider_id, &SearchQueryInput {
                query: query.clone(),
                filter: None,
            }).await {
                for section in categorized.sections {
                    for item in section.items {
                        match item {
                            SearchItem::TopResult(top) => {
                                if top.item_type == "song" || top.item_type == "video" || top.item_type.is_empty() {
                                    results.push(TrackResult {
                                        id: top.id,
                                        title: top.title,
                                        artist: top.subtitle,
                                        album: None,
                                        cover_art_url: top.cover_art_url,
                                        stream_url: None,
                                        quality_hint: None,
                                        duration_ms: None,
                                        isrc: None,
                                        plays: None,
                                    });
                                }
                            }
                            SearchItem::Track(t) => {
                                results.push(t);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn resolve(&self, provider_id: &str, track_id: &str) -> Result<ResolvedTrack, SandboxError> {
        let provider_id = provider_id.to_string();
        let track_id = track_id.to_string();
        
        let _permit = self.search_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, SEARCH_TIMEOUT_SECS)?;
        
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&track_id).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("resolve", &json_input).map(|res| res.to_vec())
        }).await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                tracing::error!("Resolve plugin call failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                tracing::error!("Resolve spawn_blocking failed for provider '{}': {}", provider_id, e);
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        let results: ResolvedTrack = serde_json::from_slice(&res_bytes).map_err(|e| {
            tracing::error!("Failed to parse resolve results from provider '{}': {}", provider_id, e);
            self.invalidate_plugin_cache(&provider_id);
            SandboxError::ScriptError {
                script: provider_id,
                message: e.to_string(),
            }
        })?;
        
        Ok(results)
    }

    pub async fn get_all_explore_modules(&self) -> Vec<AggregatedModule> {
        let explore_providers: Vec<(String, String)> = self.providers
            .read()
            .unwrap()
            .values()
            .filter(|p| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case("explore")))
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();

        let mut aggregated = Vec::new();
        for (pid, pname) in explore_providers {
            if let Ok(modules) = self.get_modules(&pid).await {
                for m in modules {
                    aggregated.push(AggregatedModule {
                        provider_id: pid.clone(),
                        provider_name: pname.clone(),
                        module: m,
                    });
                }
            }
        }
        aggregated
    }

    pub async fn get_modules(&self, provider_id: &str) -> Result<Vec<ProviderModule>, SandboxError> {
        let provider_id = provider_id.to_string();
        
        let _permit = self.explore_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, MODULE_FETCH_TIMEOUT_SECS)?;
        
        let res_bytes = spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            plugin.call::<&[u8], &[u8]>("get_modules", &[]).map(|res| res.to_vec())
        }).await.map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?.map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?;

        let results: Vec<ProviderModule> = serde_json::from_slice(&res_bytes).map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?;
        
        Ok(results)
    }

    pub async fn fetch_module(&self, provider_id: &str, module_id: &str) -> Result<ModuleData, SandboxError> {
        let provider_id = provider_id.to_string();
        let module_id = module_id.to_string();
        
        let _permit = self.explore_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, MODULE_FETCH_TIMEOUT_SECS)?;
        
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&module_id).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("fetch_module", &json_input).map(|res| res.to_vec())
        }).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        let mut results: ModuleData = serde_json::from_slice(&res_bytes).map_err(|e| {
            self.invalidate_plugin_cache(&provider_id);
            SandboxError::ScriptError {
                script: provider_id,
                message: e.to_string(),
            }
        })?;
        
        let mut seen = std::collections::HashSet::new();
        results.items.retain(|item| {
            match item {
                ModuleItem::Album(a) => {
                    !a.id.trim().is_empty() && !a.title.trim().is_empty() && !a.artist.trim().is_empty() && seen.insert(a.id.clone())
                }
                ModuleItem::Track(t) => {
                    !t.id.trim().is_empty() && !t.title.trim().is_empty() && seen.insert(t.id.clone())
                }
                ModuleItem::Spotlight(s) => {
                    !s.id.trim().is_empty() && !s.title.trim().is_empty() && seen.insert(s.id.clone())
                }
                ModuleItem::Genre(g) => {
                    !g.title.trim().is_empty() && seen.insert(format!("{}:{}", g.title.to_lowercase(), g.endpoint_params.as_deref().unwrap_or(&g.id)))
                }
                ModuleItem::Artist(a) => {
                    !a.id.trim().is_empty() && !a.name.trim().is_empty() && seen.insert(a.id.clone())
                }
                ModuleItem::Playlist(p) => {
                    !p.id.trim().is_empty() && !p.title.trim().is_empty() && seen.insert(p.id.clone())
                }
                ModuleItem::Shelf(s) => {
                    !s.title.trim().is_empty() && !s.items.is_empty()
                }
            }
        });

        Ok(results)
    }


    pub async fn browse_album(&self, provider_id: &str, album_id: &str) -> Result<AlbumDetailResult, SandboxError> {
        let plugin_arc = self.get_or_create_plugin(provider_id, 15)?;
        let album_id_str = album_id.to_string();
        let res_bytes = spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&album_id_str).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("browse_album", &json_input).map(|res| res.to_vec())
        }).await.map_err(|e| SandboxError::ScriptError { script: provider_id.to_string(), message: e.to_string() })?
        .map_err(|e| SandboxError::ScriptError { script: provider_id.to_string(), message: e.to_string() })?;

        let results: AlbumDetailResult = serde_json::from_slice(&res_bytes).map_err(|e| SandboxError::ScriptError {
            script: provider_id.to_string(),
            message: e.to_string(),
        })?;
        Ok(results)
    }

    pub async fn browse_artist(&self, provider_id: &str, artist_id: &str) -> Result<ArtistDetailResult, SandboxError> {
        let plugin_arc = self.get_or_create_plugin(provider_id, 15)?;
        let artist_id_str = artist_id.to_string();
        let res_bytes = spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&artist_id_str).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("browse_artist", &json_input).map(|res| res.to_vec())
        }).await.map_err(|e| SandboxError::ScriptError { script: provider_id.to_string(), message: e.to_string() })?
        .map_err(|e| SandboxError::ScriptError { script: provider_id.to_string(), message: e.to_string() })?;

        let results: ArtistDetailResult = serde_json::from_slice(&res_bytes).map_err(|e| SandboxError::ScriptError {
            script: provider_id.to_string(),
            message: e.to_string(),
        })?;
        Ok(results)
    }

    pub async fn resolve_url(&self, url: &str) -> Result<Option<(String, String, TrackResult)>, SandboxError> {
        let active_providers: Vec<(String, String)> = self.providers.read().unwrap().iter()
            .filter(|(_, p)| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case("url_resolver") || c.eq_ignore_ascii_case("search")))
            .map(|(id, p)| (id.clone(), p.name.clone()))
            .collect();

        for (provider_id, provider_name) in active_providers {
            let pid = provider_id.clone();
            let url_str = url.to_string();
            let plugin_arc = match self.get_or_create_plugin(&pid, 15) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let res_opt = spawn_blocking(move || {
                let mut plugin = plugin_arc.lock().unwrap();
                let json_input = serde_json::to_vec(&url_str).unwrap_or_default();
                plugin.call::<&[u8], &[u8]>("resolve_url", &json_input).map(|res| res.to_vec())
            }).await;

            if let Ok(Ok(bytes)) = res_opt {
                if let Ok(Some(track)) = serde_json::from_slice::<Option<TrackResult>>(&bytes) {
                    return Ok(Some((provider_id, provider_name, track)));
                }
            }
        }

        Ok(None)
    }


    pub fn has_capability(&self, provider_id: &str, cap: &str) -> bool {
        self.providers.read().unwrap().get(provider_id).is_some_and(|p| {
            p.capabilities.iter().any(|c| c.eq_ignore_ascii_case(cap))
        })
    }

    pub fn get_providers_with_capability(&self, cap: &str) -> Vec<(String, String)> {
        self.providers.read().unwrap().iter()
            .filter(|(_, p)| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case(cap)))
            .map(|(id, p)| (id.clone(), p.name.clone()))
            .collect()
    }

    pub async fn get_related(&self, provider_id: &str, seed: &CanonicalSeedV1) -> Result<Vec<TrackResult>, SandboxError> {
        let provider_id = provider_id.to_string();
        let seed_clone = seed.clone();
        
        let _permit = self.explore_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Explore semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, 4)?;
        
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&seed_clone).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("get_related", &json_input).map(|res| res.to_vec())
        }).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        let results: Vec<TrackResult> = serde_json::from_slice(&res_bytes).map_err(|e| {
            self.invalidate_plugin_cache(&provider_id);
            SandboxError::ScriptError {
                script: provider_id,
                message: format!("Invalid get_related JSON: {}", e),
            }
        })?;

        Ok(results)
    }

    pub async fn get_radio(&self, provider_id: &str, seed: &CanonicalSeedV1) -> Result<RadioStreamResultV1, SandboxError> {
        let provider_id = provider_id.to_string();
        let seed_clone = seed.clone();
        
        let _permit = self.explore_semaphore.acquire().await.map_err(|_| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: "Explore semaphore closed".into(),
        })?;

        let plugin_arc = self.get_or_create_plugin(&provider_id, 4)?;
        
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&seed_clone).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("get_radio", &json_input).map(|res| res.to_vec())
        }).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
            Err(e) => {
                self.invalidate_plugin_cache(&provider_id);
                return Err(SandboxError::ScriptError {
                    script: provider_id,
                    message: e.to_string(),
                });
            }
        };

        let results: RadioStreamResultV1 = serde_json::from_slice(&res_bytes).map_err(|e| {
            self.invalidate_plugin_cache(&provider_id);
            SandboxError::ScriptError {
                script: provider_id,
                message: format!("Invalid get_radio JSON: {}", e),
            }
        })?;

        Ok(results)
    }

    pub async fn notify_playback_completed(&self, provider_id: &str, event: &PlaybackTelemetryEventV1) -> Result<(), SandboxError> {
        if !self.has_capability(provider_id, "telemetry_reporting") {
            return Ok(());
        }
        let provider_id = provider_id.to_string();
        let event_clone = event.clone();

        let plugin_arc = match self.get_or_create_plugin(&provider_id, 4) {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };

        let _ = spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&event_clone).unwrap_or_default();
            let _ = plugin.call::<&[u8], &[u8]>("on_playback_event", &json_input);
        }).await;

        Ok(())
    }
}
pub fn check_url_allowed(url: &str) -> Result<(), String> {
    if url.starts_with("file://") {
        return Err("Local file URLs are not allowed from providers".into());
    }
    if url.contains("localhost") || url.contains("127.0.0.1") {
        return Err("Localhost URLs are not allowed from providers".into());
    }
    Ok(())
}


#[cfg(test)]
mod provider_unit_tests {
    use super::*;

    #[test]
    fn test_module_item_serialization_roundtrip() {
        let track_item = ModuleItem::Track(TrackResult {
            id: "track123".to_string(),
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: Some("Test Album".to_string()),
            cover_art_url: Some("https://example.com/art.jpg".to_string()),
            stream_url: None,
            quality_hint: Some("#1".to_string()),
            duration_ms: Some(180000),
            isrc: None,
            plays: None,
        });

        let json_track = serde_json::to_string(&track_item).unwrap();
        assert!(json_track.contains(r#""type":"Track""#));
        let decoded_track: ModuleItem = serde_json::from_str(&json_track).unwrap();
        assert_eq!(track_item, decoded_track);

        let album_item = ModuleItem::Album(AlbumItem {
            id: "MPREb_123".to_string(),
            title: "Hit Album".to_string(),
            artist: "Star Artist".to_string(),
            year: Some("2026".to_string()),
            cover_art_url: Some("https://example.com/album.jpg".to_string()),
        });

        let json_album = serde_json::to_string(&album_item).unwrap();
        assert!(json_album.contains(r#""type":"Album""#));
        let decoded_album: ModuleItem = serde_json::from_str(&json_album).unwrap();
        assert_eq!(album_item, decoded_album);

        let genre_item = ModuleItem::Genre(GenreItem {
            id: "genre_chill".to_string(),
            title: "Chill Vibes".to_string(),
            endpoint_params: Some("params_xyz".to_string()),
            color_hex: Some("#336699".to_string()),
        });

        let json_genre = serde_json::to_string(&genre_item).unwrap();
        assert!(json_genre.contains(r#""type":"Genre""#));
        let decoded_genre: ModuleItem = serde_json::from_str(&json_genre).unwrap();
        assert_eq!(genre_item, decoded_genre);
    }

    #[test]
    fn test_module_layout_serde() {
        let l1: ModuleLayout = serde_json::from_str(r#""chart4row""#).unwrap();
        assert_eq!(l1, ModuleLayout::Chart4Row);

        let l2: ModuleLayout = serde_json::from_str(r#""genrecloud""#).unwrap();
        assert_eq!(l2, ModuleLayout::GenreCloud);

        let l3: ModuleLayout = serde_json::from_str(r#""carousel""#).unwrap();
        assert_eq!(l3, ModuleLayout::Carousel);

        let l4: ModuleLayout = serde_json::from_str(r#""unknown_custom""#).unwrap();
        assert_eq!(l4, ModuleLayout::Unknown);
    }

    #[test]
    fn test_warmup_capability_filter() {
        let mut providers = std::collections::HashMap::new();
        providers.insert("yt".to_string(), ActiveProvider {
            id: "yt".to_string(),
            name: "YouTube".to_string(),
            script_path: std::path::PathBuf::from("/tmp/yt.wasm"),
            config: std::collections::HashMap::new(),
            capabilities: vec!["search".to_string(), "WARMUP".to_string(), "explore".to_string()],
        });
        providers.insert("local".to_string(), ActiveProvider {
            id: "local".to_string(),
            name: "Local".to_string(),
            script_path: std::path::PathBuf::from("/tmp/local.wasm"),
            config: std::collections::HashMap::new(),
            capabilities: vec!["search".to_string()],
        });

        let eligible: Vec<String> = providers.iter()
            .filter(|(_, p)| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case("warmup")))
            .map(|(id, _)| id.clone())
            .collect();

        assert_eq!(eligible, vec!["yt".to_string()]);
    }
}
