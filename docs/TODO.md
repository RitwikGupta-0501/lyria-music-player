# TODO

> For completed features, fixes, and release history, see [CHANGELOG.md](../CHANGELOG.md).

## Audio Engine & Playback Polish
- [ ] Handle audio hardware disconnection & device drop (detect USB DAC / Bluetooth drop, transition to Paused, and notify frontend)
- [ ] Fix in-flight seek vs. track load race condition (`AudioCommand::Load` must increment `seek_generation` in `src-tauri/src/audio/mod.rs`)
- [ ] Wire up terminal `track-ended` event in frontend or emit `player-sync` Stopped upon queue exhaustion
- [ ] Implement auto-advance on corrupted / deleted audio paths in `SymphoniaSource::from_path`
- [ ] Implement comprehensive audio subsystem unit tests (benchmarking is not equivalent to functional tests; test decoding, seeking, underruns, and gapless callbacks in `src-tauri/src/audio`)
- [ ] Implement gapless playback and configurable crossfade (1-5s) between consecutive tracks
- [ ] Implement ReplayGain / volume normalization (EBU R128 / LUFS standard) to prevent loudness jumps
- [ ] Implement audio output device selector in Settings (DAC, Bluetooth headphones, speakers)
- [ ] Implement sleep timer (stop after N minutes or at the end of current track)
- [ ] Implement audio format and quality badging (e.g. FLAC 24-bit/96kHz, MP3 320kbps, OPUS) on tracks and player bar

## Desktop & OS Integration
- [ ] Set up macOS Homebrew Cask recipe (`brew install --cask lyria`)
- [ ] Implement system tray integration with minimize-to-tray, close-to-tray, and quick playback controls
- [ ] Implement Discord Rich Presence integration (native or extension-driven, configurable in Settings)

## Library & Queue Management
- [ ] Implement automatic filesystem watcher for music folders (background auto-indexing of newly added files)
- [ ] Implement multi-select and batch actions (Shift/Ctrl-click to queue, add to playlist, or delete)
- [ ] Fix `RepeatMode::One` auto-advance loop bug in `QueueState` (`src-tauri/src/queue/mod.rs`):
  - In `next_normal()` and `next_shuffle()`, evaluate `RepeatMode::One` before incrementing position to ensure tracks repeat anywhere in the queue, not just at queue terminus
- [ ] Fix manual "Next" replaying last track on `RepeatMode::Off` (`src/lib/stores/audio.svelte.ts` / `commands.rs`)
- [ ] Implement the 3-Second "Previous" Rule (restart if > 3s, skip previous if <= 3s)
- [ ] Wire up runtime queue state persistence & background snapshotting:
  - Connect runtime queue mutations (`set_queue`, `add_to_queue`, `remove_from_queue`, reorder, shuffle) to debounced SQLite persistence using `queue::recovery::periodic_save` or `DbRequest::SaveQueueState`
  - Ensure playback queue, shuffle order, and cursor position survive application restarts and sudden crashes as specified in Phase 5 persistence design
  - Implement SQLite persistence unit/integration tests in `src-tauri/src/queue/persistence.rs`
- [ ] Synchronize `shuffle_state.order` when tracks are reordered during active shuffle mode
- [ ] Deterministic Seeded PRNG for Shuffle: Use `rand::rngs::StdRng::seed_from_u64(seed)` rather than unseeded `thread_rng`
## Infinite Core Loop & Autoplay
- [ ] Migrate Autoplay Core Loop Orchestration from Frontend Store (`audio.svelte.ts`) to Rust Daemon to preserve the "Frontend as Pure Remote Control" architectural invariant (ARCH-01)
- [ ] Fix local seed duration proximity calculation bypass in autoplay (`src-tauri/src/queue/autoplay.rs:18-25`, `src-tauri/src/db/queries.rs:1981-1992`):
  - In `autoplay.rs`, `TrackSourceInfo::Local` sets `seed_duration_ms` to `None`, bypassing the duration proximity formula and always awarding the fallback score of 5.0
- [ ] Fix WASM capability mismatch in federated radio fallback (`src-tauri/src/providers/recommendations.rs:420-442`, `src-tauri/src/providers/mod.rs:948`):
  - When falling back to `"related"` providers, `p_mgr.get_radio` invokes the `"get_radio"` WASM export, failing if the extension only exports `"get_related"`
- [ ] Reconcile conflicting radio timeout budgets:
  - Inner per-provider task timeout is 4000ms while outer autoplay timeout in `autoplay.rs` is 3500ms; adjust inner timeout to <= 3000ms so individual timeouts fire before the batch deadline
- [ ] Implement unit test suite for `queue::autoplay` and Markov candidate scoring (`autoplay.rs`, `db::queries.rs`):
  - Unit tests for waterfall fallback, remote deduplication, provider fallback selection, recency gates, 3-track library size relaxation, affinity vector weights, and Boltzmann temperature bounds
- [ ] Reconcile code comment (`// 3. Genre / Lexical Affinity (weight 30)`) with implemented scoring weight (+15.0) in `src-tauri/src/db/queries.rs:1969`

