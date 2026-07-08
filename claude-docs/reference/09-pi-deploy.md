# 09 — Pi deployment (Step 3.3)

Date: 2026-07-02. Target: interim Pi 3A+ (`pi@192.168.210.159`, survey in
07). This documents the **smoke-test deployment** — hand-rolled, run
inside the stock desktop session. The CHECKPOINT-4-approved end state
(two .deb packages, cage kiosk, systemd unit) is built after CHECKPOINT 5
feedback; its layout mirrors this one. **Superseded 2026-07-06 by the
packaged deployment — see 12-kiosk-packaging.md.**

## 1. Layout on the Pi

```
/home/pi/chumby/
  bin/ruffle_desktop    aarch64 release build, chumby feature (see 08)
  bin/chumby-ctl        bend/IPC trigger script (repo root copy)
  fixtures/             copy of the repo fixtures/ tree
  swf/controlpanel.swf  copied from the dev box — the Pi does NOT touch
                        /home/jan/chumby_backup (plan rule)
  run.sh                launcher (below)
```

Pushed with rsync from the dev box:

```sh
rsync -a ruffle/target/aarch64-unknown-linux-gnu/release/ruffle_desktop \
         chumby-ctl pi@192.168.210.159:chumby/bin/
rsync -a --delete fixtures/ pi@192.168.210.159:chumby/fixtures/
rsync -a /home/jan/chumby_backup/tmp/controlpanel.swf \
         pi@192.168.210.159:chumby/swf/
```

Fixtures are **unmodified** — path portability comes from the
`{FIXTURES}` token (fixtures/README.md), added after the first smoke run
failed on a hardcoded dev-box path.

## 2. Launcher (`run.sh`)

Mirror of `run-controlpanel.sh` with Pi paths; creates the
`/tmp/chumby-ctl` FIFO, exports `XDG_RUNTIME_DIR`/`WAYLAND_DISPLAY`
(defaults for the labwc autologin session), then execs:

```sh
bin/ruffle_desktop --load-behavior blocking \
    --filesystem-access-mode allow \
    --chumby-fixtures $HOME/chumby/fixtures \
    --chumby-control /tmp/chumby-ctl \
    --fullscreen -PlocalCache=1 \
    swf/controlpanel.swf
```

(`--fullscreen` since 2026-07-02, user request: panel covers the screen,
Ruffle menu bar gone; SWF is pillarboxed 4:3 on the 16:10 HDMI panel.)

Start/stop over SSH:

```sh
nohup ~/chumby/run.sh > /tmp/chumby-run.log 2>&1 &
pkill -x ruffle_desktop
~/chumby/bin/chumby-ctl bend        # summon/dismiss the panel bar
```

**`pkill -x`, never `pkill -f ruffle_desktop`:** with `-f` the pattern
also matches the SSH session's own `bash -c` command line, killing the
session itself (exit 255, and whatever came after the pkill never runs).
Cost an hour of confusion on 2026-07-02.

## 3. Dependencies installed on the Pi (2026-07-02)

`mpv` 0.40 (audio backend), `cage` 0.2 (for the kiosk stage, unused in
the smoke test), `grim` (Wayland screenshots for verification). All via
apt; passwordless sudo for user `pi`.

## 4. Audio route

PipeWire is the sound server (trixie default) and the **USB card
(UACDemoV1.0) is already the default sink**, so mpv and Ruffle's cpal
output both land on it with no configuration. Available overrides if a
different routing is ever wanted:
- mpv: `CHUMBY_AUDIO_DEVICE=pipewire/alsa_output.usb-Jieli_Technology_…`
  (env var, see fixtures/README.md);
- Ruffle UI sounds: upstream `output_device` preference
  (`~/.config/ruffle/preferences.toml`).

## 5. Graphics reality check (risk from 07 §3, confirmed)

wgpu picks lavapipe (software Vulkan; the only VK_EXT_physical_device_drm
warning in the log is lavapipe's). Measurements (fullscreen, /proc
jiffie deltas over 5 s; 4 cores = 400 % budget):

| configuration | ruffle CPU |
|---------------|-----------|
| windowed 640×480 | ~220 % (ps lifetime avg — not comparable) |
| fullscreen @ 1920×1200 | ~345 % |
| fullscreen @ 800×600 | ~315 % |
| fullscreen @ 800×600, `--quality low` | ~318 % |

Output mode and stage quality are **not** the lever — the SWF
rasterization itself dominates, and the panel still keeps up (clock
ticks, UI responds). Sustained load brushed the 3A+'s soft thermal
limit once (58.5 °C, `get_throttled` 0x80000 sticky bit) — plan a
heatsink for the final 3B+ build, and expect the cage kiosk (no desktop
apps) to free a little headroom. Revisit on the real TFT (~480×320,
SPI/DBI panel — a different display pipeline entirely).

## 6. Smoke-test results

