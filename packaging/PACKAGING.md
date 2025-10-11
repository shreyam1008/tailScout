# TailScout — Packaging Guide

Publisher: Shreyam Adhikari (shreyam1008@gmail.com)

---

## Files in this directory

| File | Purpose |
| --- | --- |
| `dev.shre.TailScout.desktop` | Desktop entry for Linux |
| `dev.shre.TailScout.metainfo.xml` | AppStream metadata for Flathub/GNOME Software |
| `dev.shre.TailScout.flatpak.yml` | Flatpak manifest for Flathub submission |
| `icons/hicolor/scalable/apps/dev.shre.TailScout.svg` | App icon |
| `snap/snapcraft.yaml` | Snap Store packaging |

---

## 1. AUR (fastest, do this first)

### What to publish

- `tailscout-bin` — downloads pre-built binary from GitHub Releases

### Steps

```bash
# 1. Create AUR account at https://aur.archlinux.org
# 2. Add your SSH key in AUR settings
# 3. Clone the (empty) AUR package repo
git clone ssh://aur@aur.archlinux.org/tailscout-bin.git
cd tailscout-bin

# 4. Create PKGBUILD (see template below)
# 5. Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 6. Test install locally
makepkg -si

# 7. Push
git add PKGBUILD .SRCINFO
git commit -m "Initial release v0.1.2"
git push
```

### PKGBUILD template

```bash
# Maintainer: Shreyam Adhikari <shreyam1008@gmail.com>
pkgname=tailscout-bin
pkgver=0.1.2
pkgrel=1
pkgdesc="Native Linux GUI for Tailscale built with Rust and GTK4/libadwaita"
arch=('x86_64')
url="https://shreyam1008.github.io/tailScout/"
license=('MIT')
depends=('tailscale' 'gtk4' 'libadwaita')
provides=('tailscout')
conflicts=('tailscout' 'tailscout-git')

source_x86_64=("tailscout::https://github.com/shreyam1008/tailScout/releases/download/v${pkgver}/tailscout-linux-amd64")
sha256sums_x86_64=('SKIP')

package() {
    install -Dm755 "$srcdir/tailscout" "$pkgdir/usr/bin/tailscout"
    install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/dev.shre.TailScout.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=TailScout
Comment=Native Tailscale GUI
Exec=tailscout
Icon=dev.shre.TailScout
Terminal=false
Categories=Network;GTK;
StartupNotify=true
EOF
}
```

---

## 2. Flathub

### Prerequisites

- Add a 512x512 PNG icon at `packaging/icons/hicolor/512x512/apps/dev.shre.TailScout.png`
- Generate `cargo-sources.json` for offline Cargo build:

```bash
pip install aiohttp toml
python3 flatpak-cargo-generator.py Cargo.lock -o packaging/cargo-sources.json
# Tool: https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo
```

- Replace placeholder commit SHA in `dev.shre.TailScout.flatpak.yml`:

```bash
git ls-remote https://github.com/shreyam1008/tailScout refs/tags/v0.1.2
# Copy the SHA and paste into the flatpak manifest
```

### Sandbox note

TailScout calls the `tailscale` CLI and LocalAPI socket on the host.
Under Flatpak strict sandbox, wrap CLI calls with:

```rust
// Instead of: Command::new("tailscale").args(...)
// Use: Command::new("flatpak-spawn").args(["--host", "tailscale", ...])
// Only when running inside a Flatpak (check FLATPAK_ID env var)
```

### Test locally

```bash
flatpak install org.gnome.Platform//48 org.gnome.Sdk//48
flatpak install org.freedesktop.Sdk.Extension.rust-stable
flatpak-builder --force-clean build-dir packaging/dev.shre.TailScout.flatpak.yml
flatpak-builder --run build-dir packaging/dev.shre.TailScout.flatpak.yml tailscout
```

### Submit to Flathub

1. Fork https://github.com/flathub/flathub
2. Create directory `dev.shre.TailScout/`
3. Add `dev.shre.TailScout.yml`, `cargo-sources.json`, icons, metainfo, desktop file
4. Submit PR — follow https://docs.flathub.org/docs/for-app-authors/submission

---

## 3. Snap Store

### Prerequisites

```bash
sudo snap install snapcraft --classic
sudo snap install lxd && sudo lxd init --minimal
```

### Build

```bash
cd /home/shre/Desktop/me/tailScout
snapcraft

# Produces: tailscout_0.1.2_amd64.snap
```

### Register and upload

```bash
snapcraft login
snapcraft register tailscout
snapcraft upload tailscout_0.1.2_amd64.snap --release=stable
```

### Snap Store dashboard

https://snapcraft.io/account

---

## Before any release: checklist

- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets` clean
- [ ] Binary built: `cargo build --release`
- [ ] GitHub Release created with binary attached
- [ ] `dev.shre.TailScout.metainfo.xml` release entry added
- [ ] Screenshot added to docs/
- [ ] AUR PKGBUILD sha256 updated
- [ ] Flathub manifest commit SHA updated
- [ ] WinGet/Scoop manifests updated (when Windows support added)
