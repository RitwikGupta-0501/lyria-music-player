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
const MODULE_FETCH_TIMEOUT_SECS: u64 = 3;

// ── Public types ─────────────────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderModule {
    pub id: String,
    pub name: String,
    pub layout: ModuleLayout,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModuleLayout {
    Grid,
    List,
    Carousel,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModuleItem {
    Track(TrackResult),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleData {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
