# Design

How the appliance is put together. Requirements: [requirements.md](requirements.md).
Building, deploying and the record of what was done to the device:
[development.md](development.md).

---

## 1. Repository layout

```
ruffle/                  submodule → chumby-ruffle. The player AND its
                         environment: fixtures/, swf-assets/, the desktop
                         launcher, chumby-ctl, chumby-widget-channel.
pkg/                     Debian packaging + build-debs.sh
docs/                    end-user documentation
claude-docs/             this engineering record
```

The player owns everything it needs to run, so that player work happens in
one repository. This repo packages it and puts it on a Pi; `build-debs.sh`
reaches into the submodule for the binary, the fixtures, the SWF and the two
helper scripts it installs.

## 2. What this repo knows about the player

Three things, and nothing more.

**How to build it.** `cargo build --profile dist -p ruffle_desktop
--target aarch64-unknown-linux-gnu`, from inside the submodule, with the
cross toolchain configured by `.cargo/config.toml` at *this* repository's
root — deliberately outside the fork, so the fork stays upstream-clean.

**How to launch it.**

```sh
ruffle_desktop \
    --load-behavior blocking \
    --filesystem-access-mode allow \
    --chumby-fixtures <dir> \
    --chumby-control /tmp/chumby-ctl \
    --fullscreen -PlocalCache=1 \
    controlpanel.swf
```

`--load-behavior blocking` is not optional: the panel jumps to frame labels
that a streaming load has not parsed yet. `-PlocalCache=1` selects the
in-movie widget path. `--fullscreen` hides Ruffle's menu bar.

**What data it wants.** A fixtures directory — `rootfs/`, `exec/`, `http/`,
`widgets/` — which ships inside the submodule. The keys are the panel's own
request strings, so a `chumby_host=info` log line names exactly the file that
would have answered it.

Everything else about the player — the vendor-call table, the host trait, the
UI policy, how `file://` is intercepted, how the widget channel is generated —
belongs to chumby-ruffle's `claude-docs/`.

## 3. Fixtures are data, and they ship with the player

The fixture tree lives in the submodule; its contents and their format are
the player's business. Two properties of it shape this repo's design.

Paths inside fixture data may contain a `{FIXTURES}` token, which the player
expands to the absolute fixtures directory when it serves the value. That is
why the same profile XML names widget SWFs correctly on the dev box and on
the Pi. The first smoke deployment failed on a hardcoded dev-box path; the
token is the fix.

The rootfs is **read-write** — all of the panel's persistence (`/psp/alarms`,
`/psp/volume`, `/psp/url_streams`, `/psp/clock_format`) lives there. A
read-only copy under `/usr/share` therefore cannot be the live tree, which
drives the state-directory design in §5. It also means a fixture change is
deployed by rsync-and-restart, with no rebuild — unlike the UI-policy rules,
which are compiled into the player.

## 4. Booting the widget channel

The player generates its single widget channel from the widgets it ships
(`chumby-widget-channel`, in the submodule — design there). This repo owns
only when that runs on the device.

There is deliberately no boot-time regeneration (the
`chumby-widget-channel.service` oneshot was dropped in 0.7.0). The data
package ships a profile generated at build time from the packaged sidecars,
the first-run seed copies it into the state directory, and a plain launch
never rewrites state — the launcher only *checks* that a non-empty profile
exists and refuses with a hint otherwise. Regeneration is opt-in:
`chumby-player-run --scan-channel` runs the generator (`--force`) against
the live tree, for the rare case of sidecars added or removed by hand.

The service had existed to pick up exactly those hand-dropped sidecars at
boot; it became redundant when the panel's own local-profile merge was
verified on our stack (2026-07-13, fork design §3): `mergeLocalProfile()`
reads the first of `/tmp/profile.xml`, `/mnt/usb/profile.xml`,
`/mnt/storage/profile.xml`, `/psp/profile.xml` via `_getFile` and
concatenates its `<widget_instance>` entries onto the loaded channel — so
"add a widget without repackaging" is a file drop into the fixtures' `/psp`,
no regeneration involved. (This is the wiki's "mixing local widgets into a
channel" trick; it *adds to* a channel and cannot replace the base profile,
which is why the generator itself stays.)

## 5. The two packages, and the kiosk

`pkg/build-debs.sh` produces two `.deb`s with `dpkg-deb` over a staging tree.
They are not policy-compliant Debian source packages; vendoring a Rust
workspace into sbuild serves no goal here.

