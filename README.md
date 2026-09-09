# Lyria Music Player

<div align="center">

**A minimal, algorithmically-driven local audio player with sandboxed WebAssembly extensions.**

[![CI](https://github.com/RitwikGupta-0501/lyria-music-player/actions/workflows/release.yml/badge.svg)](https://github.com/RitwikGupta-0501/lyria-music-player/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-amber.svg)](https://opensource.org/licenses/MIT)
[![Tauri 2.0](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://v2.tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-orange.svg)](https://svelte.dev/)
[![Extism WASM](https://img.shields.io/badge/Extism-WASM%20ABI%20v1-purple.svg)](https://extism.org/)

</div>

---

## Overview

Lyria is a desktop music player built for listeners who value deterministic performance, algorithmic music discovery, and privacy. The application couples a dedicated lock-free Rust audio engine with an embedded **Extism WebAssembly (WASM)** extension sandbox and a responsive Svelte 5 frontend.

Rather than relying on manual playlist curation or heavy web views, Lyria centers on an **infinite core playback loop**: play any seed track, and the daemon dynamically resolves upcoming music via local Markov random walks or sandboxed WebAssembly extensions querying remote APIs.

---

## Architectural Highlights

### 1. Dedicated Audio Engine & Thread Isolation
- **Non-blocking OS thread:** The `rodio` audio sink and event loop operate on a dedicated OS thread outside the Tokio async runtime.
- **Lock-free message passing:** Playback controls (play, pause, seek, volume, queue transitions) communicate strictly via non-blocking `std::sync::mpsc` channels without mutex contention.
- **Symphonia decoding:** High-fidelity decoding support for **FLAC, MP3, WAV, OGG, M4A, and OPUS**.
- **Frontend interpolation:** Continuous playback time is interpolated at 60fps on the frontend using `requestAnimationFrame`, throttling Tauri IPC progress updates to ≤ 4Hz to keep the bridge lean.

### 2. Extism WebAssembly Extension Sandbox
- **Secure sandbox:** Embedded Extism WASM runtime executing community extensions with strict memory limits (40MB RAM budget) and execution timeouts.
- **Host capabilities:** Extensions can access async HTTP (`reqwest`) and a sandboxed QuickJS (`rquickjs`) JavaScript VM for dynamic client attestation (e.g. BotGuard attestation and PO token minting for YouTube Music).
- **Extensible ABI v1:** Standardized JSON-based ABI supporting track search, metadata scraping, URL streaming resolution, category browsing, and radio generation.
- **Safe import pipeline:** Single-click dropdown and drag-and-drop import supporting `.wasm`, `.zip` archives, and unpackaged folders with Zip Slip protection.

### 3. Dual-Mode Infinite Core Loop
- **Tier 1 — Federated WASM Radio:** When playback nears queue completion, the background watcher invokes active extensions (`get_radio` / `get_related`) to dynamically compile candidate tracks matching the seed's acoustic space.
- **Tier 2 — Local Markov Random Walk:** If offline or if remote streams fail, Lyria executes a Markov random walk across your local library using tag distance metrics and a 2-hour recency penalty to avoid repetitive playback.
- **Tier 3 — Graceful Exhaustion:** Seamless fallback to idle state when the library and remote providers are exhausted.

### 4. Real-Time Observability & Diagnostics (v0.2.0)
- **Standalone Debug Console (`/debug`):** Dedicated diagnostic window featuring live 2Hz batch streaming from a 2,000-entry in-memory ring buffer.
- **4-Tier Log Aggregation:**
  - *Rust Backend:* Custom `tracing_subscriber::Layer` (`BufferLogLayer`) capturing engine, database, and audio thread events.
  - *WASM Plugins:* Extension logs bridged directly under the `WASM` category via `host_log`.
  - *SvelteKit Frontend:* Non-blocking, recursion-safe telemetry hook in `app.html` capturing uncaught window exceptions and console diagnostics.
  - *JS Sandbox VM:* VM evaluation lifecycle, execution timeouts, and CSP violation telemetry.
- **Diagnostic Controls:** Real-time pause/resume, search filtering, origin badge filtering, severity toggles (INFO, WARN, ERROR, DEBUG), auto-scroll square checkbox, buffer clearing, and formatted clipboard export.
- **Silent Terminal by Default:** Raw terminal stdout is silenced to keep terminal output clean, with optional override via `RUST_LOG_STDOUT=1`.

### 5. Desktop & OS Integration
- **Native Media Controls:** Integrated OS media controls via `souvlaki` (MPRIS on Linux, SMTC on Windows, hardware media keys, Now-Playing sync).
- **Universal AppImage Packaging:** Automated CI sanitization pipeline in release workflows that purges host-conflicting display and graphics drivers (`libwayland-client.so*`, `libEGL.so*`, etc.), ensuring seamless Wayland EGL negotiation across all distributions.
- **Arch Linux Support:** Native Arch package (`.pkg.tar.zst`) release assets and `PKGBUILD`.

---

## Tech Stack

| Layer | Technologies |
|---|---|
| **Frontend** | SvelteKit (SPA Mode), Svelte 5 (Runes: `$state`, `$derived`, `$effect`), TypeScript, Vite, Phosphor Icons |
| **Backend** | Rust, Tauri 2.0, Tokio (async I/O), dedicated OS thread (audio) |
| **Audio Engine** | `rodio`, `symphonia`, `souvlaki` (MPRIS/SMTC) |
| **Database** | Embedded SQLite (`rusqlite`) via dedicated database actor thread |
| **Extension Sandbox** | Extism WASM runtime, `rquickjs` (QuickJS VM) |
| **Tooling & Runtime** | Bun, Cargo, Clippy |

---

## Project Structure

```
lyria-music-player/
├── src/                          # SvelteKit frontend (SPA)
│   ├── app.css                   # Design tokens & typography
│   ├── app.html                  # HTML shell & frontend telemetry hook
│   ├── lib/
│   │   ├── components/           # Svelte 5 UI components
│   │   │   ├── explore/          # Explore feed, search results, spotlight
│   │   │   ├── settings/         # Playback, appearance, advanced settings
│   │   │   ├── AlbumGrid.svelte  # Virtualized album collection
│   │   │   ├── PlayerBar.svelte  # Persistent bottom playback controls
│   │   │   └── TrackRow.svelte   # Virtualized track item
│   │   ├── stores/               # Svelte 5 rune stores
│   │   │   ├── audio.svelte.ts   # Audio engine & queue bridge
│   │   │   ├── library.svelte.ts # SQLite cache & local collections
│   │   │   ├── explore.svelte.ts # Dynamic feed & remote browsing
│   │   │   ├── settings.svelte.ts# Persistent user preferences
│   │   │   └── toast.svelte.ts   # Notification toasts
│   │   └── utils/
│   │       └── format.ts         # Canonical normalization & formatting
│   └── routes/
│       ├── +page.svelte          # Main player interface
│       └── debug/
│           └── +page.svelte      # Standalone Real-Time Debug Console
├── src-tauri/                    # Rust backend & Tauri 2.0 daemon
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs               # Desktop entry point
│   │   ├── lib.rs                # IPC commands & application state
│   │   ├── logger.rs             # 4-tier ring buffer & tracing subscriber
│   │   ├── audio/                # Dedicated OS thread audio engine
│   │   ├── db/                   # Database actor & SQLite queries
│   │   ├── providers/            # Extism WASM runner & recommendations
│   │   └── queue/                # Queue state machine & autoplay engine
│   └── permissions/              # Tauri capability & security manifests
├── extensions/                   # First-party WebAssembly extensions
│   └── youtube-wasm/             # YouTube Music extension with BotGuard
├── tests/                        # Unit and integration test suites
├── docs/                         # Specifications & extension documentation
│   ├── extensions/               # ABI spec, manifest schema, starter templates
│   ├── VERSION_SCOPE.md          # Version release roadmap
│   └── TODO.md                   # Feature tracking
└── static/                       # Static assets & JS sandbox shell
```

---

## Getting Started

### Prerequisites

- **Rust:** Stable toolchain ([rustup.rs](https://rustup.rs/))
- **Bun:** JavaScript runtime & package manager ([bun.sh](https://bun.sh/))
- **System Audio Libraries:**
  - **Linux:** `libasound2-dev`, `libssl-dev`, `pkg-config`
  - **macOS:** CoreAudio (included with Xcode tools)
  - **Windows:** No external audio dependencies

### Installation & Development

```bash
# Clone the repository
git clone https://github.com/RitwikGupta-0501/lyria-music-player.git
cd lyria-music-player

# Install frontend dependencies
bun install

# Launch desktop app in development mode
bun run tauri dev
```

---

## Build, Test & Lint Commands

| Command | Purpose |
|---|---|
| `bun run dev` | Launch Vite frontend development server |
| `bun run tauri dev` | Full desktop development mode (SvelteKit + Rust) |
| `bun run check` | Validate Svelte 5 types and template diagnostics |
| `bun test` | Execute frontend unit test suites |
| `cargo clippy --all-targets -- -D warnings` | Backend Rust linting (zero-warnings policy) |
| `cargo test` | Execute Rust unit and integration tests |
| `bun run build` | Compile optimized production web bundle |
| `bun run tauri build` | Package standalone native binary releases |

---

## Extension Authoring

Lyria can be extended by compiling any language (Rust, AssemblyScript, C, Zig) into WebAssembly targeting the Extism runtime.

- [Extism WASM ABI v1 Specification](docs/extensions/ABI_SPECIFICATION.md): Export hooks, host functions, and memory transport schemas.
- [Manifest Schema & Packaging](docs/extensions/MANIFEST_SCHEMA.md): `manifest.json` requirements and folder distribution guidelines.
- [Starter Templates](docs/extensions/STARTER_TEMPLATES.md): Boilerplate templates in Rust and TypeScript.

---

## Release Roadmap

- **v0.1.0 — Architecture Foundation & Extensibility (Shipped ✅)**
  - Dedicated lock-free audio thread, SQLite database actor, Extism WASM runtime, dual-mode infinite loop, native OS media keys (Souvlaki).
- **v0.2.0 — Observability, Diagnostics & Platform Hardening (Shipped ✅)**
  - Standalone 4-tier Debug Console (`/debug`), unified logging, muted terminal stdout, universal AppImage Wayland sanitization, album deduplication, and dynamic provider fallbacks.
- **v0.3.0 — Context Controls, Audio Polish & Usability (Planned 🔵)**
  - Global right-click context menus, 3-dot action buttons, gapless crossfade, ReplayGain volume normalization, sleep timer, system tray, and search slash commands.
- **v0.4.0 — Rich Media, Lyrics & Discovery Depth (Planned 🔵)**
  - Synchronized `.lrc` scrolling lyrics, dynamic adjacent horizons discovery, and virtualized infinite scroll.
- **v0.5.0 — Power User & Library Mastery (Planned 🔵)**
  - Automatic filesystem watcher, batch tag editor, multi-select queue actions, and smart shuffle.
- **v1.0.0 — Production Extensibility (General Availability 🚀)**

---

## License

This project is licensed under the [MIT License](LICENSE).
