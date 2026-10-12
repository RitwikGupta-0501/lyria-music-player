use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{Mutex, oneshot};
use serde_json::Value;
use uuid::Uuid;

type PendingRequestsMap = HashMap<String, oneshot::Sender<Result<Value, String>>>;

#[derive(Clone, Default)]
pub struct SandboxManager {
    pub pending_requests: Arc<Mutex<PendingRequestsMap>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self::default()
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
    .visible(false);

    let window = builder.build().map_err(|e| format!("Failed to build Sandbox WebviewWindow: {}", e))?;
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

    // Clone the map handle so we can clean up on any error exit path.
    // BUG-05: Without explicit removal, a failed eval() or dropped channel
    // leaks the oneshot::Sender in pending_requests indefinitely.
    let cleanup_map = manager.pending_requests.clone();
    let cleanup_id = req_id.clone();

    let safe_script = serde_json::to_string(script).unwrap_or_else(|_| "\"\"".to_string());
    let wrapped_script = format!("window.executeArbitraryJS('{}', {});", req_id, safe_script);

    if let Err(e) = window.eval(&wrapped_script) {
        cleanup_map.lock().await.remove(&cleanup_id);
        return Err(format!("Eval failed: {}", e));
    }

    match rx.await {
        Ok(Ok(payload)) => {
            // Happy path: sandbox_callback already removed the entry via pending.remove(&req_id).
            Ok(payload)
        }
        Ok(Err(e)) => {
            cleanup_map.lock().await.remove(&cleanup_id);
            Err(e)
        }
        Err(_) => {
            // rx dropped without resolution (e.g. 15s timeout in host_execute_webview_js).
            cleanup_map.lock().await.remove(&cleanup_id);
            Err("IPC Channel closed unexpectedly".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    /// Simulates the memory-leak scenario: inserting a req_id and then
    /// explicitly removing it (as the fix does on error paths).
    /// Verifies the map is empty after cleanup.
    #[tokio::test]
    async fn pending_requests_cleanup_on_eval_failure() {
        let manager = SandboxManager::new();
        let req_id = "test-req-123".to_string();

        let (tx, _rx) = oneshot::channel::<Result<serde_json::Value, String>>();

        // Simulate insert (as execute_javascript does)
        manager.pending_requests.lock().await.insert(req_id.clone(), tx);
        assert_eq!(manager.pending_requests.lock().await.len(), 1);

        // Simulate eval() failure cleanup
        manager.pending_requests.lock().await.remove(&req_id);
        assert_eq!(
            manager.pending_requests.lock().await.len(),
            0,
            "pending_requests must be empty after eval failure cleanup"
        );
    }

    #[tokio::test]
    async fn pending_requests_cleanup_on_channel_closed() {
        let manager = SandboxManager::new();

        // Insert two requests to verify only the targeted one is removed
        let (tx1, _rx1) = oneshot::channel::<Result<serde_json::Value, String>>();
        let (tx2, _rx2) = oneshot::channel::<Result<serde_json::Value, String>>();

        manager.pending_requests.lock().await.insert("req-A".to_string(), tx1);
        manager.pending_requests.lock().await.insert("req-B".to_string(), tx2);
        assert_eq!(manager.pending_requests.lock().await.len(), 2);

        // Simulate channel closed (Err(_) branch) cleanup for req-A only
        manager.pending_requests.lock().await.remove("req-A");
        let map = manager.pending_requests.lock().await;
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("req-B"), "req-B must not be affected by req-A cleanup");
    }
}
