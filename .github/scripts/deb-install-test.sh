#!/bin/sh
# Runs INSIDE an arm64 debian:trixie container (the Raspberry Pi OS
# base) with pkg/out mounted read-only at /debs. Proves the BC3
# acceptance: the debs install with all dependencies declared, and the
# installed binaries run — including actually starting the movie from
# the packaged SWF + fixtures.
set -eux

export DEBIAN_FRONTEND=noninteractive
apt-get update
# Install both debs together so chumby-player's dependency on
# chumby-player-data resolves. apt pulls the declared Depends; an
# undeclared library dep would surface as a load error in the run below.
apt-get install -y /debs/chumby-player_*.deb /debs/chumby-player-data_*.deb

# Binaries present and the player binary runs (links against the
# declared libraries).
test -x /usr/bin/chumby-ctl
test -x /usr/bin/chumby-player-run
/usr/lib/chumby-player/ruffle_desktop --version

# Movie-start test from the packaged data: same assertion as the
# chumby-ruffle workflow — player still alive when the timeout fires
# (exit 124) and the panel executed chumby host calls.
# libxkbcommon-x11-0: winit's X11 backend dlopens it and panics
# without it (found by the fork CI run); only the Xvfb test needs it —
# on the Pi the player runs under Wayland, where cage's dependencies
# provide libxkbcommon.
apt-get install -y --no-install-recommends \
    xvfb mesa-vulkan-drivers libgl1-mesa-dri fonts-dejavu-core \
    libxkbcommon-x11-0

export RUST_LOG=warn,chumby_host=info,avm_trace=info
set +e
xvfb-run -a timeout 45 /usr/lib/chumby-player/ruffle_desktop \
    --load-behavior blocking --filesystem-access-mode allow \
    --chumby-fixtures /usr/share/chumby-player/fixtures \
    --width 640 --height 480 -PlocalCache=1 \
    /usr/share/chumby-player/swf/controlpanel.swf > /tmp/run.log 2>&1
status=$?
set -e
echo "player exit status: $status (124 = alive at timeout, expected)"
tail -n 80 /tmp/run.log
test "$status" -eq 124
grep -q '_getPlatform' /tmp/run.log
if grep -q 'panicked' /tmp/run.log; then
    echo "panic found in player log" >&2
    exit 1
fi
echo "deb install + run test PASSED"