**`chumby-player`** (arm64) — the cross-built `dist` binary, the launcher
`chumby-player-run`, `chumby-ctl`, the widget-channel generator,
the systemd kiosk unit, and udev rules (CEC-pointer ignore §7, USB-music
automount, backlight write access §8). `Depends:` cage, mpv, pipewire-alsa,
python3, `chumby-player-data`, plus the library dependencies read off the
binary's `NEEDED` entries (`libc6`, `libgcc-s1`, `libfontconfig1`,
`libssl3t64`, `libasound2t64`, `libudev1` — note trixie's time64 renames).
`postinst` enables the unit but does not start it; installing means "player
mode on next boot". It also ships the owner config
`/etc/chumby-player/player.toml` as a dpkg **conffile** (default content =
the fork's committed `player.toml.example`): the player reads
`<fixtures>/player.toml` once at start, and the launcher symlinks the /etc
file into the live fixtures root on every start — so an owner edit (e.g.
`access_chumby_com = 1`) survives deb upgrades *and* the fixture re-seed
wipe that discards everything else under `/var/lib/chumby/fixtures`.

The launcher also owns the **boot-time intro**, reproducing rcS's
`start_intro`: before exec'ing the panel it runs the same binary on
`fixtures/rootfs/usr/widgets/intro.swf`, standalone but against the same
fixtures root — so the tour's enable/disable buttons really toggle
`/psp/disable_intro` — and skips it once that flag exists. The tour always
quits itself (end of tour and both control-screen buttons
`fscommand("quit")`, which the fork only swallows *inside* the panel), so
the wait is bounded; a crash falls through to the panel. `start_intro`'s
`/mnt/usb/intro.swf` override is deliberately not reproduced — auto-running
a SWF off whatever stick is inserted is a factory/debug hook, not a
behavior to keep.

**`chumby-player-data`** (all) — the `fixtures/` tree and `controlpanel.swf`.
Private use only (NFR1).

The kiosk unit runs **cage**, a single-application Wayland compositor,
launching the player fullscreen. The decisions inside it:

- **`User=pi` + `PAMName=login` + `TTYPath=/dev/tty1`** gives the service a
  real logind session on seat0. Without one, cage cannot open the DRM device
  and audio has no route: an SSH session cannot take seat0, and the
  experiments that ran as root needed `LIBSEAT_BACKEND=builtin` plus a
  PipeWire runtime-dir bridge. With a logind seat, cage opens the display as
  plain `pi` and audio lands in pi's own PipeWire. No root, no bridge.
  `Conflicts=getty@tty1.service` frees the VT.
- **`WLR_BACKENDS=drm,libinput`, `WLR_RENDERER=pixman`,
  `WLR_DRM_DEVICES=/dev/dri/by-path/…`** — the TFT is a tiny DRM card with no
  GPU, so pixman rather than a GL renderer, and a `by-path` name because card
  numbering drifts across boots (NFR3).
- **`StateDirectory=chumby`** gives `/var/lib/chumby` owned by `pi`. The
  launcher seeds `fixtures/` there from `/usr/share` on first run, because
  the panel writes into the rootfs (§3). The consequence is that after a
  `chumby-player-data` upgrade you must `rm -rf /var/lib/chumby/fixtures` to
  pick up new fixture content — discarding panel-made settings. Accepted.
- **`Restart=on-failure`**: a clean exit (the panel quit) does not respawn; a
  crash does.
- **`EnvironmentFile=-/etc/default/chumby-player`** is the override point for
  every one of the above.
- Packaged default `RUST_LOG=warn`. Per-call host logging is the right
  default for a debug session and the wrong one for an appliance journal.

## 6. Display

The TFT is an ILI9486 480×320 SPI clone with XPT2046 resistive touch. The
stock `piscreen` overlay binds it to the **fbtft** staging driver, which
gives `/dev/fb0` and nothing else — and fbdev is invisible to KMS/Wayland,
so cage cannot output to it.

The kernel already ships the mainline DRM tiny driver, and the stock overlay
has a `drm` parameter that switches to it. So the whole problem is one
overlay parameter, no custom driver, reversible at runtime. The rejected
alternative was keeping fbtft and writing a screencopy client to mirror the
headless compositor into `/dev/fb0`: custom software, continuous CPU cost,
two moving parts.

Two gotchas that cost time. DRM's base orientation is landscape, so fbtft's
`rotate=90` corresponds to DRM `rotate=0`. And the touch axes needed
`swapxy=on` plus `invy=on`, where `swapxy=on` *deletes* the
`touchscreen-swapped-x-y` property — an inverted boolean.

Throughput ceiling: 480×320×16 bit is 300 KiB a frame, so 24 MHz SPI allows
about 9.7 full frames per second. The DRM driver flushes only dirty
rectangles, so the mostly-static panel feels fine. This is the bus, not the
driver — fbtft has the same ceiling.

## 7. Audio, input, and the control channel

PipeWire is the sound server. The USB card is the default sink, so mpv and
the player's `cpal` output both land on it with no configuration —
`pipewire-alsa` must be installed for the ALSA path to exist at all.
`CHUMBY_AUDIO_DEVICE` overrides mpv's sink.

`chumby-ctl` writes to a FIFO the player reads: `bend`, `bend down|up`,
`click X Y`, `drag X1 Y1 X2 Y2`. It is the bend button until GPIO17 is
wired, the remote-control path for scripted testing over SSH, and the
intended home of the eventual `exit-player` magic key.

The kiosk showed a mouse cursor on a device with no mouse. The cause was not
the client: `libinput list-devices` traced the seat's only pointer capability
to `vc4-hdmi`, the HDMI-CEC input device. wlroots drew a cursor for that
phantom pointer. The fix is a udev rule marking that device
`LIBINPUT_IGNORE_DEVICE`. A real USB mouse still produces a cursor, which is
what we want, and the CEC device only ever carried remote-control keys.

## 8. Brightness: what a future display must have

The current panel cannot dim (requirements §3); the display purchase is the
open half. The player half shipped 2026-07-13 (fork FR16: slider mode onto
any `/sys/class/backlight` device, or an owner `brightness_ctl` executable
for the discrete 0/1/2 path), and so did this repo's permission side: the
backlight sysfs file is root-owned and has no `/dev` node — the usual
`GROUP=`/`MODE=` udev keys do nothing — so `90-chumby-backlight.rules`
(the rule brightnessctl ships) `chgrp video`s + `g+w`s the `brightness`
file when a backlight device appears, and pi is in `video`. Inert on the
current TFT, which registers no backlight device; when a dimmable display
lands, the panel's brightness sliders work with no further appliance work.
A `brightness_ctl` program (e.g. GPIO PWM by hand) would instead be the
owner's own script named in `/etc/chumby-player/player.toml`.

Requirements for a
replacement: ~3.5" SPI HAT on the 2×20 header, touch, 480×320-ish, a
**PWM-dimmable** backlight, and a mainline DRM driver so §5 and §6 carry over
unchanged.

- **Adafruit PiTFT Plus 3.5" (2441)** — recommended. HX8357D + STMPE610.
  Backlight over the STMPE's spare GPIO (on/off, appears under
  `/sys/class/backlight`) or GPIO18 = hardware PWM0 for smooth dimming.
  `pitft35-resistive.dtbo` with a `drm` parameter, `hx8357d.ko`, `stmpe-ts.ko`
  and `pwm.dtbo` are all already present on our image. Caveat: GPIO18 is
  shared with the Pi's analog audio; we use USB audio.
