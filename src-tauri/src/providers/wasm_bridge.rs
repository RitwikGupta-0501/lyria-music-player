use extism::host_fn;
use std::sync::Arc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use std::sync::mpsc::Sender;
use crate::db::DbRequest;

#[derive(Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
pub struct StorageRequest {
    pub provider_id: String,
    pub key: String,
    pub value: Option<String>,
}

// Host functions
host_fn!(pub host_log (input: String) -> () {
    crate::logger::push_log("WASM", "INFO", &input);
    tracing::info!(target: "echo_desktop::wasm", "[PLUGIN LOG] {}", input);
    Ok(())
});


fn run_async_host<F, T>(f: F) -> Result<T, extism::Error>
where
    F: std::future::Future<Output = Result<T, extism::Error>> + Send + 'static,
    T: Send + 'static,
{
    let rt = tokio::runtime::Handle::current();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    rt.spawn(async move {
        let res = f.await;
        let _ = tx.send(res);
    });
    rx.recv().map_err(|e| extism::Error::msg(e.to_string()))?
}

host_fn!(pub host_http_request (user_data: Arc<Client>; json_input: String) -> String {
    let req: HttpRequest = serde_json::from_str(&json_input).map_err(|e| extism::Error::msg(e.to_string()))?;
    let client = user_data.get()?.lock().unwrap().clone();
    
    let res = run_async_host(async move {
        let mut builder = match req.method.as_str() {
            "GET" => client.get(&req.url),
            "POST" => client.post(&req.url),
            _ => client.get(&req.url),
        };
        
        if let Some(headers) = req.headers {
            for (k, v) in headers {
                builder = builder.header(k, v);
            }
        }
        if let Some(body) = req.body {
            builder = builder.body(body);
        }
        
        let response = builder.send().await.map_err(|e| extism::Error::msg(e.to_string()))?;
        let status = response.status().as_u16();
        let mut resp_headers = std::collections::HashMap::new();
        for (k, v) in response.headers() {
            resp_headers.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
        }
        let body = response.text().await.unwrap_or_default();
        
        Ok::<HttpResponse, extism::Error>(HttpResponse { status, headers: resp_headers, body })
    })?;
    
    let out = serde_json::to_string(&res).map_err(|e| extism::Error::msg(e.to_string()))?;
    Ok(out)
});

// Dedicated host function for telemetry reporting (structurally gated by manifest capability)
host_fn!(pub host_telemetry_request (user_data: Arc<Client>; json_input: String) -> String {
    let req: HttpRequest = serde_json::from_str(&json_input).map_err(|e| extism::Error::msg(e.to_string()))?;
    let client = user_data.get()?.lock().unwrap().clone();
    
    let res = run_async_host(async move {
        let mut builder = match req.method.as_str() {
            "GET" => client.get(&req.url),
            "POST" => client.post(&req.url),
            _ => client.get(&req.url),
        };
        
        if let Some(headers) = req.headers {
            for (k, v) in headers {
                builder = builder.header(k, v);
            }
        }
        if let Some(body) = req.body {
            builder = builder.body(body);
        }
        
        let response = builder.send().await.map_err(|e| extism::Error::msg(e.to_string()))?;
        let status = response.status().as_u16();
        let mut resp_headers = std::collections::HashMap::new();
        for (k, v) in response.headers() {
            resp_headers.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
        }
        let body = response.text().await.unwrap_or_default();
        
        Ok::<HttpResponse, extism::Error>(HttpResponse { status, headers: resp_headers, body })
    })?;
    
    let out = serde_json::to_string(&res).map_err(|e| extism::Error::msg(e.to_string()))?;
    Ok(out)
});

host_fn!(pub host_telemetry_blocked (_json_input: String) -> String {
    let err: Result<String, extism::Error> = Err(extism::Error::msg("PermissionDenied: extension lacks 'telemetry_reporting' capability")); err
});

host_fn!(pub host_storage_get (user_data: Arc<Sender<DbRequest>>; json_input: String) -> String {
    let req: StorageRequest = serde_json::from_str(&json_input).map_err(|e| extism::Error::msg(e.to_string()))?;
    let db_tx = user_data.get()?.lock().unwrap().clone();
    
    let res = run_async_host(async move {
        let (tx, rx) = oneshot::channel();
        db_tx.send(DbRequest::GetProviderStorage {
            provider_id: req.provider_id,
            key: req.key,
            resp: tx,
        }).map_err(|e| extism::Error::msg(e.to_string()))?;
        
        rx.await.map_err(|e| extism::Error::msg(e.to_string()))?.map_err(extism::Error::msg)
    })?;
    
    Ok(res.unwrap_or_default())
});

host_fn!(pub host_storage_set (user_data: Arc<Sender<DbRequest>>; json_input: String) -> () {
    let req: StorageRequest = serde_json::from_str(&json_input).map_err(|e| extism::Error::msg(e.to_string()))?;
    let db_tx = user_data.get()?.lock().unwrap().clone();
    
    if let Some(value) = req.value {
        run_async_host(async move {
            let (tx, rx) = oneshot::channel();
            db_tx.send(DbRequest::SetProviderStorage {
                provider_id: req.provider_id,
                key: req.key,
                value,
                resp: tx,
            }).map_err(|e| extism::Error::msg(e.to_string()))?;
            
            rx.await.map_err(|e| extism::Error::msg(e.to_string()))?.map_err(extism::Error::msg)
        })?;
    }
    
    Ok(())
});

use tauri::Manager;

host_fn!(pub host_execute_webview_js (user_data: Arc<tauri::AppHandle>; script: String) -> String {
    let app = user_data.get()?.lock().unwrap().clone();
    let state = app.state::<crate::AppState>();
    let sandbox_mgr = state.sandbox_manager.clone();
    
    let res = run_async_host(async move {
        match tokio::time::timeout(
            std::time::Duration::from_secs(15), 
            crate::sandbox::sandbox_vm::execute_javascript(&app, &sandbox_mgr, &script)
        ).await {
            Ok(Ok(result)) => {
                crate::logger::push_log("JS Sandbox", "DEBUG", "JS execution completed successfully");
                Ok(result.to_string())
            },
            Ok(Err(err)) => {
                crate::logger::push_log("JS Sandbox", "ERROR", &format!("JS execution error: {}", err));
                Err(extism::Error::msg(err))
            },
            Err(_) => {
                crate::logger::push_log("JS Sandbox", "ERROR", "JS execution timeout after 15s");
                Err(extism::Error::msg("JS execution timeout"))
            },
        }
    })?;
    
    Ok(res)
});


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_run_async_host_inside_tokio_worker() {
        // Calling run_async_host inside a Tokio worker thread must NOT panic.
        // Before the fix, calling Handle::current().block_on inside a worker thread
        // threw: "Cannot start a runtime from within a runtime".
        let result = run_async_host(async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            Ok::<u32, extism::Error>(42)
        });
        assert_eq!(result.unwrap(), 42);
    }
}
