# Native Platform Notes

TailScout keeps each desktop shell native:

- Linux: Rust + GTK4/libadwaita in `src/ui/`
- Windows: WinUI 3 in `platform/windows/`
- macOS: SwiftUI/SwiftPM in `platform/macos/`

The shared rule is that Tailscale behavior stays CLI/API-backed and testable.

## Runtime Footprint Checks

Each client has a platform-specific measurement path:

- Linux: `scripts/measure-memory.sh` (PSS smoke test with `TAILSCOUT_MEMORY_LIMIT_MIB`)
- Windows: `platform/windows/scripts/Measure-TailScoutWindowsMemory.ps1`
- macOS: `platform/macos/Scripts/measure_rss.sh`

Current targets and exact command examples are documented in
[`docs/runtime-footprint.md`](runtime-footprint.md).

## Tailscale Surface

The cross-platform surface that is stable enough for all three native clients is
the `tailscale` CLI:

- `tailscale status --json` for daemon state, this device, peers, users, health,
  Taildrop capability, exit-node flags, and version metadata.
- `tailscale up`, `tailscale down`, `tailscale login`, and `tailscale logout`
  for connection lifecycle.
- `tailscale switch --list --json` and `tailscale switch <account>` for saved
  profiles.
- `tailscale set --exit-node=<target>` and `tailscale set --exit-node=` for
  exit-node selection.
- `tailscale set --advertise-exit-node=true|false` for advertising this device
  as an exit node.
- `tailscale file cp <file> <target>:` and `tailscale file get <directory>` for
  Taildrop where the installed client supports it.
- `tailscale version`, `tailscale netcheck`, and `tailscale bugreport` for
  diagnostics.

Official references:

- Tailscale CLI: https://tailscale.com/docs/reference/tailscale-cli
- `tailscale up`: https://tailscale.com/docs/reference/tailscale-cli/up
- tailscaled daemon/platform differences: https://tailscale.com/docs/reference/tailscaled
- Taildrop: https://tailscale.com/docs/features/taildrop
- macOS variants: https://tailscale.com/docs/concepts/macos-variants
- Windows install/runtime: https://tailscale.com/docs/install/windows

## LocalAPI Limits

Linux uses LocalAPI through a Unix socket for fast reads and falls back to the
CLI. That path remains Linux-first because LocalAPI transport differs elsewhere:

- Windows runs Tailscale as a Windows service and LocalAPI access is not the
  same Unix socket path used on Linux.
- macOS has Standalone, App Store, and CLI-only variants. The Standalone app is
  the recommended user install, but sandboxing and packaging affect how local
  daemon access is exposed.

For the first Windows and macOS clients, TailScout intentionally uses the CLI
for both reads and writes. That is less aggressive than hand-rolling named-pipe
or sandbox-token LocalAPI access, but it preserves feature parity and keeps the
native shells small. Faster per-platform LocalAPI readers can be added after the
basic native apps are validated on real Windows and macOS machines.

## Feature Parity

| Feature | Linux GTK | Windows WinUI | macOS SwiftUI | Notes |
| --- | --- | --- | --- | --- |
| Status/device list | LocalAPI, CLI fallback | CLI | CLI | Uses `status --json`. |
| Connect/disconnect | CLI | CLI | CLI | `up` may open browser for auth. |
| Login/logout | CLI | CLI | CLI | Native shells show CLI output/errors. |
| Profile switch | CLI | CLI | CLI | Uses `switch --list --json`. |
| Exit-node selection | CLI | CLI | CLI | Uses `tailscale set`. |
| Advertise exit node | CLI | CLI | CLI | Still requires admin approval in Tailscale. |
| Taildrop send | CLI | CLI | CLI | Taildrop is alpha and same-owner limited. |
| Taildrop receive | CLI + folder setting | CLI + picker | CLI + picker | macOS may also place received files in Downloads depending on client variant. |
| Diagnostics | CLI | CLI | CLI | Version, netcheck, bugreport. |
