# Lyria - Arch Linux Packaging

This directory contains Arch Linux packaging files for Lyria.

## 1. Direct Installation via Pacman (Pre-built Release)

You can install the official `.pkg.tar.zst` release directly with `pacman`:

```bash
# Download and install the package
sudo pacman -U https://github.com/RitwikGupta-0501/lyria-music-player/releases/download/v0.2.3/lyria-0.2.3-1-x86_64.pkg.tar.zst
```

## 2. Building with `makepkg`

To build and install locally from this repository:

```bash
cd packaging/arch
makepkg -si
```

## 3. Arch User Repository (AUR)

This `PKGBUILD` is structured for publishing to the AUR as `lyria-bin`.
To update checksums for a specific release:

```bash
updpkgsums
makepkg --printsrcinfo > .SRCINFO
```
