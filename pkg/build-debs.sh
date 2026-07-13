#!/bin/sh
# Build the chumby deb:
#   chumby-player  arm64  binary, launcher, helpers, kiosk unit and the
#                         git-clean fixtures tree — publishable, needs
#                         no .swf to build or install
#
# Input: the cross-compiled dist binary (claude-docs/development.md §3),
# nothing else. The copyrighted files (controlpanel.swf, widget SWFs,
# alarm tones, intro.swf) never enter a package: the owner copies them
# from a chumby or its backup into /var/lib/chumby — see
# chumby-player-run and design.md §5.
# Output: pkg/out/*.deb, staging in pkg/build/ (both gitignored).

set -eu

cd "$(dirname "$0")"
REPO=$(cd .. && pwd)

VERSION="${VERSION:-0.8.3}"
# dist = release + fat LTO + codegen-units=1 (what upstream ships);
# measurably lighter on the Pi's CPU-bound rasterization (doc 11).
BIN="$REPO/ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop"
BUILD="$REPO/pkg/build"
OUT="$REPO/pkg/out"

[ -x "$BIN" ] || { echo "missing $BIN — cross-build with --profile dist first (claude-docs/development.md §3)" >&2; exit 1; }

rm -rf "$BUILD"
mkdir -p "$BUILD" "$OUT"

# --- chumby-player (arm64) ---
P="$BUILD/chumby-player"
mkdir -p "$P/DEBIAN" "$P/usr/bin" "$P/usr/lib/chumby-player" \
         "$P/lib/systemd/system" "$P/usr/lib/udev/rules.d" \
         "$P/media/chumby-usb" "$P/etc/chumby-player" \
         "$P/etc/default" "$P/usr/share/chumby-player"
sed "s/@VERSION@/$VERSION/" chumby-player/DEBIAN/control > "$P/DEBIAN/control"
install -m 755 chumby-player/DEBIAN/postinst chumby-player/DEBIAN/prerm "$P/DEBIAN/"
# Owner config (fork FR14/FR15): shipped as a dpkg conffile so local edits
# survive upgrades. Default content is the fork's committed template; the
# launcher links it into the live fixtures root at every start.
install -m 644 chumby-player/DEBIAN/conffiles "$P/DEBIAN/"
install -m 644 "$REPO/ruffle/fixtures/player.toml.example" \
    "$P/etc/chumby-player/player.toml"
# Kiosk env overrides (DRM device, renderer, audio device) — all
# commented out; the unit's built-in defaults suit the reference Pi 3.
install -m 644 chumby-player/chumby-player.default "$P/etc/default/chumby-player"
install -m 755 "$BIN" "$P/usr/lib/chumby-player/ruffle_desktop"
install -m 755 "$REPO/ruffle/chumby-ctl" "$P/usr/bin/chumby-ctl"
install -m 755 chumby-player/chumby-local-widgets "$P/usr/bin/chumby-local-widgets"
install -m 755 chumby-player/chumby-download-firmware "$P/usr/bin/chumby-download-firmware"
install -m 755 chumby-player/chumby-player-run "$P/usr/bin/chumby-player-run"
install -m 644 chumby-player/chumby-player.service "$P/lib/systemd/system/"
install -m 644 chumby-player/90-chumby-ignore-cec-pointer.rules "$P/usr/lib/udev/rules.d/"
# Backlight write access for the player (fork FR16); inert until a
# display with a kernel backlight is installed.
install -m 644 chumby-player/90-chumby-backlight.rules "$P/usr/lib/udev/rules.d/"
# USB music: the automount pair (rule + templated mount unit). The deb
# also ships /media/chumby-usb itself, so the panel's /mnt/usb symlink
# always resolves — empty dir = no stick, the state the panel handles.
install -m 644 chumby-player/99-chumby-usb-music.rules "$P/usr/lib/udev/rules.d/"
install -m 644 chumby-player/chumby-usb-mount@.service "$P/lib/systemd/system/"
# The fixtures tree, git-tracked files only: everything the panel needs
# except the copyrighted binaries, which are gitignored and ship (or are
# owner-copied) separately. cp keeps working-tree edits; the prune drops
# untracked files (the private overlay, local junk, the dev guid).
cp -a "$REPO/ruffle/fixtures" "$P/usr/share/chumby-player/fixtures"
( cd "$REPO/ruffle" && git ls-files -o fixtures ) | while read -r f; do
    rm -f "$P/usr/share/chumby-player/$f"
done
find "$P/usr/share/chumby-player/fixtures" -type d -empty -delete
find "$P" -type d -exec chmod 755 {} +
dpkg-deb --build --root-owner-group "$P" "$OUT/chumby-player_${VERSION}_arm64.deb"

ls -lh "$OUT"