## Display Modes
- [ ] Implement compact floating mini-player mode (always-on-top picture-in-picture widget)
- [ ] Implement immersive full-screen / cinematic view (large album artwork, synced lyrics, ambient background)

## Discovery, Search & Recommendations
- [ ] Implement dynamic Adjacent Horizons engine:
  - Dynamically invert 7-day SQLite listening habits to pair unlistened genres with dominant artist styles
  - Fetch 3 real, dynamic preview tracks (artwork, duration, IDs) with streaming/local fallbacks instead of static definitions
- [ ] Implement infinite scroll pagination for Explore feed and Search results:
  - Consume `continuation_token` from providers and browse endpoints
  - Implement virtualized scroll-boundary prefetching (~300px before edge)
- [ ] Extend Global Search (<kbd>Ctrl+K</kbd> / <kbd>Cmd+K</kbd>) into full Command Palette:
  - Add quick slash commands (`/play`, `/shuffle`, `/liked`, `/queue`, `/clear`)
  - Support instant command shortcuts and direct navigation

## Context Menus & Action Controls
- [ ] Implement global custom right-click context menu system (tracks, albums, artists, playlists, and queue)
- [ ] Add 3-dot more options button (DotsThreeVertical) to TrackRow, QueueSidebar, AlbumCard, and TrackCard:
  - Play Next
  - Add to Queue
  - Add to Playlist (with playlist picker sub-menu)
  - Go to Album / Go to Artist
  - Toggle Like / Favorite
  - Show in File Explorer (for local tracks)
  - Remove from Queue (queue-specific action)

## Extension Capabilities & System
- [ ] Implement user-governed extension priorities (reorderable list in Settings/Extensions UI, stored in DB)
- [ ] Implement multi-tier fallback resolution chain across extensions (failover if primary provider cannot resolve)
- [ ] Implement extension lyrics capability (`lyrics`):
  - Add `get_lyrics` WASM export hook
  - Implement native synced/plain lyrics UI with playback timestamp auto-scrolling
  - Implement SQLite lyrics caching and local .lrc / embedded tag fallback
- [ ] Implement extension equalizer profiles (`equalizer`):
  - Add native biquad filter DSP on the audio thread
  - Implement extension/preset schema for frequency bands and curve definitions
- [ ] Implement native download manager with extension stream resolution:
  - Asynchronous background download engine with chunking and retry support
  - Native metadata tagging (ID3, Vorbis, embedded album art)
  - Automatic indexing of downloaded files into local library database
- [ ] Implement extension playback lifecycle hooks (`scrobble` / `integration`):
  - Forward playback events (`start`, `pause`, `progress_50`, `complete`) to extensions
  - Support Discord Rich Presence, Last.fm, and ListenBrainz extensions
- [ ] Implement extension metadata scraper capability (`metadata`):
  - Add `get_artist_info` / `get_album_info` WASM export hooks
  - Render native artist biography and discography inspection sheet
- [ ] Implement SSRF mitigation and security containment for WASM HTTP host functions (`host_http_request`):
  - Block loopback addresses (`127.0.0.1`, `::1`, `localhost`), cloud metadata endpoints (`169.254.169.254`), and RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`)
  - Enforce `https://` scheme restriction (strictly reject `file://`, `gopher://`, etc.)
  - Implement domain allowlisting and follow-redirect limits
  - Add per-extension rate limiting to prevent network flood and resource exhaustion
- [ ] Implement extension theme support (`theme`):
  - Support theme packages with CSS variable overrides and custom palettes
  - Add theme selection dropdown in Settings

## Database Layer & Concurrency
- [ ] Configure SQLite WAL mode and busy timeout in `schema.rs`:
  - Set `PRAGMA journal_mode = WAL;` and `PRAGMA busy_timeout = 5000;` on database initialization
  - Eliminate reader-blocked-by-writer contention (`SQLITE_BUSY`) between read connections (`open_read_conn`) and the single-writer actor channel (`db_tx`) during long scans or batch writes
  - Add connection pool tuning and benchmark concurrent read/write query latencies
- [ ] Enforce strict single-writer actor architecture across all subsystems:
  - Eliminate direct database writes via `RecommendationCompiler::open_write_conn` in `src-tauri/src/providers/recommendations.rs`
  - Route recommendation caching (`save_cache`), manual dedup overrides (`save_override`), and extension telemetry (`record_provider_success`, `record_provider_failure`) through dedicated `DbRequest` message variants in `db_tx`
  - Eliminate multi-writer SQLite lock contention and race conditions
- [ ] Offload high-frequency and heavy read queries from `db_tx` to dedicated concurrent read connections:
  - Migrate library queries (`search_library`, `get_local_tracks`, explore feed queries) to `open_read_conn` or a scoped read-connection pool using `tokio::task::spawn_blocking`
  - Prevent UI query serialization behind long-running batch write operations (`InsertTracks`)

## Branding & Visual Identity
- [ ] Design and generate custom Lyria application icon (replacing generic default Tauri logo across 32x32, 512x512, Windows .ico, and macOS .icns)

## Documentation & Organization
- [ ] Organize docs directory with proper segregation and hierarchy
- [ ] Write comprehensive project and developer documentation
