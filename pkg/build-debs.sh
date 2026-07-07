#!/bin/sh
# Build the two chumby debs (survey 07 §4.2, approved CHECKPOINT 4):
#   chumby-player       arm64  binary, launcher, chumby-ctl, kiosk unit
#   chumby-player-data  all    fixtures tree + controlpanel.swf (PRIVATE)
#
# Inputs: the cross-compiled dist binary (claude-docs/reference/
# 08-pi-build.md) and controlpanel.swf from swf-assets/ (override with
# CHUMBY_SWF). Output: pkg/out/*.deb. Staging in pkg/build/ (both
# gitignored).

set -eu

cd "$(dirname "$0")"
REPO=$(cd .. && pwd)

VERSION="${VERSION:-0.2.0}"
# dist = release + fat LTO + codegen-units=1 (what upstream ships);
# measurably lighter on the Pi's CPU-bound rasterization (doc 11).
BIN="$REPO/ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop"
SWF="${CHUMBY_SWF:-$REPO/swf-assets/controlpanel.swf}"
BUILD="$REPO/pkg/build"
OUT="$REPO/pkg/out"

[ -x "$BIN" ] || { echo "missing $BIN — cross-build with --profile dist first (08-pi-build.md)" >&2; exit 1; }
[ -f "$SWF" ] || { echo "missing $SWF — put controlpanel.swf in swf-assets/ (docs/setup.md §2) or set CHUMBY_SWF" >&2; exit 1; }

rm -rf "$BUILD"
mkdir -p "$BUILD" "$OUT"

# --- chumby-player (arm64) ---
P="$BUILD/chumby-player"
mkdir -p "$P/DEBIAN" "$P/usr/bin" "$P/usr/lib/chumby-player" \
         "$P/lib/systemd/system" "$P/usr/lib/udev/rules.d"
sed "s/@VERSION@/$VERSION/" chumby-player/DEBIAN/control > "$P/DEBIAN/control"
install -m 755 chumby-player/DEBIAN/postinst chumby-player/DEBIAN/prerm "$P/DEBIAN/"
install -m 755 "$BIN" "$P/usr/lib/chumby-player/ruffle_desktop"
install -m 755 "$REPO/chumby-ctl" "$P/usr/bin/chumby-ctl"
install -m 755 chumby-player/chumby-player-run "$P/usr/bin/chumby-player-run"
install -m 644 chumby-player/chumby-player.service "$P/lib/systemd/system/"
install -m 644 chumby-player/90-chumby-ignore-cec-pointer.rules "$P/usr/lib/udev/rules.d/"
find "$P" -type d -exec chmod 755 {} +
dpkg-deb --build --root-owner-group "$P" "$OUT/chumby-player_${VERSION}_arm64.deb"

# --- chumby-player-data (all) ---
D="$BUILD/chumby-player-data"
mkdir -p "$D/DEBIAN" "$D/usr/share/chumby-player/swf"
sed "s/@VERSION@/$VERSION/" chumby-player-data/DEBIAN/control > "$D/DEBIAN/control"
cp -a "$REPO/fixtures" "$D/usr/share/chumby-player/fixtures"
install -m 644 "$SWF" "$D/usr/share/chumby-player/swf/controlpanel.swf"
find "$D" -type d -exec chmod 755 {} +
dpkg-deb --build --root-owner-group "$D" "$OUT/chumby-player-data_${VERSION}_all.deb"

ls -lh "$OUT"
