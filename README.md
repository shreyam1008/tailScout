# TailDesk

A clean, **native Rust GUI for Tailscale** on Linux — built with GTK4 + libadwaita.

No official Linux desktop client exists for Tailscale. TailDesk fills that gap: a
modern, low-footprint window to see your tailnet, connect/disconnect, manage exit
nodes, and send files over Taildrop — without living in the terminal.

**Built native.** TailDesk is written in Rust and uses GTK4 + libadwaita — the same
toolkit GNOME itself uses. No Electron, no web views, no bundled browser engine.
RAM usage is ~40–50 MB at runtime; that is the GTK4 framework floor, not bloat.

**Works across Linux desktops.** GTK4 runs natively on GNOME, KDE Plasma, XFCE,
Cinnamon, MATE, Budgie, and tiling compositors (i3, Sway, Hyprland). That covers the
vast majority of Linux desktop users. On GNOME it looks perfectly at home. On KDE and
others it renders its own Adwaita chrome — it won't match Breeze pixel-for-pixel, but
it works correctly and KDE's `breeze-gtk` package helps it blend in.

> Status: early (v0.1). Linux first. Windows and macOS planned.

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
- Auto version-matched: TailDesk reads the running daemon's version

## Requirements

- A running `tailscaled` and the `tailscale` CLI in `$PATH`
- The current user set as operator (so the GUI can act without root). TailDesk can open the native password prompt for this, or you can run:

  ```bash
  sudo tailscale set --operator=$USER
  ```

- Build deps: `libgtk-4-dev`, `libadwaita-1-dev`, `build-essential`, `pkg-config`

## Build & run

```bash
cargo run --release
```

## Architecture

- `src/tailscale/` — pure, unit-tested client + data models (CLI wrapper + LocalAPI socket reader)
- `src/settings.rs` — tiny XDG config helper for local preferences
- `src/ui/` — thin libadwaita view layer (`window.rs`, `device_row.rs`, `dialogs.rs`, `help.rs`, CSS)
- `tests/` — parsing and integration tests

See `AGENTS.md` for contributor rules and `CHANGELOG.md` for version history.

## License

MIT
