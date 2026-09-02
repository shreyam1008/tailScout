# Shared backend contract

TailScout keeps each interface native, but every platform implements the same small
backend contract. This directory is the source of truth for behavior that crosses
the Rust/GTK, C#/WinUI, and Swift/SwiftUI boundaries.

## Status model

All clients parse `tailscale status --json` and must:

- accept missing or `null` fields as empty/default values;
- preserve unknown backend-state strings;
- prefer `Version`, falling back to `ClientVersion`, for display;
- prefer an IPv4 Tailscale address, then the first available address;
- use that address, then MagicDNS, as the target for CLI actions;
- display hostname, then MagicDNS name, then IP, then `unknown`;
- sort peers online-first, then case-insensitively by display name;
- expose Taildrop only when the peer is online, has a positive
  `TaildropTarget`, has no `NoFileSharingReason`, and belongs to the same user;
- treat non-host routes in `AllowedIPs` as subnet-router capability; and
- expose peers with `ExitNodeOption` as exit-node choices.

The canonical parser inputs live in [`fixtures/`](fixtures/). Platform tests must
read these files directly rather than copy their contents into test source.

Saved profiles use nickname, account, tailnet, then ID as the display fallback;
use ID, nickname, account, then tailnet as the switch target. Ignore entries that
have no usable display value.

## CLI contract

| Operation | Arguments |
| --- | --- |
| Status | `status --json` |
| Profiles | `switch --list --json` |
| Connect | `up --timeout=30s` |
| Disconnect | `down` |
| Login | `login --timeout=30s` |
| Logout | `logout` |
| Switch profile | `switch <id-or-name>` |
| Set exit node | `set --exit-node=<target>` |
| Clear exit node | `set --exit-node=` |
| Advertise exit node | `set --advertise-exit-node=true|false` |
| Send Taildrop file | `file cp <path> <target>:` |
| Receive Taildrop files | `file get --conflict=rename <directory>` |
| Diagnostics | `version`, `netcheck`, `bugreport` |

Reads use the Linux LocalAPI first when available. Mutations always use the
installed Tailscale CLI so TailScout stays version-matched with the daemon.

## UI contract

The three native shells use their platform controls, but they present one
product vocabulary and information hierarchy:

- show connection state, tailnet, this device, Tailscale IP, version, and
  health before secondary controls;
- list peer devices only (the local device belongs in the overview), using the
  shared online-first ordering;
- show device name and `Online`/`Offline` first, followed by OS, Tailscale IP,
  owner, and relevant Taildrop/exit-node/subnet-router capability;
- keep controls grouped as `Accounts and Tailnets`, `Selected Device`,
  `Taildrop`, `Exit Node`, and `Diagnostics` wherever the native layout has a
  persistent control pane; and
- use the same action names: `Refresh`, `Connect`, `Disconnect`, `Log In`,
  `Log Out`, `Switch`, `Send File`, `Receive Files`, `Use Exit Node`,
  `Clear Exit Node`, `Advertise This Device`, `Tailscale Version`,
  `Network Check`, and `Bug Report`.

An OS may expose genuinely platform-specific affordances, such as Linux
operator setup or macOS security-scoped file access. Those additions must not
rename or reorder the common workflows.

## Architecture boundary

- `src/tailscale/`, `TailScout.Windows.Core`, and `TailScoutCore` contain parsing,
  policy, validation, and process execution. They must not depend on UI toolkits.
- GTK, WinUI, and SwiftUI code renders state, collects user input, and invokes the
  platform core asynchronously.
- Do not add a cross-language FFI layer or helper process merely to remove a few
  equivalent model declarations. For a one-person project, native debugging and
  packaging are more valuable than literal source sharing.
- Add a shared fixture and assertions on every platform when the wire contract
  changes.
