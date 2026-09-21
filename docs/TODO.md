# TODO

> For completed features, fixes, and release history, see [CHANGELOG.md](../CHANGELOG.md).

## Audio Engine & Playback Polish
- [x] Handle audio hardware disconnection & device drop (Fixed in v0.3.0 - BUG-04):
  - Replaced panicking `.expect()` / `.unwrap()` calls in `src-tauri/src/audio/mod.rs` with `recreate_sink` fallback and `Option<Sink>`.
  - Emits `device-disconnected` to frontend and drops frames cleanly instead of crashing when DAC/headphones are removed or missing.
- [ ] Fix in-flight seek vs. track load race condition (`AudioCommand::Load` must increment `seek_generation` in `src-tauri/src/audio/mod.rs`)
- [ ] Fix gapless queue loss and state desync on seek (`src-tauri/src/audio/mod.rs:426`, `src/lib/stores/audio.svelte.ts:307-319`):
  - When seeking, `audio/mod.rs` executes `queued_tracks.clear()` and stops/recreates the `rodio::Sink`, destroying the pre-buffered next track.
  - `AudioStore.seek()` in `audio.svelte.ts` never triggers `queueNextAudio()` after seeking.
  - When the current track reaches the end, the sink is empty and `queued_tracks` is empty, causing gapless transition to fail and triggering an abrupt fallback `track-ended` event.
- [x] Fix multichannel/surround audio corruption in `OpusSymphoniaDecoder` (Fixed in v0.3.0 - BUG-05) (`src-tauri/src/audio/opus_decoder.rs:25, 68-73`):
  - Clamped decoder channel mapping to stereo (2 channels) for multichannel surround streams (5.1 / 7.1).
  - Symphonia `SignalSpec` and output sample buffer allocation now match (2 channels), preventing out-of-bounds heap memory reads and segmentation faults.
- [ ] Fix HLS audio stutter, seek desynchronization, and live stream truncation (`src-tauri/src/audio/hls.rs:85-121`, `src-tauri/src/audio/symphonia_source.rs:118-127`):
  - `fetch_next_segment()` calls `runtime.block_on()` synchronously only after `current_segment_data` hits EOF, stalling the audio OS thread during network transit and causing audible dropouts between segments. Implement background prefetching for segment $N+1$.
  - In `symphonia_source.rs:125-126`, `from_hls_seeked` initializes `HlsStream` with segment boundary alignment but passes `seek_to: None` to `open()`. `audio/mod.rs` sets `seek_offset = pos`, causing up to a 10s desynchronization between audible audio and visual playback position.
  - Non-VOD / sliding-window live HLS streams terminate prematurely because `HlsStream` never refreshes `.m3u8` playlists.
- [x] Wire up terminal `track-ended` event and emit `player-sync` Stopped upon queue exhaustion (Fixed in v0.3.0 - BUG-06):
  - In `src-tauri/src/audio/mod.rs:241-250`, the audio thread clears `current_track_path` and emits `track-ended`, but never emits `player-sync` with state `"Stopped"`.
  - The Svelte frontend (`src/lib/stores/audio.svelte.ts`) does not listen to `track-ended`, leaving the player bar and media controls permanently stuck in `"Playing"` at 100% position after the queue drains.
- [ ] Fix premature audio cutoff on non-fatal decode errors in `SymphoniaSource` (`src-tauri/src/audio/symphonia_source.rs:299-328`):
  - In the background decoding loop, `format.next_packet()` breaking on any error treats recoverable packet/decode errors as EOF (`eof_clone.store(true)`), abruptly cutting off the song.
  - Distinguish between fatal EOF (`SymphoniaError::IoError(UnexpectedEof)`) and recoverable packet decode errors so playback skips damaged frames rather than stopping.
- [ ] Implement Offline Resilience & Network Disconnect Handling:
  - Add reactive frontend connectivity monitoring (`navigator.onLine`, `window.addEventListener('online' | 'offline')`) and display a subtle offline status badge in the UI.
  - On remote stream resolution failure due to network loss, gracefully fall back to local library seed tracks or cached media rather than halting with unhandled socket errors.
  - In `symphonia_source.rs`, distinguish transient socket read disconnects from true EOF and implement a short reconnect window (e.g. 5-10s) before terminating playback.
