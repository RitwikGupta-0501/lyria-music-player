# LRCLIB Lyrics Extension

The complete documentation and development guide for the **LRCLIB WebAssembly Extension** can be found at:

👉 [extensions/lrclib-wasm/README.md](../../extensions/lrclib-wasm/README.md)

### Quick Summary
- **Language**: TypeScript / AssemblyScript (`asc`)
- **Plugin ID**: `lrclib-wasm`
- **Capability**: `lyrics`
- **Priority**: `10`
- **Binary Size**: ~35 KB
- **API Endpoints**:
  - Exact Lookup: `https://lrclib.net/api/get`
  - Fallback Search: `https://lrclib.net/api/search`
- **Exported Function**: `get_lyrics`
- **Host Functions Used**: `host_http_request` (HTTP GET requests)
- **Status**: Live tested with synchronized cue resolution in Lyria Rust engine.
