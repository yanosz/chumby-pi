#!/bin/sh
# Build the two chumby debs:
#   chumby-player       arm64  binary, launcher, chumby-ctl, kiosk unit
#   chumby-player-data  all    fixtures tree + controlpanel.swf (PRIVATE)
#
# Inputs: the cross-compiled dist binary (claude-docs/development.md §3)
# and controlpanel.swf. Both, plus the fixtures and the helper scripts,
# live in the ruffle/ submodule — the player owns its own environment;
# this repo only packages it. Override the SWF with CHUMBY_SWF.
# Output: pkg/out/*.deb. Staging in pkg/build/ (both gitignored).

set -eu

cd "$(dirname "$0")"
REPO=$(cd .. && pwd)

VERSION="${VERSION:-0.6.0}"
# dist = release + fat LTO + codegen-units=1 (what upstream ships);
# measurably lighter on the Pi's CPU-bound rasterization (doc 11).
BIN="$REPO/ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop"
SWF="${CHUMBY_SWF:-$REPO/ruffle/swf-assets/controlpanel.swf}"
BUILD="$REPO/pkg/build"
OUT="$REPO/pkg/out"

[ -x "$BIN" ] || { echo "missing $BIN — cross-build with --profile dist first (claude-docs/development.md §3)" >&2; exit 1; }
[ -f "$SWF" ] || { echo "missing $SWF — put controlpanel.swf in ruffle/swf-assets/ (docs/setup.md §2) or set CHUMBY_SWF" >&2; exit 1; }

rm -rf "$BUILD"
mkdir -p "$BUILD" "$OUT"

# --- chumby-player (arm64) ---
P="$BUILD/chumby-player"
mkdir -p "$P/DEBIAN" "$P/usr/bin" "$P/usr/lib/chumby-player" \
         "$P/lib/systemd/system" "$P/usr/lib/udev/rules.d" \
         "$P/media/chumby-usb" "$P/etc/chumby-player"
sed "s/@VERSION@/$VERSION/" chumby-player/DEBIAN/control > "$P/DEBIAN/control"
install -m 755 chumby-player/DEBIAN/postinst chumby-player/DEBIAN/prerm "$P/DEBIAN/"
# Owner config (fork FR14/FR15): shipped as a dpkg conffile so local edits
# survive upgrades. Default content is the fork's committed template; the
# launcher links it into the live fixtures root at every start.
install -m 644 chumby-player/DEBIAN/conffiles "$P/DEBIAN/"
install -m 644 "$REPO/ruffle/fixtures/player.toml.example" \
    "$P/etc/chumby-player/player.toml"
install -m 755 "$BIN" "$P/usr/lib/chumby-player/ruffle_desktop"
install -m 755 "$REPO/ruffle/chumby-ctl" "$P/usr/bin/chumby-ctl"
install -m 755 "$REPO/ruffle/chumby-widget-channel" "$P/usr/bin/chumby-widget-channel"
install -m 755 chumby-player/chumby-player-run "$P/usr/bin/chumby-player-run"
install -m 644 chumby-player/chumby-player.service "$P/lib/systemd/system/"
install -m 644 chumby-player/chumby-widget-channel.service "$P/lib/systemd/system/"
install -m 644 chumby-player/90-chumby-ignore-cec-pointer.rules "$P/usr/lib/udev/rules.d/"
# Backlight write access for the player (fork FR16); inert until a
# display with a kernel backlight is installed.
install -m 644 chumby-player/90-chumby-backlight.rules "$P/usr/lib/udev/rules.d/"
# USB music: the automount pair (rule + templated mount unit). The deb
# also ships /media/chumby-usb itself, so the panel's /mnt/usb symlink
# always resolves — empty dir = no stick, the state the panel handles.
install -m 644 chumby-player/99-chumby-usb-music.rules "$P/usr/lib/udev/rules.d/"
install -m 644 chumby-player/chumby-usb-mount@.service "$P/lib/systemd/system/"
find "$P" -type d -exec chmod 755 {} +
dpkg-deb --build --root-owner-group "$P" "$OUT/chumby-player_${VERSION}_arm64.deb"

# --- chumby-player-data (all) ---
D="$BUILD/chumby-player-data"
mkdir -p "$D/DEBIAN" "$D/usr/share/chumby-player/swf"
sed "s/@VERSION@/$VERSION/" chumby-player-data/DEBIAN/control > "$D/DEBIAN/control"
cp -a "$REPO/ruffle/fixtures" "$D/usr/share/chumby-player/fixtures"
# Never ship the build box's generated dev identity (fork FR10).
rm -f "$D/usr/share/chumby-player/fixtures/rootfs/psp/guid"
# Ship a profile that matches the packaged widget sidecars.
"$REPO/ruffle/chumby-widget-channel" --fixtures "$D/usr/share/chumby-player/fixtures" --force --quiet
install -m 644 "$SWF" "$D/usr/share/chumby-player/swf/controlpanel.swf"
find "$D" -type d -exec chmod 755 {} +
dpkg-deb --build --root-owner-group "$D" "$OUT/chumby-player-data_${VERSION}_all.deb"

ls -lh "$OUT"