- [ ] Enforce connection and read timeouts on remote stream initialization (`src-tauri/src/audio/symphonia_source.rs:137-164`):
  - `from_url_internal` executes `runtime_handle.block_on(HttpStream::new(...))` synchronously on the audio OS thread without connect or read timeouts.
  - A stalled remote CDN response indefinitely freezes the audio thread, deadlocking player commands (`pause`, `stop`, `seek`) and OS media controls.
- [ ] Implement auto-advance on corrupted / deleted audio paths in `SymphoniaSource::from_path`
- [ ] Implement comprehensive audio subsystem unit tests (benchmarking is not equivalent to functional tests; test decoding, seeking, underruns, and gapless callbacks in `src-tauri/src/audio`)
- [ ] Implement gapless playback and configurable crossfade (1-5s) between consecutive tracks
- [ ] Implement ReplayGain / volume normalization (EBU R128 / LUFS standard) to prevent loudness jumps
- [ ] Implement Audio Output Device Selection on the Player Bar (Single device output, default):
  - UI on Player Bar: Add a dedicated device picker button (speaker/headphones icon) on the right flank utilities of `PlayerBar.svelte` (adjacent to Queue and Volume) with a glassmorphism popover menu listing available audio sinks, an active selection indicator, and "System Default".
  - Backend enumeration: Use `cpal::default_host().output_devices()` to discover all attached hardware outputs (DACs, Bluetooth headsets, internal speakers).
  - Tauri commands: Expose `get_audio_output_devices() -> Vec<String>` and `set_audio_output_device(device_name: Option<String>)`.
  - Audio thread hot-swap: Send `AudioCommand::SetOutputDevice(Option<String>)` over the lock-free mpsc channel; re-initialize `OutputStream::try_from_device(&device)` and rebuild `rodio::Sink` preserving current track position, duration, and pause/play state.
  - Settings integration: Mirror the selected audio device in Settings -> Playback as well.
- [ ] Research & Implement Multi-Device Simultaneous Audio Output (Experimental / Post-1.0):
  - Use Cases: Dual-headphone listening (sharing audio on laptop/travel), simultaneous local speakers + Bluetooth room speaker, or capture/broadcast workflows.
  - Validity & Architectural Considerations:
    * Clock Drift: Different hardware audio interfaces run on independent physical crystal clocks (e.g. 48,000 Hz vs 48,003 Hz). Without software PLL resampling (or delegating to PipeWire/PulseAudio `module-combine-sink`), multi-sink buffers diverge over time causing audio underruns or buffer clicks.
    * Latency Asymmetry: Bluetooth A2DP typically introduces ~100-200ms latency, while onboard 3.5mm/speakers have <20ms. Direct synchronous mirroring produces an audible slap-back echo unless configurable per-device millisecond delay compensation is added.
    * Implementation Strategy:
      1. Phase 1 (OS Delegation): Provide a helper/toggle to bind into native OS combined sinks (PipeWire virtual combined node, PulseAudio `module-combine-sink`, macOS Audio MIDI Aggregate Device).
      2. Phase 2 (Native Multi-Sink Engine): Support `Vec<(OutputStream, Sink)>` in the audio thread with synchronized frame distribution and optional latency-offset buffers.
- [ ] Implement sleep timer (stop after N minutes or at the end of current track)
- [ ] Implement audio format and quality badging (e.g. FLAC 24-bit/96kHz, MP3 320kbps, OPUS) on tracks and player bar

## Desktop & OS Integration
- [x] Wire up OS & hardware media control event listeners in Svelte frontend (`src-tauri/src/audio/media_controls.rs:59-67`, `src/lib/stores/audio.svelte.ts:190-198`):
  - `media_controls.rs` handles keyboard media keys (Play/Pause, Next, Previous) and native OS controls (MPRIS/SMTC/NowPlaying) via `souvlaki` and emits `"media-play-pause"`, `"media-next"`, and `"media-prev"`.
  - Connected listeners in `AudioStore` to `togglePlay()`, `skipForward()`, and `skipBackward()`, with unlistener cleanup in `destroy()`.
