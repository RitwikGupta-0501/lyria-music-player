# 🎤 Lyrics in Lyria — User & Community Guide

Welcome to lyrics in Lyria! This guide covers everything you need to know about how lyrics work, how to enable community lyrics extensions, set your preferred sources, calibrate timing, and enjoy timed lyrics completely offline.

---

## 1. What Makes Lyria’s Lyrics Unique?

Unlike conventional streaming apps that lock you into a single proprietary database with missing songs and regional blocks, Lyria is built on an **open, sandboxed extension architecture**:

* **Universal Choice**: You decide where your lyrics come from. Whether it's open-source repositories like [LRCLIB](https://lrclib.net/), community databases, or personal servers, any developer can build a provider.
* **Intelligent Fallbacks**: Enabled multiple lyrics extensions? Lyria automatically tries your preferred provider first. If a niche track or live version isn't found, it seamlessly cascades to your secondary sources behind the scenes.
* **100% Local-First & Offline**: Once Lyria fetches lyrics for a song in your library, they are cached in your local SQLite database. You can hop on a flight or disconnect from Wi-Fi and still sing along with line-by-line sync.
* **Zero Telemetry Leakage**: Extension queries run inside a secure WebAssembly sandbox. Third-party providers cannot track your device, library, or playback history.

---

## 2. Quick Start: Enabling Your First Lyrics Provider

1. Open Lyria and click **Settings (⚙️)** in the sidebar.
2. Select **Extensions**.
3. Under the **Lyrics** section, click **Import Extension** or drop a `.wasm` / `.zip` lyrics package into your extensions directory.
4. Click the toggle switch next to your extension to set its status to **Enabled**.

Once enabled, whenever you play a song and open the lyrics drawer, Lyria will automatically resolve and sync the lyrics for you.

---

## 3. Preferred Providers & Fallbacks Explained

If you have more than one lyrics extension installed, you have complete control over how Lyria fetches them.

### Setting Your Preferred Provider
In **Settings → Extensions → Lyrics**:
* Select your **Preferred Provider** from the dropdown menu (e.g. *LRCLIB*).
* Whenever you play a track, Lyria will query this provider first.

### How the Fallback Cascade Works
```
           ┌─────────────────────────────┐
           │        Track Starts         │
           └──────────────┬──────────────┘
                          │
                          ▼
           ┌─────────────────────────────┐
           │  Check Local SQLite Cache   │ ─── [Hit] ───> Display Lyrics (<2ms)
           └──────────────┬──────────────┘
                          │ [Miss]
                          ▼
           ┌─────────────────────────────┐
           │   Preferred Provider Query  │ ─── [Match] ──> Cache & Display
           └──────────────┬──────────────┘
                          │ [Not Found / Error]
                          ▼
           ┌─────────────────────────────┐
           │  Secondary Providers (by    │ ─── [Match] ──> Cache & Display
           │  priority order)            │
           └──────────────┬──────────────┘
                          │ [All Exhausted]
                          ▼
           ┌─────────────────────────────┐
           │  "No lyrics found" Pill     │
           └─────────────────────────────┘
```

If your preferred source experiences network downtime or does not have the lyrics for an obscure B-side, you won't be left staring at a blank screen—Lyria queries your next enabled provider automatically without interrupting audio playback.

---

## 4. Switching Providers On-The-Fly

Sometimes a song might have lyrics on multiple services—for example, one provider might offer the original language while another provides Romanized or translated lines.

1. Open the **Lyrics Drawer** while listening to a song.
2. Click the **Source Pill** (e.g. `Source: LRCLIB`).
3. Select another enabled provider from the menu to immediately fetch and switch lyrics for the current song.

---

## 5. Adjusting Timing & Sync Offset

Occasionally, an external lyrics file might be slightly ahead of or behind your specific audio file (due to different masterings, intros, or silence padding).

* In the Lyrics drawer, use the **Offset (+/-)** controls to nudge timing in increments of `50ms` or `100ms`.
* An offset of `+500ms` delays the lyrics highlight by half a second to match late vocals.
* An offset of `-500ms` advances the lyrics highlight for early vocals.
* Lyria remembers your offset preference for that track.

---

## 6. Frequently Asked Questions (FAQ)

### Can I clear cached lyrics if a provider had a typo?
Yes! In **Settings → Storage & Cache**, click **Clear Lyrics Cache**, or right-click the track in your library and select **Refresh Lyrics** to fetch a fresh copy from your providers.

### Does Lyria support instrumental tracks?
Yes. If an extension marks a track as instrumental, Lyria gracefully displays an *Instrumental Track* visual indicator rather than a blank page.

### How much storage do cached lyrics use?
Negligible! Lyrics are stored as lightweight text and timestamp arrays in your local SQLite database. A library of 10,000 cached songs typically consumes less than 15MB of disk space.
