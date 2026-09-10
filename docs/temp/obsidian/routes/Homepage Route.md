---
title: "Homepage Route"
type: route-documentation
aliases: ["/", "Home", "Homepage", "HomeView"]
tags:
  - type/docs
  - layer/frontend
  - ui/route
status: stable
related:
  - "[[Architecture MOC]]"
  - "[[Home Store]]"
  - "[[Home Shelves Architecture]]"
  - "[[Audio Engine Service]]"
  - "[[API - Home Recommendations Endpoints]]"
---

# Homepage Route

## In Plain English: What is the Homepage?
The **Homepage** is the front door of Echo. Instead of greeting you with a static list of files or an empty screen, it acts like a **personalized algorithmic cockpit**.

When you open Echo:
1. **Time-Aware Greeting:** It greets you based on your time of day (*Good morning*, *Good afternoon*, *Good evening*).
2. **Mood Selector:** You can immediately tailor what you want to hear (e.g. *Deep Focus*, *Energy & Drive*, *Late Night Drift*).
3. **Smart Personalization:** If you're a regular listener, it presents your favorite tracks, unfinished albums, fresh daily discoveries, and curated radio stations.
4. **Gentle Onboarding (Cold Start):** If it's your first time opening the app with zero listening history, it welcomes you and offers starter tracks directly from your local music library.

```mermaid
graph TD
    AppShell["+page.svelte (App Shell)"] -->|activeView === 'home'| HomeView["HomeView.svelte"]
    HomeView --> Header["Header (Time Greeting + Refresh)"]
    HomeView --> MoodBar["Mood Filter Bar (Focus, Chill, Energy, etc.)"]
    
    HomeView -->|New User (Zero History)| ZeroState["Welcome Hero & Local Library Starter Seeds"]
    HomeView -->|Returning User (Has History)| ActiveCockpit["The 7 Discovery Shelves"]

    HomeView -.->|reads & updates| HomeStore["[[Home Store]]"]
    HomeStore -.->|plays music| AudioService["[[Audio Engine Service]]"]
```

---

## Key User Experiences & Features

### 1. The Dynamic Mood Filter Bar
At the top of the homepage, six tactile mood pills let you instantly reshape your recommendation stream:
* **All** — Your complete, balanced listening rotation.
* **Deep Focus** — Low-distraction, instrumental, and ambient sounds for studying or coding.
* **Relax & Chill** — Laid-back, acoustic, and serene melodies for winding down.
* **Energy & Drive** — High-tempo, motivating beats for workouts or staying alert.
* **Commute** — Catchy favorites and road-trip mixes.
* **Late Night Drift** — Lo-fi beats, synthwave, and midnight jazz.

Clicking any mood pill instantly re-sorts your quick picks and generates tailored radio stations matching that vibe.

### 2. Seamless Onboarding: Cold-Start vs. Active Cockpit
Echo intelligently switches its layout depending on whether it has listening data:

| User Scenario | What the User Sees | Why It Exists |
| :--- | :--- | :--- |
| **New User (Zero Plays)** | **Welcome Hero Banner + Starter Seeds** | Avoids an awkward empty screen. Introduces the app's features and shows random tracks from the user's scanned library to kick off listening. |
| **Active Listener** | **The 7 Discovery Shelves** | Full personalized feed with quick picks, ongoing albums, new recommendations, and throwback favorites. |

---

## Technical Architecture & Implementation

### File Structure & Coordinates
* **Route Container:** `src/routes/+page.svelte` (controls high-level view switching and drawer open states)
* **Home View Component:** `src/lib/components/HomeView.svelte` (renders header, mood bar, and shelves)
* **State Management:** [[Home Store]] (`src/lib/stores/home.svelte.ts`)
* **Visual Components:** [[Home Shelves Architecture]] (`src/lib/components/home/*.svelte`)
* **Backend Bridge:** [[API - Home Recommendations Endpoints]] (`src-tauri/src/lib.rs`)

### Responsive Three-Column Canvas
The homepage lives within a fluid, responsive 3-column app container:
1. **Left Sidebar:** Fixed navigation bar (`240px`).
2. **Main Canvas:** Scrollable feed (`1fr`) with `10rem` bottom padding so the floating `[[PlayerBar Component]]` never blocks content.
3. **Right Drawer:** Slide-out drawer (`0px` to `400px` controlled by `--drawer-w`) for inspectable album/playlist details and the Up Next queue.

```svelte
<!-- src/routes/+page.svelte excerpt -->
<script lang="ts">
    let activeView = $state("home");
    let queueOpen = $state(false);
</script>

<div class="app-container">
    <Sidebar bind:activeView />
    <main class="main-content">
        {#if activeView === "home"}
            <HomeView bind:activeView />
        {/if}
    </main>
    <RightDrawer ... />
</div>
```

---

## Global Shortcuts & Event Integration

The homepage responds to keyboard shortcuts and custom application events:
* **Global Search (`echo:search`):** Opens the fuzzy search overlay (`Ctrl+K` or `/`).
* **Explore Navigation (`echo:navigate-explore`):** Switches directly to the global explore page.
* **Escape (`echo:escape`):** Closes open details drawers and search modals without losing your place.

---

## Related Notes
* [[Home Store]] — The reactive state store powering all homepage data.
* [[Home Shelves Architecture]] — The detailed guide to each of the 7 recommendation shelves.
* [[API - Home Recommendations Endpoints]] — Rust backend endpoints calculating recommendations.
* `[[Audio Engine Service]]` — Audio playback engine that receives music when items are clicked.
