# Lyria

<div align="center">

**A clean, lightweight, premium, offline-first desktop music player.**

[![Release](https://img.shields.io/github/v/release/RitwikGupta-0501/lyria-music-player?color=amber&label=release)](https://github.com/RitwikGupta-0501/lyria-music-player/releases)
[![Platform: Linux | Windows | macOS](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-blue)](#installation)
[![License: MIT](https://img.shields.io/badge/license-MIT-emerald.svg)](LICENSE)

</div>

---

## What is Lyria?

**Lyria** is a clean, lightweight, premium, offline-first desktop music player designed for listeners who value audio fidelity, aesthetic simplicity, and ownership of their music.

Built with Rust, Tauri, and Svelte, Lyria delivers a fast, distraction-free experience for your local music collection. It stays lean by default, while offering a sandboxed WebAssembly extension system to expand the player's capabilities—such as streaming from remote platforms, fetching metadata, or adding community integrations—without compromising the core offline player.

---

## Why Lyria?

In a desktop ecosystem dominated by heavy Electron-based music players that bundle full Chromium and Node.js runtimes—routinely consuming hundreds of megabytes of RAM just to idle—Lyria was built with a fundamental ideology: **software should be featherlight**.

A music player should be a respectful background companion that launches instantly, sips minimal CPU and battery, and never competes with your workflow for memory. At the same time, classic offline audio players often feel dated, lack modern design polish, and cannot interface with modern music ecosystems.

Lyria was created as the modern, lightweight alternative:

- **Featherlight Alternative to Electron:** Built on a compiled Rust core and OS-native webviews via Tauri 2.0. Lyria uses a fraction of the memory of Electron applications, launches in milliseconds, and conserves battery life.
- **Offline-First & Private:** Your local library, metadata, and listening history live entirely on your device. Zero sign-in walls, zero analytics, zero background tracking.
- **Clean & Premium Design:** A distraction-free dark interface crafted with thoughtful typography, smooth micro-interactions, virtualized scrolling, and zero visual noise.
- **Modular & Extensible:** Instead of bloating the core player with bundled third-party services, Lyria features a sandboxed WebAssembly extension system. You choose if and when to plug in remote streaming providers, scrobblers, or metadata sources.

---

## Key Features

- **High-Fidelity Audio Playback:** Crystal-clear audio engine supporting lossy and lossless formats: **FLAC, MP3, WAV, OGG, M4A, and OPUS**.
- **Fast Local Library Browsing:** Effortlessly scan and organize thousands of local tracks with virtualized scrolling and instant search.
- **WebAssembly Extension Sandbox:** Safely extend the player with community extensions (e.g. YouTube Music remote playback) running isolated with strict memory and execution limits.
- **Continuous Listening:** Seamless queue management with intelligent autoplay that keeps the music going once your queue finishes.
- **Native OS Controls:** Integrates natively with desktop media controls, lock-screen widgets, and hardware media keys (MPRIS on Linux, SMTC on Windows).
- **Built-in Diagnostics:** Includes an optional standalone debug console (`/debug`) to monitor audio engine events, extension logs, and performance in real time.

---

## Installation

### Pre-built Releases

Pre-compiled packages for **Windows** and **Linux** are available on the [**Releases**](https://github.com/RitwikGupta-0501/lyria-music-player/releases) page:

#### Windows
- **Installer (`.exe`):** Standard desktop installer with desktop and Start Menu integration.
- **MSI Package (`.msi`):** Windows Installer package for system-wide and enterprise deployments.

#### Linux
- **AppImage:** Download the `.AppImage`, make it executable (`chmod +x Lyria*.AppImage`), and run it directly on any modern distribution.
- **Arch Linux:** Install via the pre-built `.pkg.tar.zst` release asset or the provided `PKGBUILD`.
- **Debian / Ubuntu:** Native `.deb` package.

*(macOS `.dmg` bundles are in active development).*

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

- Check out the [ABI Specification](docs/extensions/ABI_SPECIFICATION.md) to see available export hooks and host capabilities.
- Read the [Manifest Schema](docs/extensions/MANIFEST_SCHEMA.md) for extension configuration and packaging.
- Get started quickly using the [Starter Templates](docs/extensions/STARTER_TEMPLATES.md).

---

## Roadmap & What's Next

Lyria is actively developed. Follow along with our upcoming milestones in [docs/VERSION_SCOPE.md](docs/VERSION_SCOPE.md) and [docs/TODO.md](docs/TODO.md):

- **v0.1.0** — Core audio engine, lock-free architecture, WASM extension sandbox, OS media keys. *(Shipped)*
- **v0.2.0** — Real-time diagnostic console, universal Linux packaging hardening, and dynamic provider fallbacks. *(Shipped)*
- **v0.3.0** — Custom right-click context menus, gapless crossfade, ReplayGain volume normalization, and system tray. *(In Progress)*
- **v0.4.0** — Synchronized scrolling lyrics and dynamic musical horizon exploration. *(Planned)*

---

## Contributing & Feedback

We welcome feedback, feature suggestions, and bug reports! If you encounter an issue or have ideas to improve Lyria, please feel free to [open an issue on GitHub](https://github.com/RitwikGupta-0501/lyria-music-player/issues).

---

## License

Lyria is free, open-source software released under the [MIT License](LICENSE).
