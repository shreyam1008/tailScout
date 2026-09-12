# TailScout distribution evidence

## 12 September 2026 — personal Ubuntu PC

- v0.1.4 GitHub release artifacts were checked against their published SHA-256
  receipts. The Snap hash is
  `4a74d49c75ba0dfe65bcfd22d30fb6c8ff44e8e2d52fbf020b511066c985f551`.
- The exact `tailscout` Snap name was registered to `shreyam1008`. Upload failed
  Store review: classic confinement requires approval and the existing artifact
  incorrectly declares plugs. The source manifest now omits plugs; a corrected
  artifact and Canonical classic review are required before candidate publication.
  No stable Snap is available.
- `snap-publish.yml` can verify an existing release artifact and upload only to
  candidate once scoped `SNAPCRAFT_STORE_CREDENTIALS` are configured. Stable
  promotion remains manual after installation and host-integration acceptance.
- Signed APT preparation and Debian lifecycle evidence: [APT signing](apt-signing.md).
- Linux formatting, all 20 Rust tests, clippy and release-truth checks passed.
  Native Windows screenshots and interactive acceptance remain outstanding.