- [ ] Set up macOS Homebrew Cask recipe (`brew install --cask lyria`)
- [ ] Implement system tray integration with minimize-to-tray, close-to-tray, and quick playback controls
- [ ] Implement Discord Rich Presence integration (native or extension-driven, configurable in Settings)

## Library & Queue Management
- [x] Fix "Clear Player When Finished" / `collapse_idle` not clearing the queue (`src/lib/stores/audio.svelte.ts:127-140`):
  - When `queueCompletionBehavior` is set to `"collapse_idle"`, the `Stopped` handler resets `currentTime = 0`, `duration = 0`, and `currentTrack = "None"`, but never calls `this.clearQueue()` or invokes `clear_queue`.
  - Extracted dedicated `handleQueueCompletion()` function and execute `await this.clearQueue()` when `collapse_idle` triggers on `Stopped` state.
- [ ] Add `.opus` extension to local library scanner (`src-tauri/src/lib.rs:990`):
  - `scan_local_directory` checks `ext == "mp3" || ext == "flac" || ext == "wav" || ext == "m4a" || ext == "ogg"`, ignoring `.opus` files despite first-class OPUS decoder support in the audio pipeline.
- [ ] Implement pagination / infinite scrolling for Library collections (`src/lib/stores/library.svelte.ts:84-115`):
  - `fetchAlbums`, `fetchPlaylists`, and `fetchLikedSongs` hardcode `{ limit: 500, offset: 0 }` with no pagination continuation.
  - Libraries exceeding 500 albums, playlists, or liked tracks are silently truncated in the UI.
- [ ] Implement automatic filesystem watcher for music folders (background auto-indexing of newly added files)
- [ ] Implement multi-select and batch actions (Shift/Ctrl-click to queue, add to playlist, or delete)
- [ ] Fix `RepeatMode::One` auto-advance loop bug in `QueueState` (`src-tauri/src/queue/mod.rs`):
  - In `next_normal()` and `next_shuffle()`, evaluate `RepeatMode::One` before incrementing position to ensure tracks repeat anywhere in the queue, not just at queue terminus.
  - Fix gapless pre-buffering in `QueueState::peek_next()` (`src-tauri/src/queue/mod.rs:220-231`): `QueueMode::Normal` unconditionally returns `self.current_position + 1`, causing `queueNextAudio()` to pre-buffer the *next* track into the audio sink and gaplessly advance instead of repeating when `RepeatMode::One` is active.
- [ ] Fix manual "Next" replaying last track on `RepeatMode::Off` (`src/lib/stores/audio.svelte.ts` / `commands.rs`):
  - When at queue terminus with repeat off, `queue.next()` returns `Ok(None)` leaving `current_position` unchanged; `skip_forward` then returns that last track, causing frontend to reload and replay it forever.
- [ ] Implement the 3-Second "Previous" Rule (restart if > 3s, skip previous if <= 3s) in `audioStore.skipBackward()` (`src/lib/stores/audio.svelte.ts:578-599`).
- [ ] Implement `remove_from_queue` Tauri command and `QueueState::remove_track` (`src-tauri/src/queue/mod.rs`, `commands.rs`):
  - Currently `QueueState` only implements `add_track`, `set_queue`, `clear`, and `reorder`. There is no backend method or Tauri command to remove individual tracks from the active queue.
- [ ] Fix cursor corruption and order desync in `QueueState::reorder()` during active shuffle (`src-tauri/src/queue/mod.rs:162-170`):
  - When `self.mode == QueueMode::Shuffle`, `self.current_position` is completely ignored during reordering. If tracks are moved while shuffle is active, toggling shuffle off (`set_shuffle(false)`) causes the queue cursor to jump to a completely arbitrary track or out-of-bounds index.
  - Also synchronize `shuffle_state.order` with the reordered positions.