See `docs/progress.md` (2026-07-02 M3 entry) and screenshots
`images/m3-pi-clock.png`, `images/m3-pi-panel-bar.png`. Boot → widget,
bend summon → B2 bar: pass, zero log errors. Alarm ring-through, My
Streams playback, and audible volume are deliberately left for the user
test at CHECKPOINT 5 (they need ears at the device).

## 7. Addendum 2026-07-06 — final 3B+, headless cage, TFT driver

The SD card moved unchanged to the final **Pi 3B+** (new MAC → new DHCP
lease, `pi@192.168.210.137`). Setup changes for the CHECKPOINT-5 sound
verification:

- **Desktop GUI disabled**: `systemctl disable --now lightdm`, with
  `loginctl enable-linger pi` set FIRST so the user manager (and
  PipeWire) survives without any login session.
- **Panel runs under headless cage** (no usable display: TFT had no
  driver, HDMI unplugged). Start/stop over SSH:

  ```sh
  nohup env WLR_BACKENDS=headless cage -- /home/pi/chumby/run.sh \
      > /tmp/chumby-run.log 2>&1 &
  pkill -x mpv; pkill -x ruffle_desktop; pkill -x cage
  ```

  Verification is screenshot-based: `WAYLAND_DISPLAY=wayland-0 grim x.png`.
  **Kill mpv too**: SIGTERM on ruffle skips destructors, so its mpv child
  survives as an unkillable-via-panel orphan that still plays audio.
