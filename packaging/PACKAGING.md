# Packaging status

GitHub Releases are TailScout's only supported distribution channel today. The
release workflow builds Linux, Windows, and macOS archives and publishes SHA-256
checksums. The Linux installer consumes that release archive.

This directory contains the Linux desktop entry, icon, and AppStream metadata used
by the supported archive. Keep them aligned with `Cargo.toml` and validate them in
CI through `scripts/check-release-truth.py`.

Do not commit speculative AUR, Flatpak, Snap, WinGet, Homebrew, or app-store
manifests. Add a packaging target only when it:

1. builds from a clean checkout;
2. can reach the host Tailscale CLI or LocalAPI safely;
3. is exercised in CI or by a documented release check; and
4. contains no provisional versions, hashes, generated-file references, or paths.

Flatpak and strict Snap need an explicit host-daemon/CLI integration design before
they can satisfy those conditions. Track that work as an issue rather than keeping
non-runnable manifests in the main branch.
