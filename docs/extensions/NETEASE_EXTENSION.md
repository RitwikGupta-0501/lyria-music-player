# NetEase Lyrics Extension

The complete documentation and development guide for the **NetEase Cloud Music WebAssembly Extension** can be found at:

👉 [extensions/netease-wasm/README.md](../../extensions/netease-wasm/README.md)

### Quick Summary
- **Language**: TypeScript / AssemblyScript (`asc`)
- **Plugin ID**: `netease-wasm`
- **Capability**: `lyrics`
- **Priority**: `9`
- **Binary Size**: ~41 KB
- **API Endpoints**:
  - Suggest / Search: `https://music.163.com/api/search/suggest/web`
  - Lyric Fetch: `https://music.163.com/api/song/lyric`
- **Exported Function**: `get_lyrics`
- **Host Functions Used**: `host_http_request` (HTTP GET requests)
- **Status**: Live tested with synchronized cue resolution in Lyria Rust engine.
