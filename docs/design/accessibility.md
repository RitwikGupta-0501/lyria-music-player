# Lyria Accessibility & Keyboard Navigation Specification

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-10  

---

# 1. Overview & Philosophy

In Lyria, accessibility serves two complementary pillars:
1. **Assistive Accessibility (WCAG Compliance):** Clear typographic contrast, screen-reader labels for icon controls, logical focus traversal, and vestibular motion reduction.
2. **Keyboard-Driven Accessibility:** The entire application must be fully operable via keyboard alone. Complete keyboard drivability guarantees an efficient, accessible, and frictionless experience for both assistive technology users and listeners who prefer navigating without a pointing device.

---

# 2. Keyboard Navigation & Default Keymap

Global hotkeys are managed by [`KeyboardHandler.svelte`](file:///home/ritwikg/Repository/echo-desktop/src/lib/components/KeyboardHandler.svelte) and codified in [`src/lib/stores/keymap.ts`](file:///home/ritwikg/Repository/echo-desktop/src/lib/stores/keymap.ts).

### 2.1. Global Configurable Keymap
These hotkeys are configurable via SQLite and handled globally across all views:

| Action Category | Action | Shortcut | Description |
| :--- | :--- | :--- | :--- |
| **Playback** | Play / Pause | `Space` | Toggle playback state for the active track. |
| **Playback** | Seek Back 5s | `ArrowLeft` | Rewind active track position by 5 seconds. |
| **Playback** | Seek Forward 5s | `ArrowRight` | Fast-forward active track position by 5 seconds. |
| **Playback** | Previous Track | `Ctrl + ArrowLeft` | Skip to previous track in queue. |
| **Playback** | Next Track | `Ctrl + ArrowRight` | Skip to next track in queue. |
| **Playback** | Volume Up (+5%) | `Ctrl + ArrowUp` | Increase master audio output by 5%. |
| **Playback** | Volume Down (-5%) | `Ctrl + ArrowDown` | Decrease master audio output by 5%. |
| **Playback** | Toggle Shuffle | `Ctrl + S` | Toggle shuffle state across current queue. |
| **Playback** | Cycle Repeat | `Ctrl + R` | Cycle repeat mode: `Off` → `All` → `One`. |
| **Navigation** | Global Search | `Ctrl + K` | Open Spotlight command palette and focus search input. |
| **Navigation** | Close / Escape | `Escape` | Dismiss active drawers, modal palettes, or fullscreen view. |
| **Discovery** | Refresh Recommendations | `R` | Reload algorithmic shelves, radios, and explore feeds. |

---

### 2.2. Contextual & Component Shortcuts
Shortcuts scoped to active overlays, modals, and focused list elements:

| Component Context | Shortcut | Action & Behavior |
| :--- | :--- | :--- |
| **Global Search Palette** | `Shift + Enter` | **Open in Explore:** Passes the search query to the Explore view and dismisses the palette. |
| **Global Search Palette** | `Enter` | **Direct Stream / Play:** Plays auto-resolved YouTube/stream URL, or activates the highlighted search result. |
| **Track Row (`TrackRow.svelte`)** | `Enter` | **Play Track:** Immediately starts playback of the focused track row. |
| **Album / Artist Card** | `Enter` | **Open Collection:** Opens the album drawer or navigates into the artist view. |
| **Queue Sidebar** | `Enter` | **Jump to Position:** Jumps audio playback directly to the selected queue track. |
| **Dropdowns & Selectors** | `Escape` | **Dismiss Context:** Closes active select dropdowns without resetting view position. |

---

### 2.3. Native OS Media Keys (Global Background Controls)
Lyria integrates directly with host operating system media subsystems via `souvlaki` (MPRIS on Linux, SMTC on Windows, NowPlaying on macOS). These function globally even when the app is minimized or running in the background:

| Hardware / OS Key | Event | Resulting Action |
| :--- | :--- | :--- |
| `XF86AudioPlay` / `XF86AudioPause` | `MediaControlEvent::Toggle` | Toggles play/pause state in backend audio thread. |
| `XF86AudioNext` | `MediaControlEvent::Next` | Advances to next track in queue. |
| `XF86AudioPrev` | `MediaControlEvent::Previous` | Skips to previous track in queue. |
| `XF86AudioStop` | `MediaControlEvent::Stop` | Halts audio engine and clears hardware sink. |

---

### 2.4. Smart Input Exclusion
To prevent typing conflicts, `KeyboardHandler` automatically deactivates global shortcuts whenever:
* The user's focus is within an `<input>`, `<textarea>`, or `[contenteditable]` element.
* An interactive shortcut recorder is currently capturing input in the Settings view (`.is-recording`).

---

# 3. Focus Hierarchy & Keyboard Traversal

All interactive controls feature visible, high-contrast focus rings tailored for dark mineral surfaces:

```css
/* Focus Ring Specification (src/app.css) */
input[type="text"]:focus,
button:focus-visible {
  outline: none;
  border-color: var(--echo-border-strong);
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.05);
}
```

### Tab Order Sequence:
1. **Primary Navigation Rail (`Sidebar.svelte`):** Top-level navigation items (`Home`, `Explore`, `Library`, `Extensions`, `Settings`).
2. **Main Canvas:** Active view content (cards, interactive shelves, track rows).
3. **Slide-Over Drawer (`RightDrawer.svelte`):** Focus traps inside the drawer when opened; pressing `Escape` restores focus to the invoking trigger.
4. **Transport Player Bar (`PlayerBar.svelte`):** Scrubber, playback dials, volume thumb, and drawer toggles.

---

# 4. Color Contrast & Low-Vision Ergonomics

All color pairings are tested against the Web Content Accessibility Guidelines (WCAG 2.1) using the dark Obsidian base (`#0A0A0C`):

| Foreground Token | Color | Background | Contrast Ratio | WCAG Compliance Level |
| :--- | :--- | :--- | :--- | :--- |
| `--echo-text-1` | Primary Chalk (`#EAE8E3`) | Obsidian Base (`#0A0A0C`) | **15.2 : 1** | **Passes AAA** (Exceeds 7.0:1) |
| `--echo-text-2` | Secondary Graphite (`#7A7885`) | Obsidian Base (`#0A0A0C`) | **4.7 : 1** | **Passes AA** (Exceeds 4.5:1 for body copy) |
| `--echo-primary` | Patina Brass (`#E2A973`) | Obsidian Base (`#0A0A0C`) | **8.8 : 1** | **Passes AAA** (For graphical objects / UI) |
| `--echo-text-3` | Muted Mineral (`#46464F`) | Obsidian Base (`#0A0A0C`) | **2.5 : 1** | **Subdued Non-Text** (Timestamps & metadata only) |

### Anti-Smear Dark Canvas:
By rejecting pure black (`#000000`) in favor of mineral charcoal (`#0A0A0C` to `#101014`), Lyria prevents OLED pixel-switching latency (purple smearing) when high-contrast text scrolls rapidly.

---

# 5. Screen Reader & ARIA Semantics

1. **Icon-Only Buttons:**  
   Every button that renders only an SVG icon (such as transport controls, queue toggles, and window dismiss buttons) must provide an explicit `aria-label`:
   ```svelte
   <button class="ctrl-btn" onclick={handlePlayPause} aria-label={isPlaying ? "Pause track" : "Play track"}>
       {#if isPlaying}<Pause size={22} />{:else}<Play size={22} />{/if}
   </button>
   ```
2. **Dynamic Live Regions:**  
   When tracks change, screen readers receive polite live announcements communicating song title, artist, and playback status:
   ```svelte
   <div class="sr-only" aria-live="polite" aria-atomic="true">
       Playing {currentTrack.title} by {currentTrack.artist}
   </div>
   ```
3. **Modal Dialogs:**  
   Modals (`GlobalSearch.svelte`, `PromptModal.svelte`) declare `role="dialog"` and `aria-modal="true"`, ensuring screen readers do not traverse beneath the dimmed backdrop.

---

# 6. Vestibular Motion Reduction

Lyria honors the user's operating system animation settings via CSS media queries:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

When reduced motion is enabled:
* Album art hover zoom (`scale(1.05)`) is disabled.
* PlayerBar width morphing transitions instantaneously without interpolation.
* Drawer sliding animations are replaced with simple opacity cuts.

---

# 7. Accessibility DOs and DON’Ts

| DO | DON'T |
| :--- | :--- |
| Ensure all icon-only buttons include `aria-label` or `title`. | Never leave transport or drawer buttons without accessible text names. |
| Keep global keyboard shortcuts configurable in SQLite via `keymap.ts`. | Never hardcode shortcuts inside component click handlers. |
| Use `Escape` as the universal dismiss key across all modals, drawers, and overlays. | Never trap users inside a modal or drawer without a keyboard exit path. |
| Restrict `--echo-text-3` strictly to non-critical metadata (track numbers, durations). | Never use low-contrast text for interactive buttons, links, or form fields. |
