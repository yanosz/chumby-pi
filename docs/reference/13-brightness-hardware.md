# 13 — Step 3.4 brightness: hardware finding & display alternatives

Date: 2026-07-06. Step 3.4 (E2 brightness, B4 night mode) hit a
hardware blocker on the current TFT; this doc records the evidence and
the researched alternatives. User decisions 2026-07-06: night mode's
dark rendering is sufficient as-is — **no software-dimming feature
wanted**; instead find a display with real backlight control.

## 1. Finding: backlight is hardwired on the current clone panel

The panel side was never in doubt (contract 03): the SWF writes
`slider×655.35` (0–65535) to `/proc/sys/sense1/brightness` via
`_putFile`, and night mode uses `_setLCDMute` (5,20). The blocker is
the board.

Evidence, collected over SSH (all reversible, GPIO experiments only):

1. `/sys/class/backlight` is empty under the DRM tiny driver (the
   `piscreen` overlay's `led-gpios` is an fbtft-ism; the ili9486 DRM
   driver wants a `backlight` phandle it doesn't have).
2. The overlay does declare `led-gpios` = GPIO22 active-high
   (`dtc -I dtb -O dts piscreen.dtbo`), and `gpioinfo` shows GPIO22
   unclaimed (input) while the backlight is at full brightness — so
   nothing drives it, yet it is lit.
3. Empirical: `timeout 3 gpioset -c gpiochip0 22=0` (and `22=1`) —
   user watching the panel both times: **no effect**.

Conclusion: on this ILI9486 clone the LED rail is tied directly to
3.3 V; GPIO22 is not routed. This matches the widely documented
Waveshare-3.5-style design (backlight anode on the AMS1117 3.3 V rail;
dimming requires cutting a trace and soldering in an NPN transistor —
see raspberrypi.com forum t=149887). No software path exists.

## 2. Alternatives researched (similar form factor, real dimming)

Requirements: ~3.5" SPI HAT on the 2×20 header, touch, 480×320-ish,
**PWM-dimmable backlight**, and mainline/DRM driver support so the
kiosk stack (12) carries over unchanged.

### Recommended: Adafruit PiTFT Plus 3.5" (product 2441)

- 480×320 HX8357D + STMPE610 resistive touch — same size/resolution
  class as the current panel; HAT form factor.
- **Real backlight control, two documented paths**
  (learn.adafruit.com PiTFT 3.5 guide):
  - STMPE610's spare GPIO drives the backlight boost converter →
    appears as `/sys/class/backlight/soc:backlight` (on/off);
  - **GPIO18 = hardware PWM0** → smooth dimming (Adafruit documents
    1 kHz PWM, 0–1023). For us: `dtoverlay=pwm` + PiHost writing the
    pwm sysfs (or a small custom overlay adding a `pwm-backlight`
    node for a proper `/sys/class/backlight` device with levels).
- **Everything is already on our Pi** (verified on-device 2026-07-06):
  `pitft35-resistive.dtbo` with a **`drm` parameter** (hx8357d DRM
  tiny driver — same architecture as today's piscreen route),
  `hx8357d.ko`, `stmpe-ts.ko`, `pwm.dtbo`. Expected changes: one
  config.txt line, by-path DRM name update (SPI CS stays spi0.0 →
  likely unchanged), touch axis calibration, backlight plumbing.
- Caveats: GPIO18/PWM0 is shared with the Pi's analog audio (we use
  USB audio; disable onboard audio if they fight). ~US$45, stocked by
  Adafruit/RS/Newark/Jameco (RS delivers to Norway).

### Rejected / fallback options

- **Waveshare 3.5" (A)/(B)**: same hardwired-backlight design as the
  current clone (trace-cut + transistor mod required) — buying one
  changes nothing.
- **Waveshare 3.5" (C)** (corrected 2026-07-06 after parsing the wiki
  with curl — WebFetch got 403): unlike (A)/(B) it DOES have dimming
  hardware on board, disconnected by default — bridge a solder pad
  ("0R resistor or solder"), then GPIO18 hardware PWM dims it (their
  instructions use the obsolete wiringPi `gpio` tool; on trixie it
  would be the `pwm` overlay + sysfs, same as the PiTFT plan). 480×320
  resistive, 125 MHz SPI, cheaper than the PiTFT. BUT: its official
  driver route is the legacy LCD-show/FBCP stack; it is NOT the
  piscreen/ili9486 wiring, and no mainline DRM tiny driver is known to
  support the (C)'s high-speed mode — driver risk lands exactly on the
  part of the stack that took the 10-tft-display.md experiments to get
  right. Only worth it if the price difference matters and a
  fbtft/`panel-mipi-dbi` detour is acceptable.
- **Check the current clone for the same pad**: since the (C) design
  ships dimming hardware behind an unpopulated bridge, the no-name
  clone may too — worth flipping the board and looking for an
  unpopulated 2-pad jumper near the backlight/LED circuit before
  buying anything.
- **Hardware-mod the current clone** (cut 3.3 V trace at the LED
  anode, NPN transistor + 10 k from GPIO18): zero cost, proven mod,
  but soldering on the only panel we have; keep as fallback.
- **Adafruit PiTFT 2.8" capacitive (2423)**: 320×240 = pixel-perfect
  Chumby Classic resolution (no scaling, likely less CPU) and PWM
  backlight — but 2.8" is noticeably smaller than the chumby's 3.5".
  Listed in case size matters less than fidelity.
- **DPI panels (HyperPixel 4 etc.)**: real vc4 DRM plane + PWM
  backlight, but DPI consumes the whole GPIO header — kills the
  planned GPIO17 bend button (07 §4.4) and SPI. Out.

## 3. Status

**Step 3.4 MOVED TO THE VERY END OF THE PLAN (user, 2026-07-06)** —
the current clone PCB has no dimming pad (user inspected the board),
so brightness control waits for new hardware. A dimmable panel will be
ordered but **which model is still open** — §2 above is the decision
input (PiTFT 2441 = drop-in driver path; Waveshare (C) = cheaper +
solder-pad bridge but expect fbtft-or-`panel-mipi-dbi` driver work).
The plan now carries this as its final milestone.

When the new panel arrives: bridge the pad, swap the overlay line,
recalibrate touch axes, wire the backlight into `PiHost` (map the
0–65535 `/proc/sys/sense1/brightness` writes + `_setLCDMute` onto the
real backlight; fixture behavior stays for desktop), then B4 night
mode on-device test. Until then, next work item is Step 3.5
(11-perf-and-input-cleanup.md).
