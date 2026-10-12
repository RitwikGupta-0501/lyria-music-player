# Offline Music & Downloads in Lyria

Welcome to Lyria's offline audio guide. This document explains how music downloads work, where your audio files are stored, how extension fallbacks keep your downloads reliable, and how you retain 100% ownership of your music library.

---

## 🎵 The Lyria Philosophy: True File Ownership

Unlike traditional closed streaming platforms that lock your offline songs inside proprietary encrypted caches, **Lyria believes you should own your music**.

* **Standard Audio Formats**: Every song downloaded through Lyria is saved as a clean, DRM-free audio file (`.flac`, `.opus`, `.m4a`, or `.mp3`).
* **Open Portability**: You can drag your downloaded songs to a USB drive, transfer them to a dedicated Digital Audio Player (DAP), or open them in external media players like VLC or Foobar2000.
* **Instant Offline Library**: The moment a download finishes, Lyria immediately indexes the track into your local music library (`Albums`, `Artists`, and `Tracks`), ready for gapless offline playback even with your Wi-Fi disconnected.

---

## ⚙️ How Downloading Works

Lyria's desktop app is an agile audio engine and local library daemon. It does not hardcode connections to any single music store or streaming platform. Instead, **community extensions** provide download capabilities.

```
 [ You click "Download" ]
            │
            ▼
 ┌──────────────────────────────────────────────────────────────┐
 │               Lyria Download Orchestrator                    │
 │  1. Checks your Preferred Download Extension                 │
 │  2. Resolves audio stream, bitrate, and container format     │
 │  3. Falls back seamlessly if a provider is missing the song  │
 └──────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
 ┌──────────────────────────────────────────────────────────────┐
 │                     Resumable Downloader                     │
 │  - Streams in chunks with pause/resume support               │
 │  - Saves to temporary '.part' file until 100% verified       │
 └──────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
 ┌──────────────────────────────────────────────────────────────┐
 │                     Instant Library Ingestion                │
 │  - Organizes into: ~/Music/Lyria/{Artist}/{Album}/...        │
 │  - Adds track to local database for immediate playback       │
 └──────────────────────────────────────────────────────────────┘
```

---

## 🧩 Setting Up Your Download Providers

To download music in Lyria, you need at least one extension with the `download` capability installed and enabled.

### 1. Enabling Extensions
1. Open **Settings** (gear icon in the navigation bar).
2. Go to **Extensions**.
3. Toggle on the extensions you want to use for downloads (e.g., lossless FLAC providers or high-bitrate web audio providers).

### 2. Choosing Your Preferred Provider
If you have multiple download extensions enabled, you can designate your favorite one as the **Preferred Provider**:
* Navigate to **Settings > Downloads**.
* Select your preferred extension from the dropdown menu.
* When you queue a song or album, Lyria will **always query your preferred provider first**.

### 3. Automatic Fallbacks
What happens if your preferred provider does not have the song or experiences a temporary rate limit?
* You don't have to do anything!
* Lyria automatically tests the next enabled download extension in your list.
* If a provider experiences consecutive network timeouts, Lyria's built-in **circuit breaker** temporarily pauses queries to that provider so your remaining queue continues downloading at full speed.

---

## 📂 Where Your Files Are Stored

By default, Lyria organizes your music in your system's native music folder:

* **Linux**: `~/Music/Lyria/`
* **macOS**: `~/Music/Lyria/`
* **Windows**: `C:\Users\<YourName>\Music\Lyria\`

### Clean, Predictable Folder Structure
Every song is organized cleanly into artist and album subfolders:

```
Music/
└── Lyria/
    └── Daft Punk/
        └── Random Access Memories/
            ├── 01 - Give Life Back to Music.flac
            ├── 02 - Get Lucky.flac
            └── 03 - Lose Yourself to Dance.flac
```

* **No Messy Filenames**: Special characters forbidden by operating systems (`:`, `/`, `\`, `*`, `?`) are automatically cleaned up.
* **Track Ordering**: Track numbers are padded with leading zeroes (`01`, `02`) so external players display albums in the correct track sequence.

---

## ⏯️ Managing Your Download Queue

In the sidebar or top bar, click the **Downloads** icon to open your Download Manager.

* **Progress & Speed**: See real-time progress bars and transfer speeds for active tracks.
* **Pause & Resume**: Have an unstable internet connection or need to save bandwidth? Click **Pause**. When you hit **Resume**, Lyria continues right where it left off using HTTP range streaming without re-downloading bytes you already received.
* **Cancel**: Stop a download and automatically remove any incomplete temporary `.part` files from your drive.

---

## ❓ Frequently Asked Questions

#### Can I download an entire album at once?
**Yes.** On any album view, click the **Download Album** button. Lyria enqueues all tracks in that album as a single coordinated job and downloads them sequentially or with up to 3 concurrent connections.

#### What happens if I go offline during a download?
If your Wi-Fi disconnects, the active track pauses safely. Once you reconnect to the internet, simply click **Resume** on the download card to continue from the last received byte.

#### What audio quality can I expect?
Quality depends on the extension fulfilling the request. Lyria's download subsystem prioritizes Lossless (FLAC/WAV) when offered by extensions, followed by high-bitrate Opus (256kbps), AAC, and MP3 (320kbps).
