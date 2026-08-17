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

use errors::SandboxError;

// ── Constants ────────────────────────────────────────────────────────────────
const SEARCH_TIMEOUT_SECS: u64 = 15;
const MODULE_FETCH_TIMEOUT_SECS: u64 = 15;

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
#[serde(tag = "type", content = "data")]
pub enum ModuleItem {
    Track(TrackResult),
    Album(AlbumItem),
    Playlist(PlaylistItem),
    Genre(GenreItem),
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
    providers: std::collections::HashMap<String, ActiveProvider>,
    app_handle: tauri::AppHandle,
    reqwest_client: reqwest::Client,
    db_tx: Sender<DbRequest>,
    pub search_semaphore: Arc<Semaphore>,
    pub explore_semaphore: Arc<Semaphore>,
    plugin_cache: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<std::sync::Mutex<Plugin>>>>>,
}

impl ProviderManager {
    pub fn new(app_handle: tauri::AppHandle, reqwest_client: reqwest::Client, db_tx: Sender<DbRequest>) -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            app_handle,
            reqwest_client,
            db_tx,
            search_semaphore: Arc::new(Semaphore::new(4)),
            explore_semaphore: Arc::new(Semaphore::new(4)),
            plugin_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn sync_registry(&mut self, providers_info: Vec<crate::ProviderInfo>) {
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
        self.providers = new_registry;
        // Clear cache so updated plugins reload
        self.plugin_cache.lock().unwrap().clear();
        tracing::info!("Provider registry synced with {} enabled providers", self.providers.len());
    }

    fn get_or_create_plugin(&self, provider_id: &str, timeout_secs: u64) -> Result<Arc<std::sync::Mutex<Plugin>>, SandboxError> {
        let mut cache = self.plugin_cache.lock().unwrap();
        if let Some(plugin) = cache.get(provider_id) {
            return Ok(plugin.clone());
        }

        let provider = self.providers.get(provider_id).ok_or_else(|| SandboxError::ScriptError {
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
            .with_memory_max(400)
            .with_timeout(Duration::from_secs(timeout_secs));

        let reqwest_client = Arc::new(self.reqwest_client.clone());
        let db_tx = Arc::new(self.db_tx.clone());

        let plugin = extism::PluginBuilder::new(manifest)
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
                extism::UserData::new(reqwest_client),
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
            )
            .build()
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
        let provider_name = self.providers.get(provider_id).map(|p| p.name.clone()).unwrap_or_else(|| provider_id.to_string());
        
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
            .iter()
            .filter(|(_, p)| p.capabilities.iter().any(|c| c.eq_ignore_ascii_case("warmup")))
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn invalidate_plugin_cache(&self, provider_id: &str) {
        let mut cache = self.plugin_cache.lock().unwrap();
        if cache.remove(provider_id).is_some() {
            tracing::warn!("Evicted failed plugin instance from cache for provider: {}", provider_id);
        }
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
        
        let res_bytes = match spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&query).unwrap_or_default();
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

        let results: Vec<TrackResult> = serde_json::from_slice(&res_bytes).map_err(|e| {
            tracing::error!("Failed to parse search results from provider '{}': {}", provider_id, e);
            self.invalidate_plugin_cache(&provider_id);
            SandboxError::ScriptError {
                script: provider_id,
                message: e.to_string(),
            }
        })?;
        
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
        
        let res_bytes = spawn_blocking(move || {
            let mut plugin = plugin_arc.lock().unwrap();
            let json_input = serde_json::to_vec(&module_id).unwrap_or_default();
            plugin.call::<&[u8], &[u8]>("fetch_module", &json_input).map(|res| res.to_vec())
        }).await.map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?.map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?;

        let results: ModuleData = serde_json::from_slice(&res_bytes).map_err(|e| SandboxError::ScriptError {
            script: provider_id.clone(),
            message: e.to_string(),
        })?;
        
        Ok(results)
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
mod tests {
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