- **Waveshare 3.5" (C)** — cheaper, and it *does* have dimming hardware, but
  disconnected by default behind a solder pad. Its driver route is the legacy
  LCD-show/FBCP stack, and no mainline tiny driver is known for its
  high-speed mode. The risk lands exactly on the part that took the most
  effort to get right.
- **Waveshare 3.5" (A)/(B)** — same hardwired-backlight design as the current
  clone. Buying one changes nothing.
- **Hardware-modding the current clone** (cut the LED anode trace, add an NPN
  transistor from GPIO18) is a documented, zero-cost fallback that involves
  soldering on the only panel we have.
- **DPI panels** (HyperPixel and friends) consume the whole GPIO header,
  killing SPI and the bend button. Out.

## 9. CI

One chumby workflow per repository (the fork also carries upstream's, kept
so merges stay conflict-free), and the acceptance criterion in both is **"the
movie starts"** — not "it builds". An upstream merge can compile clean and
still leave the vendor-call hooks dead.

That criterion is only exercised **off** pull requests, though: starting the
movie needs the copyrighted SWF, so a PR builds and stops, and the real check
runs on push to the default branch. The pre-merge gate is a local run.

The check: run the player headless under `timeout 45` with
`chumby_host=info`; success is exit code 124 (still alive when the timeout
fired) *and* `_getPlatform` in the log (a chumby native actually executed)
*and* no `panicked` line.

`controlpanel.swf` is fetched from a private WebDAV share by rclone
configured purely through environment variables, so the endpoint appears in
neither repo. Only `URL`, `USER` and `PASS` are secrets (`PASS` holds
rclone's *obscured* form); `TYPE` and `VENDOR` are plain workflow values and
reveal nothing.

This repo's job runs on a GitHub-hosted **arm64** runner: it builds the `dist`
binary natively, builds the debs, and installs them into an arm64
`debian:trixie` container — which is what actually proves the declared
dependencies are complete — then runs the movie-start check from the
*packaged* SWF and fixtures. Only the SWF-free `chumby-player` deb is
uploaded as an artifact.
