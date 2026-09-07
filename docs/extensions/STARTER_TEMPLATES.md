# Lyria Extension Starter Templates

This document provides complete, production-ready boilerplate starter templates for creating Lyria extensions in **Rust** and **AssemblyScript (TypeScript)**.

---

## 1. Rust Starter Template

Rust is the recommended language for Lyria extensions due to zero-cost abstractions, robust memory safety, and native Extism PDK support.

### 1.1 Project Structure

```
my-extension/
├── .cargo/
│   └── config.toml
├── Cargo.toml
├── manifest.json
└── src/
    ├── host.rs
    ├── lib.rs
    └── models.rs
```

### 1.2 `Cargo.toml`

```toml
[package]
name = "my-extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"     # Optimize for minimum binary size
lto = true          # Enable link-time optimization
codegen-units = 1
panic = "abort"
strip = true        # Strip symbols
```

### 1.3 `.cargo/config.toml`

```toml
[build]
target = "wasm32-wasip1"
```

### 1.4 `src/models.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
pub struct StorageRequest {
    pub provider_id: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTrack {
    pub stream_url: String,
    pub quality_hint: Option<String>,
    pub duration_ms: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
}
```

### 1.5 `src/host.rs`

```rust
use extism_pdk::*;
use std::collections::HashMap;
use crate::models::{HttpRequest, HttpResponse, StorageRequest};

#[host_fn]
extern "ExtismHost" {
    pub fn host_log(msg: String);
    pub fn host_http_request(req: String) -> String;
    pub fn host_storage_get(req: String) -> String;
    pub fn host_storage_set(req: String);
}

pub fn log(msg: &str) {
    unsafe {
        let _ = host_log(msg.to_string());
    }
}

pub fn http_get(url: &str, headers: Option<HashMap<String, String>>) -> FnResult<HttpResponse> {
    let req = HttpRequest {
        method: "GET".to_string(),
        url: url.to_string(),
        headers,
        body: None,
    };
    let json_req = serde_json::to_string(&req)?;
    let res_json = unsafe { host_http_request(json_req)? };
    let res: HttpResponse = serde_json::from_str(&res_json)?;
    Ok(res)
}
```

### 1.6 `src/lib.rs`

```rust
use extism_pdk::*;
mod host;
mod models;

use host::{http_get, log};
use models::{ResolvedTrack, TrackResult};

#[plugin_fn]
pub fn warmup(_input: String) -> FnResult<String> {
    log("Warming up my-extension...");
    Ok("ok".to_string())
}

#[plugin_fn]
pub fn search(query: String) -> FnResult<String> {
    log(&format!("Searching for: {}", query));

    // Example: fetch from external search API
    // let resp = http_get(&format!("https://api.example.com/search?q={}", query), None)?;

    let results = vec![
        TrackResult {
            id: "example-1".to_string(),
            title: format!("Match for '{}'", query),
            artist: "Demo Artist".to_string(),
            album: Some("Demo Album".to_string()),
            cover_art_url: Some("https://example.com/cover.jpg".to_string()),
            stream_url: None,
            quality_hint: Some("320kbps MP3".to_string()),
            duration_ms: Some(180000),
        }
    ];

    Ok(serde_json::to_string(&results)?)
}

#[plugin_fn]
pub fn resolve(track_id: String) -> FnResult<String> {
    log(&format!("Resolving track ID: {}", track_id));

    let resolved = ResolvedTrack {
        stream_url: format!("https://cdn.example.com/audio/{}.mp3", track_id),
        quality_hint: Some("320kbps MP3".to_string()),
        duration_ms: Some(180000),
        headers: None,
    };

    Ok(serde_json::to_string(&resolved)?)
}
```

### 1.7 Building the Rust Extension

Ensure the `wasm32-wasip1` target is installed:
```bash
rustup target add wasm32-wasip1
cargo build --release
# Compiled output is at: target/wasm32-wasip1/release/my_extension.wasm
```

---

## 2. AssemblyScript (TypeScript) Starter Template

If you prefer TypeScript or JavaScript syntax, AssemblyScript compiles directly into lean WebAssembly.

### 2.1 Project Structure

```
my-ts-extension/
├── asconfig.json
├── manifest.json
├── package.json
└── assembly/
    ├── host.ts
    ├── index.ts
    └── models.ts
```

### 2.2 `package.json`

```json
{
  "name": "my-ts-extension",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "build": "asc assembly/index.ts --target release --outFile build/my_ts_extension.wasm --optimizeSpeed"
  },
  "devDependencies": {
    "assemblyscript": "^0.27.0",
    "assemblyscript-json": "^1.1.0"
  }
}
```

### 2.3 `asconfig.json`

```json
{
  "targets": {
    "release": {
      "outFile": "build/my_ts_extension.wasm",
      "optimize": true,
      "shrinkLevel": 2,
      "converge": true,
      "noAssert": true
    }
  },
  "options": {
    "bindings": "raw"
  }
}
```

### 2.4 `assembly/host.ts`

```typescript
// Declare Extism host imports
@external("ExtismHost", "host_log")
export declare function host_log(ptr: u64): void;

@external("ExtismHost", "host_http_request")
export declare function host_http_request(ptr: u64): u64;

// Memory read/write utilities via Extism memory management
@external("extism", "alloc")
declare function extism_alloc(size: u64): u64;

@external("extism", "load_u8")
declare function extism_load_u8(ptr: u64): u8;

@external("extism", "store_u8")
declare function extism_store_u8(ptr: u64, val: u8): void;

export function log(msg: string): void {
  // Convert string to UTF-8 and pass pointer to host_log
}
```

### 2.5 `assembly/index.ts`

```typescript
import { log } from "./host";

// Extism entry point for search
export function search(): i32 {
  // Read input query from Extism input memory
  // Process query
  // Return JSON serialized TrackResult[]
  return 0;
}

export function resolve(): i32 {
  // Read track ID and return ResolvedTrack JSON
  return 0;
}
```

---

## 3. Packaging & Verification

1. Place your `manifest.json` and compiled `.wasm` into a folder:
   ```
   my-extension/
   ├── manifest.json
   ├── my_extension.wasm
   └── icon.png
   ```
2. Compress into a `.zip` archive or copy directly to:
   * **Linux**: `~/.local/share/Lyria/extensions/my-extension/`
   * **Windows**: `%APPDATA%\Lyria\extensions\my-extension\`
3. In Lyria, open **Settings > Extensions**. Click **Sync** or drag-and-drop the `.zip` to test!
