#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CALLER_DIR=$(pwd)

APP_NAME=${APP_NAME:-TailScout}
APP="$ROOT/.build/app/${APP_NAME}.app"
APP_OVERRIDDEN=0
BUILD_MODE=auto
BUILD_CONF=${BUILD_CONF:-release}
SAMPLES=${TAILSCOUT_RSS_SAMPLES:-8}
INTERVAL_SECONDS=${TAILSCOUT_RSS_INTERVAL_SECONDS:-1}
SETTLE_SECONDS=${TAILSCOUT_RSS_SETTLE_SECONDS:-8}
LAUNCH_TIMEOUT_SECONDS=${TAILSCOUT_RSS_LAUNCH_TIMEOUT_SECONDS:-20}
BASELINE_MIB=${TAILSCOUT_RSS_BASELINE_MIB:-${TAILSCOUT_MAX_RSS_MIB:-}}
KEEP_OPEN=0
MEASURED_PID=

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Build or launch the packaged TailScout.app, wait for it to settle, then sample
the app process RSS with built-in macOS tools.

Options:
  --app PATH                 App bundle to measure (default: .build/app/TailScout.app)
  --build [CONFIG]           Run Scripts/package_app.sh before measuring
                             (default CONFIG: release)
  --no-build                 Require an existing packaged app bundle
  --samples N                RSS samples to take after settle (default: 8)
  --interval SECONDS         Delay between samples (default: 1)
  --settle SECONDS           Delay after launch before sampling (default: 8)
  --launch-timeout SECONDS   Seconds to wait for the app process (default: 20)
  --baseline-mib MIB         Fail if sampled peak RSS exceeds this MiB value
  --max-rss-mib MIB          Alias for --baseline-mib
  --keep-open                Leave the measured app running
  --help, -h                 Show this help

Environment:
  APP_NAME, BUILD_CONF, TAILSCOUT_RSS_BASELINE_MIB,
  TAILSCOUT_RSS_SAMPLES, TAILSCOUT_RSS_INTERVAL_SECONDS,
  TAILSCOUT_RSS_SETTLE_SECONDS, TAILSCOUT_RSS_LAUNCH_TIMEOUT_SECONDS
USAGE
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

