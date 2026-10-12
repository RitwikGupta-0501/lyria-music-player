# Building Download Extensions for Lyria

Welcome to the developer guide for creating **Download Provider Extensions** in Lyria.

This manual explains how to declare download capabilities, export the `resolve_download` WebAssembly entry point, return high-resolution audio assets, pass custom authentication headers, and handle error fallbacks with elegance.

---

## 1. Architectural Philosophy: Downloads vs. Streaming

In Lyria, **audio streaming** and **audio downloading** are intentionally separated into two distinct capabilities:

| Feature | Streaming (`resolve`) | Downloading (`resolve_download`) |
|---|---|---|
| **Primary Goal** | Real-time playback with minimum latency (TTFB) | High-resolution file preservation & permanent offline caching |
| **Output Type** | Direct stream URL for `rodio` sink | Direct file URL, container format, bitrate & metadata |
| **Integrity** | Ephemeral, chunked playback buffers | Full-file byte verification, tag embedding & atomic disk write |
| **Capability Flag** | `"resolve"` | `"download"` |

An extension can implement **either** capability, or **both**. When a user clicks "Download", Lyria exclusively queries providers declaring the `"download"` capability.

---

## 2. Manifest Declaration (`manifest.json`)

To register your extension as a download provider, declare `"download"` in your extension manifest's `capabilities` array:

```json
{
  "id": "qobuz-hifi",
  "name": "Qobuz Hi-Res Downloader",
  "author": "Community Developer",
  "version": "1.0.0",
  "main": "plugin.wasm",
  "capabilities": [
    "search",
    "download"
  ],
  "priority": 10,
  "description": "Fetches 24-bit 96kHz FLAC audio files for offline music collection."
}
```

* **`priority`**: Default priority ranking (higher numbers are queried earlier in the fallback sequence).
* **`capabilities`**: Must include `"download"`.

---

## 3. The `resolve_download` Export Function

Extensions export a function named `resolve_download` using the standard [Extism](https://extism.org/) plugin calling convention.

* **Export Name**: `resolve_download`
* **Input**: UTF-8 JSON string (`DownloadQueryInput`)
* **Output**: UTF-8 JSON string (`DownloadSourceResult`) or an error message

### 3.1 Input Payload: `DownloadQueryInput`

When Lyria requests a download link for a track, it passes the following JSON:

```json
{
  "title": "Get Lucky",
  "artist": "Daft Punk",
  "album": "Random Access Memories",
  "duration_ms": 369000,
  "isrc": "USX111300001",
  "native_id": "track_12345",
  "quality_preference": "lossless"
}
```

#### Fields Description:
* `title` *(string)*: Name of the song.
* `artist` *(string)*: Primary performing artist.
* `album` *(string, optional)*: Album or release title.
* `duration_ms` *(integer, optional)*: Expected track length in milliseconds.
* `isrc` *(string, optional)*: International Standard Recording Code for exact acoustic catalog matching.
* `native_id` *(string, optional)*: Native identifier if the user originated the track from your provider.
* `quality_preference` *(string)*: User's audio preference:
  * `"lossless"`: FLAC, ALAC, or WAV.
  * `"high"`: ~320kbps MP3 or ~256kbps Opus/AAC.
  * `"standard"`: ~128kbps - ~192kbps.
  * `"smallest_size"`: Lowest bandwidth stream.

---

### 3.2 Output Payload: `DownloadSourceResult`

Your extension must return a JSON payload with direct download parameters:

```json
{
  "download_url": "https://cdn.example.com/audio/track_flac_96k.flac",
  "codec": "flac",
  "bitrate_kbps": 1411,
  "sample_rate_hz": 44100,
  "bit_depth": 16,
  "content_length_bytes": 45120300,
  "headers": {
    "Authorization": "Bearer eyJhbGciOi...",
    "User-Agent": "Lyria/1.0"
  },
  "cover_art_url": "https://cdn.example.com/art/album_cover.jpg",
  "lyrics_lrc": "[00:05.12] Like the legend of the phoenix..."
}
```

#### Codec Formats Supported:
`codec` must be one of:
* `"flac"` (Lossless Free Lossless Audio Codec)
* `"opus"` (High-efficiency Ogg Opus)
* `"aac"` (Advanced Audio Coding)
* `"mp3"` (MPEG-1 Audio Layer III)
* `"m4a"` (MPEG-4 Audio container)
* `"wav"` (Uncompressed PCM)
* `"vorbis"` (Ogg Vorbis)

---

## 4. Complete Rust PDK Reference Implementation

Here is a complete, working example of a download provider using the `extism-pdk` crate:

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct DownloadQueryInput {
    title: String,
    artist: String,
    album: Option<String>,
    duration_ms: Option<u64>,
    isrc: Option<String>,
    native_id: Option<String>,
    quality_preference: Option<String>,
}

#[derive(Serialize)]
struct DownloadSourceResult {
    download_url: String,
    codec: String,
    bitrate_kbps: Option<u32>,
    sample_rate_hz: Option<u32>,
    bit_depth: Option<u8>,
    content_length_bytes: Option<u64>,
    headers: Option<HashMap<String, String>>,
    cover_art_url: Option<String>,
    lyrics_lrc: Option<String>,
}

#[plugin_fn]
pub fn resolve_download(input_json: String) -> FnResult<String> {
    let query: DownloadQueryInput = serde_json::from_str(&input_json)
        .map_err(|e| WithReturnCode::new(e, 1))?;

    // 1. Perform catalog search or token resolution
    let download_url = format!(
        "https://api.myprovider.com/download?artist={}&title={}",
        urlencoding::encode(&query.artist),
        urlencoding::encode(&query.title)
    );

    // 2. Prepare headers (e.g., Bearer auth or cookies)
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Lyria-Extension/1.0".to_string());

    // 3. Construct response
    let result = DownloadSourceResult {
        download_url,
        codec: "flac".to_string(),
        bitrate_kbps: Some(1411),
        sample_rate_hz: Some(44100),
        bit_depth: Some(16),
        content_length_bytes: None,
        headers: Some(headers),
        cover_art_url: None,
        lyrics_lrc: None,
    };

    let output_json = serde_json::to_string(&result)
        .map_err(|e| WithReturnCode::new(e, 1))?;

    Ok(output_json)
}
```

---

## 5. Error Etiquette & Automatic Fallbacks

When your extension fails to resolve a download link, **fail fast with a clear error string**:

```rust
if track_not_found {
    return Err(WithReturnCode::new(
        extism_pdk::Error::msg("TRACK_NOT_FOUND: Song is not available in catalog"),
        2,
    ));
}
```

### Why Clear Errors Matter:
1. **Instant Fallback**: When your extension returns an error, Lyria's core engine catches it and **instantly queries the next enabled download extension** without delaying the user's queue.
2. **Circuit Breaker Protection**: If an extension encounters 3 consecutive network timeouts or 5xx server crashes, Lyria automatically trips its circuit breaker for 60 seconds. This prevents unresponsive extensions from stalling multi-track album batches.

---

## 6. Testing Your Extension in Lyria

1. Compile your WebAssembly binary:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```
2. Copy `plugin.wasm` and `manifest.json` into your local Lyria extensions directory:
   * **Linux/macOS**: `~/.local/share/Lyria/extensions/my-download-provider/`
   * **Windows**: `%APPDATA%\Lyria\extensions\my-download-provider\`
3. Open Lyria, navigate to **Settings > Extensions**, toggle your provider on, and initiate a track or album download!
