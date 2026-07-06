# 10 — Panel-on-TFT display route (ILI9486 480×320 SPI, XPT2046 touch)

Investigation date: 2026-07-06 (read-only survey over SSH, Pi 3B+ at
192.168.210.137, kernel `6.18.34+rpt-rpi-v8`). Decision context: the
cage kiosk (M3 packaging step) must know which output it drives.

## Problem recap

The TFT (ILI9486 clone, `piscreen` overlay, SPI0 at 24 MHz, rotate=90)
currently binds the **fbtft staging driver** `fb_ili9486` → `/dev/fb0`
only. fbdev is invisible to KMS/Wayland: cage/wlroots cannot output to
it. The panel therefore ran under *headless* cage for CHECKPOINT 5.

## Current state (verified on device)

| Item | Finding |
|---|---|
| DT node compatible | `ilitek,ili9486` (+ `ti,ads7846` for touch) — binds fbtft |
| `/dev/fb0` | owned by `fb_ili9486` (fbtft) |
| `/dev/dri/card0` | vc4 (connectors: HDMI-A-1, Writeback-1) |
| Touch | ads7846 bound, `/dev/input/event3`, physically **untested** |
| config.txt | `dtoverlay=piscreen,speed=24000000,rotate=90` |

## Key findings

1. **The kernel already ships the mainline DRM tiny driver**:
   `/lib/modules/6.18.34+rpt-rpi-v8/kernel/drivers/gpu/drm/tiny/ili9486.ko.xz`,
   binding compatibles `ozzmaker,piscreen` and `waveshare,rpi-lcd-35`.
   (Also present as a generic fallback: `panel-mipi-dbi.ko` +
   `mipi-dbi-spi.dtbo`, which needs an init-sequence firmware blob —
   only relevant if the clone's init table differs.)

2. **The stock `piscreen` overlay has a built-in `drm` parameter.**
   Decompiling `/boot/firmware/overlays/piscreen.dtbo` shows an
   `__overrides__` entry `drm` that rewrites
   `compatible=waveshare,rpi-lcd-35` and patches the `reset-gpios`
   flags cell to 0 (active-high, the polarity the DRM driver expects;
   fbtft used active-low). The `rotate` parameter writes **both** the
   fbtft `rotate` and the DRM `rotation` properties, so `rotate=90`
   carries over unchanged.

   ⇒ Switching to the DRM driver is a **one-parameter change**, no
   custom overlay:
   `dtoverlay=piscreen,speed=24000000,rotate=90,drm`
   It can even be tried at runtime, reversibly, without a reboot:
   `sudo dtoverlay -r piscreen && sudo dtoverlay piscreen speed=24000000 rotate=90 drm`

3. **Throughput ceiling**: 480×320×16 bit = 300 KiB/frame; at
   24 Mbit/s SPI that is ≈9.7 full frames/s. The DRM driver flushes
   dirty rectangles only, so the mostly-static control panel should
   feel fine; full-screen animation will be chunky (identical ceiling
   under fbtft — this is the bus, not the driver).

## Options

### A. DRM tiny driver (recommended)
Add `,drm` to the overlay line → TFT appears as a second DRM card
(e.g. `/dev/dri/card1`, driver `ili9486`); cage outputs to it directly
(`WLR_DRM_DEVICES=/dev/dri/card1`, likely `WLR_RENDERER=pixman` since
the tiny card has no GPU).

- Pros: zero custom code; mainline driver; kiosk unit is a normal
  cage-on-DRM unit; touch/rotation handled by the overlay.
- Open questions (answerable in one ~15-min reversible experiment):
  1. Does cage come up on the tiny card (pixman renderer)?
  2. How does `ruffle_desktop` (wgpu) render when the compositor
     advertises no dmabuf — llvmpipe fallback CPU cost on the 3B+?
     (CHECKPOINT-5 CPU numbers were taken under headless cage with the
     vc4 render node available.)
  3. Physical image correct (rotation, BGR order)? Needs eyes on the
     device.
  4. Touch works end-to-end (first physical test of ads7846).

### B. Frame-mirror from headless cage
Keep fbtft; keep the proven headless-cage setup; add a mirror process
(wlr-screencopy client writing to `/dev/fb0` — no off-the-shelf tool
found, would be a small custom program; a `grim` loop is too slow).

- Pros: ruffle's rendering path stays exactly as verified at
  CHECKPOINT 5.
- Cons: custom software to write and maintain; continuous copy burns
  CPU on a 3B+; two moving parts in the kiosk unit; vc4's
  Writeback-1 connector does not help — it writes to memory, not to
  the SPI panel, so a copier process is needed regardless.

## Recommendation

Try **Option A** first: it is a stock-overlay parameter, reversible at
runtime, and if the experiment answers 1–2 acceptably it deletes an
entire custom component from the kiosk design. Fall back to B only if
cage or wgpu cannot produce acceptable output on the tiny card.

Experiment plan (needs user approval — changes live device state):
1. `sudo dtoverlay -r piscreen; sudo dtoverlay piscreen speed=24000000 rotate=90 drm`
2. Verify `ili9486` DRM card + ads7846 still bound; check dmesg.
3. `WLR_DRM_DEVICES=/dev/dri/cardN WLR_RENDERER=pixman cage -- run.sh`,
   grim screenshot + user confirms image on the physical TFT.
4. User taps the screen; verify events land (evtest) and the panel
   reacts.
5. If good: make `,drm` permanent in config.txt; record renderer env
   in the kiosk unit design. If not: revert (`dtoverlay -r piscreen`,
   re-add without `drm`) and design the mirror (Option B).

## Status

**DECIDED 2026-07-06: Option A (DRM tiny driver), user-verified on the
device.** Experiment results:

1. Runtime overlay swap worked; `/dev/dri/card1` (driver `ili9486`,
   connector `SPI-1`) appeared, ads7846 re-bound.
2. **Rotation gotcha**: DRM's base orientation is landscape — fbtft
   `rotate=90` ≙ DRM `rotate=0` (`rotate=90` gave 320×480 portrait).
3. cage runs on the tiny card with `WLR_DRM_DEVICES=/dev/dri/card1
   WLR_RENDERER=pixman` (needs a seat: as root + builtin libseat for
   the experiment; SSH sessions can't take seat0 — kiosk unit will use
   a logind seat via `TTYPath`/`PAMName=login`). Image on the physical
   TFT confirmed correct by the user.
4. Ruffle CPU ~275 % at 480×320 — below the ~315 % headless-800×600
   baseline. It was already software-rendering; no pixman penalty.
5. **Touch**: hardware fine; upstream Ruffle ignores Wayland touch →
   hook H11 added (touch → mouse events + ≥1 s stationary hold →
   `tap_bend()`). Axis calibration from four physical taps:
   screen-x = device-y, screen-y = inverted device-x ⇒ overlay flags
   `swapxy=on` (**inverted-boolean**: `on` *deletes*
   `touchscreen-swapped-x-y`) + `invy=on` (adds
   `touchscreen-inverted-y`). Verified: taps land under the finger.
6. Audio via the user session's PipeWire as before (root experiment
   bridged with `PIPEWIRE_RUNTIME_DIR=/run/user/1000`; not needed once
   the kiosk runs as `pi`).

Permanent config.txt line (old fbtft line kept commented, backup
`config.txt.bak-tft`; **not yet reboot-tested**):

```
dtoverlay=piscreen,speed=24000000,rotate=0,drm,swapxy=on,invy=on
```