set_app_path() {
  local path="$1"
  case "$path" in
    /*) APP="$path" ;;
    *) APP="$CALLER_DIR/$path" ;;
  esac
  APP_OVERRIDDEN=1
}

is_positive_int() {
  [[ "$1" =~ ^[0-9]+$ ]] && [[ "$1" -gt 0 ]]
}

is_nonnegative_number() {
  [[ "$1" =~ ^[0-9]+([.][0-9]+)?$ ]]
}

require_value() {
  local option="$1"
  local value="${2:-}"
  [[ -n "$value" ]] || die "$option requires a value"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app)
      require_value "$1" "${2:-}"
      set_app_path "$2"
      shift
      ;;
    --app=*)
      set_app_path "${1#*=}"
      ;;
    --build)
      BUILD_MODE=build
      if [[ $# -ge 2 && "$2" != --* ]]; then
        BUILD_CONF="$2"
        shift
      fi
      ;;
    --build=*)
      BUILD_MODE=build
      BUILD_CONF="${1#*=}"
      ;;
    --no-build)
      BUILD_MODE=none
      ;;
    --samples)
      require_value "$1" "${2:-}"
      SAMPLES="$2"
      shift
      ;;
    --samples=*)
      SAMPLES="${1#*=}"
      ;;
    --interval|--interval-seconds)
      require_value "$1" "${2:-}"
      INTERVAL_SECONDS="$2"
      shift
      ;;
    --interval=*|--interval-seconds=*)
      INTERVAL_SECONDS="${1#*=}"
      ;;
    --settle|--settle-seconds)
      require_value "$1" "${2:-}"
      SETTLE_SECONDS="$2"
      shift
      ;;
    --settle=*|--settle-seconds=*)
      SETTLE_SECONDS="${1#*=}"
      ;;
    --launch-timeout|--launch-timeout-seconds)
      require_value "$1" "${2:-}"
      LAUNCH_TIMEOUT_SECONDS="$2"
      shift
      ;;
    --launch-timeout=*|--launch-timeout-seconds=*)
      LAUNCH_TIMEOUT_SECONDS="${1#*=}"
      ;;
    --baseline-mib|--max-rss-mib)
      require_value "$1" "${2:-}"
      BASELINE_MIB="$2"
      shift
      ;;
    --baseline-mib=*|--max-rss-mib=*)
      BASELINE_MIB="${1#*=}"
      ;;
    --keep-open)
      KEEP_OPEN=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
  shift
done

is_positive_int "$SAMPLES" || die "--samples must be a positive integer"
is_nonnegative_number "$INTERVAL_SECONDS" || die "--interval must be a nonnegative number"
is_nonnegative_number "$SETTLE_SECONDS" || die "--settle must be a nonnegative number"
is_positive_int "$LAUNCH_TIMEOUT_SECONDS" || die "--launch-timeout must be a positive integer"
if [[ -n "$BASELINE_MIB" ]]; then
  is_nonnegative_number "$BASELINE_MIB" || die "--baseline-mib must be a nonnegative number"
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  die "measure_rss.sh must be run on macOS"
fi

abs_path() {
  local path="$1"
  local dir
  local base

  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
    return
  fi

  dir=$(dirname "$path")
  base=$(basename "$path")
  (cd "$dir" && printf '%s/%s\n' "$(pwd -P)" "$base")
}

plist_executable_name() {
  local plist="$1/Contents/Info.plist"
  local executable

  if [[ -x /usr/libexec/PlistBuddy && -f "$plist" ]]; then
    executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist" 2>/dev/null || true)
    if [[ -n "$executable" ]]; then
      printf '%s\n' "$executable"
      return
    fi
  fi

  printf '%s\n' "$APP_NAME"
}

matching_app_pids() {
  local pids
  local pid
  local command

  pids=$(pgrep -x "$APP_EXECUTABLE_NAME" 2>/dev/null || true)
  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    command=$(ps -ww -p "$pid" -o command= 2>/dev/null || true)
    case "$command" in
      "$APP_EXECUTABLE"*) printf '%s\n' "$pid" ;;
    esac
  done <<< "$pids"
}

find_app_pid() {
  local pid

  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    printf '%s\n' "$pid"
    return 0
  done < <(matching_app_pids)

  return 1
}

terminate_pid() {
  local pid="$1"
  local deadline

  kill "$pid" 2>/dev/null || return 0
  deadline=$((SECONDS + 5))
  while kill -0 "$pid" 2>/dev/null; do
    if [[ "$SECONDS" -ge "$deadline" ]]; then
      kill -9 "$pid" 2>/dev/null || true
      return 0
    fi
    sleep 1
  done
}

terminate_existing_app() {
  local pid

  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    printf 'Stopping existing %s process %s\n' "$APP_EXECUTABLE_NAME" "$pid"
    terminate_pid "$pid"
  done < <(matching_app_pids)
}

wait_for_app_pid() {
  local deadline
  local pid

  deadline=$((SECONDS + LAUNCH_TIMEOUT_SECONDS))
  while [[ "$SECONDS" -lt "$deadline" ]]; do
    if pid=$(find_app_pid); then
      printf '%s\n' "$pid"
      return 0
    fi
    sleep 1
  done

  return 1
}

sample_rss_kib() {
  local pid="$1"
  local rss

  rss=$(ps -o rss= -p "$pid" 2>/dev/null | awk '{print $1}')
  [[ -n "$rss" && "$rss" =~ ^[0-9]+$ ]] || return 1
  printf '%s\n' "$rss"
}

cleanup() {
  local status=$?

  if [[ "$KEEP_OPEN" != "1" && -n "${MEASURED_PID:-}" ]]; then
    terminate_pid "$MEASURED_PID"
  fi

  return "$status"
}
trap cleanup EXIT

if [[ "$BUILD_MODE" == "build" || ( "$BUILD_MODE" == "auto" && "$APP_OVERRIDDEN" == "0" && ! -d "$APP" ) ]]; then
  printf 'Packaging %s (%s)\n' "$APP_NAME" "$BUILD_CONF"
  cd "$ROOT"
  SIGNING_MODE=${SIGNING_MODE:-adhoc} "$ROOT/Scripts/package_app.sh" "$BUILD_CONF"
fi

[[ -d "$APP" ]] || die "app bundle not found: $APP"
APP=$(abs_path "$APP")
APP_EXECUTABLE_NAME=$(plist_executable_name "$APP")
APP_EXECUTABLE="$APP/Contents/MacOS/$APP_EXECUTABLE_NAME"
[[ -x "$APP_EXECUTABLE" ]] || die "app executable not found or not executable: $APP_EXECUTABLE"

terminate_existing_app

printf 'Launching %s\n' "$APP"
open "$APP"

if ! MEASURED_PID=$(wait_for_app_pid); then
  die "timed out waiting for $APP_EXECUTABLE_NAME to launch"
fi

printf 'Waiting %ss for idle settle\n' "$SETTLE_SECONDS"
sleep "$SETTLE_SECONDS"

RSS_VALUES=()
for ((i = 1; i <= SAMPLES; i += 1)); do
  if ! rss=$(sample_rss_kib "$MEASURED_PID"); then
    die "$APP_EXECUTABLE_NAME exited before RSS sample $i"
  fi
  RSS_VALUES+=("$rss")
  if [[ "$i" -lt "$SAMPLES" ]]; then
    sleep "$INTERVAL_SECONDS"
  fi
done

stats=$(printf '%s\n' "${RSS_VALUES[@]}" | awk '
  NR == 1 {
    current = $1
    peak = $1
  }
  {
    current = $1
    sum += $1
    if ($1 > peak) {
      peak = $1
    }
  }
  END {
    printf "%.1f %.1f %.1f\n", current / 1024, peak / 1024, (sum / NR) / 1024
  }
')

set -- $stats
CURRENT_MIB="$1"
PEAK_MIB="$2"
AVERAGE_MIB="$3"

printf '\n%s RSS measurement\n' "$APP_EXECUTABLE_NAME"
printf '  app: %s\n' "$APP"
printf '  pid: %s\n' "$MEASURED_PID"
printf '  samples: %s every %ss after %ss settle\n' "$SAMPLES" "$INTERVAL_SECONDS" "$SETTLE_SECONDS"
printf '  current RSS: %s MiB\n' "$CURRENT_MIB"
printf '  peak RSS: %s MiB\n' "$PEAK_MIB"
printf '  average RSS: %s MiB\n' "$AVERAGE_MIB"

if [[ -n "$BASELINE_MIB" ]]; then
  printf '  baseline: %s MiB peak max\n' "$BASELINE_MIB"
  if awk -v peak="$PEAK_MIB" -v baseline="$BASELINE_MIB" 'BEGIN { exit !(peak > baseline) }'; then
    printf 'ERROR: peak RSS %s MiB exceeds baseline %s MiB\n' "$PEAK_MIB" "$BASELINE_MIB" >&2
    exit 2
  fi
fi
