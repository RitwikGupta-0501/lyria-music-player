# Lyria Visual Language & Aesthetic Specification

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-10  

---

# 1. Overview & Core Metaphor

The Lyria visual language is defined by two foundational concepts: **Tactile Audio Hardware** and **Quiet Luxury**.

Lyria is designed to feel like dedicated audio hardware rather than a typical web application. The interface emphasizes restraint, tactile precision, and quiet luxury - keeping the music and artwork front and center while ensuring controls remain understated, responsive, and tactile.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           QUIET LUXURY                                  │
│   Confidence through restraint. No garish neons. No loud gradients.     │
│   Every element exists with purpose. The music and artwork are heroes.  │
├─────────────────────────────────────────────────────────────────────────┤
│                     TACTILE AUDIO HARDWARE                              │
│   Deep mineral surfaces, brushed patina brass, and smoked glass.        │
│   Controls feel weighted, mechanical, and calibrated for precision.     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

# 2. Core Philosophy: The Pillars of Quiet Luxury

### 1. Content Before Chrome
The interface never competes with the musical content. Album artwork, musical dynamics, and artist narratives are the primary emotional anchors. The player chrome (navigation, transport controls, borders, and indicators) remains understated and respectful until called upon.

### 2. Quiet Confidence Through Restraint
Loud software screams for attention using oversaturated neon highlights, high-frequency animated banners, and aggressive gamification badges. Lyria achieves luxury through:
* Generous, intentional negative space (giving collections room to breathe).
* Masterful typographic hierarchy.
* Subtle, physics-damped interactions.
* Whisper-thin structural borders instead of jarring container outlines.

### 3. Native Fluidity as Ultimate Luxury
A laggy or stuttering interface destroys any illusion of luxury. Visual quality must never compromise:
* 60fps / 120fps smooth scrolling and rendering.
* Zero layout shift (CLS = 0) through reserved aspect ratios and skeleton geometry.
* Instantaneous state updates powered by Svelte 5 Runes directly synchronized with the underlying Rust audio daemon.

---

# 3. The Digital Materials

Instead of flat, arbitrary colors, Lyria’s interface is constructed from five simulated digital materials:

| Material | Physical Equivalent | Token Mapping | Role & Purpose |
| :--- | :--- | :--- | :--- |
| **Obsidian Slate** | Dark-room volcanic mineral stone | `--echo-void`, `--echo-base`, `--echo-surface` | Deep base canvas; eliminates eye fatigue during night listening. |
| **Chalk & Graphite** | Unbleached natural chalk | `--echo-text-1`, `--echo-text-2`, `--echo-text-3` | Soft, readable typography with zero high-contrast vibrating glare. |
| **Patina Brass** | Burnished acoustic brass / copper | `--echo-primary`, `--echo-primary-dark` | Warm, physical accent for needles, active states, and volume meters. |
| **Smoked Liquid Glass** | Precision optical studio glass | `--liquid-glass-bg`, `--liquid-glass-blur` | Semi-translucent floating decks letting album colors bleed softly. |
| **Whisper Borders** | Precision-machined chassis seams | `--echo-border`, `--echo-border-medium` | Micro-opacity structural lines defining boundaries imperceptibly. |

---

## 3.1. The Obsidian Mineral Canvas
Pure digital black (`#000000`) creates jarring optical vibration when paired with bright text on high-contrast OLED and IPS displays, and leads to severe pixel-smearing on scrolling.

Lyria’s background is a stepped hierarchy of deep mineral tones:
* **The Void (`#050507`):** The deepest foundation layer, reserved for the application window frame and backdrop recesses.
* **The Base (`#0a0a0c`):** The primary listening room canvas where scrollable content resides.
* **The Surface (`#101014`):** The baseline material for cards, list containers, and structural sections.
* **The Raised Layer (`#15151a`):** Elevated elements that respond to user presence (card hovers, active rows).

```css
/* Surface Hierarchy (src/app.css) */
--echo-void:    #050507;
--echo-base:    #0a0a0c;
--echo-surface: #101014;
--echo-raised:  #15151a;
--echo-overlay: #1a1a21;
```

---

## 3.2. Chalk on Slate Typography
Pure stark white (`#FFFFFF`) is forbidden for body and standard display copy. It tires the eye during prolonged listening sessions and feels stark and sterile.

Text hierarchy is rendered in warm, natural chalk tones:
* **Primary Chalk (`#eae8e3`):** High legibility for track titles, artist headlines, and active controls.
* **Graphite (`#7a7885`):** Secondary metadata, artist bylines, and inactive icons.
* **Muted Mineral (`#46464f`):** Timestamps, technical bitrates, track numbers, and disabled indicators.

```css
/* Text Hierarchy (src/app.css) */
--echo-text-1: #eae8e3; /* primary chalk */
--echo-text-2: #7a7885; /* secondary graphite */
--echo-text-3: #46464f; /* dim mineral / timestamps */
```

---

## 3.3. Patina Brass & Copper Accents
Digital music players frequently rely on electric neon blues or vivid magentas. Lyria draws its signature identity from **acoustic metals** - specifically brushed brass and burnished copper:

* **Brass Primary (`#e2a973`):** Evokes the warm metal of brass instruments, tube amplifiers, and turntable counterweights. Used for playback position heads, active repeat/shuffle indicators, and focused inputs.
* **Deep Copper (`#b58e62`):** Used for hover states, button borders, and secondary accents.

```css
/* Metallic Accent Tokens (src/app.css) */
--echo-primary:      #e2a973;
--echo-primary-dark: #b58e62;
```