- [ ] Deterministic Seeded PRNG for Shuffle (`src-tauri/src/queue/mod.rs:430-448`):
  - `regenerate_shuffle_order` calculates a timestamp `seed` and stores it in `ShuffleState.seed`, but shuffles using unseeded `rand::thread_rng()`. Replace with deterministic `rand::rngs::StdRng::seed_from_u64(seed)` to enable reproducible queue session recovery.
- [ ] Wire up runtime queue state persistence & background snapshotting:
  - `recover_on_startup` is executed on boot in `lib.rs:1633`, but `save_queue_state` is never called on any queue mutations (`add_track`, `set_queue`, `reorder`, `skip_forward`, `set_shuffle`), leaving persistence completely dormant.
  - Connect runtime queue mutations to debounced SQLite persistence using `queue::recovery::periodic_save` or `DbRequest::SaveQueueState`.
  - Ensure playback queue, shuffle order, and cursor position survive application restarts and sudden crashes.
  - Implement SQLite persistence unit/integration tests in `src-tauri/src/queue/persistence.rs`.
## Infinite Core Loop & Autoplay
- [x] Fix Autoplay provider misattribution and 2-track infinite loop (`src-tauri/src/queue/autoplay.rs:48-70`, `src-tauri/src/providers/recommendations.rs:448-456`):
  - Preserved provider IDs in `compile_federated_radio` and attached to `TrackResult.provider_id`.
  - Repaired `autoplay.rs` to read `selected.provider_id` instead of misinterpreting `isrc` as a provider identifier.
  - Added recent queue history deduplication to break 2-track ping-pong loops.
- [ ] Migrate Autoplay Core Loop Orchestration from Frontend Store (`audio.svelte.ts`) to Rust Daemon to preserve the "Frontend as Pure Remote Control" architectural invariant (ARCH-01)
- [ ] Fix local seed duration proximity calculation bypass in autoplay (`src-tauri/src/queue/autoplay.rs:18-25`, `src-tauri/src/db/queries.rs:1981-1992`):
  - In `autoplay.rs`, `TrackSourceInfo::Local` sets `seed_duration_ms` to `None`, bypassing the duration proximity formula and always awarding the fallback score of 5.0
- [ ] Fix WASM capability mismatch in federated radio fallback (`src-tauri/src/providers/recommendations.rs:420-442`, `src-tauri/src/providers/mod.rs:948`):
  - When falling back to `"related"` providers, `p_mgr.get_radio` invokes the `"get_radio"` WASM export, failing if the extension only exports `"get_related"`
- [x] Reconcile conflicting radio timeout budgets:
  - Reduced inner per-provider task timeout to 3000ms so individual timeouts fire before the 3500ms batch deadline in `autoplay.rs`.
- [ ] Implement unit test suite for `queue::autoplay` and Markov candidate scoring (`autoplay.rs`, `db::queries.rs`):
  - Unit tests for waterfall fallback, remote deduplication, provider fallback selection, recency gates, 3-track library size relaxation, affinity vector weights, and Boltzmann temperature bounds
- [ ] Reconcile code comment (`// 3. Genre / Lexical Affinity (weight 30)`) with implemented scoring weight (+15.0) in `src-tauri/src/db/queries.rs:1969`

## Display Modes
- [ ] Implement compact floating mini-player mode (always-on-top picture-in-picture widget)
- [ ] Implement immersive full-screen / cinematic view (large album artwork, synced lyrics, ambient background)

## Discovery, Search & Recommendations
- [ ] Implement dynamic Adjacent Horizons engine:
  - In `src-tauri/src/providers/recommendations.rs:500-575`, replace hardcoded static mock tuples ("Divenire", "Water No Get Enemy") which currently pass arbitrary canonical key strings as `remote_track_id` to `provider_manager.resolve()`, causing stream playback to fail.
  - Dynamically invert 7-day SQLite listening habits to pair unlistened genres with dominant artist styles.
  - Fetch 3 real, dynamic preview tracks (artwork, duration, IDs) with streaming/local fallbacks instead of static definitions.
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
- [x] Fix memory leak and hang hazard in JavaScript sandbox VM (Fixed in v0.3.0 - BUG-05) (`src-tauri/src/sandbox/sandbox_vm.rs:38-60`, `src-tauri/src/providers/wasm_bridge.rs:168-184`):
  - `execute_javascript` inserts a `req_id` into `manager.pending_requests`. If `window.eval()` fails or the 15-second timeout in `wasm_bridge.rs` triggers, the sender is never removed from `pending_requests`, leaking memory indefinitely.
  - Ensure `execute_javascript` uses an RAII cleanup guard or explicit removal on error/timeout, and verify the `sandbox-vm` window has finished loading before evaluating scripts.
