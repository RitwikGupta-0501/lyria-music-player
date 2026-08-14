use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{Mutex, oneshot};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct SandboxManager {
    pub pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Result<Value, String>>>>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub fn spawn_sandbox_webview(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("sandbox-vm") {
        return Ok(window);
    }
    
    let builder = WebviewWindowBuilder::new(
        app,
        "sandbox-vm",
        WebviewUrl::App("sandbox.html".into()),
    )
    .title("Sandbox Debugger")
    .visible(true); // Temporarily visible for debugging

    let window = builder.build().map_err(|e| format!("Failed to build Sandbox WebviewWindow: {}", e))?;
    
    // Open devtools automatically so we can see the console.logs
    #[cfg(debug_assertions)]
    window.open_devtools();
    
    Ok(window)
}

pub async fn execute_javascript(
    app: &AppHandle,
    manager: &SandboxManager,
    script: &str,
) -> Result<Value, String> {
    let window = spawn_sandbox_webview(app)?;
    let req_id = Uuid::new_v4().to_string();
    
    let (tx, rx) = oneshot::channel();
    manager.pending_requests.lock().await.insert(req_id.clone(), tx);
        
    let safe_script = serde_json::to_string(script).unwrap_or_else(|_| "\"\"".to_string());
    let wrapped_script = format!("window.executeArbitraryJS('{}', {});", req_id, safe_script);
    
    window.eval(&wrapped_script)
        .map_err(|e| format!("Eval failed: {}", e))?;
        
    match rx.await {
        Ok(Ok(payload)) => Ok(payload),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("IPC Channel closed unexpectedly".to_string()),
    }
}
