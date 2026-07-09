# Design

How the appliance is put together. Requirements: [requirements.md](requirements.md).
Building, deploying and the record of what was done to the device:
[development.md](development.md).

---

## 1. Repository layout

```
ruffle/                  submodule → chumby-ruffle (the player)
fixtures/                everything the player answers the panel with
  rootfs/                the panel's virtual filesystem (/psp, /tmp, /LICENSES…)
  exec/                  canned stdout for the panel's shell commands
  http/<host>/<path>     canned chumby.com responses
  widgets/               widget SWFs + per-widget XML sidecars + thumbnails
swf-assets/              controlpanel.swf — gitignored, you supply it
pkg/                     Debian packaging + build-debs.sh
chumby-widget-channel    boot-time generator of the single local channel
chumby-ctl               shell client for the player's control FIFO
run-controlpanel.sh      desktop run
docs/                    end-user documentation
claude-docs/             this engineering record
```

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

**What data it wants.** A fixtures directory, laid out as above. The keys are
the panel's own request strings — a command line, a URL path, a filesystem
path — so a `chumby_host=info` log line names exactly the file that would
have answered it.

Everything else about the player — the vendor-call table, the host trait, how
`file://` is intercepted — belongs to chumby-ruffle's `claude-docs/`.

## 3. Fixtures are data, not code

Two consequences worth stating.

Changing what the panel is told needs **no rebuild**. The network fixture,
the widget channel and the seeded `/psp` values are all files; an rsync and
a service restart are the whole deployment for a change that touches only
them. The UI-policy rules are the exception — they live in the player and
are compiled in, because which of the panel's controls are dead is a fact
about the panel, not about this packaging.

Paths inside fixture data may contain a `{FIXTURES}` token, which the player
expands to the absolute fixtures directory when it serves the value. That is
why the same profile XML names widget SWFs correctly on the dev box and on
the Pi. The first smoke deployment failed on a hardcoded dev-box path; the
token is the fix.

The rootfs is **read-write** — all of the panel's persistence (`/psp/alarms`,
`/psp/volume`, `/psp/url_streams`, `/psp/clock_format`) lives there. A
read-only copy under `/usr/share` therefore cannot be the live tree, which
drives the state-directory design in §5.

## 4. The single local widget channel

Real chumby fetched a *channel* — a list of widget instances — from
chumby.com. We generate one at boot from the widgets we ship.

Each widget carries an XML sidecar next to its SWF
(`unsubscribedclock.widget.xml`), holding exactly the `<widget>` element the
panel consumes: name, description, version, mode, access, `<movie href>` and
an optional `<thumbnail href>`. `chumby-widget-channel` (Python, XML in, XML
out) enumerates the sidecars, wraps each in the
`<widget_instance>`/`<profile>` envelope, assigns instance ids, and writes
`fixtures/http/xml.chumby.com/xml/profiles` plus the two
`/tmp/currentProfile*` files. Channel order is the sidecar's `id` attribute.

It deep-copies the whole `<widget>` element, so the thumbnail rides along
with no generator change. It skips the rewrite when a sha256 over the ordered
sidecar set is unchanged. Its faithfulness was established by regenerating
the previously hand-written, known-good profile fixture and finding the
output canonical-XML equal.

Boot wiring: a `chumby-widget-channel.service` oneshot runs
`Before=chumby-player.service`, seeding the state directory if absent and
then refreshing the profile. The player unit `Wants=` it rather than
`Requires=` it, so a generator failure still lets the panel start and hit its
own guard. The generator is deliberately **not** folded into the launcher —
a debug launch must work without regenerating first — so both launchers only
*check* that a non-empty profile exists and refuse with a hint otherwise.

The dashboard preview is a static 80×60 JPEG per widget, `loadMovie`'d from
the `<thumbnail href>`. `loadMovie` decodes an image as readily as a SWF, so
this needs no second live render. The panel gates the thumbnail on
`hasNetwork`, which our healthy `network_status.sh` answer already satisfies.

## 5. The two packages, and the kiosk

`pkg/build-debs.sh` produces two `.deb`s with `dpkg-deb` over a staging tree.
They are not policy-compliant Debian source packages; vendoring a Rust
workspace into sbuild serves no goal here.

**`chumby-player`** (arm64) — the cross-built `dist` binary, the launcher
`chumby-player-run`, `chumby-ctl`, the widget-channel generator and its unit,
the systemd kiosk unit, and a udev rule. `Depends:` cage, mpv, pipewire-alsa,
python3, `chumby-player-data`, plus the library dependencies read off the
binary's `NEEDED` entries (`libc6`, `libgcc-s1`, `libfontconfig1`,
`libssl3t64`, `libasound2t64`, `libudev1` — note trixie's time64 renames).
`postinst` enables the unit but does not start it; installing means "player
mode on next boot".

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

The current panel cannot dim (requirements §3). Requirements for a
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

One workflow per repository, and the acceptance criterion in both is **"the
movie starts"** — not "it builds". An upstream merge can compile clean and
still leave the vendor-call hooks dead.

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
