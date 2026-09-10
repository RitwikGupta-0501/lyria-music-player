# Echo Music Player - Ideal End-Goal Screens & Views

This document defines the ultimate vision for the application's interface. Every screen should feel instantly responsive, logically structured, and visually breathtaking.

## 1. The Contextual Command Center (Dashboard)
* **The Ultimate Goal:** The app should know what you want to hear before you do.
* **Vision:**
  * **Dynamic Greetings:** A clean, time-aware top section (e.g., "Good Evening") paired with an algorithmically generated "Jump Back In" mix.
  * **Living Grids:** Album covers in the "Recently Added" or "Heavy Rotation" sections aren't just static images; they have subtle hover states that reveal play counts and instantly actionable "Play Next" buttons.
  * **Zero Clutter:** No ads, no social feeds—just your music, beautifully presented.

## 2. The Immersive "Now Playing" Experience (Cinematic Mode)
* **The Ultimate Goal:** A screen so beautiful you'll want to leave it open on a second monitor just to look at it.
* **Vision:**
  * **Living Backgrounds:** The background is a heavily blurred, slow-moving, fluid gradient extracted from the dominant colors of the current album cover. It subtly pulses and shifts with the music.
  * **Perfectly Synced Lyrics:** Large, beautifully rendered typography scrolls in perfect time with the vocals, with the current line highlighted brightly and future lines dimmed.
  * **Floating Glass Transport:** Playback controls sit on a frosted glass panel that seemingly floats above the background, fading away entirely when the mouse stops moving.

## 3. The Infinite Library Browser
* **The Ultimate Goal:** Navigating 100,000 tracks should feel exactly as fast as navigating 10 tracks.
* **Vision:**
  * **Buttery Smooth Scrolling:** Hardware-accelerated virtual lists mean you can drag the scrollbar from A to Z instantly without a single dropped frame.
  * **Deep Artist Profiles:** Clicking an artist slides open a breathtaking biography page (fetched via Lua plugins) alongside a perfectly categorized discography (LPs, EPs, Singles, Compilations).
  * **Instant Search:** A global search bar that updates results instantaneously as you type, intelligently grouping results by Track, Album, Artist, and Playlist.

## 4. The Intelligent Queue & Playlist Manager
* **The Ultimate Goal:** Total control over the listening session with zero friction.
* **Vision:**
  * **Drag-and-Drop Mastery:** The active queue sits in a sleek right-hand sidebar. You can drag albums, tracks, or entire folders into the queue seamlessly.
  * **The "Infinite" Toggle:** A beautifully animated toggle switch at the bottom of the queue. When engaged, the Lua algorithmic engine visually appends upcoming tracks to the queue, showing you exactly *why* they were chosen (e.g., "Matched by BPM and Genre").

## 5. The Power-User Hub (Settings & Plugins)
* **The Ultimate Goal:** Exposing extreme configuration power without intimidating the user.
* **Vision:**
  * **The Plugin Store:** A built-in, beautifully designed manager for Lua scripts. Install a new metadata scraper or lyrics fetcher with one click.
  * **Real-time Engine Stats:** A section for power users showing real-time stats from the Rust backend: audio buffer health, SQLite query times, and Lua memory usage.
  * **Aesthetic Control:** Sliders for UI scaling, background blur intensity, and custom typography choices that apply instantly without requiring an app restart.
