# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-09-10

### 📦 Packaging & Distribution
* **Linux AppImage Display Sanitization**:
  - Restored AppImage post-build sanitization step in release workflow to scrub host-conflicting display and graphics libraries (`libwayland*`, `libEGL*`, `libGL*`, `libgbm*`, `libdrm*`).
  - Resolves `Could not create default EGL display: EGL_BAD_PARAMETER` startup crashes on Wayland and NVIDIA systems (CachyOS/Arch, Fedora, openSUSE, Ubuntu 24.04+).

### ⚙️ Continuous Integration & Quality Gates
* Added Apple Silicon macOS (`macos-14`) job to CI workflow to maintain warm build caches across all platforms.
* Added automated test suite runs (`bun test`, `cargo test`) to CI pipeline.
* Configured smart concurrency cancellation for PRs while ensuring cache persistence on `main`.

---

## [0.2.1] - 2026-09-10

### 📦 Packaging & Distribution
* **Native macOS Support (`.dmg` & `.app`)**:
  - Added automated GitHub Actions release workflow for Apple Silicon macOS (`macos-14`, `aarch64-apple-darwin`).
  - Configured DMG disk image packaging with customized installer window dimensions.
  - Documented Gatekeeper security bypass procedures (`xattr -cr /Applications/Lyria.app`) for open-source distributions.

### ⚖️ Licensing
* Aligned all repository manifests (`package.json`, `PKGBUILD`, release workflows, and README) to **GNU General Public License v3.0 (GPL-3.0)**, matching root `LICENSE`.

---

## [0.2.0] - 2026-09-09

### 🚀 Observability & Diagnostics
* **Dedicated Real-Time Debug Console (`/debug`)**:
  - 4-tier log aggregation capturing Rust backend tracing (`tracing-subscriber`), WASM plugin logs, frontend console telemetry, and sandboxed JS attestation events.
  - Native multi-window architecture with high-visibility origin badges (`backend`, `wasm`, `frontend`, `sandbox`) and distinct error/warning row accenting.
  - Discarded legacy development `#debug-overlay` DOM injection in favor of native telemetry window.
  - Terminal stdout muted by default in production, gated behind `RUST_LOG_STDOUT=1`.

### 🛠️ Fixes & Improvements
* **Library Deduplication**:
  - Fixed duplicate album rendering when favoriting local albums via canonical normalization (`normalizeCanonicalString`, `getCanonicalKey`, `isCanonicalEntityMatch`).
* **Configurable Provider Fallbacks**:
  - Eliminated hardcoded `'youtube-wasm'` fallbacks across Explore feeds, Search, Queue, and Recommendation actors.
  - Dynamic fallback resolution to user's configured default remote provider (`settingsStore.getEffectiveRemoteProvider()`).
* **Universal Linux Packaging Hardening**:
  - Sanitized AppImage release pipeline by excluding bundled `libwayland-client.so*` and host GPU drivers to prevent Wayland/EGL conflicts.

### 📚 Documentation & Governance
* Re-anchored project README around a clean, premium, lightweight, offline-first desktop music player identity.
* Consolidated agent governance and system rules into a unified `AGENTS.md`.
* Documented native Windows (`.exe`, `.msi`) and Linux distribution channels.

---

## [0.1.1] - 2026-09-07

### 🚀 Features & Enhancements
* **Extension Import Pipeline**:
  - Safe archive extraction with Zip Slip path-traversal protection.
  - Standardized application data path to `Lyria/extensions` with automated migration from legacy paths.
  - 1-depth subdirectory manifest inspection (`manifest.json` / `extension.json`).
  - WebKitGTK UI freeze fix by offloading archive extraction and disk I/O to background blocking worker threads.
  - Unified import dropdown and full-screen drag-and-drop overlay with zero layout shift.
* **Extension Details & Descriptions**:
  - Added extension descriptions across SQLite schema, migrations, IPC commands, and frontend details view.
  - Toast notification system for import status, sync events, deletion, and settings updates.
  - Adaptive scrolling for short viewports and tab management.

### 📦 Packaging & Distribution
* **Arch Linux Support**: Added native Arch Linux `.pkg.tar.zst` release package generation and repository `PKGBUILD` for AUR distribution.

### 📚 Documentation & Specifications
* Added Extism WASM ABI v1 specification (`docs/extensions/ABI_SPECIFICATION.md`).
* Added Manifest JSON schema and packaging requirements (`docs/extensions/MANIFEST_SCHEMA.md`).
* Added extension boilerplate starter templates for Rust and AssemblyScript (`docs/extensions/STARTER_TEMPLATES.md`).
* Reorganized project roadmap into version scopes (`docs/VERSION_SCOPE.md` and `docs/TODO.md`).

---

## [0.1.0] - 2026-09-06

### 🚀 Features
* Dedicated non-blocking OS audio thread running `rodio` with lock-free `mpsc` message passing.
* SQLite database layer isolated with synchronous actor pattern and connection isolation.
* Embedded Extism WebAssembly sandbox with async HTTP (`reqwest`) and quickjs attestation capabilities.
* YouTube WASM extension (`WEB_REMIX`) with dynamic BotGuard attestation and PO token minting.
* Dual-mode infinite core loop: Local Markov random walk and federated WASM radio replenishment (`get_radio`).
* Native OS media controls via `souvlaki` (MPRIS on Linux, SMTC on Windows, hardware media keys).
* Virtualized collection drawer and real-time Liked Songs telemetry.
