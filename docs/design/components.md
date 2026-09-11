# Lyria Component Anatomy & Layout Specification

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-10  

---

# 1. Component Philosophy

Lyria’s component architecture follows the **Quiet Luxury** doctrine:
1. **Content Over Chrome:** Components provide clear structural scaffolding without competing with album artwork or typography.
2. **Tactile Boundaries:** Components rely on whisper borders (`rgba(255, 255, 255, 0.05)`), stepped obsidian surfaces, and optical glass highlights rather than heavy background fills.
3. **Strict Clearance Architecture:** Floating controls (such as the bottom player bar) never clip or obscure underlying scrollable content.

All component styling is codified in [`src/app.css`](file:///home/ritwikg/Repository/echo-desktop/src/app.css) and isolated in dedicated Svelte 5 single-file components under `src/lib/components/`.

---

# 2. Shell Layout & Clearance Architecture

The top-level shell ([`src/routes/+page.svelte`](file:///home/ritwikg/Repository/echo-desktop/src/routes/+page.svelte)) consists of a three-column fluid layout with floating overlay controls:

```
┌───────────┬───────────────────────────────────────────┬──────────────────┐
│  Sidebar  │            Main Content Canvas            │   RightDrawer    │
│  (Fixed)  │            (Fluid Scroll Area)            │   (Slide-out)    │
│  83px     │                                           │   0px / 400px    │
│           │                                           │                  │
│           │                                           │                  │
│           │                                           │                  │
│           ├───────────────────────────────────────────┤                  │
│           │      Clearance Buffer (10rem / 160px)     │                  │
└───────────┴───────────────────────────────────────────┴──────────────────┘
            │ ──── Floating PlayerBar (Fixed Bottom) ── │
```

### Layout Clearance Tokens (`src/app.css`)
To ensure scrollable content is never occluded by the floating transport bar or side drawers:

| Token | Value | Purpose |
| :--- | :--- | :--- |
| `--sidebar-w` | `83px` | Fixed width of the primary left navigation bar. |
| `--player-clearance` | `10rem` (160px) | Bottom margin added to all page views (`.main-content`) so cards clear the player bar. |
| `--player-scroll-padding` | `10rem` | Scroll padding ensuring jumped-to elements do not align beneath the player. |
| `--drawer-scroll-padding` | `8rem` | Clearance for bottom elements inside the right slide-over drawer. |
| `--drawer-w` | `0px` / `400px` | Dynamically toggled CSS variable defining drawer open/closed state. |

---

# 3. Core Component Anatomy

---

## 3.1. Floating Player Bar (`PlayerBar.svelte`)
The `PlayerBar` is the primary transport deck. It floats permanently fixed at bottom-center of the screen.

### Anatomy:
```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Top-Edge Seek Scrubber: 2px idle -> 4px hover + Brass Needle Thumb]                   │
├──────────────┬─────────────────────────────────────────┬───────────────────────────────┤
│ Now Playing  │ Transport Controls                      │ Utility Cluster               │
│ - 44px Cover │ - Shuffle Toggle (Phosphor Shuffle)     │ - Format Badge (Hi-Res FLAC)  │
│ - Song Title │ - Skip Back (Phosphor SkipBack)         │ - Volume Slider + Mute Toggle │
│ - Artist     │ - Play/Pause Dial (44px Brass Pill)     │ - Queue Toggle (Up Next)      │
│ - Like Heart │ - Skip Forward (Phosphor SkipForward)   │ - Fullscreen Expand           │
│              │ - Repeat Toggle (Off / All / One)       │                               │
└──────────────┴─────────────────────────────────────────┴───────────────────────────────┘
```

### Key Technical Characteristics:
* **State Morphing:** Automatically animates from `340px` (idle/empty state) to `720px` (active playback) over `0.5s` using `--ease-liquid`.
* **Fitts’s Law Seek Bar:** A full-width interactive hitbox (`top: -10px`, `height: 20px`) along the top edge that smoothly expands from `2px` to `4px` on hover.
* **Optical Glass Finish:** Blurs the underlying content by `28px` with specular highlight catch-lights (`inset 0 1.5px 0 0 rgba(255, 255, 255, 0.55)`).

---

## 3.2. Track Row (`TrackRow.svelte`)
The standard row item for playlists, album listings, and search results.

### Anatomy:
```
┌────┬─────────┬───────────────────────────────┬─────────────────┬──────────┬────────┐
│ #  │ [Cover] │ Title & Artist Details        │ Album Title     │ Duration │  •••   │
│ 01 │ (42px)  │ The Sound of Silence          │ Wednesday Morning│ 03:05    │ (Opts) │
│    │         │ Simon & Garfunkel             │                 │          │        │
└────┴─────────┴───────────────────────────────┴─────────────────┴──────────┴────────┘
```

### Visual & Interactive States:
1. **Resting State:** Transparent background, `--echo-text-1` for title, `--echo-text-2` for artist, `--echo-text-3` for track number and duration.
2. **Hover State:** Background lightens to `rgba(255, 255, 255, 0.04)`, track number `#` transitions to a Play icon, 3-dots context menu button (`•••`) reveals.
3. **Playing State:** Title turns Brass (`#e2a973`), track number is replaced by an animated 3-bar equalizer wave (`EqualizerWave.svelte`).

---

## 3.3. Album & Collection Card (`AlbumCard.svelte`)
The primary discovery container for albums, playlists, and artists.

### Anatomy:
* **Artwork Frame:** Strict 1:1 square aspect ratio with rounded corners (`border-radius: 12px`).
* **Whisper Border:** `1px solid rgba(255, 255, 255, 0.05)`.
* **Play Overlay Deck:** Frosted backdrop (`backdrop-filter: blur(2px)`) that fades in on hover (`200ms`), revealing a centered circular `CardPlayButton` and a top-right `Heart` button.
* **Metadata Stack:**
  * **Title:** Primary chalk (`#eae8e3`), single-line truncated with ellipsis.
  * **Byline:** Graphite (`#7a7885`), displaying artist name, release year, or item count.

---

## 3.4. Pill Button (`PillButton.svelte`)
The universal action button used across headers, spotlight hero banners, and dialogs.

### Variants:
* **`primary`:** Patina Brass fill (`#e2a973`), dark void text (`#050507`), slight amber drop shadow. Used for "Play Album" or confirmation.
* **`secondary`:** Smoked glass background (`rgba(255, 255, 255, 0.08)`), white chalk text, whisper border. Used for "Explore Release" or secondary actions.
* **`ghost`:** Borderless, transparent resting background, brightening on hover.
* **`danger`:** Deep crimson tint (`#ef4444`) with subtle red glow. Used for destructive deletions.

### Sizing Standards:
* **`sm`:** Padding `0.42rem 0.9rem`, radius `20px`, icon size `14px`, text `0.8rem`.
* **`md`:** Padding `0.6rem 1.3rem`, radius `24px`, icon size `16px`, text `0.88rem`.
* **`lg`:** Padding `0.75rem 1.6rem`, radius `28px`, icon size `18px`, text `0.95rem`.

---

## 3.5. Right Slide-Over Drawer (`RightDrawer.svelte`)
A persistent right-hand drawer used for the **Up Next Queue** and **Collection/Album Details**.

### Anatomy:
* **Fixed Width:** `400px` when open; slides out smoothly from right edge.
* **Chassis Background:** Volcanic mineral charcoal (`#08080a`) with a 1px whisper border along the left seam (`border-left: 1px solid rgba(255, 255, 255, 0.05)`).
* **Header Bar:** Sticky top header with title and close icon button (`Phosphor X`).
* **Content Area:** Smooth 150ms crossfade on title/content changes using `svelte/transition` (`fade`).

---

## 3.6. Navigation Spine (`Sidebar.svelte`)
The persistent left-hand navigation column that anchors primary routing.

### Anatomy:
* **Dimensions:** Width `83px` (`--sidebar-w`), expanding to `256px` when opened, `100vh` height.
* **Top Hub:** Application logo / identity anchor.
* **Primary Route Cluster:**
  * **Home** (`House` icon)
  * **Explore** (`Compass` icon)
  * **Library** (`Disc` icon, active for both Albums and Playlists)
  * **Extensions** (`PuzzlePiece` icon, sandboxed WASM provider hub)
* **Bottom Utility Cluster:**
  * **Settings** (`Gear` icon)
* **Active Indicator Pill:** A burnished brass specular pill that tracks the active navigation tab, translating smoothly along the Y-axis using `cubic-bezier(0.16, 1, 0.3, 1)` (`0.24s`).
* **Smoked Glass Material:** In glassy mode, blurs the underlying canvas by `28px` with whisper border along the right edge (`border-right: 1px solid rgba(255, 255, 255, 0.05)`).

---

## 3.7. Command Palette & Global Search (`GlobalSearch.svelte`)
A modal spotlight overlay triggered via `Ctrl+K` or `/` for instant, keyboard-driven navigation and playback.

### Anatomy:
* **Modal Backdrop:** Translucent dark obsidian overlay (`rgba(0, 0, 0, 0.65)`) with hardware `backdrop-filter: blur(12px)`.
* **Palette Window:** Width `640px`, height `500px` (max-width `92vw`), rounded corners (`16px`), surface background (`#101015`), whisper border (`rgba(255, 255, 255, 0.10)`).
* **Search Input Field:** Full-width autofocus text field with `MagnifyingGlass` leading icon, clear button (`X`), and soft chalk placeholder text.
* **Filter Filter Bar:** Scope pills (`All`, `Local`, `Remote`) for instant category filtering.
* **Direct Stream URL Card:** Special card that auto-detects pasted YouTube / streaming URLs, providing instant single-click or `Enter` playback.
* **Grouped Results:** Sectioned rows for Local Library Matches, Remote WASM Streams, and Albums/Artists with arrow-key keyboard navigation.

---

## 3.8. System Feedback & Toasts (`ToastContainer.svelte`)
Non-intrusive transient notifications communicating asynchronous task completion, errors, and system state.

### Anatomy:
* **Fixed Anchor:** Placed safely above the floating player bar: `bottom: 120px; right: 1.5rem; z-index: 200;`.
* **Toast Card:** Max-width `380px`, rounded corners (`12px`), smoked glass backing (`rgba(24, 24, 27, 0.65)`, blur `20px`), whisper border.
* **Semantic Iconography:**
  * **Success:** `CheckCircle` in Emerald (`#10b981`).
  * **Error:** `WarningCircle` in Crimson (`#ef4444`).
  * **Info:** `Info` in Brass (`#e2a973`).
* **Transition Physics:** Fluid upward entrance via Svelte `fly({ y: 20, duration: 400, easing: quintOut })` with smooth dynamic list reordering via `animate:flip`.

---

# 4. Badges & Technical Indicators

Technical metadata is treated with utmost precision to satisfy audiophile and power-user expectations:

| Indicator | Typography | Styling | Purpose |
| :--- | :--- | :--- | :--- |
| **Audio Format** | `IBM Plex Mono` (9px) | Subdued border, pill radius | Displays stream format: `FLAC`, `OPUS`, `MP3`, `WAV`. |
| **Hi-Res Badge** | `IBM Plex Mono` (9px) | Brass text (`#e2a973`), amber border | Highlights lossless streams >= 24-bit / 96kHz. |
| **Provider Tag** | `Inter Tight` (10px) | Muted charcoal background | Identifies track origin: `Local`, `YouTube`, `Radio`. |

---

# 5. Component Engineering DOs and DON’Ts

| DO | DON'T |
| :--- | :--- |
| Always respect `--player-clearance` (`10rem`) on scroll containers. | Never let scroll lists end flush with the bottom of the window (they will be hidden by PlayerBar). |
| Use Svelte 5 snippets (`children?.()`, `rightSnippet?.()`) for clean composition. | Never duplicate component structures just to add one trailing icon. |
| Rely on `--ease-liquid` for standard component hover and active transitions. | Never introduce ad-hoc CSS transition curves that feel disconnected from the app. |
| Maintain strict text truncation (`text-overflow: ellipsis; white-space: nowrap;`). | Never allow long song titles or artist names to wrap and cause layout shift. |
