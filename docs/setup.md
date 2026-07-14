# Setup: from a bare Raspberry Pi to a Chumby

This walkthrough reproduces the reference device. If your display,
sound card, or Pi model differs, read [hardware.md](hardware.md)
alongside — every hardware-specific value has an override point.

Reference hardware:

- Raspberry Pi 3B+ (any arm64-capable Pi should work; the 3B+ is the
  verified floor — the panel runs software-rendered at about one core)
- 3.5″ 480×320 SPI TFT with ILI9486 controller and XPT2046/ADS7846
  resistive touch (sold as "piscreen", Waveshare 3.5″ (B), and many
  clones)
- USB audio adapter (the Pi's headphone jack works too)
- Raspberry Pi OS **arm64 Lite** (Debian 13 "trixie" base), SSH access

No build environment is needed — everything installs from a package
repository.

## 1. Configure the display

**HDMI and DSI displays need nothing** — skip to step 2.

An SPI TFT cannot be auto-detected, so it must be declared once. Append
to `/boot/firmware/config.txt` and reboot:

```
dtparam=spi=on
dtoverlay=piscreen,speed=24000000,rotate=0,drm,swapxy=on,invy=on
```

- **Skip the display vendor's driver instructions.** Overlays like
  `waveshare35b-v2` use the legacy fbtft framebuffer driver, which the
  kiosk cannot use — it needs a DRM device, and the mainline kernel's
  driver (the `drm` option above) provides one for this panel.
- `rotate=0` is landscape in DRM terms; `swapxy=on,invy=on` aligns the
  touch axes at this rotation ([hardware.md](hardware.md) if yours
  differs).

After the reboot the kernel boot console appears on the TFT. The kiosk
finds the panel by itself on any Pi model — there is nothing to
configure on the software side.

## 2. Install the package

The signed apt repository is served from this project's GitHub Pages:

```sh
curl -fsSL https://yanosz.github.io/chumby-pi/apt/chumby-archive.gpg \
  | sudo tee /usr/share/keyrings/chumby-archive.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/chumby-archive.gpg] https://yanosz.github.io/chumby-pi/apt ./' \
  | sudo tee /etc/apt/sources.list.d/chumby.list
sudo apt update
sudo apt install chumby-player
```

The install transcript ends with next-step guidance derived from your
machine's actual state — it tells you if a display overlay or the
firmware files are still missing.

## 3. Get the firmware files

The package contains everything **except** the copyrighted chumby
firmware files. Two ways to get them:

**Download (no chumby needed):**

```sh
chumby-download-firmware
```

It explains what it fetches and from where (Chumby's servers still
carry the classic firmware image and the newest control panel), asks
before touching the network, and verifies checksums. It gets you the
control panel, the guided-tour intro and the alarm tones — a complete
chumby that boots to a clock; widgets are optional extras.

**Copy from your own chumby or its backup** — the same files, plus any
widgets you want; left side is the path on the chumby:

| On the chumby | On the Pi | Needed for |
|---|---|---|
| `/usr/widgets/controlpanel.swf` | `/var/lib/chumby/controlpanel.swf` | everything (skip if downloaded) |
| `/usr/widgets/*.swf` + `*.jpg` | `/var/lib/chumby/widgets/` | more widgets — then run `chumby-local-widgets` |
| `/usr/chumby/alarmtones/*.mp3` | `/var/lib/chumby/alarmtones/` | alarm sounds; **keep the filenames** (the panel hardcodes them) |
| `/usr/widgets/intro.swf` | `/var/lib/chumby/intro.swf` | the guided-tour INTRO (its button stays dimmed without it) |

Files must be readable by the `pi` user — if you copied with `sudo`,
`sudo chmod 644` them (the player says exactly this in
`systemctl status chumby-player` if it can't read something).

## 4. Start it

```sh
sudo systemctl start chumby-player    # installing enabled it for next boot
```

The panel appears on the TFT and from now on comes up on every boot.

### Operating it

- **Touch** = the chumby's touchscreen. A **stationary long-press
  (≥1 s)** is the squeeze of the chumby's top button: it summons (and
  dismisses) the control panel bar, and snoozes a ringing alarm.
  `chumby-ctl bend` does the same from a shell.
- `sudo systemctl stop chumby-player` — leave player mode until next
  boot; `sudo systemctl disable --now chumby-player` — leave it for
  good; logs: `journalctl -u chumby-player`.
- Panel state (alarms, streams, volume — everything set in the UI)
  persists in `/var/lib/chumby/fixtures` and survives upgrades.

### Your own widgets

Drop widget `.swf` files (plus an optional same-named `.jpg` preview)
into `/var/lib/chumby/widgets`, run `chumby-local-widgets`, restart
the player.

## 5. Configuration

Two files, both surviving upgrades:

- **`/etc/chumby-player/player.toml`** — owner policy: volume cap,
  `access_chumby_com` (opt-in: internet-radio directories, device
  registration and your account's widget channels from the still-alive
  chumby.com), `merge_local_remote_widgets` (whether local widgets ride
  along inside those account channels), `enable_lyrion`,
  `brightness_ctl`. Each key is documented in the file.
- **`/etc/default/chumby-player`** — environment overrides: pin a
  specific display (`WLR_DRM_DEVICES`), route audio to a specific
  output (`CHUMBY_AUDIO_DEVICE`), log verbosity (`RUST_LOG`). Also
  documented in place.

Sound needs no setup: it follows the system default output (a USB
adapter if present).

## 6. Desktop run (no Pi needed)

For trying it out or hacking on the player, any Linux desktop works —
that path needs a source build; see the
[chumby-ruffle](https://github.com/yanosz/chumby-ruffle) repository:

```sh
git clone --recursive https://github.com/yanosz/chumby-pi.git
cd chumby-pi/ruffle && cargo build -p ruffle_desktop
# drop controlpanel.swf into swf-assets/ (see step 3 for sources)
./run-controlpanel.sh
```

The panel opens in a window; a mouse long-press plays the squeeze, and
`/tmp/chumby-run.log` records every request the panel makes of its
fake chumby.
