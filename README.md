# TailScout

A clean, **native Rust GUI for Tailscale** on Linux — built with GTK4 + libadwaita.

No official Linux desktop client exists for Tailscale. TailScout fills that gap: a
modern, low-footprint window to see your tailnet, connect/disconnect, manage exit
nodes, and send files over Taildrop — without living in the terminal.

**Built native.** TailScout is written in Rust and uses GTK4 + libadwaita — the same
toolkit GNOME itself uses. No Electron, no web views, no bundled browser engine.
RAM usage is ~40–50 MB at runtime; that is the GTK4 framework floor, not bloat.

**Works across Linux desktops.** GTK4 runs natively on GNOME, KDE Plasma, XFCE,
Cinnamon, MATE, Budgie, and tiling compositors (i3, Sway, Hyprland). That covers the
vast majority of Linux desktop users. On GNOME it looks perfectly at home. On KDE and
others it renders its own Adwaita chrome — it won't match Breeze pixel-for-pixel, but
it works correctly and KDE's `breeze-gtk` package helps it blend in.

> Status: early (v0.1). Linux first. Windows and macOS planned.

## Quick install

Install the latest GitHub Release:

```bash
curl -fsSL https://raw.githubusercontent.com/shreyam1008/tailScout/main/scripts/install.sh | bash
```

The installer downloads the latest `tailscout-linux-x86_64.tar.gz` release asset,
detects existing installs, upgrades old versions automatically, refreshes launcher
metadata, and installs:

- **Binary:** `/usr/local/bin/tailscout`
- **Desktop launcher:** `/usr/local/share/applications/dev.shre.TailScout.desktop`
- **App icon:** `/usr/local/share/icons/hicolor/scalable/apps/dev.shre.TailScout.svg`

It uses `sudo` only if those system directories are not writable or if GTK4/libadwaita
runtime packages are missing. TailScout still needs Tailscale itself installed and
running.

## Features (v0.1)

- See your tailnet devices with online status, OS, and Tailscale IP
- Connect / disconnect (`tailscale up` / `down`)
- Set operator permission through the native Polkit password prompt
- View tailnet, signed-in user, MagicDNS, health, and Tailscale version
- Login, logout, and switch saved Tailscale accounts/tailnets
- Per-device detail: owner, IPs, routes, endpoint, relay, last seen, key expiry, traffic
- Send files to Taildrop-capable devices and receive Taildrop inbox files
- Save a default Taildrop receive folder in a tiny XDG config file
- View/select approved exit nodes and advertise this device as an exit node
- Help menu with app guide, CLI cheatsheet, netcheck, admin console, version, bugreport, and about
- Copyable command output/error dialogs for troubleshooting
- Live status refresh
- Auto version-matched: TailScout reads the running daemon's version

## Requirements

- A running `tailscaled` and the `tailscale` CLI in `$PATH`
- The current user set as operator (so the GUI can act without root). TailScout can open the native password prompt for this, or you can run:

  ```bash
  sudo tailscale set --operator=$USER
  ```

- Build deps: `libgtk-4-dev`, `libadwaita-1-dev`, `build-essential`, `pkg-config`

## Build & run

The local release binary is created at:

```text
target/release/tailscout
```

`target/` is gitignored on purpose. Do **not** commit compiled binaries to `main`;
GitHub Actions builds them from source and uploads them to GitHub Releases.

```bash
cargo run --release
```

To build and package a local release archive:

```bash
scripts/package-release.sh
```

This creates local artifacts under `dist/`:

- `tailscout-linux-x86_64.tar.gz`
- `tailscout-v<version>-linux-x86_64.tar.gz`
- `SHA256SUMS`

`dist/` is also gitignored because release files belong in GitHub Releases.

## Release process

Every push or pull request to `main` runs CI:

```bash
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

To publish a release:

```bash
git checkout main
git pull
scripts/tag-release.sh
```

That creates and pushes a tag like `v0.1.0`. GitHub Actions then builds the optimized
Linux binary, packages it, publishes a GitHub Release, and uploads checksums.

Package-manager installs (`.deb`, RPM, AUR, Flatpak, distro repos, app stores) are
planned later. For now the supported install path is the GitHub Release installer above.

## Website

The lightweight landing page lives in [`docs/`](docs/). It is a static GitHub Pages
site with no build step. The page uses Tailwind via CDN, a tiny local JavaScript
file, and SEO/discovery files including `robots.txt`, `sitemap.xml`, `llms.txt`,
`llms-full.txt`, and `site.webmanifest`.

To publish it, enable GitHub Pages in the repository settings:

- **Source:** Deploy from a branch
- **Branch:** `main`
- **Folder:** `/docs`

## Architecture

- `src/tailscale/` — pure, unit-tested client + data models (CLI wrapper + LocalAPI socket reader)
- `src/settings.rs` — tiny XDG config helper for local preferences
- `src/ui/` — thin libadwaita view layer (`window.rs`, `device_row.rs`, `dialogs.rs`, `help.rs`, CSS)
- `tests/` — parsing and integration tests

See `AGENTS.md` for contributor rules, `CONTRIBUTING.md` for how to contribute, and `CHANGELOG.md` for version history.

## Contributing

Bug reports, ideas, and pull requests are all welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — Copyright (c) 2026 Shreyam Adhikari. See [LICENSE](LICENSE).
