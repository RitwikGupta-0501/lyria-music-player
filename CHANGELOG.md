# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
