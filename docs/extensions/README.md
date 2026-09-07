# Lyria Extension System Documentation

Welcome to the Lyria Extension developer documentation. Lyria's architecture is powered by sandboxed Extism WebAssembly modules that allow developers to integrate new music streaming providers, search engines, catalog exploration feeds, lyrics sources, and audio algorithms.

## Documentation Index

1. **[Extism WASM ABI v1 Specification & Host Functions](ABI_SPECIFICATION.md)**
   * Extism execution sandbox & WASI environment
   * Complete host function signatures (`host_log`, `host_http_request`, `host_storage_get`, `host_storage_set`, `host_execute_webview_js`, `host_telemetry_request`)
   * Plugin export entry points (`search`, `search_categorized`, `resolve`, `resolve_url`, `get_modules`, `fetch_module`, `browse_album`, `browse_artist`, `get_related`, `get_radio`, `on_playback_event`)
   * JSON data models and type schemas

2. **[Manifest Schema & Packaging Requirements](MANIFEST_SCHEMA.md)**
   * `manifest.json` schema and field references
   * Capability definitions and runtime enforcement (`search`, `warmup`, `explore`, `url_resolver`, `telemetry_reporting`)
   * Folder layouts, Zip Slip safe extraction, and installation paths

3. **[Boilerplate Starter Templates](STARTER_TEMPLATES.md)**
   * Rust starter template (`extism-pdk`, `serde`, `wasm32-wasip1`)
   * AssemblyScript / TypeScript starter template
   * Step-by-step build and installation walkthrough
