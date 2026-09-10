# Echo — Feature Roadmap & Ideas

A living document tracking every feature idea for Echo. Organized by priority tier and implementation complexity.

**Legend:**  
🟢 Easy (< 1 session) · 🟡 Medium (1–3 sessions) · 🔴 Hard (multi-session) · ⚫ Research needed  
`[ ]` Not started · `[~]` In progress · `[x]` Done

---

## Tier 0: Core Player (Done ✅)
- [x] Audio engine (rodio + background thread)
- [x] Local library scanning (walkdir + ID3)
- [x] Relational DB (albums, tracks, playlists)
- [x] Lazy artwork caching
- [x] Interpolation-based progress bar
- [x] Seek, Queue, Skip, Shuffle, Repeat
- [x] SPA component architecture

---

## Tier 1: Essential Polish (Next Up)

### 🎨 UI Overhaul — Consistent Design Language
**Priority: HIGH** · 🟡 Medium
- [x] Replace emoji icons (🔀⏮▶⏭🔁) with a proper SVG icon set (Lucide or Phosphor)
- [ ] Consistent spacing, padding, and typography scale across all views
- [ ] Hover states, active states, and focus rings on every interactive element
- [ ] Animated view transitions (crossfade between Albums → Album Detail)
- [ ] "Now Playing" highlight on the currently playing track in any list
- [x] Volume slider in PlayerBar
- [ ] Make the UI responsive (collapse sidebar on small screens, scale player bar layout)
- [ ] Drag-to-reorder queue (future)

### ⚙️ Comprehensive Settings Page
**Priority: HIGH** · 🟡 Medium
- [ ] **Audio:** Output device selector, crossfade duration, gapless playback toggle
- [ ] **Library:** Default scan directories (multi-path), auto-scan on launch toggle
- [ ] **Appearance:** Theme selector (dark/darker/custom accent color)
- [ ] **Playback:** Default repeat mode, default shuffle state, "restart vs previous" threshold (currently hardcoded 3s)
- [ ] **Data:** Export/import playlists, database location display, storage usage stats
- [ ] **About:** App version, credits, links

### 🔊 Volume Control (Done ✅)
**Priority: HIGH** · 🟢 Easy
- [x] Volume slider in PlayerBar (rodio `Sink::set_volume()`)
- [x] Mute toggle
- [x] Persist volume in settings DB

---

## Tier 2: Power User Features

### 📝 Playlist Management (Full)
**Priority: HIGH** · 🟡 Medium
- [ ] View playlist tracks (PlaylistDetail view)
- [ ] Add tracks to playlist from album view (right-click or "+" button)
- [ ] Remove tracks from playlist
- [ ] Reorder tracks within a playlist (drag & drop)
- [ ] Rename / delete playlists
- [ ] Play entire playlist (populate queue)
- [ ] Playlist artwork (mosaic of first 4 album covers)

### 🔍 Search
**Priority: HIGH** · 🟡 Medium
- [ ] Global search bar (Ctrl+K / Cmd+K)
- [ ] Search across tracks, albums, artists, playlists
- [ ] Fuzzy matching (Rust-side using `fuzzy-matcher` or similar)
- [ ] Search results grouped by category

### 🎤 Live Lyrics
**Priority: MEDIUM** · 🔴 Hard
- [ ] Fetch lyrics from a Lua provider (e.g., scrape lyrics APIs)
- [ ] Synced lyrics (LRC format) — highlight current line based on timestamp
- [ ] Unsynced lyrics fallback (plain text, auto-scroll)
- [ ] Lyrics panel that slides in from the right or overlays the main view
- [ ] Cache fetched lyrics in SQLite to avoid re-fetching
- [ ] Embedded lyrics support (read from ID3 `USLT` frame)

### ⌨️ Keyboard Shortcuts
**Priority: MEDIUM** · 🟢 Easy
- [ ] Space = Play/Pause
- [ ] ← / → = Seek ±5s
- [ ] Ctrl+← / Ctrl+→ = Previous / Next
- [ ] Ctrl+↑ / Ctrl+↓ = Volume Up / Down
- [ ] Ctrl+S = Toggle Shuffle
- [ ] Ctrl+R = Cycle Repeat
- [ ] Ctrl+K = Focus Search
- [ ] Escape = Close modals / back navigation

---

## Tier 3: Differentiators (What Makes Echo Special)

