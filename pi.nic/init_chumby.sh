#!/bin/sh
set -e

SRC=$(dirname "$0")

mkdir -p /psp/rfs1
cp "$SRC/userhook0" "$SRC/userhook1" /psp/rfs1/
chmod +x /psp/rfs1/userhook0 /psp/rfs1/userhook1
sh -n /psp/rfs1/userhook0
sh -n /psp/rfs1/userhook1
