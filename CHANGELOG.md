# Changelog

All notable changes to TailScout are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

We also track the **Tailscale version** TailScout is verified against, since the
backend wraps the `tailscale` CLI and LocalAPI, which evolve over time.

## [Unreleased]

## [0.1.2] - 2026-06-10

### Changed
- Moved Admin Console access into the main Overview section instead of the header bar.
- Polished copyable output/error windows with clearer titles, larger layout, text padding, and a Copy All button with copied feedback.
- Improved device-detail popover spacing and key/value styling.
- Clarified Taildrop policy wording for sends that depend on Tailscale file-sharing rules.
- Replaced the app icon with a cleaner black-and-white TailScout mark inspired by Tailscale node geometry.
- Improved the one-line installer with install/upgrade detection, clearer status output, launcher/icon refresh, runtime checks, and recorded installed version metadata.
- Added a lightweight Tailwind-based GitHub Pages landing page with SEO metadata, schema, sitemap, robots, manifest, and LLM discovery files.

### Fixed
- Prevented duplicate file chooser windows and reset file chooser state on cancel/success to avoid the UI feeling stuck.

### Verified
- Verified against **Tailscale 1.98.4** on Linux.
- `cargo test`: 19 passing.
- `cargo clippy --all-targets`: clean.

## [0.1.1] - 2026-06-10

### Changed
- Replaced cramped device detail alerts with a scrollable key/value popover that dismisses when clicking outside.
- Collapsed Overview into one clickable row, with full overview details available on demand.
- Added direct Admin Console access.
- Improved permission and account dialogs with clearer logged-in vs operator-permission wording.
- Expanded operator permission commands with the real current username instead of a literal `$USER` placeholder.

### Fixed
- Hid Taildrop send actions for peers owned by a different Tailscale user when detectable.
- Added friendly Taildrop error messaging for `cannot send files: peer is owned by a different user`.

### Verified
- Verified against **Tailscale 1.98.4** on Linux.
- `cargo test`: 19 passing.
- `cargo clippy --all-targets`: clean.

## [0.1.0] - 2026-06-09

Verified against **Tailscale 1.98.4**.

### Added
- Initial native Rust + GTK4 + libadwaita GUI for Linux.
- Tailscale backend module:
  - Typed data models for `tailscale status --json` / LocalAPI state.
  - CLI wrappers for status, connect, disconnect, login/logout, Taildrop, profiles, exit nodes, diagnostics, and operator setup.
  - LocalAPI unix-socket client for `/localapi/v0/status` reads.
- Main two-column window with overview, devices, Taildrop, and exit-node controls.
- Account/tailnet dialog backed by `tailscale switch --list --json`.
- Taildrop send and receive actions.
- Default Taildrop receive-folder preference stored in XDG config.
- Help menu with TailScout guide, CLI cheatsheet, netcheck, admin console, version, bugreport, and About dialog.
- GitHub Actions CI/release workflow, release packaging scripts, one-line installer, desktop launcher, and icon packaging.
- MIT license and contributing guide.

### Changed
- Renamed the project from TailDesk to TailScout, including crate, binary, app ID, desktop file, icon, docs, scripts, and release assets.
- `tailscale up` uses a bounded timeout so the GUI does not wait forever.
- `NeedsLogin` makes the primary action `Log In` instead of trying a normal connect first.
- Permission errors open a repair dialog that can launch the Polkit operator setup.

### Fixed
- Status parsing tolerates `null` fields from `tailscale status --json` / LocalAPI.
- Connect/disconnect no longer gets stuck with the spinner/busy state after the command finishes.

### Verified
- `cargo test`: 19 passing.
- `cargo clippy --all-targets`: clean.
