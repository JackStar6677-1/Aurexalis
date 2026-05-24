#!/usr/bin/env bash
# Empaqueta Aurexalis para Linux: .deb (Ubuntu/Debian), .rpm (Fedora) y .pkg.tar.zst (Arch).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(grep -m1 '^version' "$ROOT/crates/aurexalis-shell/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')}"
DIST="$ROOT/dist"
STAGE="$ROOT/.linux-stage"
PKGROOT="$STAGE/pkgroot"

echo "[INFO] Empaquetando Aurexalis $VERSION para Linux"

export PATH="$HOME/.cargo/bin:$PATH"
cd "$ROOT"
cargo build --release -p aurexalis-shell

rm -rf "$STAGE"
mkdir -p "$PKGROOT/opt/aurexalis/bin" "$PKGROOT/opt/aurexalis/browser" "$PKGROOT/usr/bin"

cp "$ROOT/target/release/aurexalis" "$PKGROOT/opt/aurexalis/bin/aurexalis"
chmod 755 "$PKGROOT/opt/aurexalis/bin/aurexalis"
cp -R "$ROOT/browser/." "$PKGROOT/opt/aurexalis/browser/"
cp "$ROOT/LICENSE" "$PKGROOT/opt/aurexalis/LICENSE"
mkdir -p "$PKGROOT/opt/aurexalis/profiles/default"

cat > "$PKGROOT/usr/bin/aurexalis" <<'WRAPPER'
#!/bin/sh
exec /opt/aurexalis/bin/aurexalis --launch-installed "$@"
WRAPPER
chmod 755 "$PKGROOT/usr/bin/aurexalis"

mkdir -p "$DIST"

if ! command -v fpm >/dev/null 2>&1; then
  echo "[INFO] Instalando fpm (gem)..."
  sudo gem install --no-document fpm
fi

if ! command -v bsdtar >/dev/null 2>&1; then
  echo "[WARN] bsdtar no encontrado; paquete Arch puede fallar (instala libarchive-tools)"
fi

echo "[INFO] Generando .deb (Ubuntu/Debian)..."
fpm -s dir -t deb -n aurexalis -v "$VERSION" --architecture amd64 \
  --maintainer "Aurexalis <noreply@aurexalis.local>" \
  --description "Navegador Aurexalis — shell y tema Gecko" \
  --url "https://github.com/JackStar6677-1/Aurexalis" \
  --license MIT \
  --depends libc6 \
  -C "$PKGROOT" \
  --package "$DIST/aurexalis_${VERSION}_amd64.deb"

echo "[INFO] Generando .rpm (Fedora/RHEL)..."
fpm -s dir -t rpm -n aurexalis -v "$VERSION" --architecture x86_64 \
  --maintainer "Aurexalis" \
  --description "Navegador Aurexalis — shell y tema Gecko" \
  --url "https://github.com/JackStar6677-1/Aurexalis" \
  --license MIT \
  -C "$PKGROOT" \
  --package "$DIST/aurexalis-${VERSION}-1.x86_64.rpm"

echo "[INFO] Generando .pkg.tar.zst (Arch Linux)..."
fpm -s dir -t pacman -n aurexalis -v "$VERSION" --architecture x86_64 \
  --maintainer "Aurexalis" \
  --description "Navegador Aurexalis — shell y tema Gecko" \
  --url "https://github.com/JackStar6677-1/Aurexalis" \
  --license MIT \
  -C "$PKGROOT" \
  --package "$DIST/aurexalis-${VERSION}-x86_64.pkg.tar.zst"

echo "[INFO] Generando tarball portable..."
tar -czf "$DIST/aurexalis-runtime-linux-x86_64.tar.gz" -C "$PKGROOT/opt" aurexalis

rm -rf "$STAGE"
echo "[SUCCESS] Paquetes Linux en $DIST"
