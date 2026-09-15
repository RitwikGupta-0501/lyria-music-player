# Lyria Color System & Palette Reference

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-10  

---

# 1. Overview

Lyria’s color system is engineered for **long-duration listening comfort** and **tactile elegance**.

The color architecture enforces three core rules:
1. **No Pure Extremes:** Pure `#000000` and pure `#FFFFFF` are strictly forbidden. High-contrast stark white on pitch black causes ocular fatigue, text vibration, and OLED pixel smearing.
2. **Mineral Hierarchy:** Backgrounds use stepped volcanic mineral tones (Obsidian, Charcoal, Slate) that give physical depth to cards and drawers.
3. **Restrained Metallic Accents:** Color is used for purpose, not decoration. The signature accent is acoustic brass (`#E2A973`), used with restraint for active needles, scrubber thumbs, and focused states.

All color variables are codified in [`src/app.css`](file:///home/ritwikg/Repository/echo-desktop/src/app.css).

---

# 2. Surface Hierarchy (Obsidian Canvas)

Surfaces are arranged from the deepest foundational background to foreground floating layers:

| Token | Hex Value | Role | Usage Context |
| :--- | :--- | :--- | :--- |
| `--echo-void` | `#050507` | Foundation Void | Application window perimeter, backdrop gutters, and root background. |
| `--echo-base` | `#0a0a0c` | Primary Canvas | Default listening room surface for scrollable views (Home, Explore, Library). |
| `--echo-surface` | `#101014` | Neutral Container | Resting state for album cards, tracklist headers, and section tiles. |
| `--echo-raised` | `#15151a` | Elevated Element | Card hover states, active list selections, and popover panels. |
| `--echo-overlay` | `#1a1a21` | Foreground Deck | Modal sheets, context menus, and elevated floating widgets. |

### CSS Definition
```css
/* Surface Hierarchy (src/app.css) */
--echo-void:    #050507;
--echo-base:    #0a0a0c;
--echo-surface: #101014;
--echo-raised:  #15151a;
--echo-overlay: #1a1a21;
```

---

# 3. Text & Typographic Hierarchy (Chalk on Slate)

Typography uses warm, unbleached chalk and graphite tones to ensure high legibility without harsh glare:

| Token | Hex Value | Name | Usage & Role | Contrast (vs Base) |
| :--- | :--- | :--- | :--- | :--- |
| `--echo-text-1` | `#eae8e3` | Primary Chalk | Track titles, artist headlines, album headers, and active action labels. | ~15.2:1 (AAA) |
| `--echo-text-2` | `#7a7885` | Secondary Graphite | Artist bylines, album artist credits, navigation labels, and inactive icons. | ~4.7:1 (AA) |
| `--echo-text-3` | `#46464f` | Muted Mineral | Timestamps, durations, technical bitrates, track index numbers, and subtle badges. | ~2.5:1 (Subdued) |

### Usage Rules:
* Always use `var(--echo-text-1)` for text requiring immediate legibility.
* Never use `var(--echo-text-3)` for interactive buttons or critical form labels; reserve it for non-essential technical metadata.

---

# 4. Metallic Accents (Acoustic Brass & Copper)

Lyria rejects generic neon primaries (saturated blues, purples, or greens) in favor of warm, burnished metallic tones:

| Token | Hex Value | Role | Usage |
| :--- | :--- | :--- | :--- |
| `--echo-primary` | `#e2a973` | Patina Brass | Scrubber progress head, active repeat/shuffle buttons, audio visualizer needles, and primary action buttons. |
| `--echo-primary-dark` | `#b58e62` | Burnished Copper | Hover states on primary buttons, focused borders, and secondary accent fills. |

### CSS Definition
```css
/* Metallic Accent Tokens (src/app.css) */
--echo-primary:      #e2a973;
--echo-primary-dark: #b58e62;
```

> **Note on Ambient Reactivity:** Currently, Brass (`#e2a973`) is the fixed signature accent across all controls and transport needles. Ambient backdrop reactivity is achieved via blurred CSS image projections (`filter: blur(...)`) in the Fullscreen and Spotlight views rather than dynamic programmatic color extraction. Global palette extraction remains a planned roadmap enhancement.

---

# 5. Whisper Borders & Specular Edges

Borders are translucent structural seams made from pure white at microscopic opacities. This provides clear architectural organization without harsh, opaque lines:

| Token | CSS Value | Purpose | Usage |
| :--- | :--- | :--- | :--- |
| `--echo-border` | `rgba(255, 255, 255, 0.05)` | Structural Seam | Default divider lines, list borders, and inactive card outlines. |
| `--echo-border-medium` | `rgba(255, 255, 255, 0.10)` | Interactive Edge | Card hover borders, input fields, and panel boundaries. |
| `--echo-border-strong` | `rgba(255, 255, 255, 0.15)` | Active Highlight | Selected rows, keyboard focus rings, and active toggle borders. |

---

# 6. Smoked Glass Materials

Lyria uses dark smoked glass with hardware-accelerated diffusion for floating decks (e.g. `PlayerBar`, `Sidebar`, and `QueueDrawer`):

| Token | Value | Role |
| :--- | :--- | :--- |
| `--liquid-glass-bg` | `rgba(12, 14, 18, 0.25)` | Deep semi-transparent smoked tint. |
| `--liquid-glass-blur` | `28px` | Heavy optical diffusion. |
| `--liquid-glass-saturate` | `190%` | Color amplification of background content bleeding through. |
| `--liquid-border-outer` | `rgba(255, 255, 255, 0.16)` | Catch-light edge for floating glass decks. |
| `--liquid-shadow-ambient` | `0 8px 24px -4px rgba(0, 0, 0, 0.40)` | Soft ambient occlusion under floating glass. |

---

# 7. Semantic Status Tokens

Status colors are balanced to ensure high visibility without jarring the dark listening room aesthetic:

| State | Color / Token | Hex Value | Usage |
| :--- | :--- | :--- | :--- |
| **Info / Focus** | `var(--echo-primary)` | `#e2a973` | Information toasts, active connection indicators, and focus rings. |
| **Success** | Emerald | `#10b981` | Completed track scan toasts, active extension verification, and saved playlist confirmation. |
| **Warning** | Brass / Amber | `#f59e0b` | Network retry indicators, rate limit notices, and storage warnings. |
| **Error** | Crimson | `#ef4444` | Decoders failure, missing audio files, and sandbox panic alerts. |

---

# 8. Implementation Guide for Components

### Do:
* Use standard surface tokens directly:
  ```svelte
  <div class="card">...</div>
  <style>
    .card {
      background: var(--echo-surface);
      border: 1px solid var(--echo-border);
      color: var(--echo-text-1);
    }
    .card:hover {
      background: var(--echo-raised);
      border-color: var(--echo-border-medium);
    }
  </style>
  ```
* Use text tokens strictly according to hierarchy (Text 1 for titles, Text 2 for artists, Text 3 for durations).

### Don't:
* Never introduce arbitrary hardcoded hex codes (e.g., `#222`, `#333`, `#fff`).
* Never use saturated primaries (e.g. `#007acc` or `#6366f1`) for buttons or sliders.
* Never set opacity on text elements (e.g. `opacity: 0.7`); use the dedicated `--echo-text-2` and `--echo-text-3` tokens instead to preserve crisp sub-pixel font rendering.
