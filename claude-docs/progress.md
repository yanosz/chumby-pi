# Progress log (M2, M3)

Newest first. Per plan step 2.2.5: every newly reached screen gets a
screenshot here. Ruffle fork branch `chumby`; run command in
`fixtures/README.md`.

## 2026-07-06 (late) — Step 3.5: CPU halved, mouse cursor gone (0.1.2)

Both backlog items closed same-day (11-perf-and-input-cleanup.md):

- **CPU ~215 % → ~103 %** idle on the clock. The load was never the
  SWF (12 fps, 480×320): ~80 % was lavapipe's four raster worker
  threads. `LP_NUM_THREADS=1` (launcher default) halves total CPU;
  the debs now also ship the `dist` (fat-LTO) build like upstream.
  Frame pacing checked and clean — no busy loop; ~1 core is the floor
  for the software-Vulkan renderer at this workload.
- **Cursor**: the seat's only "pointer" was the vc4-hdmi **CEC** input
  device — a phantom pointer on a mouseless kiosk. One packaged udev
  rule ignores it; a real USB mouse still brings the cursor back. The
  client-side attempt (H12 hook) did not work and was reverted —
  recorded as a dead end. User-verified: cursor gone, touch fine.

## 2026-07-06 (evening) — Player mode: debs installed, boot-to-panel works

Step 3.3 end state reached (12-kiosk-packaging.md). The two
CHECKPOINT-4-approved debs (`chumby-player` 0.1.0 arm64,
`chumby-player-data` 0.1.0 all, built by `pkg/build-debs.sh`) are
installed on the Pi; `chumby-player.service` runs cage+ruffle as `pi`
with a real logind seat on tty1 (no root, no PipeWire bridge). The
permanent TFT overlay line passed its reboot test — with one catch:
**DRM card numbers flipped across the reboot** (TFT card1 → card0), so
the unit now targets `/dev/dri/by-path/platform-3f204000.spi-cs-0-card`.
Panel state lives in `/var/lib/chumby/fixtures` (seeded on first run
because the panel writes into `fixtures/rootfs`).

Verified: cold boot → clock on the TFT untouched (~40 s), tap + ≥1 s
long-press summon work (user, at the device), ruffle ~202 % CPU.

![boot-to-panel](reference/images/m3-kiosk-boot.png)

Noted for 3.4: no `/sys/class/backlight` device under the DRM driver —
backlight control path TBD. Old `/home/pi/chumby` superseded, deletion
left to Jan.

## 2026-07-06 — Panel on the TFT: DRM route works, touch live, long-press bend

The panel-on-TFT decision (deferred at CHECKPOINT 5) is resolved:
**DRM tiny driver** (10-tft-display.md Option A), user-verified on the
physical screen. No frame-mirror component needed.

