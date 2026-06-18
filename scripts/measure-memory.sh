#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BIN=${TAILSCOUT_MEMORY_BINARY:-"$ROOT/target/release/tailscout"}
LIMIT_MIB=${TAILSCOUT_MEMORY_LIMIT_MIB:-100}
METRIC=${TAILSCOUT_MEMORY_METRIC:-pss}
SETTLE_SECONDS=${TAILSCOUT_MEMORY_SETTLE_SECONDS:-8}
SAMPLES=${TAILSCOUT_MEMORY_SAMPLES:-10}
INTERVAL_SECONDS=${TAILSCOUT_MEMORY_INTERVAL_SECONDS:-1}
LOG_FILE=${TAILSCOUT_MEMORY_LOG:-"$ROOT/target/tailscout-memory.log"}

die() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

need_number() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]] || die "$name must be numeric (got: $value)"
}

need_integer() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+$ ]] || die "$name must be an integer (got: $value)"
}

need_number TAILSCOUT_MEMORY_LIMIT_MIB "$LIMIT_MIB"
need_integer TAILSCOUT_MEMORY_SETTLE_SECONDS "$SETTLE_SECONDS"
need_integer TAILSCOUT_MEMORY_SAMPLES "$SAMPLES"
need_integer TAILSCOUT_MEMORY_INTERVAL_SECONDS "$INTERVAL_SECONDS"
[[ "$SAMPLES" -gt 0 ]] || die "TAILSCOUT_MEMORY_SAMPLES must be greater than zero"

case "$METRIC" in
  pss | rss | private) ;;
  *) die "TAILSCOUT_MEMORY_METRIC must be one of: pss, rss, private" ;;
esac

if [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" && "${TAILSCOUT_MEMORY_DBUS:-0}" != "1" ]]; then
  if command -v dbus-run-session >/dev/null 2>&1; then
    export TAILSCOUT_MEMORY_DBUS=1
    exec dbus-run-session -- "$0" "$@"
  fi
fi

if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" && "${TAILSCOUT_MEMORY_XVFB:-0}" != "1" ]]; then
  if command -v xvfb-run >/dev/null 2>&1; then
    export TAILSCOUT_MEMORY_XVFB=1
    exec xvfb-run -a "$0" "$@"
  fi
  die "no graphical display found; install xvfb or run from a desktop session"
fi

if [[ ! -x "$BIN" ]]; then
  if [[ "${TAILSCOUT_MEMORY_SKIP_BUILD:-0}" == "1" ]]; then
    die "binary not executable: $BIN"
  fi
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked --release
fi

existing_pids=$(pgrep -x tailscout 2>/dev/null || true)
if [[ -n "$existing_pids" && "${TAILSCOUT_MEMORY_ALLOW_EXISTING:-0}" != "1" ]]; then
  die "tailscout is already running; close it or set TAILSCOUT_MEMORY_ALLOW_EXISTING=1"
fi

mkdir -p "$(dirname "$LOG_FILE")"
: > "$LOG_FILE"

"$BIN" >"$LOG_FILE" 2>&1 &
pid=$!

cleanup() {
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    sleep 1
    kill -KILL "$pid" 2>/dev/null || true
  fi
  wait "$pid" 2>/dev/null || true
}
trap cleanup EXIT

sleep "$SETTLE_SECONDS"

if ! kill -0 "$pid" 2>/dev/null; then
  printf 'TailScout exited before memory sampling could start.\n' >&2
  sed -n '1,120p' "$LOG_FILE" >&2
  exit 1
fi

sample_memory() {
  local proc_dir="/proc/$pid"
  if [[ -r "$proc_dir/smaps_rollup" ]]; then
    awk '
      /^Rss:/ { rss = $2 }
      /^Pss:/ { pss = $2 }
      /^Private_Clean:/ { private_clean = $2 }
      /^Private_Dirty:/ { private_dirty = $2 }
      END { printf "%d %d %d\n", rss, pss, private_clean + private_dirty }
    ' "$proc_dir/smaps_rollup"
    return
  fi

  local rss
  rss=$(ps -o rss= -p "$pid" | tr -d '[:space:]')
  [[ -n "$rss" ]] || return 1
  printf '%d %d %d\n' "$rss" "$rss" 0
}

format_mib() {
  awk -v kib="$1" 'BEGIN { printf "%.1f", kib / 1024 }'
}

printf 'TailScout Linux memory smoke test\n'
printf 'Binary: %s\n' "$BIN"
printf 'PID: %s\n' "$pid"
printf 'Metric limit: %s <= %s MiB\n\n' "$METRIC" "$LIMIT_MIB"
printf '%-8s %10s %10s %12s\n' "Sample" "RSS MiB" "PSS MiB" "Private MiB"

max_rss=0
max_pss=0
max_private=0

for ((sample = 1; sample <= SAMPLES; sample++)); do
  read -r rss_kib pss_kib private_kib < <(sample_memory)
  (( rss_kib > max_rss )) && max_rss=$rss_kib
  (( pss_kib > max_pss )) && max_pss=$pss_kib
  (( private_kib > max_private )) && max_private=$private_kib

  printf '%-8d %10s %10s %12s\n' \
    "$sample" \
    "$(format_mib "$rss_kib")" \
    "$(format_mib "$pss_kib")" \
    "$(format_mib "$private_kib")"

  if [[ "$sample" -lt "$SAMPLES" ]]; then
    sleep "$INTERVAL_SECONDS"
  fi
done

case "$METRIC" in
  pss) actual_kib=$max_pss ;;
  rss) actual_kib=$max_rss ;;
  private) actual_kib=$max_private ;;
esac
actual_mib=$(format_mib "$actual_kib")

printf '\nMaximum sampled memory:\n'
printf '  RSS:     %s MiB\n' "$(format_mib "$max_rss")"
printf '  PSS:     %s MiB\n' "$(format_mib "$max_pss")"
printf '  Private: %s MiB\n' "$(format_mib "$max_private")"

if awk -v actual="$actual_mib" -v limit="$LIMIT_MIB" 'BEGIN { exit !(actual > limit) }'; then
  printf '\nMemory baseline exceeded: %s %s MiB > %s MiB\n' "$METRIC" "$actual_mib" "$LIMIT_MIB" >&2
  exit 2
fi

printf '\nMemory baseline met: %s %s MiB <= %s MiB\n' "$METRIC" "$actual_mib" "$LIMIT_MIB"
