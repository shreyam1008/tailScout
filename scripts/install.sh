#!/usr/bin/env bash
set -euo pipefail

repo="${TAILSCOUT_REPO:-shreyam1008/tailScout}"
bin_dir="${TAILSCOUT_BIN_DIR:-/usr/local/bin}"
app_dir="${TAILSCOUT_APP_DIR:-/usr/local/share/applications}"
version="${TAILSCOUT_VERSION:-latest}"
tmp_dir="$(mktemp -d)"
cleanup() { rm -rf "$tmp_dir"; }
trap cleanup EXIT

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

need_cmd curl
need_cmd tar
need_cmd uname

has_runtime_libs() {
  ldconfig -p 2>/dev/null | grep -q 'libgtk-4.so' &&
    ldconfig -p 2>/dev/null | grep -q 'libadwaita-1.so'
}

install_runtime_deps() {
  if has_runtime_libs; then
    return
  fi
  if [[ "${TAILSCOUT_SKIP_DEPS:-0}" == "1" ]]; then
    echo "Skipping runtime dependency install because TAILSCOUT_SKIP_DEPS=1." >&2
    return
  fi

  echo "Installing TailScout runtime dependencies (GTK4 + libadwaita)."
  if command -v apt-get >/dev/null 2>&1; then
    need_cmd sudo
    sudo apt-get update
    sudo apt-get install -y libgtk-4-1 libadwaita-1-0
  elif command -v dnf >/dev/null 2>&1; then
    need_cmd sudo
    sudo dnf install -y gtk4 libadwaita
  elif command -v pacman >/dev/null 2>&1; then
    need_cmd sudo
    sudo pacman -S --needed gtk4 libadwaita
  elif command -v zypper >/dev/null 2>&1; then
    need_cmd sudo
    sudo zypper install -y gtk4 libadwaita-1-0
  else
    echo "Warning: could not detect a supported package manager for GTK4/libadwaita." >&2
    echo "Install runtime packages manually for your distro." >&2
  fi
}

if ! command -v tailscale >/dev/null 2>&1; then
  echo "Warning: tailscale CLI was not found. Install Tailscale before using TailScout." >&2
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os" in
  linux) ;;
  *) echo "TailScout installer currently supports Linux only (got: $os)." >&2; exit 1 ;;
esac
case "$arch" in
  x86_64 | amd64) arch="x86_64" ;;
  *) echo "TailScout installer currently supports x86_64 Linux only (got: $arch)." >&2; exit 1 ;;
esac

asset="tailscout-${os}-${arch}.tar.gz"
if [[ "$version" == "latest" ]]; then
  url="https://github.com/${repo}/releases/latest/download/${asset}"
else
  url="https://github.com/${repo}/releases/download/${version}/${asset}"
fi

archive="$tmp_dir/$asset"
echo "Downloading TailScout from $url"
curl --fail --location --progress-bar "$url" --output "$archive"

tar -xzf "$archive" -C "$tmp_dir"
install_from="$tmp_dir/tailscout-${os}-${arch}/tailscout"
if [[ ! -x "$install_from" ]]; then
  echo "release archive did not contain an executable tailscout binary" >&2
  exit 1
fi

mkdir_cmd=(mkdir -p "$bin_dir")
install_cmd=(install -m 0755 "$install_from" "$bin_dir/tailscout")
if [[ -w "$bin_dir" || ( ! -e "$bin_dir" && -w "$(dirname "$bin_dir")" ) ]]; then
  "${mkdir_cmd[@]}"
  "${install_cmd[@]}"
else
  need_cmd sudo
  sudo "${mkdir_cmd[@]}"
  sudo "${install_cmd[@]}"
fi

desktop_file="$tmp_dir/tailscout-${os}-${arch}/share/applications/dev.shre.TailScout.desktop"
if [[ -f "$desktop_file" ]]; then
  app_mkdir_cmd=(mkdir -p "$app_dir")
  app_install_cmd=(install -m 0644 "$desktop_file" "$app_dir/dev.shre.TailScout.desktop")
  if [[ -w "$app_dir" || ( ! -e "$app_dir" && -w "$(dirname "$app_dir")" ) ]]; then
    "${app_mkdir_cmd[@]}"
    "${app_install_cmd[@]}"
  else
    need_cmd sudo
    sudo "${app_mkdir_cmd[@]}"
    sudo "${app_install_cmd[@]}"
  fi
fi

icon_src="$tmp_dir/tailscout-${os}-${arch}/share/icons/hicolor/scalable/apps/dev.shre.TailScout.svg"
icon_dir="${TAILSCOUT_ICON_DIR:-/usr/local/share/icons/hicolor/scalable/apps}"
if [[ -f "$icon_src" ]]; then
  icon_mkdir_cmd=(mkdir -p "$icon_dir")
  icon_install_cmd=(install -m 0644 "$icon_src" "$icon_dir/dev.shre.TailScout.svg")
  if [[ -w "$icon_dir" || ( ! -e "$icon_dir" && -w "$(dirname "$icon_dir")" ) ]]; then
    "${icon_mkdir_cmd[@]}"
    "${icon_install_cmd[@]}"
  else
    need_cmd sudo
    sudo "${icon_mkdir_cmd[@]}"
    sudo "${icon_install_cmd[@]}"
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor 2>/dev/null || true
  fi
fi

install_runtime_deps

if ! has_runtime_libs; then
  echo "Warning: GTK4 runtime library was not found by ldconfig." >&2
  echo "Debian/Ubuntu: sudo apt install libgtk-4-1 libadwaita-1-0" >&2
  echo "Fedora: sudo dnf install gtk4 libadwaita" >&2
  echo "Arch: sudo pacman -S gtk4 libadwaita" >&2
fi

echo "Installed TailScout to $bin_dir/tailscout"
echo "Run it with: tailscout"
