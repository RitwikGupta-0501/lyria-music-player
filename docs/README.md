# Lyria Documentation Hub

Welcome to the canonical engineering and architectural documentation suite for the **Lyria Music Player**.

This documentation hub is grounded strictly in the implemented codebase (Rust, Tauri 2.0, Svelte 5, SQLite, and Extism WebAssembly), providing a comprehensive, cross-referenced reference manual for core contributors, extension authors, and performance engineers.

---

## 🗺️ Documentation Architecture Map

```
docs/
├── README.md                      # Central Documentation Index & Navigation Map (This file)
├── VERSION_SCOPE.md               # Version Release Scope & Milestone Boundaries
├── TODO.md                        # Active Product Backlog & Technical Debt
│
├── architecture/                  # Deep Subsystem Specifications (Grounded in Code)
│   ├── README.md                  # High-Level Architectural Topology & System Invariants
│   ├── audio-engine.md            # Audio OS Thread, Rodio Sink, Symphonia Decoders & Rubato Resampling
│   ├── database.md                # SQLite Single-Writer Actor, Concurrent WAL Reads & Deduplication
│   ├── queue-system.md            # Queue State Machine, Shuffle Order & 35+ Playback Edge Cases
│   ├── core-loop.md               # Infinite Playback Loop (Markov Walk & Federated Radio Fallback)
│   ├── discovery.md               # Editorial Spotlight Hero & Adjacent Horizons Harmonic Contrast
│   ├── observability.md           # Real-Time 4-Tier Diagnostic Pipeline (/debug) & Logging Rules
│   └── os-integration.md          # Desktop Media Controls (souvlaki: MPRIS/SMTC/NowPlaying) & Keyring
│
├── design/                        # Visual Identity & "Quiet Luxury" UX Design System
│   ├── README.md                  # Design System Foundation & Component Principles
│   ├── visual-language.md         # Physical Audio Hardware Metaphor, Quiet Luxury & 5 Materials
│   ├── color-system.md            # Obsidian Surfaces, Chalk Text Tokens & Acoustic Brass Accents
│   ├── typography.md              # Tripartite Font Hierarchy (Newsreader, Inter Tight, IBM Plex Mono)
│   ├── motion.md                  # Damped Physics Micro-Animations & Fitts's Law Seek Scrubbing
│   ├── components.md              # Component Anatomy: PlayerBar, TrackRow, PillButton & Drawers
│   └── accessibility.md           # Keyboard-First Navigation, Focus Management & Screen Readers
│
├── extensions/                    # Sandboxed WebAssembly (WASM) Extension System
│   ├── README.md                  # Extension Ecosystem Architecture & Security Sandbox
│   ├── ABI_SPECIFICATION.md       # Extism WASM ABI v1, Memory Bounding & Host Functions
│   ├── MANIFEST_SCHEMA.md         # manifest.json Schema & Permission Capabilities
│   └── STARTER_TEMPLATES.md       # Polyglot Starter Templates (Rust & AssemblyScript)
│
├── development/                   # Developer Workflows, Quality Gates & Benchmarking
│   ├── guide.md                   # Contributor Onboarding: Setup, Building, Testing & Linting
│   └── benchmarking.md            # Performance Runbook: Scenarios S1–S8, Threshold Gates & Leaks
│
└── research/                      # Archived Research Spikes & Performance History
    ├── metrolist-analysis.md      # Metrolist Streaming & Anonymous Recommendation Spike
    └── performance.md             # Algorithmic Queue Latency & Historical Resource Benchmarks
```

---

## 📚 Section Directory

