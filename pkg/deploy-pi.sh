#!/bin/bash
# (Re-)build the chumby debs and deploy them to a Pi:
#   pkg/deploy-pi.sh <pi-host>
#
# Always rebuilds the dist binary first — a stale binary has faked test
# results before (claude-docs/development.md §7). Installs into a clean
# directory on the Pi so leftover debs from earlier deploys can't be
# picked up by the glob.

set -euo pipefail

[ $# -eq 1 ] || { echo "usage: $0 <pi-host>" >&2; exit 1; }
PI="pi@$1"

cd "$(dirname "$0")/.."

cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu --manifest-path ruffle/Cargo.toml

# A clean out/ so leftover debs from earlier versions aren't deployed too.
rm -rf pkg/out
pkg/build-debs.sh

ssh "$PI" 'rm -rf /tmp/chumby-debs && mkdir -p /tmp/chumby-debs'
scp pkg/out/*.deb "$PI:/tmp/chumby-debs/"
# --reinstall: redeploying the same version must still replace the files.
ssh "$PI" 'sudo apt install --reinstall -y /tmp/chumby-debs/*.deb &&
           sudo systemctl restart chumby-player'

echo "deployed to $1:"
ssh "$PI" 'dpkg-query -W chumby-player chumby-player-data &&
           systemctl is-active chumby-player'
