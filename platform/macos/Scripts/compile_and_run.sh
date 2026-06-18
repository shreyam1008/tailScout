#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_NAME=${APP_NAME:-TailScout}
APP="$ROOT/.build/app/${APP_NAME}.app"

pkill -f "${APP_NAME}.app/Contents/MacOS/${APP_NAME}" 2>/dev/null || true
pkill -x "$APP_NAME" 2>/dev/null || true

RUN_TESTS=0
for arg in "$@"; do
  case "$arg" in
    --test|-t) RUN_TESTS=1 ;;
    --help|-h)
      printf 'Usage: %s [--test]\n' "$(basename "$0")"
      exit 0
      ;;
  esac
done

cd "$ROOT"

if [[ "$RUN_TESTS" == "1" ]]; then
  swift test
fi

SIGNING_MODE=adhoc "$ROOT/Scripts/package_app.sh" debug
open "$APP"
