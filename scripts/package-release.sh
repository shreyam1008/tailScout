#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

version="${TAILSCOUT_VERSION:-}"
if [[ -z "$version" ]]; then
  if [[ -n "${GITHUB_REF_NAME:-}" && "${GITHUB_REF_NAME}" == v* ]]; then
    version="${GITHUB_REF_NAME#v}"
  else
    version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
  fi
fi

os="linux"
arch="$(uname -m)"
case "$arch" in
  x86_64 | amd64) arch="x86_64" ;;
  *) echo "unsupported release architecture: $arch" >&2; exit 1 ;;
esac

if [[ "${TAILSCOUT_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --locked --release
fi

binary="target/release/tailscout"
if [[ ! -x "$binary" ]]; then
  echo "release binary not found at $binary" >&2
  exit 1
fi

asset="tailscout-${os}-${arch}"
dist_dir="dist"
stage_dir="$(mktemp -d)"
trap 'rm -rf "$stage_dir"' EXIT

mkdir -p "$dist_dir" "$stage_dir/$asset"
install -m 0755 "$binary" "$stage_dir/$asset/tailscout"
cp README.md CHANGELOG.md "$stage_dir/$asset/"
mkdir -p "$stage_dir/$asset/share/applications"
cp packaging/dev.shre.TailScout.desktop "$stage_dir/$asset/share/applications/"
mkdir -p "$stage_dir/$asset/share/icons/hicolor/scalable/apps"
cp packaging/icons/hicolor/scalable/apps/dev.shre.TailScout.svg \
   "$stage_dir/$asset/share/icons/hicolor/scalable/apps/"

tar -C "$stage_dir" -czf "$dist_dir/$asset.tar.gz" "$asset"
cp "$dist_dir/$asset.tar.gz" "$dist_dir/tailscout-v${version}-${os}-${arch}.tar.gz"

(
  cd "$dist_dir"
  sha256sum "$asset.tar.gz" "tailscout-v${version}-${os}-${arch}.tar.gz" > SHA256SUMS
)

printf 'created %s\n' "$dist_dir/$asset.tar.gz"
printf 'created %s\n' "$dist_dir/tailscout-v${version}-${os}-${arch}.tar.gz"
printf 'created %s\n' "$dist_dir/SHA256SUMS"
