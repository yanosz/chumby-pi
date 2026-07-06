# chumby-pi — the Chumby control panel on a Raspberry Pi

This project boots a Raspberry Pi with a small touchscreen straight
into the **original Chumby Classic control panel** — the real
`controlpanel.swf` from the device firmware, unmodified — using
[chumby-ruffle](https://github.com/yanosz/chumby-ruffle), a fork of the
[Ruffle](https://ruffle.rs) Flash emulator that recreates the chumby's
runtime environment (its vendor ActionScript natives, `exec://` URLs,
device filesystem, and audio player). The panel believes it is running
on a chumby: the clock widget plays, alarms ring, internet radio
streams, volume and settings persist.

What the fork does and how is documented in its
[CHUMBY.md](https://github.com/yanosz/chumby-ruffle/blob/chumby/CHUMBY.md).
This repo carries everything around it:

| Path | What |
|------|------|
| `ruffle/` | the chumby-ruffle fork, as a git submodule |
| `fixtures/` | the virtual chumby the panel talks to: HTTP endpoints, shell command responses, a writable rootfs ([fixtures/README.md](fixtures/README.md)) |
| `pkg/` | Debian packaging: two debs + a boot-to-panel systemd/cage kiosk unit |
| `run-controlpanel.sh` | run the panel in a window on a desktop machine |
| `docs/` | end-user documentation — start with [docs/setup.md](docs/setup.md) |
| `claude-docs/` | the internal engineering record (how every decision was reached) |
| `swf-assets/` | drop zone for the copyrighted SWF assets (empty, gitignored) |

## Quick start

```sh
git clone --recursive https://github.com/yanosz/chumby-pi.git
cd chumby-pi
# put controlpanel.swf into swf-assets/ (see below), then either:

# a) desktop window (any Linux machine)
(cd ruffle && cargo build -p ruffle_desktop --features chumby)
./run-controlpanel.sh

# b) Raspberry Pi kiosk — full walkthrough in docs/setup.md
```

Using a different display or sound device than the reference hardware
(Pi 3B+, ILI9486 3.5″ SPI TFT, USB audio)? See
[docs/hardware.md](docs/hardware.md).

## The SWF assets are not included

`controlpanel.swf`, the widget SWFs, and the chumby alarm tones are
copyrighted chumby firmware and are **not in this repository** and
never will be. Get them from your own chumby's backup, or contact the
maintainer. Where each file goes: [docs/setup.md §2](docs/setup.md).
Anything you drop into `swf-assets/`, `fixtures/widgets/`, or
`fixtures/rootfs/usr/chumby/alarmtones/` is gitignored. Never publish
built `chumby-player-data` debs — they contain these files.

## Status

Working on-device: boot-to-panel kiosk, clock widget, touch (with
long-press standing in for the bend-sensor squeeze), control panel,
alarms, My-Streams internet radio, volume/mute/night mode, settings
persistence. Not yet: backlight/brightness control (blocked on
dimmable panel hardware), widget channel management, USB music.

Ruffle is Apache-2.0/MIT; this repo's own scripts, fixtures, and docs
carry the same licenses.
