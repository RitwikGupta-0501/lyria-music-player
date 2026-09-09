# TODO

> For completed features, fixes, and release history, see [CHANGELOG.md](../CHANGELOG.md).

## Audio Engine & Playback Polish
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
- [ ] Implement extension theme support (`theme`):
  - Support theme packages with CSS variable overrides and custom palettes
  - Add theme selection dropdown in Settings

## Branding & Visual Identity
- [ ] Design and generate custom Lyria application icon (replacing generic default Tauri logo across 32x32, 512x512, Windows .ico, and macOS .icns)

## Documentation & Organization
- [ ] Organize docs directory with proper segregation and hierarchy
- [ ] Write comprehensive project and developer documentation
