# Lyria

<div align="center">

**A minimal, distraction-free desktop music player with algorithmic discovery.**

[![Release](https://img.shields.io/github/v/release/RitwikGupta-0501/lyria-music-player?color=amber&label=release)](https://github.com/RitwikGupta-0501/lyria-music-player/releases)
[![Platform: Linux | Windows | macOS](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-blue)](#installation)
[![License: MIT](https://img.shields.io/badge/license-MIT-emerald.svg)](LICENSE)

</div>

---

## What is Lyria?

**Lyria** is a lightweight, modern desktop music player designed for listeners who want to enjoy their music without the clutter of social feeds, intrusive telemetry, or heavy web apps. 

It pairs your local music collection with an intelligent **infinite playback loop**: drop in your favorite tracks, and Lyria will seamlessly queue up what comes next—whether that's through a local random walk across your collection or through community-built streaming extensions.

---

## Why Lyria?

Most modern music apps have become overloaded with podcasts, algorithmic recommendations you didn't ask for, and sluggish interfaces that consume hundreds of megabytes of RAM. Meanwhile, classic offline audio players often feel dated and lack ways to explore beyond what's already saved on your drive.

Lyria was created to strike the balance:
- **Instant & Lightweight:** Built on Rust and native desktop webviews. It launches immediately, responds smoothly, and stays quiet in the background.
- **Privacy First:** Your listening history and library stay on your computer. No analytics, no account requirements, no tracking.
- **Algorithmic, Not Curated:** Instead of manually building and maintaining static playlists, play a seed track and let Lyria organically explore your catalog or discover new music.
- **Extensible via WebAssembly:** Want to stream audio from external platforms? Community extensions run in a secure, sandboxed environment without bloating the core app.

---

## Highlights & Features

- **Infinite Playback Loop:** Play any song as a seed. When your queue nears the end, Lyria intelligently recommends and queues up related tracks using local acoustic tag distance or connected extensions.
- **High-Fidelity Audio:** Crystal-clear decoding for your local collection, supporting **FLAC, MP3, WAV, OGG, M4A, and OPUS**.
- **Modern, Distraction-Free Interface:** A clean, dark-mode interface built with subtle typography, fast virtualized scrolling for large libraries, and zero visual clutter.
- **Sandboxed Extensions:** Add new music sources via WebAssembly extensions with single-click import or drag-and-drop. Extensions run safely isolated with strict memory limits.
- **Native OS Controls:** Seamless integration with your desktop’s media keys, system notifications, and lock-screen controls (MPRIS on Linux, SMTC on Windows).
- **Built-in Diagnostics:** Includes an optional standalone debug console to inspect real-time audio telemetry, plugin logs, and engine events when you need them.

---

## Installation

### Pre-built Releases

Pre-compiled packages for Linux are available on the [**Releases**](https://github.com/RitwikGupta-0501/lyria-music-player/releases) page:

- **Linux AppImage:** Download the `.AppImage`, make it executable (`chmod +x Lyria*.AppImage`), and run it directly on any modern distribution.
- **Arch Linux:** Download the pre-built `.pkg.tar.zst` package or install via the provided `PKGBUILD`.
- **Debian / Ubuntu:** `.deb` packages are available with each release.

*(Windows and macOS native installers are planned for upcoming releases).*

---

## Building from Source

If you prefer to build Lyria from source, you will need:
- [Rust](https://rustup.rs/) (stable toolchain)
- [Bun](https://bun.sh/) (fast JavaScript runtime & package manager)
- System development packages (on Linux: `libasound2-dev`, `libssl-dev`, `pkg-config`)

```bash
# 1. Clone the repository
git clone https://github.com/RitwikGupta-0501/lyria-music-player.git
cd lyria-music-player

# 2. Install dependencies
bun install

# 3. Run in development mode
bun run tauri dev

# 4. Build a release binary for your platform
bun run tauri build
```

---

## Creating Extensions

Lyria features a sandboxed plugin engine powered by Extism WebAssembly. Anyone can build extensions in Rust, TypeScript, AssemblyScript, or any language that compiles to WebAssembly:

- Check out the [ABI Specification](docs/extensions/ABI_SPECIFICATION.md) to see available hooks.
- Read the [Manifest Schema](docs/extensions/MANIFEST_SCHEMA.md) for extension structure and packaging.
- Start quickly with the [Starter Templates](docs/extensions/STARTER_TEMPLATES.md).

---

## Roadmap & What's Next

Lyria is actively developed. Follow along with our upcoming milestones in [docs/VERSION_SCOPE.md](docs/VERSION_SCOPE.md) and [docs/TODO.md](docs/TODO.md):

- **v0.1.0** — Core audio engine, lock-free architecture, WASM extension sandbox, OS media keys. *(Shipped)*
- **v0.2.0** — Real-time diagnostic console, universal Linux packaging hardening, and dynamic provider fallbacks. *(Shipped)*
- **v0.3.0** — Custom right-click context menus, gapless crossfade, ReplayGain volume normalization, and system tray. *(In Progress)*
- **v0.4.0** — Synchronized scrolling lyrics and dynamic musical horizon exploration. *(Planned)*

---

## Contributing

Contributions, bug reports, and suggestions are welcome! Feel free to open an issue or submit a pull request on GitHub.

For developer guidelines and technical architecture rules, please read [AGENTS.md](AGENTS.md).

---

## License

Lyria is free, open-source software released under the [MIT License](LICENSE).
