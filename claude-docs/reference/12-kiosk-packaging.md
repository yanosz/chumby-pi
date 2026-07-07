# 12 — Kiosk packaging: two debs + systemd/cage unit

Date: 2026-07-06. This is the CHECKPOINT-4-approved end state of Step
3.3 (survey 07 §4.2/§4.5): the hand-rolled `/home/pi/chumby` smoke-test
deployment (09) replaced by two Debian packages and a boot-to-panel
kiosk unit. Built entirely on the dev box (Pi was offline); numbering
note: 11 is reserved for the Step 3.5 perf/input cleanup doc.

## 1. Packages

Built by `pkg/build-debs.sh` (staging `pkg/build/`, output `pkg/out/`,
both gitignored). Not policy-compliant Debian source packages —
`dpkg-deb --build --root-owner-group` over a staging tree, as decided
in 07 §4.2. `VERSION` env overrides the default 0.1.0.

### chumby-player_0.1.0_arm64.deb (~8.6 MB)

| path | content |
|------|---------|
| `/usr/lib/chumby-player/ruffle_desktop` | cross-built release binary, `--features chumby` (08) |
| `/usr/bin/chumby-player-run` | launcher: seeds state, makes FIFO, execs ruffle fullscreen |
| `/usr/bin/chumby-ctl` | repo-root FIFO trigger script, copied at build time (single source) |
| `/lib/systemd/system/chumby-player.service` | cage kiosk unit (§2) |

`Depends: cage, mpv, pipewire-alsa, chumby-player-data`. Library deps
(libasound2, libudev1, …) are NOT declared — the binary is a private
cross-build already verified on this exact trixie image. postinst runs
`systemctl enable chumby-player` (player mode on next boot, per 07
§4.2 "installing = player mode") but does not start it; prerm disables
and stops on removal.

### chumby-player-data_0.1.0_all.deb (~1.2 MB)

| path | content |
|------|---------|
| `/usr/share/chumby-player/fixtures/` | repo `fixtures/` tree verbatim (exec/, http/, rootfs/, widgets/, README) |
| `/usr/share/chumby-player/swf/controlpanel.swf` | from `/home/jan/chumby_backup/tmp/` (read-only source; build script only reads it) |

**PRIVATE USE ONLY** — controlpanel.swf and the widget SWFs are
copyrighted chumby firmware; never publish these debs in an apt repo
(also stated in the package description).

## 2. Kiosk unit design

`pkg/chumby-player/chumby-player.service` — decisions and why:

- **`User=pi` + `PAMName=login` + `TTYPath=/dev/tty1`**: gives the
  service a real logind session on seat0. This is the fix recorded in
  10 §Status(3): SSH sessions can't take seat0, and the root experiment
  needed `LIBSEAT_BACKEND=builtin` + a PipeWire runtime-dir bridge.
  With a logind seat, cage opens the DRM device as plain `pi` and
  audio lands in pi's own PipeWire — no root, no bridge.
  `Conflicts=getty@tty1.service` frees the VT; TTYReset/TTYVHangup/
  TTYVTDisallocate + UtmpIdentifier are the standard cage-kiosk tty
  hygiene set.
- **`Environment=WLR_BACKENDS=drm,libinput WLR_RENDERER=pixman
  WLR_DRM_DEVICES=/dev/dri/by-path/platform-3f204000.spi-cs-0-card`**:
  the verified TFT config from 10 (ili9486 tiny DRM card, no GPU →
  pixman) — but by its **by-path name**, not `cardN`: the numbering
  drift is real, observed on the very next boot (TFT was card1 on
  2026-07-06 during the experiments, card0 after the reboot).
  `3f204000.spi` is the BCM2837 SPI0 controller — a different Pi model
  changes that address; `EnvironmentFile=-/etc/default/chumby-player`
  is the override point.
- **`StateDirectory=chumby`**: systemd creates `/var/lib/chumby` owned
  by `pi`. The launcher seeds `fixtures/` there from `/usr/share` on
  first run, because **the panel writes into `fixtures/rootfs`** (all
  /psp persistence: alarms, url_streams, volume, settings) — a
  read-only `/usr/share` copy cannot be the live tree. Consequence:
  after a chumby-player-data upgrade, `rm -rf /var/lib/chumby/fixtures`
  re-seeds on next start, discarding panel-made settings. Acceptable
  for now; revisit if fixture updates become frequent.
- **`Restart=on-failure`**: clean exit (panel quit) does not respawn —
  matches "magic key" semantics; crash does.
- Packaged default `RUST_LOG=warn` (launcher): the smoke-test's
  `chumby_host=info,avm_trace=info` logs per-call traffic — wrong
  default for an appliance journal and implicated in the Step 3.5 CPU
  question. Re-enable via `/etc/default/chumby-player` when debugging.
