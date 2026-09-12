# Packaging status

GitHub Releases remain the source of truth for TailScout. The release workflow
builds Linux, Windows, and macOS archives, a Debian package, a classic Snap
candidate, and SHA-256 checksums. The Linux installer consumes the release
archive, while Debian and Snap users can use the matching package artifact.

This directory contains the Linux desktop entry, icon, and AppStream metadata used
by the supported archive. Keep them aligned with `Cargo.toml` and validate them in
CI through `scripts/check-release-truth.py`.

Do not add speculative AUR, Flatpak, WinGet, Homebrew, or app-store manifests.
Add a packaging target only when it:

1. builds from a clean checkout;
2. can reach the host Tailscale CLI or LocalAPI safely;
3. is exercised in CI or by a documented release check; and
4. contains no provisional versions, hashes, generated-file references, or paths.

## Debian

The tag workflow runs `scripts/package-deb.sh <version>` after the verified Linux
build. The package installs the native GTK4 application, desktop entry, icon,
AppStream metadata, license, and documentation under `/usr`. It depends on the
distribution GTK4/libadwaita runtime and suggests the separately installed
`tailscale` CLI. Build locally on a Debian-based host with:

```sh
TAILSCOUT_SKIP_BUILD=1 scripts/package-deb.sh 0.1.4 dist
```

The v0.1.4 package is attached to the [GitHub release](https://github.com/shreyam1008/tailScout/releases/tag/v0.1.4) with its
[SHA-256 receipt](https://github.com/shreyam1008/tailScout/releases/download/v0.1.4/tailscout_0.1.4_amd64.deb.sha256).
The artifact was backfilled by the hosted [Debian workflow run](https://github.com/shreyam1008/tailScout/actions/runs/34678900557).

## Snap

`snap/snapcraft.yaml` derives its version from `Cargo.toml` and builds the same
native binary and desktop metadata. The release workflow attaches
`tailscout_<version>_amd64.snap` beside the Debian artifact. TailScout uses
classic confinement because it must invoke the user's Tailscale CLI and LocalAPI
on the host; Canonical review and a maintainer smoke test are required before a
Snap Store release is advertised.

The v0.1.4 candidate is attached to the [GitHub release](https://github.com/shreyam1008/tailScout/releases/tag/v0.1.4) with its
[SHA-256 receipt](https://github.com/shreyam1008/tailScout/releases/download/v0.1.4/tailscout_0.1.4_amd64.snap.sha256).
It was built by the hosted [Snap workflow run](https://github.com/shreyam1008/tailScout/actions/runs/34677309505).

```sh
snapcraft --destructive-mode
snap install --dangerous ./tailscout_0.1.4_amd64.snap
```

Flatpak and strict Snap still need an explicit host-daemon/CLI integration design.
Keep those channels as follow-up work until the portals, permissions, and runtime
behavior are tested on a real Linux desktop.
