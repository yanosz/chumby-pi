#!/bin/sh
# Runs INSIDE an arm64 debian:trixie container (the Raspberry Pi OS
# base) with pkg/out mounted read-only at /debs, and — on main, where CI
# fetched it — controlpanel.swf mounted at /swf. Proves the deb installs
# with all dependencies declared and the launcher behaves: refuses
# helpfully without the owner's controlpanel.swf, and starts the movie
# from an owner-copied one when /swf provides it (skipped by default).
set -eux

export DEBIAN_FRONTEND=noninteractive
apt-get update
# apt pulls the declared Depends; an undeclared library dep would
# surface as a load error in the run below.
apt-get install -y /debs/chumby-player_*.deb

# Binaries present and the player binary runs (links against the
# declared libraries).
test -x /usr/bin/chumby-ctl
test -x /usr/bin/chumby-local-widgets
test -x /usr/bin/chumby-player-run
/usr/lib/chumby-player/ruffle_desktop --version

# The public deb carries the fixtures but never a SWF.
test -d /usr/share/chumby-player/fixtures
if dpkg -L chumby-player | grep -iq '\.swf$'; then
    echo "a .swf leaked into the public deb" >&2
    exit 1
fi

# Without controlpanel.swf the launcher must refuse with instructions
# (it exits before needing any display).
set +e
/usr/bin/chumby-player-run > /tmp/noswf.log 2>&1
status=$?
set -e
test "$status" -eq 1
grep -q 'controlpanel.swf not found' /tmp/noswf.log

# Movie-start test: needs the copyrighted SWF, so it runs only when CI
# mounted one (push to main) and is skipped by default — PRs stop here.
if [ ! -f /swf/controlpanel.swf ]; then
    echo "no /swf/controlpanel.swf mounted — movie-start test skipped"
    exit 0
fi

# libxkbcommon-x11-0: winit's X11 backend dlopens it and panics
# without it (found by the fork CI run); only the Xvfb test needs it —
# on the Pi the player runs under Wayland, where cage's dependencies
# provide libxkbcommon.
apt-get install -y --no-install-recommends \
    xvfb mesa-vulkan-drivers libgl1-mesa-dri fonts-dejavu-core \
    libxkbcommon-x11-0

# Owner setup exactly as documented, then the real launcher: seeding,
# symlinks and SWF resolution are all exercised. Same assertion as the
# chumby-ruffle workflow — player still alive when the timeout fires
# (exit 124) and the panel executed chumby host calls.
mkdir -p /var/lib/chumby
cp /swf/controlpanel.swf /var/lib/chumby/controlpanel.swf
export RUST_LOG=warn,chumby_host=info,avm_trace=info
set +e
xvfb-run -a timeout 45 /usr/bin/chumby-player-run > /tmp/run.log 2>&1
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