- **Display**: the stock `piscreen` overlay's `drm` parameter binds the
  mainline `drm/tiny/ili9486` driver instead of fbtft →
  `/dev/dri/card1`, 480×320 landscape at `rotate=0` (DRM's base
  orientation differs from fbtft's: fbtft `rotate=90` ≙ DRM `rotate=0`).
  cage runs on it with `WLR_DRM_DEVICES=/dev/dri/card1
  WLR_RENDERER=pixman`. Ruffle CPU ~275 % — *below* the ~315 % headless
  baseline (it was already software-rendering; pixman changes nothing
  for the client).
- **Touch (first physical test)**: ADS7846 hardware fine, but upstream
  ruffle_desktop drops `WindowEvent::Touch` — Wayland touch is not a
  pointer. New hook **H11** maps single-touch to left-button mouse
  events. Axes calibrated from four user taps: overlay flags
  `swapxy=on` (removes `touchscreen-swapped-x-y` — inverted-boolean
  param!) + `invy=on`. Taps land under the finger, slider drag works.
- **Bend summon by long-press** (user decision): stationary ≥1 s touch
  fires `tap_bend()` (H11 cont.) — same path as `chumby-ctl bend`;
  toggles the control panel like a real squeeze.
- **Audio**: unchanged; streams verified (Groove Salad by ear). The
  test ran cage as root, bridged via `PIPEWIRE_RUNTIME_DIR=
  /run/user/1000` — stopgap only; the kiosk unit runs cage as `pi`
  with a logind seat and needs no bridge.
- config.txt now carries the DRM overlay line permanently (old fbtft
  line kept commented; backup at config.txt.bak-tft). Boot-time
  behavior not yet reboot-tested.

![control panel on TFT](reference/images/m3-tft-panel.png)

## 2026-07-06 — CHECKPOINT 5 sound verification passed (final 3B+)

All three ear-tests confirmed by the user at the device (USB speaker):
**alarm ring-through (B6)** — pre-armed fixture alarm fired on schedule,
rang at full alarm volume, dismissable; **My Streams playback (C2)** —
SomaFM Groove Salad streamed live (first real-network audio, per the
CHECKPOINT-4 decision); **volume slider (E1)** — dragging the knob sends
`_setSystemVolume` per glide step and mpv volume follows live over IPC.

![alarm ringing](reference/images/m3-alarm-ringing.png)
![streams playing](reference/images/m3-streams-playing.png)

Setup: SD card moved unchanged to the final Pi 3B+ (new IP; 09 §7).
lightdm disabled, panel now runs under **headless cage** (TFT driverless
at session start, HDMI unplugged); driven remotely via the control FIFO
and verified via grim screenshots. The TFT turned out to be an ILI9486
480×320 clone — `piscreen` overlay works, screen shows a test pattern,
touch device present (09 §7; panel-on-TFT is a next-session decision:
fbtft has no DRM device for cage).

Bugs found only by real-hardware testing, all fixed and re-verified:

1. **Alarm fixture `time` is minutes, not seconds** since midnight — the
   seconds value parsed silently and never fired (fixtures/README.md).
2. **mpv IPC-socket race** (`chumby/audio.rs`): on the loaded Pi, mpv
   needs >1 s to create its control socket; the one-shot wait gave up and
   dropped every later volume command, so the alarm rang **silently**
   (fade-in stuck at spawn volume 0). Fix: lazy reconnect in `send_ipc`.
3. **Audio state constants** (`chumby/fixture.rs`): we returned
   Playing=1, but the SWF's PLAYING is 2 (1 = WAITING) — the stream
   watchdog (`DirectURLPlayer.THRESHOLD`) killed every stream after 5 s.
4. **Zombie mpv** on stop (kill without reap) — `stop_internal` waits now.
5. **Stream fixture needs `mimetype`** — without the attribute,
   `playStream()` is a silent no-op (fixtures/README.md).

New capability (hook H10 + `chumby/input.rs`): control-channel commands
`click X Y` and `drag X1 Y1 X2 Y2` inject pointer events through the
frontend event loop, one per tick — this is how all screens were driven
with no input device attached (wlrctl's virtual pointer never reaches
clients under headless cage). Also new on the Pi: `pipewire-alsa`
(Ruffle/cpal had no audio device without it).

**Next (user, new session):** the CHECKPOINT-4-approved end state — two
debs (chumby-player + chumby-player-data), cage+systemd kiosk — plus the
panel-on-TFT display route decision, then 3.4 brightness/night mode.

## 2026-07-02 — M3 step 3.3: first run on the Raspberry Pi

The control panel runs on the interim Pi 3A+ (`ssh pi@192.168.210.159`,
deploy layout in `reference/09-pi-deploy.md`): cross-built aarch64 binary
(reference/08), stock Raspberry Pi OS, inside the existing labwc Wayland
session (windowed 640×480, wgpu on **lavapipe** software Vulkan as
predicted in 07 §3).

- Boot → normal operation → clock widget rendering: **works**
  (![pi clock](reference/images/m3-pi-clock.png)).
- Bend summon via `chumby-ctl bend` over SSH → full B2 control-panel bar
  incl. signal-strength polling: **works**
  (![pi panel](reference/images/m3-pi-panel-bar.png)).
- Zero errors in the run log after the fixture-path fix (see below).
- Load: ~220 % CPU of 400 % at 640×480, ~115 MB RSS of 415 MB. Fine for
  the smoke test; output resolution is the lever for the kiosk config.

Found & fixed: the channel profile fixture hardcoded the dev-box fixtures
path — widgets failed to load on the Pi (first screenshot was a black
widget area). Fixture bodies now support a `{FIXTURES}` token expanded by
FixtureHost at read time (`fixtures/README.md`). Audio: PipeWire already
defaults to the USB card; mpv can additionally be pinned via
`CHUMBY_AUDIO_DEVICE`.

Untested (needs ears/hands on the device — CHECKPOINT 5): alarm
ring-through (B6), My Streams real playback, volume slider vs USB card.

Addendum (same evening, user request): launcher switched to
`--fullscreen` — panel now covers the screen, menu bar gone. CPU
measurements and the `pkill -x` (not `-f`!) operational gotcha in
09 §2/§5: ~345 % of 400 % at native mode, and neither output mode nor
`--quality` changes that materially; still responsive; brushed the soft
thermal limit once. True kiosk (cage, no desktop) is the next step after
CHECKPOINT 5 feedback.

## 2026-07-02 — Milestone 2 accepted; M3 (Pi deployment) defined

Step 2.3 acceptance met: documented run command reaches normal operation;
`fixtures/README.md` and `docs/patch-notes.md` audited current (upstream
diff = 9 cfg-gated files, feature-off `cargo check` clean). User decision:
alarm ring-through and My Streams playback stay untested on the desktop —
they will be exercised first on real hardware. Milestone 3 (Raspberry Pi
deployment) is defined in the plan; waiting for SSH access to the Pi.

## 2026-07-02 — 2.2.6 verified: alarms (B5), My Streams (C0/C2), volume (E0/E1) reached

![music source list](reference/images/m2-music-c0.png)
![my streams](reference/images/m2-streams-c2.png)
![settings menu](reference/images/m2-settings-e0.png)
![volume panel](reference/images/m2-volume-e1.png)
![alarms panel](reference/images/m2-alarms-b5.png)

All five screens captured by one unattended `./verify-screens.sh all` run.
The fixtures verify visibly: E1's volume slider sits at 60 % (`/psp/volume`
default) with the touch-click checkbox unchecked (`/psp/touchclick` = 0);
C2 lists the SomaFM Groove Salad entry from `/psp/url_streams`; B5 shows
Quick/Custom Alarm and "Next alarm: None". Stream playback was NOT tested —
it would hit the real network (SomaFM), which per the M2 network posture
needs a user decision first.