> **Accent Application & Ambient Backdrop:** Currently, Brass (`#e2a973`) is the fixed signature accent across all controls and transport needles. Ambient backdrop reactivity is achieved via blurred CSS image projections (`filter: blur(...)`) in the Fullscreen and Spotlight views rather than dynamic programmatic color extraction. Global palette extraction remains a planned roadmap enhancement.

---

## 3.4. Smoked Liquid Glass
Lyria utilizes hardware-accelerated `backdrop-filter: blur()` to simulate heavy, smoked optical glass. This material is used exclusively on floating decks (the bottom transport player bar, the sidebar navigation, and slide-over drawers).

It allows the vibrant colors of album art and content underneath to softly permeate the controls without ever degrading the legibility of text or scrubber controls.

```css
/* Liquid Glass Tokens (src/app.css) */
--liquid-glass-bg:        rgba(12, 14, 18, 0.25);
--liquid-glass-card-bg:   linear-gradient(180deg, rgba(255, 255, 255, 0.045) 0%, rgba(255, 255, 255, 0.015) 50%, rgba(10, 12, 16, 0.12) 100%);
--liquid-glass-blur:      28px;
--liquid-glass-saturate:  190%;
```

---

## 3.5. Whisper Borders
To preserve visual calm, Lyria avoids heavy solid-color borders. Sections, cards, and dividers are delineated by **Whisper Borders** - white light projected at microscopic opacities:

```css
/* Whisper Borders (src/app.css) */
--echo-border:        rgba(255, 255, 255, 0.05); /* Default structural line */
--echo-border-medium: rgba(255, 255, 255, 0.10); /* Interactive card border */
--echo-border-strong: rgba(255, 255, 255, 0.15); /* Active / Focus border */
```

---

# 4. Spatial Architecture: The Z-Index Depth Hierarchy

Lyria establishes physical depth along the Z-axis. Elements are arranged in strict physical strata:

```
[Layer 50] Context Menus, Tooltips, Volume HUD   (z-index: 50)
    │
[Layer 40] Slide-over Drawers (Queue, Detail)     (z-index: 40)
    │
[Layer 30] Floating Smoked Decks (PlayerBar, Nav) (z-index: 30)
    │
[Layer 20] Interactive Cards, TrackRows, Shelves (z-index: 20)
    │
[Layer 10] Viewport Base & Canvas Content         (z-index: 10)
    │
[Layer 00] Ambient Album Art Projection Canvas   (z-index: 0)
```

### Depth Rules:
1. **The Backdrop (Layer 00):** Sits behind all content. Renders dynamic, blurred chromatic projections of the current track's artwork (blur radius >= 80px, opacity <= 0.15).
2. **The Chassis (Layer 10 & 20):** All scrolling content (albums, artists, search results, tracks) moves within this middle realm.
3. **The Floating Decks (Layer 30):** The bottom `PlayerBar` and left `Sidebar` permanently float *above* the chassis, casting soft, diffused ambient drop shadows (`box-shadow: 0 16px 32px rgba(0,0,0,0.4)`).
4. **The Transients (Layer 40 & 50):** Overlays, drawers, and menus cast deep shadows and dim the background layers subtly to focus listener intent.

---

# 5. Tactile Interaction & Damped Physics

Physical Hi-Fi equipment features weighted knobs lubricated with damping grease that resist erratic movement and deliver smooth, controlled feedback. Software interactions in Lyria mirror this physical reality:

* **Inertial Easing:** Transitions avoid harsh linear motion and springy cartoonish bounces. All state transitions use custom cubic-bezier curves with gentle deceleration:
  ```css
  transition: all 220ms cubic-bezier(0.16, 1, 0.3, 1);
  ```
* **Weighted Hover:** Hovering over an album card or action item does not jerk the element upwards. Instead:
  * The border opacity steps up from `0.05` to `0.12`.
  * The card surface lightens subtly from `--echo-surface` to `--echo-raised`.
  * The shadow deepens slightly to simulate lifting off the chassis.
* **Micro-Scale on Press:** Active mouse clicks or spacebar taps apply a micro-scale factor (`transform: scale(0.985)`), providing immediate tactile confirmation of the press.

---

# 6. Visual Design Rules: DOs and DON'Ts

| Aspect | DO | DON'T |
| :--- | :--- | :--- |
| **Surfaces** | Use deep mineral obsidian (`#050507` - `#101014`). | Never use pure `#000000` or sterile `#1a1a1a` grays. |
| **Typography** | Use soft chalk (`#eae8e3`) and reserve `Newsreader` for editorial titles and `IBM Plex Mono` for bitrates. | Never use pure `#ffffff` or generic default sans-serifs for headings. |
| **Accents** | Use burnished brass (`#e2a973`) with intentional restraint. | Never use saturated electric blues, greens, or purples. |
| **Borders** | Use whisper borders (`rgba(255,255,255,0.05)`). | Never use opaque 1px solid gray (`#333333`) borders. |
| **Glass** | Use heavy 28px blur with dark tint and subtle specular borders. | Never use thin 4px blurs or high-transparency frosty white backdrops. |
| **Spacing** | Provide generous padding (`2rem`–`3rem` page margins, `10rem` player clearance). | Never cram controls together to fill whitespace. |
| **Motion** | Use calibrated, damped decelerating curves (150ms–250ms). | Never use bouncy spring physics or long linear animations. |

---

# 7. Relationship to Code

All visual language tokens and classes are directly codified in:
* **Tokens & Material Variables:** [`src/app.css`](file:///home/ritwikg/Repository/echo-desktop/src/app.css)
* **Typeface Definitions:** [`static/fonts/fonts.css`](file:///home/ritwikg/Repository/echo-desktop/static/fonts/fonts.css)
* **Typographic Hierarchy:** [`docs/design/typography.md`](file:///home/ritwikg/Repository/echo-desktop/docs/design/typography.md)
