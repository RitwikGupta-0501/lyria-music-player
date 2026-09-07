# Lyria Extension Manifest Schema & Packaging Specification

This document defines the schema for `manifest.json` (or `extension.json`), capability declarations, folder packaging requirements, and distribution mechanisms for Lyria extensions.

---

## 1. Extension Manifest Specification

Every Lyria extension must provide metadata via a JSON manifest named `manifest.json` (or `extension.json`) located in the root of the extension folder, or as a sibling `<id>.json` file for standalone `.wasm` files.

### 1.1 Complete Schema Example

```json
{
  "$schema": "https://raw.githubusercontent.com/echo-desktop/echo-desktop/main/docs/extensions/schema.json",
  "id": "youtube-wasm",
  "name": "YouTube Music",
  "author": "Ritwik Gupta",
  "version": "1.0.0",
  "description": "High-fidelity audio streaming and recommendation provider via YouTube Music.",
  "main": "youtube-wasm.wasm",
  "icon": "icon.png",
  "homepage": "https://github.com/RitwikGupta-0501/music-player",
  "capabilities": [
    "search",
    "warmup",
    "explore",
    "url_resolver",
    "telemetry_reporting"
  ],
  "settings_schema": "{\"api_key\": {\"type\": \"string\", \"label\": \"API Token\"}}"
}
```

### 1.2 Field Definitions

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | **Yes** | Unique identifier for the extension. Must be kebab-case or snake_case (e.g. `youtube-wasm`, `lrclib-lyrics`, `spotify-sync`). If omitted, falls back to the folder name or filename stem. |
| `name` | `string` | **Yes** | Human-readable display name shown in the UI settings and provider list (e.g. `"YouTube Music"`). |
| `author` | `string` | No | Author or organization name. Defaults to `"Unknown"`. |
| `version` | `string` | No | SemVer version string (e.g. `"1.0.0"`). Defaults to `"0.0.0"`. |
| `description` | `string` | No | Short synopsis of what the extension provides. |
| `main` | `string` | No | Path to the compiled WebAssembly binary relative to the extension folder. Defaults to looking for `main.wasm`, `plugin.wasm`, or `<id>.wasm`. |
| `icon` | `string` | No | Path to an icon file relative to the extension directory (PNG, SVG, or WebP) or an external HTTPS URL. |
| `homepage` | `string` | No | URL to documentation or project source code. |
| `capabilities` | `string[]` | **Yes** | List of functional capabilities this extension declares and implements. |
| `settings_schema`| `string` | No | Optional JSON schema definition for user-configurable settings rendered in the UI. |

---

## 2. Capability Definitions

Capabilities declare which APIs and behaviors the extension participates in. The Lyria daemon uses capabilities to build invocation pipelines and enforce sandbox security.

| Capability | Purpose | Sandbox / Runtime Behavior |
|---|---|---|
| `search` | Track and catalog search | Enables routing user queries to `search` and `search_categorized` export functions. |
| `warmup` | Session pre-heating | The host calls the extension's `warmup` entry point during application launch in the background. |
| `explore` | Browse & discovery feeds | The host registers the extension with the Home/Explore page module aggregator (`get_modules`, `fetch_module`, `browse_album`, `browse_artist`, `get_related`, `get_radio`). |
| `url_resolver` | Arbitrary URL translation | Allows the extension to intercept external URLs (e.g., pasted links) and resolve them to playable audio streams via `resolve_url`. |
| `telemetry_reporting` | Analytics & Scrobbling | **Security-critical**: Enables the `host_telemetry_request` host function. If an extension attempts to send telemetry without this capability, requests are blocked by the host with `PermissionDenied`. |
| `lyrics` *(planned)* | Synchronized / plain lyrics | Registers the provider for real-time and synced LRC lyrics resolution. |
| `equalizer_presets` *(planned)*| Parametric EQ profiles | Exposes custom DSP/EQ presets to the audio engine. |

> [!IMPORTANT]
> **Priority Governance**:
> Extension priorities are strictly governed by the user via the Settings & Extensions UI. Author-defined priority in manifests is disregarded to prevent third-party extensions from hijacking the playback pipeline.

---

## 3. Packaging & Folder Layout

Extensions are distributed in three supported formats:

### 3.1 Folder Distribution (Recommended for Development & Multi-Asset Extensions)
A folder placed directly inside the user's `extensions/` directory containing the manifest, compiled `.wasm`, and asset files:

```
Lyria/extensions/
└── youtube-wasm/
    ├── manifest.json
    ├── youtube-wasm.wasm
    ├── icon.png
    └── README.md
```

* Rules:
  * The Lyria daemon performs **bounded 1-depth directory traversal**. Nested subdirectories within `extensions/` will not be scanned recursively to prevent directory traversal attacks or symlink loops.
  * The `main` field in `manifest.json` must be a relative path pointing inside the folder (e.g. `"main": "youtube-wasm.wasm"`).

### 3.2 Zip Archive Distribution (`.zip`)
Packages can be dragged and dropped directly onto the Extensions view in the Lyria UI, or imported via the file picker:

```
youtube-wasm.zip
├── manifest.json
├── youtube-wasm.wasm
└── icon.png
```

* **Zip Slip Protection**: The native Rust extractor sanitizes all zip entry paths using `.enclosed_name()` to prevent path traversal outside the target directory.
* If the zip contains files at the root level without an enclosing folder, the host automatically wraps them in a directory named after the zip file stem.

### 3.3 Standalone `.wasm` + Companion `.json`
For simple single-binary extensions:

```
Lyria/extensions/
├── simple-search.wasm
└── simple-search.json
```

---

## 4. Host Installation Paths

Extensions reside in the platform-standard Lyria data directory:

* **Windows**: `%APPDATA%\Lyria\extensions\` (e.g. `C:\Users\<User>\AppData\Roaming\Lyria\extensions\`)
* **Linux**: `~/.local/share/Lyria/extensions/` (or `$XDG_DATA_HOME/Lyria/extensions/`)
* **macOS**: `~/Library/Application Support/Lyria/extensions/`

Users can easily access this directory using the **"Open Folder"** button in the Lyria Extensions Settings panel.
