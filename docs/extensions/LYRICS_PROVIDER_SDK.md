# 🛠️ Lyria Extension SDK: Building a Lyrics Provider

This guide walks you through building, testing, and distributing a **Lyrics Extension** for the Lyria music player.

Lyria extensions run in a sandboxed WebAssembly (WASM) environment powered by [Extism](https://extism.org/) and WASI. Extensions can fetch metadata from any REST or GraphQL API using secure host HTTP functions while remaining completely isolated from the user's host filesystem and OS.

---

## 1. Quick Overview & Architecture

When a song plays or the user opens the Lyrics drawer, Lyria's `LyricsManager`:
1. Strips extraneous noise from the track's metadata (e.g. `(Official Video)`, `feat. Artist`).
2. Checks the local SQLite cache for an existing match.
3. If not cached, invokes your extension's exported `get_lyrics` function in a sandboxed WebAssembly runtime.
4. Accepts either a structured JSON payload (`LyricsResult`) or a standard raw LRC string.
5. Caches the result locally for future offline playback.

---

## 2. Manifest Configuration (`manifest.json`)

To declare that your extension provides lyrics, add `"lyrics"` to the `capabilities` array in your `manifest.json`:

```json
{
  "id": "lrclib-wasm",
  "name": "LRCLIB Lyrics Provider",
  "author": "Lyria Community",
  "version": "1.0.0",
  "description": "Fetches synchronized line-by-line and plain text lyrics from LRCLIB",
  "capabilities": ["lyrics"],
  "main": "plugin.wasm",
  "homepage": "https://github.com/lyria-player/lrclib-extension"
}
```

---

## 3. The `get_lyrics` WASM Contract

### 3.1 Export Signature
Every lyrics extension must export a single WebAssembly function named `get_lyrics`:

* **Rust (Extism PDK)**: `#[plugin_fn] pub fn get_lyrics(input: String) -> FnResult<String>`
* **AssemblyScript / TypeScript**: `export function get_lyrics(): i32` (using Extism AS SDK)

### 3.2 Input Payload (`LyricsQuery`)
The input is passed as a JSON string containing the track's details:

```json
{
  "title": "Bohemian Rhapsody",
  "artist": "Queen",
  "album": "A Night at the Opera",
  "duration_ms": 354000,
  "isrc": "GBUM71029606",
  "track_id": null
}
```

| Field | Type | Description |
|---|---|---|
| `title` | `String` | Sanitized track title (lowercased/cleaned of common video noise). |
| `artist` | `String` | Track artist name. |
| `album` | `Option<String>` | Album title if known. |
| `duration_ms` | `Option<u64>` | Track duration in milliseconds (useful for fuzzy duration matching). |
| `isrc` | `Option<String>` | International Standard Recording Code if available. |
| `track_id` | `Option<String>` | Provider-specific track ID if resolved by an upstream provider. |

### 3.3 Output Payload (`LyricsResult`)
Your function should return a JSON string representing `LyricsResult`. If no matching lyrics were found for the song, return an empty string `""` or `"null"`.

```json
{
  "provider_id": "lrclib-wasm",
  "provider_name": "LRCLIB Lyrics Provider",
  "format": "synced",
  "lines": [
    {
      "time_ms": 500,
      "text": "Is this the real life?",
      "end_time_ms": 3200
    },
    {
      "time_ms": 3200,
      "text": "Is this just fantasy?",
      "end_time_ms": 7850
    }
  ],
  "raw_plain": null,
  "raw_lrc": "[00:00.50]Is this the real life?\n[00:03.20]Is this just fantasy?",
  "offset_ms": 0,
  "instrumental": false,
  "copyright": "© Queen Music Ltd."
}
```

> **Tip — Simplified Return**: If your provider already has a standard LRC string, you can simply return the raw LRC string! Lyria's host runtime will automatically parse timestamps, expand multi-bracket repeats, calculate `end_time_ms`, and format the lyrics for you.

---

## 4. Host Functions Available to Your Extension

Lyrics extensions can utilize host functions provided in the `ExtismHost` namespace:

### `host_http_request`
Used to communicate with third-party lyrics APIs:
```rust
#[derive(serde::Serialize)]
struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(serde::Deserialize)]
struct HttpResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}
```

### `host_log`
Logs a message to Lyria's `/debug` console and file logger:
```rust
// In Rust with extism-pdk:
// host_log("Searching LRCLIB for title: Bohemian Rhapsody");
```

### `host_storage_get` & `host_storage_set`
Scoped key-value persistence in Lyria's SQLite database (great for caching access tokens, rate limit counters, or session cookies).

---

## 5. Complete Rust Reference Implementation

Below is a complete, production-ready implementation of an LRCLIB lyrics extension built in Rust:

### `Cargo.toml`
```toml
[package]
name = "lrclib-extension"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
urlencoding = "2.1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### `src/lib.rs`
```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct LyricsQuery {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Deserialize)]
struct LrclibResponse {
    pub synced_lyrics: Option<String>,
    pub plain_lyrics: Option<String>,
    pub instrumental: Option<bool>,
}

#[derive(Serialize)]
struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Deserialize)]
struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[host_fn]
extern "ExtismHost" {
    fn host_http_request(req_json: String) -> String;
}

#[plugin_fn]
pub fn get_lyrics(input: String) -> FnResult<String> {
    let query: LyricsQuery = serde_json::from_str(&input)?;

    // Construct URL for LRCLIB API
    let mut url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}",
        urlencoding::encode(&query.title),
        urlencoding::encode(&query.artist),
    );

    if let Some(ref alb) = query.album {
        url.push_str(&format!("&album_name={}", urlencoding::encode(alb)));
    }
    if let Some(dur) = query.duration_ms {
        url.push_str(&format!("&duration={}", dur / 1000));
    }

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Lyria-Music-Player/1.0".to_string());

    let http_req = HttpRequest {
        method: "GET".to_string(),
        url,
        headers: Some(headers),
        body: None,
    };

    let req_json = serde_json::to_string(&http_req)?;
    let resp_str = unsafe { host_http_request(req_json)? };
    let resp: HttpResponse = serde_json::from_str(&resp_str)?;

    if resp.status != 200 {
        return Ok("null".to_string());
    }

    let lrclib_data: LrclibResponse = serde_json::from_str(&resp.body)?;

    if let Some(true) = lrclib_data.instrumental {
        return Ok(serde_json::to_string(&serde_json::json!({
            "provider_id": "lrclib-wasm",
            "provider_name": "LRCLIB Provider",
            "format": "plain",
            "lines": [],
            "instrumental": true,
            "offset_ms": 0
        }))?);
    }

    // If synced lyrics exist, return as raw LRC (Lyria host parses timestamps automatically!)
    if let Some(synced) = lrclib_data.synced_lyrics {
        return Ok(synced);
    }

    // Fall back to plain lyrics
    if let Some(plain) = lrclib_data.plain_lyrics {
        return Ok(plain);
    }

    Ok("null".to_string())
}
```

### Compiling to WASM
```bash
cargo build --target wasm32-wasip1 --release
```
Your compiled binary will be located at `target/wasm32-wasip1/release/lrclib_extension.wasm`. Package it with your `manifest.json` and distribute!

---

## 6. Runtime Limits & Guardrails

* **Timeout**: Host invocations timeout after **15 seconds**. Long-hanging network connections will be cancelled cleanly without freezing the player.
* **Memory Budget**: Extension linear memory is capped at **40MB**.
* **Zero Host File Access**: Extensions have no disk I/O outside of the host `host_storage_*` KV store.
