# Setup: from a bare Raspberry Pi to a Chumby

This walkthrough reproduces the reference device. If your display,
sound card, or Pi model differs, read [hardware.md](hardware.md)
alongside — every hardware-specific value below has an override point.

Reference hardware:

- Raspberry Pi 3B+ (any arm64-capable Pi should work; the 3B+ is the
  verified floor — the panel runs software-rendered at ~2 CPU cores)
- 3.5″ 480×320 SPI TFT with ILI9486 controller and XPT2046/ADS7846
  resistive touch (sold as "piscreen", Waveshare 3.5″ (B/C), and many
  clones)
- USB audio adapter (the Pi's headphone jack works too)
- Raspberry Pi OS **arm64** Lite (Debian 13 "trixie" base), SSH access

## 1. Get the sources

```sh
git clone --recursive https://github.com/yanosz/chumby-pi.git
cd chumby-pi
```

(`--recursive` pulls the `ruffle/` submodule — the actual player.)

## 2. Get the SWF assets

The copyrighted chumby firmware files are not in the repo. From your
own chumby backup (or from the maintainer), place:

| File(s) | Where | Needed for |
|---------|-------|------------|
| `controlpanel.swf` (2.8.87b3 verified) | `swf-assets/controlpanel.swf` | everything |
| widget SWFs referenced by the fixture channel (`unsubscribedclock.swf`, `builtinclock.swf`) | `fixtures/widgets/` | the clock widget on the home screen |
| alarm tones (`*.mp3`) | `fixtures/rootfs/usr/chumby/alarmtones/` | alarm sounds (alarms themselves work without them) |

All three locations are gitignored; nothing you drop there can end up
in a commit.

## 3. Cross-build the player (on a Debian/Ubuntu dev box)

Building on the Pi itself is not practical (RAM); build on a PC and
ship debs. One-time toolchain setup:

```sh
rustup target add aarch64-unknown-linux-gnu
sudo dpkg --add-architecture arm64
sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu \
    libasound2-dev:arm64 libudev-dev:arm64 \
    libssl-dev:arm64 libwayland-dev:arm64 libfontconfig-dev:arm64
```

The repo-root `.cargo/config.toml` already routes the aarch64 linker
and pkg-config paths. Then:

```sh
cd ruffle
cargo build --profile dist -p ruffle_desktop --features chumby \
    --target aarch64-unknown-linux-gnu
cd ..
```

`--features chumby` is essential — without it you get a stock Ruffle
that knows nothing about chumbys. The `dist` profile (fat LTO) takes a
while (~15–25 min cold) but measurably lowers CPU on the Pi.

## 4. Build the debs

```sh
pkg/build-debs.sh
```

Produces in `pkg/out/`:

- **`chumby-player_*_arm64.deb`** — the player binary, launcher,
  `chumby-ctl` helper, and the kiosk systemd unit.
  Runtime dependencies: `cage`, `mpv`, `pipewire-alsa`.
- **`chumby-player-data_*_all.deb`** — the fixtures tree plus your
  `controlpanel.swf`. **This deb contains copyrighted material — for
  your private use only, never publish it.**

## 5. Configure the display (on the Pi)

Add to `/boot/firmware/config.txt` (and comment out any old fbtft
line for the same panel):

```
dtoverlay=piscreen,speed=24000000,rotate=0,drm,swapxy=on,invy=on
```

- `drm` selects the DRM tiny driver (required — the player renders
  through a Wayland compositor, not the legacy framebuffer).
- `rotate=0` is landscape in DRM terms (DRM's base orientation is
  landscape; fbtft's `rotate=90` ≙ DRM `rotate=0`).
- `swapxy=on,invy=on` is the touch calibration for this panel at this
  rotation — verify and adjust per [hardware.md](hardware.md).

Reboot; you should see the kernel boot console on the TFT
(`ili9486` DRM driver). The screen going black afterwards is normal —
the kiosk owns it from here.

## 6. Install and run

```sh
# from the dev box
scp pkg/out/*.deb pi@<pi>:

# on the Pi
sudo apt install ./chumby-player_*_arm64.deb ./chumby-player-data_*_all.deb
sudo systemctl start chumby-player     # installing enabled it for next boot
```

The panel appears on the TFT and from now on comes up on every boot.

### Operating it

- **Touch** = the chumby's touchscreen. A **stationary long-press
  (≥1 s)** is the squeeze of the chumby's top button: it summons (and
  dismisses) the control panel bar, and snoozes a ringing alarm.
- `chumby-ctl bend` does the same from a shell; `chumby-ctl click X Y`
  and `chumby-ctl drag X1 Y1 X2 Y2` inject pointer input (useful over
  SSH).
- `sudo systemctl stop chumby-player` — exit player mode until next
  boot; `sudo systemctl disable --now chumby-player` — leave player
  mode; logs: `journalctl -u chumby-player`.
- Panel state (alarms, streams, volume — everything set in the UI)
  persists in `/var/lib/chumby/fixtures`. It survives package
  upgrades; after upgrading `chumby-player-data`, delete it to re-seed
  from the new fixtures (this discards panel-made settings).
- Overrides (display device, audio device, log level, …) go in
  `/etc/default/chumby-player` — see [hardware.md](hardware.md).

## 7. Desktop run (no Pi needed)

For trying it out or hacking on fixtures, any Linux desktop works:

```sh
cd ruffle && cargo build -p ruffle_desktop --features chumby && cd ..
./run-controlpanel.sh
```

This opens the panel in a window, reads `swf-assets/controlpanel.swf`
(override with `CHUMBY_SWF=<path>`), uses the repo's `fixtures/`
directly, tees a full host-traffic log to `/tmp/chumby-run.log`, and
creates `/tmp/chumby-ctl` — so `echo bend > /tmp/chumby-ctl` (or
typing `bend` + Enter in the launch terminal, or pressing Home with
the window focused) plays the squeeze. `fixtures/README.md` explains
how every mocked answer can be changed.
