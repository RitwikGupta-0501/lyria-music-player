# Contributor & Developer Guide

This document is the canonical source of truth for configuring a local development environment, building, testing, linting, and contributing to the Lyria music player codebase.

---

## 1. The What

Lyria is a minimal, algorithmically-driven local audio player engineered as a hybrid desktop application:
* **Native Backend**: Written in Rust using Tauri 2.0, Tokio async runtime, dedicated OS audio threads (`rodio`/`symphonia`), embedded SQLite (`rusqlite`), and an embedded WebAssembly runtime (`extism`).
* **Frontend Webview**: Written in TypeScript using SvelteKit in SPA mode, powered exclusively by **Svelte 5 Runes** (`$state`, `$derived`, `$effect`, `$props`), and bundled with Vite and Bun.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 CORE DEVELOPER STACK                                   │
├──────────────────────────┬─────────────────────────────┬───────────────────────────────┤
│ BACKEND / DAEMON         │ FRONTEND / UI               │ TESTING & LINTING             │
├──────────────────────────┼─────────────────────────────┼───────────────────────────────┤
│ • Rust 1.78+ (2021 ed.)  │ • Bun 1.1+ (JS runtime/pkg) │ • `cargo clippy` (-D warnings)│
│ • Tauri 2.0 (IPC & OS)   │ • SvelteKit (SPA Mode)      │ • `cargo test` (63+ tests)    │
│ • Tokio (Async runtime)  │ • Svelte 5 (Runes only)     │ • `bun run check` (TypeScript)│
│ • Rodio + Symphonia      │ • Vanilla CSS + Design Tokens│ • `bun test` (Format/Utils)   │
│ • Extism WASM Engine     │ • Phosphor Icons            │ • `./scripts/benchmark-memory.sh`│
└──────────────────────────┴─────────────────────────────┴───────────────────────────────┘
```

---

## 2. The Why

### High-Fidelity Audio Demands Disciplined Engineering
Unlike standard web or desktop applications, an audio player operates under hard real-time latency constraints. A dropped buffer or an unhandled async block creates audible clicks or skips. Contributing to Lyria requires adhering to strict architectural boundaries:
1. **Frontend as Remote Control**: The Svelte 5 UI holds zero audio state and does not execute DSP or playback timers. It purely displays state and dispatches user actions.
2. **Zero Tokio Blocking**: All synchronous operations (SQLite database queries, Symphonia file decoding) are strictly forbidden on Tokio threads. They must run on dedicated OS threads or within `tokio::task::spawn_blocking`.
3. **Rigorous Quality Gates**: No code is merged without passing backend clippy lints, frontend typecheck, unit test suites, and empirical memory threshold assertions.

---

## 3. System Prerequisites

### 3.1 Linux (Debian / Ubuntu / Pop!_OS)
Install required C compilers, WebKitGTK 4.1, ALSA, and GStreamer development libraries:

```bash
sudo apt update && sudo apt install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libasound2-dev \
    libglib2.0-dev \
    gstreamer1.0-plugins-base \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev
```

### 3.2 Linux (Arch Linux)
```bash
sudo pacman -S --needed \
    base-devel \
    curl \
    wget \
    openssl \
    gtk3 \
    webkit2gtk-4.1 \
    libappindicator-gtk3 \
    librsvg \
    alsa-lib \
    glib2 \
    gstreamer \
    gst-plugins-base
```

### 3.3 macOS
1. Install **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```
2. Install **Homebrew** and dependencies:
   ```bash
   brew install pkg-config openssl
   ```

### 3.4 Windows
1. Install **Microsoft Visual Studio C++ Build Tools** (Select "Desktop development with C++").
2. Ensure **WebView2 Runtime** is installed (pre-installed on Windows 10/11).

### 3.5 Language Toolchains (All Platforms)
1. **Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   rustup component add clippy rustfmt
   ```
2. **Bun**:
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```

---

## 4. Repository Setup & Local Development

### 4.1 Clone and Install Dependencies
```bash
git clone https://github.com/RitwikGupta-0501/lyria-music-player.git
cd echo-desktop

# Install frontend dependencies
bun install
```

### 4.2 Running the Development Application

#### Option A: Full Native Desktop App (Recommended)
Launches the native compiled Rust backend alongside the Vite frontend hot-reload server:
```bash
bun run tauri dev
```

#### Option B: Frontend SPA Mode (Browser View Only)
For rapid visual styling and layout prototyping in standard web browsers (mocking IPC calls):
```bash
bun run dev
```

