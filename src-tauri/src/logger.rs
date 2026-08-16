use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, Emitter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use std::sync::atomic::AtomicBool;

static LOG_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static LOG_COLLECTION_ENABLED: AtomicBool = AtomicBool::new(true);
const MAX_LOG_ENTRIES: usize = 500;

pub fn is_log_collection_enabled() -> bool {
    LOG_COLLECTION_ENABLED.load(Ordering::Relaxed)
}

pub fn set_log_collection_active(enabled: bool) {
    LOG_COLLECTION_ENABLED.store(enabled, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: i64,
    pub category: String,
    pub level: String,
    pub message: String,
}

lazy_static::lazy_static! {
    static ref LOG_BUFFER: Mutex<VecDeque<LogEntry>> = Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES));
}

pub fn push_log(category: &str, level: &str, message: &str) {
    if !is_log_collection_enabled() {
        return;
    }

    let entry = LogEntry {
        id: LOG_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0),
        category: category.to_string(),
        level: level.to_string(),
        message: message.to_string(),
    };

    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        if buffer.len() >= MAX_LOG_ENTRIES {
            buffer.pop_front();
        }
        buffer.push_back(entry);
    }
}

pub fn init_logging(app: &AppHandle) {
    // 1. Configure rolling log file appender on disk
    if let Ok(app_dir) = app.path().app_data_dir() {
        let log_dir = app_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);

        let file_appender = tracing_appender::rolling::daily(&log_dir, "echo.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false);

        let stdout_layer = tracing_subscriber::fmt::layer();

        let _ = tracing_subscriber::registry()
            .with(stdout_layer)
            .with(file_layer)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();
    }

    push_log("System", "INFO", "Echo Music Player logging system initialized");

    // 2. Spawn 2 Hz background batch emitter targeting the debug-logger window
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
        let mut last_sent_id: u64 = 0;

        loop {
            interval.tick().await;

            if let Some(debug_window) = app_handle.get_webview_window("debug-logger") {
                let new_logs: Vec<LogEntry> = if let Ok(buffer) = LOG_BUFFER.lock() {
                    buffer.iter().filter(|e| e.id > last_sent_id).cloned().collect()
                } else {
                    Vec::new()
                };

                if let Some(last) = new_logs.last() {
                    last_sent_id = last.id;
                }

                if !new_logs.is_empty() {
                    let _ = debug_window.emit("debug-log-batch", new_logs);
                }
            }
        }
    });
}

// ── Tauri Commands ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn open_debug_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("debug-logger") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let builder = WebviewWindowBuilder::new(
        &app,
        "debug-logger",
        WebviewUrl::App("/debug".into()),
    )
    .title("Echo Desktop — Debug Console")
    .inner_size(900.0, 600.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .visible(true);

    builder.build().map_err(|e| format!("Failed to build Debug Window: {}", e))?;
    push_log("System", "INFO", "Debug logger window opened");
    Ok(())
}

#[tauri::command]
pub fn get_debug_logs() -> Vec<LogEntry> {
    LOG_BUFFER.lock().map(|b| b.iter().cloned().collect()).unwrap_or_default()
}

#[tauri::command]
pub fn clear_debug_logs() {
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.clear();
    }
    push_log("System", "INFO", "Debug log buffer cleared");
}

#[tauri::command]
pub fn copy_debug_log_to_clipboard() -> String {
    if let Ok(buffer) = LOG_BUFFER.lock() {
        buffer
            .iter()
            .map(|e| format!("[{}] [{}] [{}] {}", e.timestamp, e.level, e.category, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    }
}

#[tauri::command]
pub fn open_log_directory(app: AppHandle) -> Result<(), String> {
    if let Ok(app_dir) = app.path().app_data_dir() {
        let log_dir = app_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let path_str = log_dir.to_string_lossy().to_string();
        
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&path_str).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&path_str).spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&path_str).spawn();
        
        Ok(())
    } else {
        Err("Failed to get log directory path".to_string())
    }
}

#[tauri::command]
pub fn sandbox_log(category: Option<String>, level: Option<String>, message: String) {
    let cat = category.unwrap_or_else(|| "JS Sandbox".to_string());
    let lvl = level.unwrap_or_else(|| "INFO".to_string());
    push_log(&cat, &lvl, &message);
}

#[tauri::command]
pub fn set_log_collection_enabled(enabled: bool) {
    set_log_collection_active(enabled);
    if enabled {
        push_log("System", "INFO", "Diagnostic log collection enabled");
    }
}

#[tauri::command]
pub fn get_log_collection_enabled() -> bool {
    is_log_collection_enabled()
}