- Exit paths: `sudo systemctl stop chumby-player` (SSH escape hatch,
  07 §4.5). The proposed bend-hold-≥5s / `chumby-ctl exit-player`
  magic key is NOT implemented yet — the FIFO knows only
  bend/tap/click/drag (core/src/chumby/input.rs).

## 3. Install / operate (on the Pi)

```sh
# from the dev box
pkg/build-debs.sh
scp pkg/out/*.deb pi@<pi>:

# on the Pi
sudo apt install ./chumby-player_0.1.0_arm64.deb \
                 ./chumby-player-data_0.1.0_all.deb
sudo systemctl start chumby-player     # or reboot: enabled by postinst
sudo systemctl stop chumby-player      # exit player mode once
sudo systemctl disable --now chumby-player   # leave player mode
chumby-ctl bend                        # summon/dismiss the panel bar
```

Removal: `sudo apt remove chumby-player chumby-player-data` (state in
`/var/lib/chumby` survives; `apt purge` does not remove it either —
StateDirectory contents are not tracked by dpkg; delete manually).

## 3b. Version history

- **0.1.0** (2026-07-06): initial two debs, kiosk unit, first deploy.
- **0.1.1**: binary switched to the `dist` (fat LTO) profile;
  `LP_NUM_THREADS=1` launcher default (doc 11 §1); carried a
  client-side cursor hook (H12) that turned out to be a dead end.
- **0.1.2**: H12 reverted; cursor fixed properly via the packaged udev
  rule `90-chumby-ignore-cec-pointer.rules` (+ postinst udev reload);
  this is the version verified in doc 11 §3.
- **0.2.0** (2026-07-07, BC3): player binary now carries the 146
  upstream commits merged at BC1; library deps declared in Depends
  (libc6, libgcc-s1, libfontconfig1, libssl3t64, libasound2t64,
  libudev1 — from the binary's NEEDED entries, trixie names), fixing
  the §1 "not declared" caveat. Verified by the CI install test
  (claude-docs/reference/15-ci.md).

## 4. Deployment results (2026-07-06 evening, Pi powered on)

All on-device steps done the same day; commands below ran over SSH as
`pi` (documented per the CLAUDE.md Pi-ops rule).

1. **Overlay reboot-test: PASSED.** First boot with the permanent line
   `dtoverlay=piscreen,speed=24000000,rotate=0,drm,swapxy=on,invy=on`:
   `[drm] Initialized ili9486` + fbcon on `ili9486drmfb` (boot console
   visible on the TFT) + ads7846 touch bound. The screen going black
   after boot messages is benign (no getty picture worth debugging —
   `consoleblank=0`, and the kiosk owns the display from then on).
   **Gotcha confirmed: DRM card numbers are not stable across boots**
   — the TFT came up as `card0` this boot (was `card1` during the 10-
   experiments). Unit switched to the by-path name before install (§2).
2. **Install:** `scp pkg/out/*.deb pi@…:` then
   `sudo apt install -y ./chumby-player_0.1.0_arm64.deb
   ./chumby-player-data_0.1.0_all.deb` — clean; postinst enabled the
   unit.
3. **First start** (`sudo systemctl start chumby-player`): PAM/logind
   session opened for `pi` on tty1, cage + ruffle running as `pi`,
   fixtures auto-seeded to `/var/lib/chumby/fixtures`. Widget clock
   rendered at 480×320 (grim screenshot); **user verified at the
   device: image on the physical TFT, tap works, ≥1 s long-press
   summons the panel bar.** Ruffle CPU ~202 % — in line with the 10-
   measurements.
4. **Boot-to-panel reboot test: PASSED.** `sudo reboot` → SSH back in
   ~25 s, `chumby-player` active, clock on the TFT with no manual
   steps: ![boot-to-panel](images/m3-kiosk-boot.png)

Observations for later steps:
- `/sys/class/backlight` is **empty** under the DRM driver — the
  piscreen `led-gpios` (GPIO22) backlight is not exposed as a
  backlight device. Step 3.4 must find the control path (panel driver
  backlight property? gpio-backlight overlay? direct gpioset?).
- Mouse pointer visible mid-screen in screenshots — Step 3.5 item 2,
  unchanged.
- Old hand-rolled `/home/pi/chumby` (37 M) is fully superseded; its
  only state deltas vs the fresh seed were throwaway (disabled test
  alarm, volume 75 vs 60, transient tmp/musicsource). **Not yet
  deleted** (left for Jan): `rm -rf /home/pi/chumby`. The copied debs
  in `/home/pi/*.deb` can go too.

Next: Step 3.4 (brightness/night mode, E2/B4).