Getting here killed two silent xdotool failure modes (the previous session's
"alarms" screenshots actually showed B2 — no scripted click had ever landed):

- **winit ignores XSendEvent input**, so `xdotool click --window` never
  reaches Ruffle; and `getwindowgeometry` can report frame-relative
  coordinates under a reparenting WM, so absolute XTEST clicks can miss the
  window entirely. Working combination: `mousemove --window` (real pointer
  warp) + plain XTEST `click 1`, confirmed against the `chumby_pick=debug`
  trace (hook H9 paying off again).
- **Bend presses are momentary and lost** if sent before the SWF restarts
  bend polling after a panel closes (`startBendSensor`); the script now
  confirms each bend via the `pressBendSensor` avm_trace line and retries.

Also new since the last commit: consecutive-duplicate suppression for the
`chumby_host` ASnative log (high-frequency polls like `_bent` collapse into
"× N" lines) and an mpv integration test in `chumby/audio.rs` (skips itself
if mpv or fixtures are absent). The test caught a real margin problem: mpv
needs ~150 ms to create its IPC socket and `play()` only waited 200 ms
before silently degrading to spawn-time volume (no live volume control —
would bite harder on the slower Pi). Wait raised to 1 s (`SOCKET_WAIT`);
the normal path still connects as soon as mpv is ready.

## 2026-06-13 — Step 2.2.6: mpv audio backend; alarms (B5/B6), My Streams (C2), volume (E1)

New module `core/src/chumby/audio.rs`: mpv-backed AudioPlayer with Unix-socket
IPC. `_playAudio` spawns mpv; `_setSystemVolume` sends a live `set_property
volume N` IPC command while audio is playing. Graceful degradation: if mpv is
not installed the state machine still responds correctly (silent-stub mode, same
as CHECKPOINT 3 decision). Volume persists across restarts via `/psp/volume`.

Fixture additions: 7 alarm-tone MP3s copied from the device backup into
`fixtures/rootfs/usr/chumby/alarmtones/` (the SWF does `_fileExists` before
calling `_playAudio` — files must be present); SomaFM Groove Salad added to
`url_streams`; `/psp/volume` (default 60) and `/psp/touchclick` (default 0)
created; exec manifest gains `rm /psp/ifalarm` prefix for alarm-dismiss path.

Screenshots and verification notes: see the 2026-07-02 entry above.

## 2026-06-13 — Widget clicks unblocked (WidgetPlayer button-mode bug)