### 1. [Architecture Specifications](architecture/README.md)
Detailed technical breakdowns of Lyria's core desktop daemon:
* **[System Topology & Invariants](architecture/README.md)**: Process boundaries, Tokio threading model, Extism sandbox isolation, and memory accounting rules.
* **[High-Fidelity Audio Engine](architecture/audio-engine.md)**: Dedicated background OS thread, lock-free MPSC command/event channels, in-memory 4-second ring buffers, `stream-download` HTTP buffering, gapless EOF handoff, and asynchronous seek generation counters.
* **[Database & Concurrency](architecture/database.md)**: Dedicated single-writer actor channel (`db_tx`), concurrent read isolation in WAL mode, schema migrations, and continuous Gaussian deduplication scoring.
* **[Queue State Machine](architecture/queue-system.md)**: Thread-safe `Mutex<QueueState>`, non-destructive shuffle state machines, 3-second previous replay threshold, and exhaustive test matrix for 35+ verified playback edge cases.
* **[Infinite Core Loop](architecture/core-loop.md)**: Dual-tier autoplay waterfall (3500ms timeout), local Markov random walk candidate scoring, 120-minute recency exclusion gates, and Boltzmann Softmax temperature sampling.
* **[Discovery Subsystem](architecture/discovery.md)**: Editorial Spotlight hero billboard on `/explore` with zero-CLS skeleton loading, and Adjacent Horizons harmonic contrast engine on `/`.
* **[Observability Pipeline](architecture/observability.md)**: 4-tier telemetry ingestion (Rust tracing, Extism plugins, frontend console, JS sandbox), circular in-memory buffer, 2 Hz batched IPC emitter, and `/debug` console.
* **[OS Integration](architecture/os-integration.md)**: Native desktop media keys via `souvlaki` (Linux MPRIS, Windows SMTC with native HWND, macOS NowPlaying), platform keychain credential vaults, and window lifecycle.

---

### 2. [Design System Suite](design/README.md)
The visual and interactive philosophy governing Lyria's UI:
* **[Visual Language](design/visual-language.md)**: The physical audio hardware metaphor, tactile feedback, and 5 foundational material finishes.
* **[Color System](design/color-system.md)**: Obsidian dark palette, semantic text tokens, and acoustic brass accents.
* **[Typography](design/typography.md)**: The tripartite font stack combining Newsreader, Inter Tight, and IBM Plex Mono.
* **[Motion & Micro-Interactions](design/motion.md)**: Non-linear spring dynamics, easing curves, and seek-bar scrubbing physics.
* **[Component Anatomy](design/components.md)**: Structural specifications for PlayerBar, TrackRow, AlbumCard, PillButton, and slide-over drawers.
* **[Accessibility & Keymaps](design/accessibility.md)**: Global keyboard shortcuts, WCAG AA compliance, and focus ring protocols.

---

### 3. [WASM Extensions](extensions/README.md)
The plugin runtime empowering modular community features:
* **[Extension Overview](extensions/README.md)**: Sandboxed WebAssembly execution via Extism, memory caps, and zero-privilege defaults.
* **[ABI Specification](extensions/ABI_SPECIFICATION.md)**: Binary interfaces, host function bindings (`host_log`, `host_http_request`), and data serialization contracts.
* **[Manifest Schema](extensions/MANIFEST_SCHEMA.md)**: Capability declarations, metadata schemas, and module types.
* **[Starter Templates](extensions/STARTER_TEMPLATES.md)**: Ready-to-compile boilerplate extensions in Rust and AssemblyScript.

---

### 4. [Development & Benchmarking](development/guide.md)
Onboarding runbooks and automated verification tools:
* **[Contributor & Developer Guide](development/guide.md)**: Host dependencies, local development commands (`bun run tauri dev`), and mandatory pre-commit verification pipelines (`cargo clippy`, `cargo test`, `bun run check`, `bun test`).
* **[Benchmarking Runbook](development/benchmarking.md)**: Automated resource verification via `scripts/benchmark_memory.py`, scenario lifecycle (S1–S8), and threshold assertion gates (`benchmarks/thresholds.json`).

---

### 5. [Research & Performance](research/performance.md)
Archived architectural research and empirical performance records:
* **[Metrolist Architectural Analysis](research/metrolist-analysis.md)**: Reverse-engineered streaming resiliency, PoToken/BotGuard attestation, and anonymous visitor tracking.
* **[Performance Telemetry](research/performance.md)**: Microbenchmark latency records for queue operations ($\mathcal{O}(1)$ skips) and resource telemetry graphs across playback lifecycles.

---

## ⚡ Quick Verification Cheat-Sheet

Before opening a pull request, ensure all local verification checks pass:

```bash
# 1. Backend Linting & Integrity
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

# 2. Frontend Typing & Tests
bun run check
bun test

# 3. Memory & Resource Assertion
./scripts/benchmark-memory.sh --quick
```
