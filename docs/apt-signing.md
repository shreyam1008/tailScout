# Signed TailScout APT repository

The pipeline is prepared for the existing v0.1.4 amd64 Debian artifact. Public
availability must be verified after the first `Publish signed APT` run; this is
not an Ubuntu default-archive package.

The persistent signing fingerprint is:

```text
E51A48647BAB749FCFD9E159677FCFBD1C8D60C8
```

After public verification, Ubuntu 24.04 amd64 users can add the repository:

```sh
curl -fsSLo /tmp/tailscout-key.asc https://tailscout.shreyam1008.com.np/apt/key.asc
test "$(gpg --show-keys --with-colons /tmp/tailscout-key.asc | awk -F: '$1 == "fpr" {print $10; exit}')" = E51A48647BAB749FCFD9E159677FCFBD1C8D60C8
sudo install -m 644 /tmp/tailscout-key.asc /usr/share/keyrings/tailscout.asc
echo 'deb [signed-by=/usr/share/keyrings/tailscout.asc] https://tailscout.shreyam1008.com.np/apt stable main' | sudo tee /etc/apt/sources.list.d/tailscout.list
sudo apt-get update
sudo apt-get install tailscout
```

Tailscale remains a separate host dependency. Removing TailScout does not remove
Tailscale or its state: `sudo apt-get remove tailscout`. To stop receiving repository
updates, remove only `/etc/apt/sources.list.d/tailscout.list` and its dedicated
`/usr/share/keyrings/tailscout.asc`. A pinned install uses `tailscout=0.1.4`.

## Maintainer operation

The manual `apt.yml` workflow verifies the release SHA-256 receipt and package
identity, builds signed indexes, verifies APT installation and removal, exercises
an upgrade from a synthetic lower-version copy of the same payload, then commits
only `docs/apt`. The existing Cloudflare Pages integration serves `docs/`.
The final job checks anonymous HTTPS installation and removal. A failed public
check is not publication acceptance. Historical-package migration is not covered
by the synthetic upgrade fixture.

`APT_SIGNING_PRIVATE_KEY` is an armored secret key in GitHub Actions;
`APT_SIGNING_KEY_ID` is the public fingerprint repository variable. The owner has
a separate private recovery copy outside Git. Never rotate it on each release,
print it, or commit it. The key expires in September 2028; renew it before expiry
and preserve its fingerprint. Rollback must use a retained, checksum-verified
older artifact; do not overwrite a release asset or downgrade users silently.

## Evidence — 12 September 2026

The GitHub v0.1.4 Debian SHA-256 is
`f213e542c4c21bbcf324a70e1ee7a207ba943afae911b18ce58e88d63c919a54`.
An isolated Ubuntu 24.04 amd64 container passed fresh install, dynamic-library
resolution, ten-second native launch with no host tailnet, removal, synthetic
lower-version upgrade and purge. The repository builder produced a valid GPG
signature with the fingerprint above. This does not claim interactive Tailscale
acceptance or Windows Store acceptance.
