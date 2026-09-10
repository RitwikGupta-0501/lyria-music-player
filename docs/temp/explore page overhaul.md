# Explore Page Overhaul: Master Implementation Plan

This plan details the complete redesign of Echo's **Explore Discovery Page**. We are replacing the standard mobile-style vertical list of carousels with an **editorial, audio-first magazine layout** that balances instant responsiveness, deep visual hierarchy, and sandboxed WebAssembly data ingestion.

---

## Visual Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────┐
│  [ 🔍 Search songs, albums, or providers...                  ] [Filter]│  ← Phase 2: Discovery Bar
├────────────────────────────────────────────────────────────────────────┤
│  FEATURED SPOTLIGHT (16:7 Editorial Banner)                            │
│  ┌──────────────────────────────────────────────────────────────────┐  │  ← Phase 2: Editorial Hero
│  │ [ High-Res Artwork / Gradient Mask ]  "Album of the Month"       │  │
│  │                                       Artist Name • 2026         │  │
│  │                                       [ Play Album ▷ ]           │  │
│  └──────────────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────────────┤
│  BROWSE BY CATEGORY (4-Column Symmetrical Grid)                        │  ← Phase 3: Category Grid
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │  Electronic  │ │  Classical   │ │ Jazz & Soul  │ │   Ambient    │   │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │  Hip-Hop     │ │  Rock        │ │ Focus/Study  │ │ Global Top   │   │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘   │
├────────────────────────────────────────────────────────────────────────┤
│  DISCOVERY SPLIT (60% / 40%)                                           │  ← Phase 3: Discovery Split
│  Top Global Tracks (Ranked 01–05)    │ New Releases (2x2 Grid)         │
│  01. Track Title — Artist   (03:45)  │ [ Art ] Album A   [ Art ] Album B│
│  02. Track Title — Artist   (04:12)  │ [ Art ] Album C   [ Art ] Album D│
│  03. Track Title — Artist   (02:58)  │ ────────────────────────────────│
│  04. Track Title — Artist   (05:10)  │ [ Art ] Album C   [ Art ] Album D│
│  05. Track Title — Artist   (03:22)  │                                 │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 4-Phase Detailed Roadmap

```
┌────────────────────────────────────────────────────────────────────────┐
│  PHASE 1: WASM ABI & Data Layer Overhaul                               │
│  • Define EditorialSpotlight struct & ModuleItem::Spotlight            │
│  • Extract Hero Spotlight & 8 curated Category Cards in youtube-wasm   │
│  • Build exploreStore.svelte.ts with instant cache hydration           │
├────────────────────────────────────────────────────────────────────────┤
│  PHASE 2: Discovery Bar & 16:7 Editorial Hero Banner                   │
│  • 44px Discovery Search Bar with #B58E62 brass accent MagnifyingGlass │
│  • 16:7 Widescreen Hero Banner with Serif typography & Play Album CTA  │
├────────────────────────────────────────────────────────────────────────┤
│  PHASE 3: Symmetrical 4-Column Category Grid & 60/40 Discovery Split   │
│  • 4-Column Grid of 16:9 squircles with pinned labels & hover glow     │
│  • 60% Ranked 01-05 Track Ledger + 40% 2x2 New Release Album Grid     │
├────────────────────────────────────────────────────────────────────────┤
│  PHASE 4: Category Hub Sub-View, Navigation & Defensive Polish         │
│  • Category Hub drilldown with smooth transition & Esc key dismissal   │
│  • 144px (pb-36) Player Island clearance & Typographic art fallbacks   │
└────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 1: WASM ABI & Data Layer Overhaul

**Purpose:** Equip our backend and sandboxed WASM extensions with the exact data structures needed to power the new layout (Hero Spotlight, 8 Category Cards, Top 5 Ranked Tracks, and 2x2 New Releases).

#### What We Will Build:
1. **Editorial Spotlight ABI (`extensions/youtube-wasm/src/lib.rs` & `src-tauri/src/providers/mod.rs`):**
   * Introduce a structured `EditorialSpotlight` model:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
     pub struct EditorialSpotlight {
         pub id: String,
         pub title: String,
         pub artist: String,
         pub cover_art_url: Option<String>,
         pub description: Option<String>,
         pub release_year: Option<String>,
         pub track_count: Option<u32>,
     }
     ```
   * Add `Spotlight(EditorialSpotlight)` as a first-class variant in `ModuleItem`.
2. **InnerTube Hero Extraction (`youtube-wasm`):**
   * Enhance `browse_innertube("FEmusic_explore")` to extract the primary featured hero release banner from InnerTube headers.
   * Provide curated categories (`Electronic`, `Classical`, `Jazz & Soul`, `Ambient`, `Hip-Hop`, `Rock`, `Focus/Study`, `Global Top`) with distinct backdrop color accents.
3. **Reactive Explore Store (`src/lib/stores/explore.svelte.ts`):**
   * Create reactive stores for:
     * `spotlight`: `EditorialSpotlight | null`
     * `rankedTracks`: `TrackResult[]` (the top 5 global tracks)
     * `newReleases2x2`: `AlbumItem[]` (4 featured albums)
     * `categoryGrid`: `GenreItem[]` (8 category cards)
     * `activeCategory`: `GenreItem | null` (current category sub-view)
   * **Instant Hydration:** Pre-populate cached/default data immediately on mount so the user never sees empty screens or layout shifts while remote WASM requests resolve in the background.

