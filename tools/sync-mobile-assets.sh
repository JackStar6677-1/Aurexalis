#!/usr/bin/env bash
# Sincroniza browser/home y browser/settings al APK Android.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/mobile/android/app/src/main/assets/aurexalis"

rm -rf "$DEST"
mkdir -p "$DEST/home" "$DEST/settings"
cp -R "$ROOT/browser/home/"* "$DEST/home/"
cp -R "$ROOT/browser/settings/"* "$DEST/settings/"

echo "[SUCCESS] Assets moviles sincronizados."
