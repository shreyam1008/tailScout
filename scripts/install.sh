#!/usr/bin/env bash
set -euo pipefail

repo="${TAILSCOUT_REPO:-shreyam1008/tailScout}"
bin_dir="${TAILSCOUT_BIN_DIR:-/usr/local/bin}"
app_dir="${TAILSCOUT_APP_DIR:-/usr/local/share/applications}"
icon_dir="${TAILSCOUT_ICON_DIR:-/usr/local/share/icons/hicolor/scalable/apps}"
meta_dir="${TAILSCOUT_META_DIR:-/usr/local/share/tailscout}"
version="${TAILSCOUT_VERSION:-latest}"
tmp_dir="$(mktemp -d)"
cleanup() { rm -rf "$tmp_dir"; }
trap cleanup EXIT

if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
  bold="$(tput bold 2>/dev/null || true)"
  dim="$(tput dim 2>/dev/null || true)"
  green="$(tput setaf 2 2>/dev/null || true)"
  yellow="$(tput setaf 3 2>/dev/null || true)"
  red="$(tput setaf 1 2>/dev/null || true)"
  reset="$(tput sgr0 2>/dev/null || true)"
else
  bold=""
  dim=""
  green=""
  yellow=""
  red=""
  reset=""
fi

say() { printf '%b\n' "$*"; }
section() { printf '\n%b==> %s%b\n' "$bold" "$*" "$reset"; }
ok() { printf '%b✓%b %s\n' "$green" "$reset" "$*"; }
warn() { printf '%b!%b %s\n' "$yellow" "$reset" "$*" >&2; }
die() { printf '%bError:%b %s\n' "$red" "$reset" "$*" >&2; exit 1; }

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "missing required command: $1"
  fi
}

strip_v() {
  local value="${1:-}"
  value="${value#v}"
  printf '%s' "$value"
}

read_version_file() {
  local file="$1"
  [[ -r "$file" ]] || return 1
  local value
  value="$(head -n 1 "$file" | tr -d '[:space:]')"
  [[ -n "$value" ]] || return 1
  strip_v "$value"
}

has_runtime_libs() {
  command -v ldconfig >/dev/null 2>&1 || return 1
  ldconfig -p 2>/dev/null | grep -q 'libgtk-4.so' &&
    ldconfig -p 2>/dev/null | grep -q 'libadwaita-1.so'
}

can_write_dir() {
  local dir="$1"
  if [[ -d "$dir" ]]; then
    [[ -w "$dir" ]]
  else
    [[ -w "$(dirname "$dir")" ]]
  fi
}

install_file() {
  local mode="$1"
  local src="$2"
  local dest="$3"
  local dest_dir
  dest_dir="$(dirname "$dest")"
  if can_write_dir "$dest_dir"; then
    mkdir -p "$dest_dir"
    install -m "$mode" "$src" "$dest"
  else
    need_cmd sudo
    sudo mkdir -p "$dest_dir"
    sudo install -m "$mode" "$src" "$dest"
  fi
}

install_runtime_deps() {
  if has_runtime_libs; then
    ok "GTK4/libadwaita runtime libraries found"
    return
  fi
  if [[ "${TAILSCOUT_SKIP_DEPS:-0}" == "1" ]]; then
    warn "Skipping runtime dependency install because TAILSCOUT_SKIP_DEPS=1"
    return
  fi

  section "Checking runtime dependencies"
  say "TailScout needs GTK4 and libadwaita runtime libraries."
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
    warn "Could not detect a supported package manager for GTK4/libadwaita"
    warn "Install runtime packages manually for your distro"
  fi
}

refresh_icon_cache() {
  command -v gtk-update-icon-cache >/dev/null 2>&1 || return
  local icon_root
  icon_root="$(dirname "$(dirname "$(dirname "$icon_dir")")")"
  gtk-update-icon-cache -f -t "$icon_root" >/dev/null 2>&1 || true
}

need_cmd curl
need_cmd tar
need_cmd uname

say "${bold}TailScout installer${reset}"
say "${dim}Native Tailscale GUI for Linux${reset}"

