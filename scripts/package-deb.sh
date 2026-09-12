#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

version="${1:-${TAILSCOUT_VERSION:-}}"
if [[ -z "$version" ]]; then
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
fi
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "invalid TailScout package version: $version" >&2
  exit 1
fi

if [[ "${TAILSCOUT_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --locked --release
fi

binary="target/release/tailscout"
[[ -x "$binary" ]] || { echo "release binary not found at $binary" >&2; exit 1; }

dist_dir="${2:-dist}"
pkg_dir="$(mktemp -d)"
trap 'rm -rf "$pkg_dir"' EXIT

mkdir -p "$pkg_dir/DEBIAN" \
  "$pkg_dir/usr/bin" \
  "$pkg_dir/usr/share/applications" \
  "$pkg_dir/usr/share/icons/hicolor/scalable/apps" \
  "$pkg_dir/usr/share/metainfo" \
  "$pkg_dir/usr/share/doc/tailscout"

install -m 0755 "$binary" "$pkg_dir/usr/bin/tailscout"
install -m 0644 packaging/dev.shre.TailScout.desktop "$pkg_dir/usr/share/applications/"
install -m 0644 packaging/icons/hicolor/scalable/apps/dev.shre.TailScout.svg "$pkg_dir/usr/share/icons/hicolor/scalable/apps/"
install -m 0644 packaging/dev.shre.TailScout.metainfo.xml "$pkg_dir/usr/share/metainfo/"
install -m 0644 LICENSE "$pkg_dir/usr/share/doc/tailscout/copyright"
install -m 0644 README.md "$pkg_dir/usr/share/doc/tailscout/README.md"
install -m 0644 CHANGELOG.md "$pkg_dir/usr/share/doc/tailscout/CHANGELOG.md"

cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: tailscout
Version: $version
Section: net
Priority: optional
Architecture: amd64
Maintainer: Shreyam Adhikari <shreyam1008@gmail.com>
Depends: libgtk-4-1, libadwaita-1-0
Suggests: tailscale
Homepage: https://tailscout.shreyam1008.com.np/
Description: Native Tailscale GUI for Linux
 TailScout is a lightweight GTK4/libadwaita desktop client for the
 Tailscale CLI and LocalAPI. Tailnet data stays on the local machine.
EOF

mkdir -p "$dist_dir"
out="$dist_dir/tailscout_${version}_amd64.deb"
dpkg-deb --build --root-owner-group "$pkg_dir" "$out" >/dev/null
printf 'created %s\n' "$out"
