# TailScout for macOS

This is the native macOS TailScout preview built with SwiftPM and SwiftUI.
It has no Xcode project. The app shells out to the official `tailscale` CLI and
keeps parsing/command logic in `TailScoutCore` so it can be tested without a UI.

## Requirements

- macOS 13 or newer
- Xcode command line tools with SwiftPM
- Tailscale installed, with `tailscale` available on `PATH`
- A running `tailscaled`/Tailscale service

## Build and Test

From this directory:

```bash
cd platform/macos
swift test
swift build -c release
```

Build a local `.app` bundle:

```bash
cd platform/macos
SIGNING_MODE=adhoc Scripts/package_app.sh release
open .build/app/TailScout.app
```

Build a universal app bundle for CI on macOS:

```bash
cd platform/macos
ARCHES="arm64 x86_64" SIGNING_MODE=adhoc Scripts/package_app.sh release
```

Fast local build, package, and launch:

```bash
cd platform/macos
Scripts/compile_and_run.sh --test
```

Measure the packaged SwiftUI app's RSS after launch/settle:

```bash
cd platform/macos
Scripts/measure_rss.sh --no-build --baseline-mib 120
```

To package first and then measure:

```bash
cd platform/macos
Scripts/measure_rss.sh --build release --baseline-mib 120
```

The RSS helper samples the app process with built-in macOS tools and fails if
the sampled peak RSS exceeds the configured baseline.

The package script writes:

- `.build/app/TailScout.app`
- `.build/app/TailScout-<version>.zip` when `ditto` is available

## CI Shape

A macOS GitHub Actions job can use:

```bash
cd platform/macos
swift test
ARCHES="arm64 x86_64" SIGNING_MODE=adhoc Scripts/package_app.sh release
```

Release signing can pass:

```bash
APP_IDENTITY="Developer ID Application: Example" Scripts/package_app.sh release
```

Notarization is not included in the preview release pipeline.

## Implemented Workflows

- Refresh status and devices via `tailscale status --json`
- Load saved accounts/tailnets via `tailscale switch --list --json`
- Connect, disconnect, login, logout
- Switch a saved account/tailnet
- Set and clear an approved exit node
- Advertise or stop advertising this Mac as an exit node
- Send one Taildrop file to a capable peer
- Receive Taildrop files into a selected folder with conflict renaming
- Run `tailscale version`, `tailscale netcheck`, and `tailscale bugreport`

All CLI calls are async and run off the main actor.

## Limitations

- This preview remains separate from the Linux GTK UI so both stay native.
- Reads use the CLI only. The macOS LocalAPI path is not implemented yet.
- Taildrop receive depends on the installed CLI supporting
  `tailscale file get --conflict=rename <folder>`.
- Taildrop send is one file at a time.
- The app does not grant operator/admin permissions. If a command needs higher
  privileges, configure Tailscale operator permissions outside the app.
- No sandbox entitlements, notarization, app icon, Sparkle updater, or menu bar
  mode are included yet.

Cross-platform behavior and UI terminology are defined in
[`../../shared/README.md`](../../shared/README.md).
