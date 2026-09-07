# Lyria Extension Architecture & Extism WASM ABI v1 Specification

This document details the architecture, runtime sandbox environment, Extism WebAssembly (WASM) Application Binary Interface (ABI v1), and host function signatures for Lyria audio player extensions.

---

## 1. Overview & Sandboxing Philosophy

Lyria extensions are sandboxed WebAssembly modules executing on the host via [Extism](https://extism.org/). The core audio engine and database daemon run natively in Rust (via Tauri 2.0), while extensions provide external data fetching, stream resolution, catalog exploration, telemetry handling, and algorithmic recommendation logic.

### Sandbox Guardrails
* **Isolation**: Extensions run in an isolated WASI context. Direct filesystem access, network sockets, and arbitrary OS system calls are blocked.
* **Controlled Host I/O**: Network requests and persistent database storage must pass through dedicated, security-gated host functions.
* **Memory Limits**: The Extism runtime limits extension WASM linear memory allocations to a hard maximum of **2048 pages (128 MB)**.
* **Execution Timeouts**: Synchronous calls are wrapped in execution timeouts (typically 15s to 30s) to prevent frozen UI threads or deadlocks.
* **WASI Target**: Extensions compile to `wasm32-wasip1` (or `wasm32-unknown-unknown`).

---

## 2. Host Function Signatures

Extensions communicate with the Lyria host runtime via functions exported by the `ExtismHost` namespace. All structured data is passed as UTF-8 JSON strings over Extism's memory bridge.

### 2.1 `host_log`
Logs a message through the host application's tracing/logging system.
* **Namespace**: `ExtismHost`
* **Signature**: `host_log(msg: String) -> ()`
* **Parameters**:
  * `msg`: String message to record. The host prepends `[PLUGIN LOG] <msg>` and routes it to the native logger (and log files).

### 2.2 `host_http_request`
Sends an asynchronous HTTP request using the host's pooled `reqwest` client.
* **Namespace**: `ExtismHost`
* **Signature**: `host_http_request(req_json: String) -> String`
* **Input Payload (`HttpRequest`)**:
  ```json
  {
    "method": "GET",
    "url": "https://api.example.com/v1/tracks",
    "headers": {
      "User-Agent": "Lyria/1.0",
      "Accept": "application/json"
    },
    "body": null
  }
  ```
* **Output Payload (`HttpResponse`)**:
  ```json
  {
    "status": 200,
    "headers": {
      "content-type": "application/json"
    },
    "body": "{\"tracks\": [...]}"
  }
  ```

### 2.3 `host_storage_get`
Retrieves a key-value entry from the host's embedded SQLite database scoped to the calling provider.
* **Namespace**: `ExtismHost`
* **Signature**: `host_storage_get(req_json: String) -> String`
* **Input Payload (`StorageRequest`)**:
  ```json
  {
    "provider_id": "youtube-wasm",
    "key": "cached_visitor_data",
    "value": null
  }
  ```
* **Output**: String value stored in SQLite, or an empty string `""` if not found.

### 2.4 `host_storage_set`
Saves or updates a key-value entry in the host's embedded SQLite database scoped to the calling provider.
* **Namespace**: `ExtismHost`
* **Signature**: `host_storage_set(req_json: String) -> ()`
* **Input Payload (`StorageRequest`)**:
  ```json
  {
    "provider_id": "youtube-wasm",
    "key": "cached_visitor_data",
    "value": "Cgt2S1lyc..."
  }
  ```

### 2.5 `host_execute_webview_js`
Executes JavaScript inside the host's isolated sandbox VM (used for cryptographic deobfuscation, challenge solving, or token extraction without running an entire browser engine).
* **Namespace**: `ExtismHost`
* **Signature**: `host_execute_webview_js(script: String) -> String`
* **Parameters**:
  * `script`: JavaScript code string to evaluate.
* **Returns**: String result of the evaluation.
* **Execution Timeout**: 15 seconds.

### 2.6 `host_telemetry_request`
Sends a telemetry/scrobble request to external analytics or music tracking endpoints.
* **Namespace**: `ExtismHost`
* **Signature**: `host_telemetry_request(req_json: String) -> String`
* **Capability Guard**: Structurally gated by the `telemetry_reporting` manifest capability. If an extension calls this function without declaring `"telemetry_reporting"` in `manifest.json`, the host redirects the call to `host_telemetry_blocked`, immediately throwing `PermissionDenied`.

---

## 3. Extension Export Functions (Plugin Entry Points)

Extensions export functions using standard Extism conventions (`#[plugin_fn]` in Rust PDK, or `// @extism-export` in AssemblyScript/JS).

All parameters and return values are serialized as JSON strings unless otherwise noted.

| Export Function | Capability Required | Input Parameter | Output Return | Description |
|---|---|---|---|---|
| `warmup` | `warmup` | `String` (`"{}"`) | `String` | Invoked on application startup or provider enable to initialize session caches and tokens. |
| `search` | `search` | `String` (search query) | `String` (`Vec<TrackResult>`) | Basic track search returning a list of audio tracks. |
| `search_categorized` | `search` | `String` (`SearchQueryInput`) | `String` (`CategorizedSearchResult`) | Categorized search returning structured tabs/sections for Songs, Albums, Artists, Playlists, etc. |
| `resolve` | *(core)* | `String` (track ID / seed) | `String` (`ResolvedTrack`) | Resolves a track ID to a playable audio stream URL and playback headers. |
| `resolve_url` | `url_resolver` | `String` (raw URL string) | `String` (`ResolvedTrack`) | Resolves an arbitrary external URL (e.g. YouTube, SoundCloud) directly to an audio stream. |
| `get_modules` | `explore` | *None* | `String` (`Vec<ProviderModule>`) | Returns catalog layout modules registered for the Explore/Home page. |
| `fetch_module` | `explore` | `String` (module ID / params) | `String` (`ModuleData`) | Returns items for a specific explore module section. |
| `browse_album` | `explore` | `String` (album ID) | `String` (`AlbumDetailResult`) | Fetches full album metadata and tracklist. |
| `browse_artist` | `explore` | `String` (artist ID) | `String` (`ArtistDetailResult`) | Fetches artist details, discography, top tracks, and related artists. |
| `get_related` | `explore` | `String` (track ID) | `String` (`Vec<TrackResult>`) | Returns related/recommended tracks for a given track seed. |
| `get_radio` | `explore` | `String` (track ID or continuation) | `String` (`RadioStreamResultV1`) | Returns infinite radio stream tracks for algorithmic playback. |
| `on_playback_event` | `telemetry_reporting` | `String` (`PlaybackTelemetryEventV1`) | `String` (`"ok"`) | Receives telemetry events when a track completes or updates playback milestones. |

---

## 4. Core Data Models & Schemas

### 4.1 Track Models

#### `TrackResult`
Represents an individual audio track in search or catalog views:
```json
{
  "id": "dQw4w9WgXcQ",
  "title": "Never Gonna Give You Up",
  "artist": "Rick Astley",
  "album": "Whenever You Need Somebody",
  "cover_art_url": "https://example.com/art.jpg",
  "stream_url": null,
  "quality_hint": "160kbps opus",
  "duration_ms": 213000,
  "isrc": null,
  "plays": "1.5B plays"
}
```

#### `ResolvedTrack`
Returned by `resolve` or `resolve_url` to feed the native audio engine (`rodio` sink):
```json
{
  "stream_url": "https://googlevideo.com/videoplayback?...",
  "quality_hint": "opus/160kbps",
  "duration_ms": 213000,
  "headers": {
    "User-Agent": "Mozilla/5.0 ...",
    "Referer": "https://music.youtube.com/"
  }
}
```

### 4.2 Categorized Search Models

#### `SearchQueryInput`
Input to `search_categorized`:
```json
{
  "query": "Hans Zimmer Interstellar",
  "filter": "all"
}
```
*Valid `filter` values*: `"all"`, `"songs"`, `"albums"`, `"artists"`, `"playlists"`.

#### `CategorizedSearchResult`
```json
{
  "sections": [
    {
      "category": "Songs",
      "items": [
        {
          "type": "Track",
          "data": { "id": "123", "title": "Cornfield Chase", "artist": "Hans Zimmer", "duration_ms": 126000 }
        }
      ]
    },
    {
      "category": "Albums",
      "items": [
        {
          "type": "Album",
          "data": { "id": "alb_456", "title": "Interstellar (OST)", "artist": "Hans Zimmer", "year": "2014" }
        }
      ]
    }
  ],
  "continuation_token": null
}
```

### 4.3 Explore & Module Models

#### `ProviderModule`
```json
{
  "id": "trending_section",
  "name": "Trending Right Now",
  "layout": "grid"
}
```
*Layout variants*: `"grid"`, `"list"`, `"carousel"`, `"chart4row"`, `"genrecloud"`.

#### `ModuleItem`
Enums are serialized using `{ "type": "<Variant>", "data": { ... } }`:
* `Track`: wrapped `TrackResult`
* `Album`: `{ "id": "...", "title": "...", "artist": "...", "year": "...", "cover_art_url": "..." }`
* `Playlist`: `{ "id": "...", "title": "...", "author": "...", "item_count": 25, "cover_art_url": "..." }`
* `Artist`: `{ "id": "...", "name": "...", "avatar_url": "...", "subscribers": "..." }`
* `Genre`: `{ "id": "...", "title": "...", "endpoint_params": "...", "color_hex": "#1DB954" }`
* `Spotlight`: `{ "id": "...", "title": "...", "artist": "...", "cover_art_url": "...", "description": "..." }`
* `Shelf`: `{ "title": "Featured", "items": [ ... ] }`

### 4.4 Telemetry Models

#### `PlaybackTelemetryEventV1`
```json
{
  "native_track_id": "dQw4w9WgXcQ",
  "duration_ms": 213000,
  "total_track_duration_ms": 213000,
  "completed": true
}
```