---

## 5. Quality Verification & Testing Pipeline

Before committing changes or opening a pull request, run the following verification steps:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              PRE-COMMIT VERIFICATION PIPELINE                          │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ 1. Backend Linting:   `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets`│
│ 2. Backend Tests:     `cargo test --manifest-path src-tauri/Cargo.toml`                │
│ 3. Frontend Typing:   `bun run check`                                                  │
│ 4. Frontend Tests:    `bun test`                                                       │
│ 5. Memory Benchmark:  `./scripts/benchmark-memory.sh --quick`                          │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.1 Backend Linting & Formatting
```bash
# Check code formatting
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Enforce zero clippy warnings across all targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

### 5.2 Backend Unit Tests
Executes the comprehensive 63+ unit test suite spanning the audio engine, queue state machine, SQLite deduplication, and Markov candidate scoring:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 5.3 Frontend Typecheck & Integrity
Validates Svelte 5 runes, props contracts, and TypeScript types:
```bash
bun run check
```

### 5.4 Frontend Unit Tests
Executes Svelte utilities and canonical string normalization tests:
```bash
bun test
```

### 5.5 Memory & Leak Assertion Check
Validates that changes adhere to the 40MB RAM budget and do not introduce heap leaks:
```bash
./scripts/benchmark-memory.sh --quick
```

---

## 6. Observability & Debugging Tools

### 6.1 Real-Time Diagnostic Console (`/debug`)
Lyria includes a dedicated, multi-window diagnostic terminal.
* **Opening the Console**: Navigate to `/debug` in the player or click **Settings > Advanced > Open Debug Console**.
* **Features**: Live streaming log feed, severity filters (`INFO`, `WARN`, `ERROR`, `DEBUG`), category segmentation (`Backend`, `Frontend`, `WASM`, `JS Sandbox`, `Audio`, `Database`, `Network`), pause/resume, and one-click clipboard copy.

### 6.2 Enabling Terminal Stdout
By default, terminal stdout logging is muted to protect terminal buffers and prevent audio thread blocking. To enable live stdout tracing in your terminal:
```bash
RUST_LOG_STDOUT=1 bun run tauri dev
```

### 6.3 Diagnostic Log Files
Rolling log files are automatically written to disk:
* **Linux**: `~/.local/share/com.ritwik.lyria/logs/echo.YYYY-MM-DD.log`
* **macOS**: `~/Library/Application Support/com.ritwik.lyria/logs/echo.YYYY-MM-DD.log`
* **Windows**: `%APPDATA%\com.ritwik.lyria\logs\echo.YYYY-MM-DD.log`

---

## 7. Troubleshooting Common Issues

| Issue / Error | Root Cause | Resolution |
| :--- | :--- | :--- |
| `pkg-config: webkit2gtk-4.1 not found` | Missing WebKitGTK dev packages on Linux | Install `libwebkit2gtk-4.1-dev` (Ubuntu/Debian) or `webkit2gtk-4.1` (Arch). |
| `ALSA lib pcm.c: No such device` | Missing or unconfigured audio sink in container/headless VM | Connect a virtual audio device or run PulseAudio/PipeWire daemon. |
| `SQLITE_BUSY: database is locked` | Concurrent write attempted outside `db_tx` actor channel | Ensure all mutations are routed through `state.db_tx.send(DbRequest::...)`. |
| `Extism plugin execution timeout` | WASM provider exceeded the 3,500ms execution budget | Inspect remote provider network latency or optimize plugin code. |
| `bun run check reports Svelte 5 syntax error` | Legacy Svelte 4 syntax (`let:`, `export let`) used | Refactor components to use Svelte 5 runes (`$state`, `$props`, `$derived`). |

---

## 8. Architectural Invariants for Contributors

1. **Frontend Hygiene**: The SvelteKit UI is strictly unstyled and purely structural. Do not introduce TailwindCSS, UnoCSS, or external component libraries without explicit architectural review.
2. **Database Actor Pattern**: Never execute direct SQLite queries inside async Tauri commands. Always use `db_tx` or `tokio::task::spawn_blocking` with `open_read_conn`.
3. **Audio Thread Isolation**: Never block the dedicated `rodio` OS thread. All state updates must occur via non-blocking `mpsc` channels.
4. **Memory Budget**: Never submit changes that increase idle anonymous memory beyond the thresholds defined in `benchmarks/thresholds.json`.
