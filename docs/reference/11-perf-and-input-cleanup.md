# 11 — Step 3.5: performance & input cleanup

Date: 2026-07-06. Backlog defined in the plan (added 2026-07-06):
(1) CPU load far above what the content should need; (2) mouse pointer
on a touch-only kiosk. Measurements on the final 3B+ running the
packaged kiosk (12), idle on the clock widget, 480×320 TFT.
Measurement method: `utime+stime` deltas from `/proc/<pid>/stat` over
20 s (HZ=100), all threads of `ruffle_desktop`; spot per-thread via
`top -bH`.

## 1. CPU load

### Where the time actually goes (baseline)

The SWF workload is tiny: `controlpanel.swf` is **12 fps** (SWF header)
at 480×320, authored for a 350 MHz ARM9. Baseline per-thread snapshot
(packaged defaults, RUST_LOG=warn):

| thread | %CPU |
|---|---|
| 4 × `llvmpipe-N` (lavapipe raster workers) | ~170 combined |
| ruffle main (AVM tick, wgpu submit) | ~43 |
| **total** | **~215** |

So ~80 % of the load is **lavapipe** (software Vulkan) rasterization,
not SWF logic, matching 09 §5's "SWF rasterization dominates" but
localizing it to Mesa's worker threads. Temp 59.6 °C with the
`get_throttled` soft-limit sticky bit set — the thermal concern is
real.

### Fix 1: LP_NUM_THREADS (measured, kept)

lavapipe spawns one raster worker per core; for a 480×320/12 fps
stage the parallelization overhead roughly **doubles** total CPU:

| LP_NUM_THREADS | ruffle total CPU |
|---|---|
| unset (4 workers) | ~215 % |
| 2 | ~173 % |
| 1 | **~103 %** |
| 0 (raster in calling thread) | ~103 % |

Set as packaged default in `chumby-player-run` (`LP_NUM_THREADS=1`,
overridable). Rendering verified intact via grim after the change.
`1` chosen over `0` to keep rasterization off the AVM thread (same
total). Pi ops during the experiments: each value was set via
`/etc/default/chumby-player` (the unit's EnvironmentFile) +
`systemctl restart chumby-player`; that file was **removed** again
once the launcher default shipped in 0.1.1, so the device carries no
local override.

### Fix 2: `dist` profile build (measured on-device below)

The repo's `[profile.dist]` = release + fat LTO + `codegen-units=1`
(what upstream ruffle ships); our debs had shipped plain `release`.
`pkg/build-debs.sh` now takes the dist binary
(`cargo build --profile dist -p ruffle_desktop --features chumby
--target aarch64-unknown-linux-gnu`). Only shaves the ruffle-side
threads (lavapipe is system Mesa, unaffected).

### Checked and cleared (no change needed)

- **Frame pacing is not broken**: `about_to_wait` reschedules with
  `ControlFlow::WaitUntil(next_frame_time)` from the SWF frame rate,
  and redraws only fire on `needs_render()` — no busy loop, no
  vsync-rate redraw of static frames. The clock widget animates every
  frame, so ~12 presents/s is inherent while a widget is visible.
- **avm_trace logging**: already off in the packaged default
  (RUST_LOG=warn since 0.1.0); the experiments in 10 that showed
  ~275 % ran with per-call info logging on — part of why the packaged
  baseline (215 %) already sits lower.
- **MSAA**: `--quality low` (sample count 1) had shown no measurable
  change (09 §5), so MSAA is not the lavapipe cost driver.

### Not pursued (recorded per plan: things that would NOT help / out of scope)

- Rebuilding Mesa or swapping lavapipe for a GL/llvmpipe path: same
  llvmpipe rasterizer underneath wgpu's GL backend; large effort, no
  expected win.
- A CPU-native 2D renderer for ruffle (skia/tiny-skia backend) does
  not exist upstream; writing one is out of scope for this milestone.

## 2. Mouse pointer on the touch-only kiosk

cage/wlroots parks an arrow cursor at output center (visible in every
screenshot); touch input never moves it. cage 0.2 has no hide-cursor
option.

**Dead end first (recorded per plan):** a client-side fix — fork hook
"H12", `set_cursor_visible(false)` on the ruffle window, gated by a
launcher check for USB mice — was implemented, deployed (0.1.1), and
did nothing: with no pointer device entering the window, the client
never owns a cursor surface; the arrow is the **server-drawn** seat
cursor. Reverted (patch-notes note under H11).

**Root cause & fix:** `libinput list-devices` showed the seat's only
pointer capability comes from **`vc4-hdmi` — the HDMI-CEC input
device** — on a system with no mouse at all. That phantom pointer is
why wlroots draws a cursor. Fix: a udev rule shipped in the deb
(`/usr/lib/udev/rules.d/90-chumby-ignore-cec-pointer.rules`):

```
SUBSYSTEM=="input", KERNEL=="event*", ATTRS{name}=="vc4-hdmi", ENV{LIBINPUT_IGNORE_DEVICE}="1"
```

Verified via grim: cursor gone. Bonus: the plan's "show it if a real
pointer is plugged in" comes for free — a USB mouse gives the seat a
genuine pointer and wlroots draws the cursor again. Nothing lost: the
CEC device only carries HDMI remote-control keys, unused here.

Pi ops trail: the rule was first tested at runtime as
`/etc/udev/rules.d/90-chumby-ignore-cec-pointer.rules` +
`udevadm control --reload && udevadm trigger /dev/input/event*` +
service restart; after the 0.1.2 deb (which ships it in
`/usr/lib/udev/rules.d/` and reloads udev in postinst) the `/etc` copy
was **removed** — the packaged rule is the only one on the device.

## 3. Results (deployed as chumby-player 0.1.2)

| | before (0.1.0) | after (0.1.2) |
|---|---|---|
| ruffle total CPU, idle clock | ~215 % | **~103 %** |
| mouse pointer on panel | yes | gone (udev; returns with real mouse) |

User-verified at the device (2026-07-06 evening): cursor gone on the
TFT, touch (incl. widget navigation) works.

The two shipped CPU levers: `LP_NUM_THREADS=1` (launcher default; the
big one) and the `dist` (LTO) build profile. Remaining CPU is lavapipe
rasterizing 12 fps × 480×320 through a full software-Vulkan pipeline —
~1 core is the floor for the current renderer architecture; further
reduction would need a CPU-native 2D backend in ruffle (out of scope).
Thermals: still warm (idle ~60 °C with the soft-limit sticky bit from
pre-fix uptime); the heatsink recommendation from 09 §5 stands, but
halving CPU should help — re-check `get_throttled` after a fresh boot
and some hours of running.
