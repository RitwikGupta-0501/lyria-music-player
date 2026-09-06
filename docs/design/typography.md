# Lyria Typography System

Version: 1.0

---

# Overview

Lyria's typography system is designed around three principles:

1. **Emotion** — music is an experience, not just data.
2. **Clarity** — the interface must remain fast and effortless.
3. **Precision** — audio information must feel technical and trustworthy.

The typography system uses three complementary typefaces:

| Role | Font | Purpose |
|---|---|---|
| Editorial | Newsreader | Emotional, artistic, musical identity |
| Interface | Inter Tight | Functional UI and interaction |
| Technical | IBM Plex Mono | Audio metadata and precision information |

Each font has a specific responsibility.

Do not mix these roles.

---

# Font Families

## 1. Editorial Font

### Newsreader

Purpose:
- Music identity
- Editorial moments
- Emotional content
- Curated experiences

CSS:

```css
--lyria-font-heading:
  "Newsreader",
  "Instrument Serif",
  "Source Serif 4",
  serif;
````

Used for content that represents the feeling of music.

---

## 2. Interface Font

### Inter Tight

Purpose:

* Navigation
* Controls
* Interaction
* General application UI

CSS:

```css
--lyria-font-body:
  "Inter Tight",
  system-ui,
  -apple-system,
  sans-serif;
```

Used for everything the user operates.

---

## 3. Technical Font

### IBM Plex Mono

Purpose:

* Audio telemetry
* Numbers
* Technical metadata
* System information

CSS:

```css
--lyria-font-mono:
  "IBM Plex Mono",
  ui-monospace,
  SFMono-Regular,
  monospace;
```

Used where precision matters.

---

# Typography Philosophy

## If it describes the music:

Use Newsreader.

Examples:

* Album names
* Artist names
* Curated shelves
* Discovery sections

## If it controls the music:

Use Inter Tight.

Examples:

* Buttons
* Navigation
* Menus
* Settings

## If it measures the music:

Use IBM Plex Mono.

Examples:

* Duration
* Bitrate
* Sample rate
* Codec information

---

# Usage Rules

---

# Newsreader

## Primary Usage

Newsreader represents the artistic layer of Lyria.

It should create the feeling of:

* vinyl sleeve notes
* music journalism
* album liner notes
* curated collections

---

## Components

### Shelf Titles

Example:

```
Forgotten Favorites

Adjacent Horizons

Heavy Rotation
```

---

### Artist Headers

Example:

```
Radiohead

A journey through alternative landscapes
```

---

### Album Titles

Example:

```
In Rainbows
```

---

### Hero Content

Example:

```
Tonight's Listening

A collection built around your recent discoveries.
```

---

### Empty States

Example:

```
Your library awaits.

Import your collection and let Lyria discover connections.
```

---

## Avoid

Do not use Newsreader for:

* Buttons
* Navigation
* Forms
* Dense lists
* Settings
* Technical information

---

# Inter Tight

## Primary Usage

Inter Tight is the operational layer of Lyria.

It should feel:

* fast
* precise
* invisible
* modern

---

## Components

### Navigation

Example:

```
Home
Library
Artists
Albums
Playlists
Settings
```

---

### Buttons

Example:

```
Play
Pause
Add to Queue
Download
```

---

### Search

Example:

```
Search your collection...
```

---

### Track Rows

Example:

```
Nude

Radiohead
In Rainbows
```

---

### Settings

Example:

```
Audio Engine

Output Device
Exclusive Mode
Buffer Size
```

---

### Tags

Example:

```
FLAC
Jazz
24-bit
96kHz
```

---

# IBM Plex Mono

## Primary Usage

IBM Plex Mono represents the engineering underneath the musical experience.

It should feel:

* precise
* stable
* instrument-like

---

## Components

### Playback Time

Example:

```
03:42 / 05:18
```

---

### Audio Quality

Example:

```
FLAC
24-bit / 96 kHz
1411 kbps
```

---

### Audio Inspector

Example:

```
Codec       FLAC
Sample      96000 Hz
Channels    Stereo
Bit Depth   24
```

---

### Keyboard Shortcuts

Example:

```
SPACE       Play/Pause

CMD + K     Search
```

---

### Extension Information

Example:

```
Provider:
WASM Audio Provider

Latency:
42ms
```

---

# Type Scale

## Display

Used for:

* Large artist names
* Hero content
* Main editorial moments

Font:
Newsreader

Weights:

* 400
* 500

Example:

```
48px
56px line-height
```

---

## Heading

Used for:

* Shelf titles
* Album titles
* Section headers

Font:
Newsreader

Weights:

* 400
* 500

Example:

```
32px
40px line-height
```

---

## UI Large

Used for:

* Important labels
* Primary actions

Font:
Inter Tight

Weights:

* 500
* 600

Example:

```
16px
24px line-height
```

---

## UI Default

Used for:

* Navigation
* Track information
* General interface

Font:
Inter Tight

Weights:

* 400
* 500

Example:

```
14px
20px line-height
```

---

## UI Small

Used for:

* Metadata
* Secondary labels

Font:
Inter Tight

Example:

```
12px
16px line-height
```

---

## Technical

Used for:

* Audio information
* Timing
* System values

Font:
IBM Plex Mono

Example:

```
12px
16px line-height
```

---

# Font Weight Guidelines

## Newsreader

Preferred:

* Regular (400)
* Medium (500)
* Italic

Avoid:

* Heavy bold usage

The editorial font should feel elegant, not loud.

---

## Inter Tight

Preferred:

| Weight | Usage             |
| ------ | ----------------- |
| 400    | Secondary text    |
| 500    | Standard UI       |
| 600    | Active states     |
| 700    | Important actions |

---

## IBM Plex Mono

Preferred:

| Weight | Usage              |
| ------ | ------------------ |
| 400    | Default telemetry  |
| 500    | Highlighted values |

Avoid excessive bold monospace.

---

# Color Pairing

Typography should work with Lyria's Obsidian & Brass palette.

## Primary Text

```
#EAE8E3
```

Use:

* Track titles
* Headings
* Active content

---

## Secondary Text

```
#7A7885
```

Use:

* Artist names
* Metadata
* Secondary navigation

---

## Technical Text

```
#46464F
```

Use:

* Durations
* Timestamps
* Low priority technical information

---

## Accent

```
#E2A973
```

Use:

* Active states
* Current track
* Important highlights

---

# Design Principle

Lyria typography should feel like:

* A vinyl record sleeve → Newsreader
* A precision instrument → Inter Tight
* A studio console → IBM Plex Mono

The typography should never compete with the music.

It should guide the user from:

Emotion → Interaction → Precision