- **`pipewire-alsa` installed** — without it ALSA apps (Ruffle's cpal)
  have no route to PipeWire and audio-device creation fails. mpv talks
  to PipeWire natively and never needed it.
- **Remote input**: wlrctl's virtual pointer does NOT reach clients
  under headless cage (installed, tried, dead end). Use the fork's
  control FIFO instead: `echo "click X Y" > /tmp/chumby-ctl`,
  `echo "drag X1 Y1 X2 Y2" > /tmp/chumby-ctl` (window pixels, match
  grim screenshots; hook H10). Navigation is screen-by-screen — always
  grim + verify before clicking, list scroll positions drift.
- **TFT driver**: the XPT2046-touch panel is an ILI9486 480×320 clone;
  stock `piscreen` overlay works (fbtft `fb_ili9486`, ~31 fps refresh,
  ads7846 touch on `/dev/input/event3`). Persisted in
  `/boot/firmware/config.txt` (backup: `config.txt.bak`):

  ```
  dtparam=spi=on
  dtoverlay=piscreen,speed=24000000,rotate=90
  ```

  Caveat for the kiosk step: fbtft is **fbdev-only, no DRM device** —
  cage/wlroots cannot output to it. Options (undecided, next session):
  DRM tiny-driver route vs. a frame-mirror from the headless output.
- Audio state for reference: USB sink (UACDemoV1.0) is default, hardware
  volume set 35 %; panel volume slider live-controls mpv (persists to
  `psp/volume`); `alarm_volume` stays a separate fixture value.

## Hot-replace deploy, 2026-07-07 (the _setTimeZone fix build)

The Pi (3B+, kiosk debs 0.1.2) was at `192.168.42.30` (DHCP — was
.210.159 on the old network). Goal: test the `_setTimeZone` fix (fork
`6c99ebd1f`) without rebuilding debs — replace the installed files in
place ("hot-replace all data", user request). dpkg still shows 0.1.2;
the on-disk files now diverge from the package until the next deb
install. Procedure (repeatable):

```sh
# build (dev box, monorepo ruffle/ at the fork commit) — doc 08 §4
cd ruffle && cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu

# transfer while the kiosk keeps running
scp ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop \
    pi@192.168.42.30:/tmp/ruffle_desktop.new
rsync -a --delete --rsync-path="sudo rsync" fixtures/ \
    pi@192.168.42.30:/usr/share/chumby-player/fixtures/

# swap + full state reset (loses persisted volume/alarms — intended)
ssh pi@192.168.42.30 '
  sudo systemctl stop chumby-player &&
  sudo install -m 755 -o root -g root /tmp/ruffle_desktop.new \
      /usr/lib/chumby-player/ruffle_desktop &&
  sudo rm -rf /var/lib/chumby/fixtures &&   # launcher re-seeds on start
  sudo systemctl start chumby-player'
```

controlpanel.swf was NOT replaced (sha256 identical to
`swf-assets/controlpanel.swf`). Result: service `active`, panel
running (~99 % CPU, the known ~103 % baseline), state dir re-seeded
(`rootfs/psp/timezone` = `Europe/Oslo`), runtime files
(`channel_names`, `controlpanelversion`) written by the SWF, no
panics in the journal. Binary sha256 verified equal on both ends
(`1e0ea2ae…`).

## Hot-replace deploy, 2026-07-07 (the UI-policy build)

Same repeatable procedure as the `_setTimeZone` deploy above, this time
for the UI-policy build (fork `adeb6058d`, submodule `ruffle/` already
at that commit; the dist binary was already built — no rebuild). Pi
still at `192.168.42.30`, kiosk debs 0.1.2 (dpkg unchanged; on-disk
files now ahead of the package again).

- New binary sha256 `8d0500dc…` (29 610 848 B), verified equal on the
  dev box and at `/usr/lib/chumby-player/ruffle_desktop` after
  `install`. Previous on-disk binary was the `_setTimeZone` build
  (`1e0ea2ae…`).
- `fixtures/` rsynced to `/usr/share/chumby-player/fixtures/` — this
  carried the new `fixtures/ui-policy.toml` (3 UI1 rules). Runtime tree
  `/var/lib/chumby/fixtures` wiped so the launcher re-seeds it with the
  policy file (confirmed present after restart).
- controlpanel.swf NOT touched. Service `active`, ruffle ~97 % CPU
  (known baseline), no panics/errors in the journal.

Display note (correcting a stale reading of §7 above): the panel now
outputs to the SPI TFT over the **DRM tiny driver** decided + verified
2026-07-06 — `card0-SPI-1` connected, HDMI disconnected, service env
`WLR_BACKENDS=drm,libinput`,
`WLR_DRM_DEVICES=/dev/dri/by-path/platform-3f204000.spi-cs-0-card`,
`WLR_RENDERER=pixman` (see `10-tft-display.md` §A and
`12-kiosk-packaging.md`). `grim` screenshots work against the cage
session (`XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-0`).

On-device verification: Jan navigated to Settings → TIME/DATE and
confirmed the NTP checkbox/label and the SET TIME ZONE globe render
disabled (2026-07-07). This closes the last open item of the UI-policy
milestone (fork CI `Build and start controlpanel.swf` was already green
on `adeb6058d`).

## Hot-replace deploy, 2026-07-08 (the I1 Licenses build)

Same repeatable procedure as above, for the Info/Licenses milestone
increment I1 (fork `86ae7f86a` = `36d4aa51a` + the `file://`→rootfs
interception in `chumby/navigator.rs`). Pi still `192.168.42.30`, kiosk
debs 0.1.2 (dpkg unchanged; on-disk files ahead of the package again).

- Rebuilt the aarch64 dist binary (`--profile dist … --target
  aarch64-unknown-linux-gnu --manifest-path ruffle/Cargo.toml`, 9m07s,
  one harmless upstream neon `inline_always` warning). New binary sha256
  `6d0cc800a34ae6bc3757927b5ec1cd6568c19e6be006b55e834bb55036e4b229`
  (29 610 960 B), verified equal on the dev box and at
  `/usr/lib/chumby-player/ruffle_desktop` after `install`. (Stale-binary
  gotcha respected: the prior on-disk binary was the UI-policy build; a
  fresh build was mandatory for the navigator change to take effect.)
- `fixtures/` rsynced to `/usr/share/chumby-player/fixtures/`, carrying
  the new `fixtures/rootfs/LICENSES/{gpl.txt,lgpl.txt,README}` and the
  updated `fixtures/ui-policy.toml` (the `settings-info` rule dropped, so
  the Settings → CHUMBY INFO icon is re-enabled — the gateway to E6/E7).
  Runtime tree `/var/lib/chumby/fixtures` wiped so the launcher re-seeds;
  confirmed after restart: LICENSES present (17617/25885/114 B, matching
  the backup) and `settings-info` absent from the seeded policy.
- controlpanel.swf NOT touched. Service `active`, panel running (pid),
  no panics/errors in the journal (only cage's benign `xkbcomp … not
  fatal`).

On-device verification: the panel main bar (B2) auto-hides on a short
inactivity timeout, so scripted `bend`→`click` over SSH races it — do
the summon+click in one command (once inside Settings, which does not
auto-hide, further clicks are unhurried). Visual confirmation of the
Licenses viewer on the TFT was left to Jan (manual, next session). The
desktop run had already proven the render + `file:// rootfs HIT` for
gpl/lgpl (doc 20, I1).

## Hot-replace deploy, 2026-07-08 (the I3 network build)

Same procedure, for I3 (real network diagnostics + blue ethernet bar,
fork `b9199a1f6`). No launcher change needed — `RealNetHost` is always
active now (the `--chumby-real-net` flag was dropped), reading the live
interface and falling back to the fixture when nothing is connected.

- aarch64 dist binary sha256 `78d40a7f…`, verified equal on the dev box
  and at `/usr/lib/chumby-player/ruffle_desktop` after `install`.
- `fixtures/` rsynced (carried the `wired-eth-bar` tint rule + the
  `type=lan` network fixture); `/var/lib/chumby/fixtures` wiped so the
  launcher re-seeds. controlpanel.swf untouched. Service `active`, panel
  running, no panics/errors in the journal.
- The device is wired `eth0` = `192.168.42.30/24`, gw `192.168.42.1`,
  dns `192.168.42.1`, MAC `b8:27:eb:93:08:b1` — so the Info screen should
  show `type: Ethernet` + those values, and the main-bar meter should
  render blue. Visual confirmation on the TFT left to Jan (manual).