if ! command -v tailscale >/dev/null 2>&1; then
  warn "tailscale CLI was not found. Install Tailscale before using TailScout."
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os" in
  linux) ;;
  *) die "TailScout installer currently supports Linux only (got: $os)" ;;
esac
case "$arch" in
  x86_64 | amd64) arch="x86_64" ;;
  *) die "TailScout installer currently supports x86_64 Linux only (got: $arch)" ;;
esac

asset="tailscout-${os}-${arch}.tar.gz"
if [[ "$version" == "latest" ]]; then
  url="https://github.com/${repo}/releases/latest/download/${asset}"
else
  url="https://github.com/${repo}/releases/download/${version}/${asset}"
fi

installed_version=""
existing_install="0"
if [[ -x "$bin_dir/tailscout" ]]; then
  existing_install="1"
fi
if installed_version="$(read_version_file "$meta_dir/VERSION" 2>/dev/null)"; then
  existing_install="1"
else
  installed_version=""
fi

section "Checking current install"
if [[ "$existing_install" == "1" && -n "$installed_version" ]]; then
  ok "Found TailScout v$installed_version at $bin_dir/tailscout"
elif [[ "$existing_install" == "1" ]]; then
  ok "Found an existing TailScout install at $bin_dir/tailscout"
else
  ok "No existing TailScout install found at $bin_dir/tailscout"
fi

section "Downloading release"
say "Source: https://github.com/${repo}"
say "Asset:  $asset"
archive="$tmp_dir/$asset"
curl --fail --location --progress-bar "$url" --output "$archive"

tar -xzf "$archive" -C "$tmp_dir"
package_dir="$tmp_dir/tailscout-${os}-${arch}"
install_from="$package_dir/tailscout"
[[ -x "$install_from" ]] || die "release archive did not contain an executable tailscout binary"

package_version=""
if package_version="$(read_version_file "$package_dir/VERSION" 2>/dev/null)"; then
  :
elif [[ "$version" != "latest" ]]; then
  package_version="$(strip_v "$version")"
else
  package_version="latest"
fi
ok "Downloaded TailScout v$package_version"

section "Installing files"
install_file 0755 "$install_from" "$bin_dir/tailscout"
ok "Installed binary to $bin_dir/tailscout"

desktop_file="$package_dir/share/applications/dev.shre.TailScout.desktop"
if [[ -f "$desktop_file" ]]; then
  install_file 0644 "$desktop_file" "$app_dir/dev.shre.TailScout.desktop"
  ok "Installed desktop launcher"
else
  warn "Release archive did not include a desktop launcher"
fi

icon_src="$package_dir/share/icons/hicolor/scalable/apps/dev.shre.TailScout.svg"
if [[ -f "$icon_src" ]]; then
  install_file 0644 "$icon_src" "$icon_dir/dev.shre.TailScout.svg"
  refresh_icon_cache
  ok "Installed app icon"
else
  warn "Release archive did not include an app icon"
fi

printf '%s\n' "$package_version" > "$tmp_dir/VERSION"
install_file 0644 "$tmp_dir/VERSION" "$meta_dir/VERSION"
ok "Recorded installed version"

install_runtime_deps

if ! has_runtime_libs; then
  warn "GTK4/libadwaita was not found by ldconfig after installation"
  warn "Debian/Ubuntu: sudo apt install libgtk-4-1 libadwaita-1-0"
  warn "Fedora: sudo dnf install gtk4 libadwaita"
  warn "Arch: sudo pacman -S gtk4 libadwaita"
fi

section "Done"
if [[ "$existing_install" == "0" ]]; then
  ok "Installed TailScout v$package_version"
elif [[ -z "$installed_version" ]]; then
  ok "Installed TailScout v$package_version over an existing unversioned install"
elif [[ "$installed_version" == "$package_version" ]]; then
  ok "TailScout v$package_version was already installed; refreshed files and checked dependencies"
else
  ok "Upgraded TailScout from v$installed_version to v$package_version"
fi
say "Run TailScout from your app launcher or with: ${bold}tailscout${reset}"
