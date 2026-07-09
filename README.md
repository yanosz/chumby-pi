# Turn a Raspberry Pi into a Chumby

> **Heads up:** this is an AI-generated hobby project, built for fun and
> not actively maintained. No support and no warranty — but it works, and
> it's yours to tinker with.

Remember the [Chumby](https://en.wikipedia.org/wiki/Chumby)? The little
squeezable Wi-Fi gadget that showed widgets and woke you up in the
morning, until its servers went dark. This project brings its control
panel back to life on a Raspberry Pi with a small touchscreen — running
the **real** `controlpanel.swf` from the original firmware, unmodified.

It does that with [chumby-ruffle](https://github.com/yanosz/chumby-ruffle),
a fork of the [Ruffle](https://ruffle.rs) Flash player that pretends to be
Chumby hardware. The panel never notices the difference: the clock ticks,
alarms ring, internet radio streams, and your volume and settings stick
around between reboots.

## What you'll need

- A **Raspberry Pi with a small touchscreen.** The reference build is a
  Pi 3B+ with a 3.5″ SPI display and a USB audio dongle; other displays
  and sound devices work too — see [docs/hardware.md](docs/hardware.md).
- The **Chumby firmware files** — `controlpanel.swf`, the widgets, and
  the alarm tones. These are copyrighted and **not included here**: pull
  them from your own Chumby's backup, or ask the maintainer. Which file
  goes where is in [docs/setup.md](docs/setup.md).

## Try it on your computer first

Before touching the Pi, you can run the panel in a window on any Linux
machine:

```sh
git clone --recursive https://github.com/yanosz/chumby-pi.git
cd chumby-pi
# drop controlpanel.swf into swf-assets/ first — see docs/setup.md
(cd ruffle && cargo build -p ruffle_desktop)
./run-controlpanel.sh
```

Your PC has no bend sensor (the squeeze the Chumby is famous for), so
long-press with the mouse to fake it.

## Put it on the Pi

The full walkthrough — flashing the card, installing the packages, and
setting up the boot-to-panel kiosk — lives in
**[docs/setup.md](docs/setup.md)**. Follow it to the end and the Pi boots
straight into the control panel, no desktop in sight.

## What's in this repo

| Directory | What's inside |
|-----------|---------------|
| `ruffle/` | the chumby-ruffle player, as a git submodule |
| `fixtures/` | the fake Chumby the panel talks to — web endpoints, canned command output, and a writable filesystem |
| `pkg/` | the Debian packages and the kiosk service that boots to the panel |
| `docs/` | the setup and hardware guides — **start with [docs/setup.md](docs/setup.md)** |
| `run-controlpanel.sh` | the desktop-window launcher used above |
| `swf-assets/` | where the copyrighted firmware files go (empty, ignored by git) |

There's also a `claude-docs/` folder with the engineering record — what
this thing has to do, how it's put together, and how to work on it. Handy
if you want to know *why* something works, but not needed to use it. The
player has its own record, in the `ruffle/` submodule.

## What works, and what doesn't

**Working on the Pi:** boot-to-panel kiosk, the clock, touch input
(long-press = squeeze), alarms, My-Streams internet radio, volume, mute,
and night mode — all with settings that persist across reboots.

**Not yet:** screen brightness control (waiting on a dimmable panel),
widget channel management, and USB music playback.

## A note on sharing

Never publish a built `chumby-player-data` package — it bundles the
copyrighted firmware files. Everything else here (scripts, fixtures, and
docs) is yours under Ruffle's Apache-2.0 / MIT licenses.
