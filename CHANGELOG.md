# Changelog

All notable changes to TailDesk are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

We also track the **Tailscale version** TailDesk is verified against, since the
backend wraps the `tailscale` CLI and LocalAPI, which evolve over time.

## [Unreleased]

### Added
- Operator setup action using the native Polkit password prompt (`pkexec tailscale set --operator=$USER`) so TailDesk does not store or ask for sudo passwords.
- Tailnet/account overview showing current tailnet, MagicDNS state, signed-in owner, Tailscale version, and daemon health messages.
- Account/tailnet dialog backed by `tailscale switch --list --json`, with profile switching.
- Login/logout actions for managing Tailscale accounts from the accounts dialog.
- Taildrop receive action using `tailscale file get --conflict=rename`.
- Exit-node section explaining what exit nodes do, listing available exit-node peers, and supporting use/clear actions.
- Exit-node advertising action for offering this device as an exit node.
- Default Taildrop receive-folder preference stored in XDG config without adding dependencies.
- Help menu with TailDesk guide, Tailscale CLI cheatsheet, netcheck, admin console, version, bugreport, and About dialog.
- Copyable command output/error/device-detail dialogs.
- Richer device details: owner, allowed IPs, endpoint, DERP relay, last seen/handshake, key expiry, Taildrop availability, subnet router state, and traffic counters.

### Changed
- `tailscale up` now uses a bounded timeout so the GUI does not wait forever.
- The main view is now a two-column layout: overview and Taildrop on the left, devices top-right, exit-node controls below devices.
- `NeedsLogin` now makes the primary action `Log In` instead of trying a normal connect first.
- Device rows now show owner names, subnet-router badges, exit-node capability, and only show Taildrop send buttons when Tailscale reports the peer can receive files.
- Permission errors now open a repair dialog that can launch the Polkit operator setup.
- Permission and account errors now expose login, admin console, copy command, and copyable details flows.
- UI help text and reusable dialogs moved into `src/ui/help.rs` and `src/ui/dialogs.rs` to keep the view layer easier to maintain.

### Fixed
- Status parsing now tolerates `null` fields from `tailscale status --json` / LocalAPI, including `Health`, peers, users, IP lists, and scalar values.
- Connect/disconnect no longer gets stuck with the spinner/busy state after the command finishes.

### Verified
- Verified against **Tailscale 1.98.4** on Linux.
- `cargo test`: 19 passing.
- `cargo clippy --all-targets`: clean.

## [0.1.0] - 2026-06-09

Verified against **Tailscale 1.98.4**.

### Added
- Initial native Rust + GTK4 + libadwaita GUI for Linux.
- Tailscale backend module:
  - Typed data models (`Status`, `Peer`, `Self`, `PeerStatus`).
  - CLI wrapper for `status --json`, `up`, `down`, `file cp`.
  - LocalAPI unix-socket client for `/localapi/v0/status`.
- Main window listing tailnet devices with online status, OS, and IP.
- Connect / disconnect toggle.
- Per-device detail pane (IPs, DNS name, OS, online state).
- Send files to online devices via Taildrop.
- Live status refresh.
- Unit tests for status JSON parsing and model edge cases.