### 🧠 Smart Shuffle
**Priority: MEDIUM** · 🔴 Hard · ⚫ Research needed
- [ ] Analyze audio features (BPM, energy, genre from ID3) to group "similar" tracks
- [ ] Weighted random: avoid playing same artist back-to-back
- [ ] "Radio mode": given a seed track, auto-generate a queue of similar tracks from library
- [ ] Optional: use a lightweight ML model (e.g., k-NN on audio embeddings) — could be a Lua provider
- [ ] Spotify-like approach: prioritize variety while maintaining mood coherence

### 📊 Listening Statistics
**Priority: LOW** · 🟡 Medium
- [ ] Track play counts (increment in DB on each play)
- [ ] "Recently Played" view
- [ ] "Most Played" view
- [ ] Total listening time (daily/weekly/monthly)
- [ ] Per-artist and per-album play counts
- [ ] Stats dashboard with charts (lightweight, CSS-only or SVG)

### 🎨 Dynamic Theming
**Priority: LOW** · 🟡 Medium
- [ ] Extract dominant colors from album artwork (Rust-side, using `image` crate)
- [ ] Dynamically tint the UI background/accents based on the currently playing album
- [ ] Smooth color transitions when tracks change
- [ ] Respect user override if they set a custom accent color in settings

### 🔌 Plugin/Provider Ecosystem
**Priority: LOW** · 🔴 Hard
- [ ] Provider manager UI (list installed Lua scripts, enable/disable)
- [ ] "Add provider from URL" (fetch .lua script and install)
- [ ] Provider marketplace / registry (future, way out)
- [ ] Provider API documentation for third-party developers

---

## Tier 4: Nice-to-Haves (Someday/Maybe)

### 🎵 Audio Processing
- [ ] Equalizer (5-band or 10-band) — requires replacing rodio with a lower-level audio pipeline
- [ ] Crossfade between tracks (blend last N seconds of current with first N of next)
- [ ] Gapless playback (pre-buffer next track)
- [ ] Replay Gain / volume normalization

### 🖥️ Desktop Integration
- [ ] System tray icon with mini controls
- [ ] MPRIS integration (Linux media keys, lock screen controls)
- [ ] Discord Rich Presence (show what you're listening to)
- [ ] File association (.mp3, .flac → open in Echo)
- [ ] Notification on track change

### 📱 Remote & Sync
- [ ] Local network remote control (phone controls desktop playback via WebSocket)
- [ ] Sync playlists/library across devices (would need a sync server — very ambitious)

### 🎨 Visual Candy
- [ ] Mini player mode (compact floating window)
- [ ] Audio visualizer (waveform or spectrum in PlayerBar)
- [ ] Album art blur as page background (already partially done with glassmorphism)

### 🖼️ Animated Album Covers (Apple Music-style)
**Priority: MEDIUM** · 🟡 Medium
Bring album art to life with subtle motion — inspired by Apple Music's "Now Playing" screen.
- [ ] **Default animations (automatic, CSS-driven):**
  - Slow Ken Burns drift (gentle pan + zoom across the artwork)
  - Subtle "breathing" scale pulse synced to playback state (playing vs paused)
  - Soft parallax tilt on mouse hover (CSS `perspective` + `transform`)
  - Ambient glow: extract dominant color from artwork and pulse a blurred halo behind the cover
- [ ] **Custom per-album/per-track animations:**
  - Store an optional `animation_preset` field in the `albums` table (e.g., `"ken_burns"`, `"rotate"`, `"pulse"`, `"none"`)
  - Allow users to assign an animation style per album from a preset gallery in Album Detail
  - Support custom looping video covers (`.webm` / `.mp4`) stored alongside artwork in the cache dir
  - Fallback: if no custom animation is set, use the default Ken Burns
- [ ] **Performance guard:**
  - Animations only run on the visible "Now Playing" / Album Detail view (paused via `IntersectionObserver` when off-screen)
  - Respect a "Reduce Motion" toggle in Settings for accessibility

---

## Technical Debt & Improvements
- [ ] Error handling: Replace all `console.error` with user-facing toast notifications
- [ ] Loading states: Skeleton loaders for album grid, track lists
- [ ] Accessibility: Proper ARIA labels, screen reader support, focus management
- [ ] Performance: Virtual scrolling for large libraries (1000+ albums)
- [ ] Testing: Rust unit tests for scanner, DB operations; Svelte component tests
- [ ] CI/CD: GitHub Actions for cross-platform builds (Linux, macOS, Windows)

---

## Notes & Decisions Log
- **2025-05-10:** Queue lives in the frontend (AudioStore), not Rust. Backend stays a dumb audio daemon.
- **2025-05-10:** Progress bar uses interpolation strategy (rAF + sync events), not IPC polling.
- **2025-05-10:** Seek emits requested position, not `sink.get_pos()` (stale after try_seek).
