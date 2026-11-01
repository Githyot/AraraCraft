#!/usr/bin/env bash
# Builds dist/araracraft-<version>-<release>.x86_64.rpm from target/release/araracraft.
# Needs rpmbuild (Fedora: dnf install rpm-build; Ubuntu: apt install rpm).
# Usage: packaging/linux/build-rpm.sh [extra-resourcepacks-dir]
#   The optional folder's .zip packs are bundled too (local builds only:
#   packs with Mojang art never go into the repository or GitHub releases).
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
RELEASE="${RELEASE:-1}"
BIN=target/release/araracraft
[ -x "$BIN" ] || { echo "build first: cargo build --release -p game" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
STAGE="$WORK/stage"
mkdir -p "$STAGE/opt/araracraft/resourcepacks" "$STAGE/icons"
install -m 0755 "$BIN" "$STAGE/opt/araracraft/araracraft"
cp -r packs shaderpacks-examples "$STAGE/opt/araracraft/"
cp README.md README.pt-BR.md LICENSE-MIT LICENSE-APACHE ASSETS-LICENSE.md docs/COMO-JOGAR.txt "$STAGE/opt/araracraft/"
mkdir -p "$STAGE/opt/araracraft/credits"
for d in assets/sounds/*/; do cp "$d/CREDITS.md" "$STAGE/opt/araracraft/credits/sounds-$(basename "$d").md"; done
if [ -n "${1:-}" ]; then
    cp "$1"/*.zip "$STAGE/opt/araracraft/resourcepacks/"
fi
cp assets/icon/icon_{32,48,64,128,256,512}.png "$STAGE/icons/"
cp packaging/linux/araracraft.sh packaging/linux/araracraft.desktop "$STAGE/"

rpmbuild -bb packaging/linux/araracraft.spec \
    --define "_topdir $WORK/rpm" \
    --define "stage $STAGE" \
    --define "game_version $VERSION" \
    --define "game_release $RELEASE" \
    --define "_binary_payload w19.zstdio"
mkdir -p dist
cp "$WORK"/rpm/RPMS/x86_64/*.rpm dist/
ls -l dist/*.rpm
