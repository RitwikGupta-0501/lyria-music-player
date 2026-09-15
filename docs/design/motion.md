# Lyria Motion & Interaction Specification

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-10  

---

# 1. Overview & Motion Philosophy

Motion in Lyria is designed around **Damped Inertia** and **Tactile Feedback**.

In line with the **Quiet Luxury** ethos, motion is never decorative or distracting. Elements do not bounce, whip, or perform playful loops. Instead, transitions feel like physical, precision-engineered hardware components—lubricated with damping grease that gives them weighted resistance and smooth deceleration.



---

# 2. Timing Functions & Easing Curves

All easing curves in Lyria are asymmetric, featuring steep initial acceleration followed by extended, smooth deceleration.

| Curve Token / Value | CSS Definition | Character | Primary Use Cases |
| :--- | :--- | :--- | :--- |
| `--ease-liquid` | `cubic-bezier(0.16, 1, 0.3, 1)` | Weighted Damping | Standard UI transitions, buttons, drawer reveals, seek bar thumb scaling. |
| `--ease-cinematic` | `cubic-bezier(0.05, 0.75, 0.15, 1)` | Heavy Inertial Glide | Artwork zoom, album card hover elevation, video/carousel sliding. |
| `--ease-standard` | `cubic-bezier(0.4, 0, 0.2, 1)` | Subtle Deceleration | Utility chevrons, toggle switches, simple opacity crossfades. |

### CSS Codification (`src/app.css`)
```css
/* Liquid Physics Curves (src/app.css) */
--ease-liquid: cubic-bezier(0.16, 1, 0.3, 1);
--duration-liquid: 0.4s;
```

---

# 3. Duration Hierarchy

Timing durations are strictly partitioned by the scale and physical weight of the UI element:

| Duration Category | Time Range | Usage Context | Example Component |
| :--- | :--- | :--- | :--- |
| **Micro / Tactile** | `120ms` – `150ms` | Active button presses, icon color changes, subtle text reveals. | Transport button press (`scale(0.985)`), volume mute toggle. |
| **Standard UI** | `180ms` – `250ms` | Popovers, hover overlays, pill button reveals, dropdown menus. | `PillButton.svelte`, `CustomSelect.svelte`, `CardPlayButton.svelte`. |
| **Structural** | `400ms` – `550ms` | Morphing containers, drawer slide-overs, artwork scale glide. | `PlayerBar.svelte` pill expansion (`340px` to `720px`), `AlbumCard.svelte` art zoom. |

---

# 4. Micro-Interactions & Hover Mechanics

### 4.1. Three-Phase Card Hover
When a listener hovers over an album card or playable shelf tile, the element does not abruptly jerk. It undergoes a synchronized three-tier lift:
1. **Surface Lift:** Card surface steps from `--echo-surface` to `--echo-raised`.
2. **Border Catch:** Border opacity transitions smoothly from `0.05` to `0.12` (`--echo-border-medium`).
3. **Art Scale & Shadow:** Artwork gently scales to `1.05` using `--ease-cinematic` (`0.55s`) while drop shadow deepens to create physical elevation:
   ```css
   /* AlbumCard.svelte */
   .art-img {
     transition: transform 0.55s cubic-bezier(0.05, 0.75, 0.15, 1);
   }
   .album-card:hover .art-img {
     transform: scale(1.05);
   }
   ```

### 4.2. Tactile Press (Click Micro-Compression)
Active mouse clicks or keyboard triggers apply a subtle physical depression to confirm input:
```css
.btn:active,
.pill-button:active {
  transform: scale(0.985);
  transition: transform 0.12s var(--ease-liquid);
}
```

### 4.3. Top-Edge Seek Scrubber (Fitts’s Law)
The playback progress bar on `PlayerBar.svelte` provides continuous, physics-damped feedback:
* **Resting State:** Thin 2px progress track with hidden thumb indicator (`opacity: 0`, `scale(0.5)`).
* **Hover State:** Track expands to 4px; brass thumb scales smoothly to `scale(1.0)` and fades in over `200ms`.
* **Drag / Scrub:** Immediate 1:1 needle tracking with zero latency, accompanied by a soft brass accent glow (`box-shadow: 0 0 8px rgba(226, 169, 115, 0.5)`).

---

# 5. Structural Transitions & Drawers

### 5.1. Player Bar State Morphing
The floating `PlayerBar` dynamically alters its footprint depending on playback state:
* **Idle State:** Compact `340px` width.
* **Active / Playing State:** Expanded `720px` full transport deck.
* **Transition:** Smooth width interpolation over `0.5s` using `cubic-bezier(0.16, 1, 0.3, 1)`, preventing abrupt layout pops.

### 5.2. Slide-Over Drawers (`RightDrawer.svelte`)
The queue and collection detail drawers slide out from the right margin:
* Width animates from `0px` to `400px`.
* Internal content smoothly fades in over `150ms` using `svelte/transition` (`fade`).
* The root viewport simultaneously adjusts CSS layout variable `--drawer-w` to prevent content clipping.

---

# 6. Performance & Rendering Guardrails

To preserve 60fps / 120fps display refresh rates across all platforms:

1. **Composite-Only Properties:**  
   Transitions must animate **only** `transform` and `opacity`. Never animate properties that trigger browser layout reflow (such as `top`, `left`, `width`, `height`, `margin`, or `padding`) during continuous micro-interactions.
2. **GPU Promotion:**  
   Elevated cards and zoom containers must include hardware-acceleration hints:
   ```css
   transform: translateZ(0);
   backface-visibility: hidden;
   will-change: transform;
   ```
3. **Respect Reduced Motion:**  
   All animations honor the user’s operating system preferences:
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

---

# 7. Motion DOs and DON’Ts

| Area | DO | DON'T |
| :--- | :--- | :--- |
| **Physics** | Use decelerating curves with smooth stops (`--ease-liquid`). | Never use spring or elastic physics that cause elements to oscillate or bounce. |
| **Pacing** | Keep micro-interactions under `200ms` to feel immediate. | Never exceed `300ms` for basic dropdowns or button states. |
| **Elevation** | Combine subtle scale (`1.02`–`1.05`) with shadow deepening. | Never translate cards by large vertical distances on hover (e.g. `-10px`). |
| **Feedback** | Use micro-compression (`scale(0.985)`) on click. | Never leave clickable controls without visible press feedback. |