#### Phase 1 Verification:
* Run `cargo test --target x86_64-unknown-linux-gnu` in `extensions/youtube-wasm` (all tests passing).
* Recompile `extensions/youtube-wasm.wasm` targeting `wasm32-wasip1`.

---

### Phase 2: Discovery Bar & 16:7 Editorial Hero Banner

**Purpose:** Build the top two visual anchors of the Explore page: an integrated, elegant discovery search bar and a cinematic 16:7 hero spotlight banner.

#### What We Will Build:
1. **Integrated Discovery Search Bar:**
   * **Dimensions:** `44px` height, full width, rounded corners (`8px`).
   * **Color & Material:** Deep slate background (`#161618`) with a subtle `1px solid rgba(255, 255, 255, 0.06)` border.
   * **Visual Styling:** Standard Phosphor SVG `<MagnifyingGlass size={18} weight="bold" />` in a refined brass accent (`#B58E62`).
   * **Typography:** Monospace input (`font-mono`) with placeholder: *"Search songs, albums, or providers..."*.
   * **Functionality:** Dispatches instant local search results and filters discovery shelves in place.
2. **16:7 Widescreen Editorial Hero Banner:**
   * **Aspect Ratio:** Ultra-wide 16:7 (or 21:9 on wide monitors) with `12px` border radius and overflow clipping.
   * **Gradient Overlay:** Deep charcoal gradient mask (`linear-gradient(to right, rgba(18,18,22,0.95) 30%, rgba(18,18,22,0.4) 100%)`) layered over high-resolution cover art.
   * **Editorial Eyebrow:** `FEATURED SPOTLIGHT` in uppercase tracking with `#B58E62` brass coloring.
   * **Title & Metadata:** Bold, large Serif title (`font-serif text-3xl font-bold text-white`), artist name, release year, and a clean `Lossless` badge.
   * **Call To Action:** Instant `[ Play Album ▷ ]` primary button with glassmorphism hover highlights.

#### Phase 2 Verification:
* Check visual layout at 1280px, 1440px, and 1920px widths.
* Test 1-click `Play Album` on the hero banner to ensure audio starts immediately.

---

### Phase 3: Symmetrical 4-Column Category Grid & 60/40 Discovery Split Section

**Purpose:** Implement the tactile category grid and the asymmetrical 60/40 discovery ledger that gives Echo its distinct editorial density.

#### What We Will Build:
1. **Browse by Category (4-Column Symmetrical Grid):**
   * **Grid System:** CSS Grid with `repeat(4, 1fr)` (responsively collapsing to 2 columns on compact windows).
   * **Card Shape:** 16:9 rectangular squircles (`border-radius: 10px`).
   * **Card Styling:** Deep slate (`#1E1E22`) with low-opacity geometric/gradient accents tailored to each genre (e.g. deep amber for Jazz, electric cyan for Electronic, crimson for Rock).
   * **Typography:** Category label pinned to the top-left in bold sans-serif (`font-semibold text-lg text-[#EAEAEA]`).
   * **Micro-Interactions:** Hovering triggers border illumination (`border-color: rgba(255,255,255,0.2)`) and a smooth scale lift (`transform: scale(1.02)`).
2. **Discovery Split Section (60% / 40% Widths):**
   * **Left Side (60%) — Ranked Track Ledger:**
     * Clean ledger listing ranked tracks **01 through 05**.
     * Leading rank number in brass monospace (`01`, `02`, `03`... `#B58E62`).
     * `40x40px` cover art squircle, title, artist, and track duration (`03:45`).
     * Hover reveals play overlay; double-click triggers instant playback.
   * **Right Side (40%) — New Releases 2x2 Grid:**
     * 2x2 grid of 4 square album cards (`1:1` aspect ratio).
     * Album title in Serif font (`font-serif`), artist name in muted slate.
     * Hovering displays a floating play bubble for 1-click listening.

#### Phase 3 Verification:
* Verify the 60/40 split ratio and track ledger alignment.
* Verify double-clicking ledger rows and clicking 2x2 album cards immediately queues audio.

---

### Phase 4: Category Hub Sub-View, Navigation & Defensive Polish

**Purpose:** Deliver seamless category drilldowns with keyboard navigation and ensure defensive edge cases (no player collisions, no missing art glitches).

#### What We Will Build:
1. **Category Hub Sub-View Flow:**
   * Clicking any category tile smoothly transitions the canvas into the **Category Hub** without a full page reload.
   * Sub-Header features an interactive back button (`← Explore`) and the active category title.
   * Renders curated playlists and tracks specific to that genre channel.
   * **Keyboard Dismissal:** Global `keydown` handler listening for the `Escape` key to instantly return to the root Explore canvas.
2. **Defensive Clearance & Typographic Fallback:**
   * **Player Pill Collision (`pb-36` / `144px`):** Fixed bottom padding of `144px` on the scrollable canvas to guarantee that cards and action buttons are never obscured by the floating Dynamic Island player.
   * **Typographic Art Fallback:** When a provider returns null or missing cover art, render a dark charcoal squircle with the artist's first initial in a Serif font (`font-serif text-xl`).

#### Phase 4 Verification:
* Navigate into a category $\rightarrow$ press `Esc` $\rightarrow$ verify smooth return to root Explore view.
* Scroll to the very bottom $\rightarrow$ verify complete visibility above the player island.
* Run `bun run check` (0 errors, 0 warnings).
* Run `cargo check in src-tauri` (0 errors).

---

## User Confirmation

Please review this descriptive 4-phase plan. Once you confirm, we will begin with **Phase 1: WASM ABI & Data Layer Overhaul**!
