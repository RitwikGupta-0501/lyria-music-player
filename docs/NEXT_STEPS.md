# Echo Music Player — Next Steps & Execution Roadmap

This document outlines the priority workstreams, architectural design requirements, and tracking milestones for upcoming development cycles of Echo Music Player.

---

## 🎯 Current Status & Recent Milestones

- [x] **Phase 1: Explore & Feed Overhaul**
  - [x] Modular aggregated Explore feed (Editorial Spotlight, Dynamic Categories, Trending, New Releases).
  - [x] Full categorized search (`Song`, `Album`, `Playlist`, `Artist`).
  - [x] YouTube Music `WEB_REMIX` provider integration with cached visitor tokens and BotGuard attestation.
- [x] **Phase 2: Unified Collection Architecture & Liked Songs**
  - [x] Unified virtualized right drawer (`CollectionDetail.svelte`) replacing separate album/playlist views.
  - [x] Rich Artist profile pages (`ArtistDetail.svelte`) with top tracks, discography carousels, and subscriber stats.
  - [x] SQLite telemetry upsert for Liked Songs (`get_liked_songs`) with real-time UI synchronization across all views.
  - [x] Custom Obsidian-and-Brass visual identity for favorites.

---

## 🚀 Priority Roadmap & Workstreams

```mermaid
graph TD
    A[Milestone Complete: Explore & Collections] --> B[Priority 1: Algorithmic Radio & Infinite Queue]
    A --> C[Priority 2: Native OS Desktop Media Controls]
    A --> D[Priority 3: Synced Lyrics System]
    B --> E[Priority 4: Infinite Scroll Pagination]
    C --> F[Priority 5: Global Quick Search Cmd+K]
```

---

## 1. 📻 Priority 1: Algorithmic Radio & Infinite Queue Continuation (Core Loop)
> **Goal:** Complete Echo’s primary identity — *“Play a seed track → Sandboxed WASM dynamically resolves the next track.”*

### Implementation Tasks
- [ ] **Daemon Queue End Detection:** Hook into audio thread queue exhaustion events to trigger continuation when queue reaches final track.
- [ ] **WASM `get_radio` / `get_related` Host Call:** Pass the current playing seed (artist, title, source ID) to the active WASM provider.
- [ ] **Seamless Track Enqueueing:** Automatically append resolved recommendations to `audioStore.queue` without audio buffer interruption.
- [ ] **Autoplay Toggle in UI:** Add a subtle toggle on the Player Bar and Queue Drawer (`Autoplay Radio: On / Off`).

---

## 2. 🎛️ Priority 2: Native Desktop Media Controls (MPRIS / SMTC / Souvlaki)
> **Goal:** Complete native desktop OS integration across Linux, Windows, and macOS.

### Implementation Tasks
- [ ] **Hardware Media Keys Integration:** Bind `Play/Pause`, `Next`, `Previous`, `Mute` to OS media controls via `souvlaki` crate.
- [ ] **OS Now-Playing Metadata Sync:** Feed current track title, artist, album, duration, playback position, and album art buffer to MPRIS (Linux D-Bus) / SMTC (Windows) / NowPlaying (macOS).
- [ ] **Tray Icon & Background Controls:** Add system tray minimization and quick-control menu.

---

## 3. 🎤 Priority 3: Synchronized / Timed Lyrics Experience
> **Goal:** High-fidelity, real-time lyrics synchronized with audio playback.

### Implementation Tasks
- [ ] **LRC Parser & Local Lyrics Support:** Support standard `.lrc` timestamped files in same directory as local tracks.
- [ ] **WASM Remote Lyrics Resolution:** Implement sandboxed `get_lyrics` hook for fetching synchronized lyric timestamps on remote tracks.
- [ ] **Dynamic Scrolling Lyrics Component:**
  - Active line highlighting with smooth spring interpolation in `<FullScreenPlayer>`.
  - Dedicated lyrics view / sidebar tab.
  - Click-to-seek by tapping on a lyric line.

---

## 4. 📜 Priority 4: Infinite Scroll Pagination (Explore & Search)
> **Goal:** Seamless catalog browsing depth without artificial limits.

### Implementation Tasks
- [ ] **Continuation Token Handler:** Consume `continuation_token` returned by `CategorizedSearchResult` and YouTube WASM browse endpoints.
- [ ] **Svelte Virtualizer Scroll Anchor:** Trigger next page chunk fetch when scroll position nears bottom of Explore feed or Search list.
- [ ] **Prefetch & Debounce:** Prefetch subsequent pages 300px before scroll boundary to eliminate loading pauses.

---

## 5. ⚡ Priority 5: Quick Command Palette (`Cmd+K` / `Ctrl+K`)
> **Goal:** Instant, keyboard-first navigation and player remote control.

### Implementation Tasks
- [ ] **Global Shortcut Listener:** Register `Ctrl+K` / `Cmd+K` global overlay hotkey.
- [ ] **Unified Fuzzy Search:** Instant fuzzy searching across local library, recent search history, artists, and remote catalog.
- [ ] **Action Shortcuts:** Fast commands (`/play`, `/shuffle`, `/liked`, `/queue`, `/clear`).

---

## 🛠️ Engineering Constraints & Guidelines (from `GEMINI.md`)
* **Audio Thread Isolation:** `rodio` sink and event loop must strictly live on a dedicated OS thread outside Tokio runtime.
* **No Tokio Blocking:** Wrap all SQLite calls in `tokio::task::spawn_blocking` or funnel through database actor channel.
* **IPC Bridge Hygiene:** Throttle high-frequency playback position events to $\le$ 4Hz; let frontend interpolate smoothly via `requestAnimationFrame`.
* **Zero UI Interfiltration:** Svelte 5 Runes exclusively (`$state`, `$derived`, `$effect`); avoid external bloated CSS frameworks.