User report: the Unsubscribed Clock's buttons (left icon column) worked
standalone but were dead inside the panel. Root cause (found via the new
`chumby_pick=debug` mouse-pick trace, hook H9): `WidgetPlayer.prototype.
onPress` — a click-STATISTICS handler — puts the widget container into AS2
button mode, so the container swallows every click before the widget's own
buttons see it. On real hardware this never bites (widgets play in a slave
flashplayer with raw touch routed to it); in our localCache in-movie mode
it killed all widget interactivity. This was the foreseen "revisit if a
widget misbehaves" case of the CHECKPOINT 3 localCache decision.

Fix: `chumby/avm.rs` deletes the prototype handler once the panel defines
it (no SWF modification; only stubbed click-stats are lost). Verified:
mini-time icon click inside the panel switches the widget to its LCARS
view; bend still summons B2. NOT a bend-button regression — that work
touches no mouse path, and tap-to-summon was always the container eating
clicks, never a real feature.

## 2026-06-13 — Unsubscribed Clock as default widget; B2 screenshot

![unsubscribed clock](reference/images/m2-unsubscribed-clock-boot.png)
![main button bar](reference/images/m2-main-button-bar-b2.png)

The fixture channel now has two widgets: **Unsubscribed Clock** (default,
instance 1 — the clock chumby.com shows unsubscribed users; downloaded from
the guide, widget GUID F1899D99-EDF7-3F38-A161-E6F3C89499E5, saved as
`fixtures/widgets/unsubscribedclock.swf`) and the previous builtinclock as
instance 2, so prev/next widget navigation has a target. Verified: boot
plays the Unsubscribed Clock; FIFO `bend` summons the button bar ("1 of 2").
This also delivers the B2 screenshot owed by the 2.2.4 entry.

## 2026-06-12 — Bend sensor wired, main button bar (B2) reached (2.2.4)

Neither dev PC nor Pi has a bend button, so the fork grew a simulated-input
control channel (decided with the user over kill signals — signals terminate
non-handling builds, carry no payload, and only two exist):

- type `bend` or `tap` + Enter in the launch terminal (ruffle reads stdin)
- `echo bend > /tmp/chumby-ctl` from any shell (FIFO, `--chumby-control`,
  created by run-controlpanel.sh)
- Home key with the window focused (chumby's own falconwing port used Home)

`bend`/`tap` reads as pressed for one `_bent` poll then released — the SWF
polls every frame, so that yields one onBend/onUnbend pair, which is all the
panel uses (B2 summon, alarm snooze; hold only feeds the dormant BendTapper
and the tilt gesture). `bend down`/`bend up` exist for explicit press/hold.

Verified end-to-end: `echo bend > /tmp/chumby-ctl` → avm_trace shows
pressBendSensor → releaseBendSensor → controlPanelMode → **main button bar
visible**; user navigated the panel interactively. (Screenshot to add on a
future run.) Ruffle commit `81515eca` (hooks H7/H8 in patch-notes). New
module `core/src/chumby/input.rs`.

**Next (user priority 2026-06-13):** alarms (B5/B6) and My Streams (C2) —
their file-I/O substrate (virtual rootfs) is already working: /psp/alarms
and /psp/url_streams writes persist and restore. Then volume (E1).
Deferred within file I/O: _getDirectoryEntry object-filling (5,320) — only
needed when USB/local-files music browsing lands. Reminder: audio is a
silent stub in M2 (CHECKPOINT 3), so alarms ring visibly, not audibly.
(Brightness moved to the Pi milestone, 2026-06-13.)

## 2026-06-12 — Normal operation reached on the real device path (2.2.3)

![clock](reference/images/m2-clock-normal-operation.png)

No FlashVars except `-PlocalCache=1` — the same path the device takes with
the downloaded panel: startup → GUID/DCID/MAC/network (exec fixtures) →
Authorizing (authorize fixture) → date validation → **normal operation**,
clock mode (fixture channel is empty). All chumby HTTP answered in-process;
nothing escapes to the live chumby.com. Remaining noise: none blocking;
FM radio and external music degrade gracefully by design.
Log: `reference/logs/m2-run12.log`. Ruffle commits `0217f5ae` (ASnative
table, 2.2.2) + `53a771a7` (navigator, 2.2.3).

**Next:** tap the screen to summon the control-panel button bar (B2) — needs
an interactive click (xdotool not installed); then volume/brightness/alarms
screens per feature-decisions.

## 2026-06-12 — Step 2.2.2: ASnative table live

First run with `--chumby-fixtures`: settings restore, version backticks,
platform identity all answered; native traffic fully logged under
`chumby_host`. Boot blocked only on `exec://` (navigator hook pending).