- [ ] Fix Extism plugin timeout cache staleness and execution serialization (`src-tauri/src/providers/mod.rs:308-312`):
  - `get_or_create_plugin` caches plugins in `Arc<std::sync::Mutex<Plugin>>`. The first creation (during warmup with a 30s timeout) locks in that timeout; subsequent calls with tighter deadlines (e.g. 4s radio recommendation) inherit the 30s timeout on cache hit.
  - Sharing a single `Mutex<Plugin>` across all calls completely serializes concurrent provider operations (search, explore feeds, autoplay radio).
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
  - [x] Fix potential Tokio runtime panic in `src-tauri/src/providers/wasm_bridge.rs:42-44` (Completed in v0.3.0): Implemented `run_async_host` using `rt.spawn()` and `sync_channel(1)` across all WASM host functions to avoid nesting `block_on` inside Tokio worker threads.
- [ ] Implement extension theme support (`theme`):
  - Support theme packages with CSS variable overrides and custom palettes
  - Add theme selection dropdown in Settings

## Frontend Store & Svelte 5 Lifecycle
- [ ] Fix log duplication in real-time debug viewer (`src-tauri/src/logger.rs:173-194`, `src/routes/debug/+page.svelte:64-80`):
  - On mount, `/debug` fetches the initial 500 logs via `get_debug_logs`. Simultaneously, the 2Hz background emitter in `logger.rs` fires with `last_sent_id = 0`, collecting and emitting all buffered logs.
  - The Svelte listener unconditionally appends `logs = [...logs, ...e.payload]`, immediately duplicating all logs in the UI upon opening the window. Implement deduplication by `id` on the frontend and track per-client `last_sent_id`.
- [ ] Migrate root layout from deprecated `<slot />` to Svelte 5 snippet props (`src/routes/+layout.svelte:54`):
  - Replace `<slot />` with `let { children } = $props();` and `{@render children()}` to eliminate Svelte 5 rune migration warnings and ensure proper reactive scope propagation.
- [x] Fix fatal runtime mutation of `$derived` runes in `AudioStore` (Fixed in v0.3.0 - BUG-04) (`src/lib/stores/audio.svelte.ts:358-372`):
  - `trackClickBehavior` and `queueCompletionBehavior` are defined at lines 57-58 using `$derived(settingsStore...)`.
  - In `setTrackClickBehavior` and `setQueueCompletionBehavior`, direct assignments (`this.trackClickBehavior = behavior;`) throw fatal Svelte 5 runtime exceptions (`Cannot assign to a derived variable`).
  - Route setting changes exclusively through `settingsStore.setSetting(...)`.
- [ ] Synchronize UI state on queue exhaustion (`src/lib/stores/audio.svelte.ts`):
  - Listen to `track-ended` or handle `player-sync` Stopped to reset `playbackState = "Stopped"`, `currentTime = 0`, and clear active player pill state when the queue drains.

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
- [x] Design and generate custom Lyria application icon (Completed in v0.3.0):
  - Created bespoke acoustic Lyria visual identity (`static/lyria.svg` and `static/favicon.png`).
  - Generated complete Tauri platform asset suite across `src-tauri/icons/` (32x32, 64x64, 128x128, 128x128@2x, 512x512 `icon.png`, Windows `.ico` and Windows tile logos, macOS `.icns`, and mobile bundles).

## Documentation & Organization
- [ ] Organize docs directory with proper segregation and hierarchy
- [ ] Write comprehensive project and developer documentation
