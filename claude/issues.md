# Open issues

One block per issue: Number, Timestamp, Title, Status, Description.
Appliance-side issues only; player issues live in ruffle/claude/issues.md.

---

Number: 1
Timestamp: 2026-07-17, 00:15 (moved 2026-07-17, 18:00)
Title: Check chumby_accel.c applicability to the Pi kernel.
Status: open — moved to the pcb-ideas branch with hardware/
Description: All PCB/hardware work (this issue, #2, and hardware/) now
lives on the pcb-ideas branch, kept out of dev's tree. This entry stays
here only as a pointer; the full text (unchanged) is on pcb-ideas.
The chumby kernel source is public (GPL), so the accelerometer
driver behind /dev/accel — chumby_accel.c, "2.1-Kionix-Ironforge", bunnie's
one-off that also carries the dcid EEPROM driver — can be read and possibly
ported. Assess what a port to the Raspberry Pi kernel would take: it
bit-bangs the KXP74 through i.MX21 GPIO calls (imx_gpio_mode/imx_gpio_write,
own spi_exchange_data), so the SPI layer would need rewriting against the
Pi's spidev/SPI subsystem, keeping the in-kernel averaging/impact logic and
the 56-byte accelReadData read() contract. Compare against the current
preference — a userspace spidev reader in PiHost feeding ASnative(5,60)/
(5,61) — which needs no kernel module; the port only pays off if something
besides PiHost must consume /dev/accel. Findings and the full access chain:
hardware/chumby-hat/accelerometer.md.

---

Number: 2
Timestamp: 2026-07-17, 01:00 (moved 2026-07-17, 18:00)
Title: Daughtercard breakout board (supersedes the HAT concept).
Status: open — moved to the pcb-ideas branch with hardware/
Description: See issue 1's note — this issue and hardware/breakout-tryout/
now live on pcb-ideas, unchanged. Reframe hardware/chumby-hat/ from a Pi HAT to a passive breakout
with a soldered-on 2x13 header the chumbilical plugs into. Mapping: DC jack →
USB-A power-only port (decide: 5 V supply into the barrel jack, or buck for
the original 12 V wart); speakers → 4-pin PH2.0 (Waveshare amp); 2x USB-A
male toward a hub carrying DATA + GND ONLY — VBUS for both jacks is the
shared P50V net, fed once from the breakout's own 5 V rail, never from the
hub's ports, so both USB ports work without back-feeding the hub; headphones
→ screw terminal; battery unconnected. Everything else on a single 2x5
dupont block mating Pi header pins 17-26 (3V3, SPI0 with both CEs, 2x GND,
GPIO24/25 for bend/reset — moves bend off FR3's GPIO17, one gpio-key config
line). Open before drawing: confirm shared P50V and the power switch sitting
in series with RAW_PWR (no button net crosses the chumbilical); HP_NOTIN
placement (screw terminal, dupont, or dropped); measure the cable-end
housing (dimensions, polarization, latches) to pick the through-hole header;
mechanical alignment of rigid USB-A plugs with a hub — fallback is a short
captive cable. Alternative still on the table: replace the connectors on the
daughtercard itself (unassessed).

---

Number: 3
Timestamp: 2026-07-17, 02:00 (updated 2026-07-17, 17:00)
Title: Plymouth boot animation, replacing the dropped Ruffle boot-opening.
Status: open — prototype built, on-device verification outstanding
Description: The 0.9.1 attempt to reproduce real hardware's opening.swf via
a sequential Ruffle run was dropped (design.md §5, "not worth the
complexity") because the animation never exits itself and a kill-timeout
approach has no good slot on a Pi 3B+. Plymouth's own model — run during
boot, get told to quit by something else — matches the original hardware
behavior (animation in parallel with boot, killed externally on
completion) far better than a sequential player ever could. Frame data
confirmed from /home/jan/chumby_backup: opening.swf is 320x240 @ 12fps, 132
frames (~11s, FWS v6, plain octopus-logo/text-typing animation, held end
frame) — matches development.md's timing notes exactly. alt_opening.swf
(320x240 @ 12fps, 145 frames) has six separate audio streams per ffprobe
and is out of scope for this pass (see audio note below). No new SWF
decoder was needed: ruffle/exporter (this repo's own submodule, built for
test fixtures) rasterizes opening.swf's 132 frames to PNG 1:1 via
`--frames all` — verified locally, frame 060 and 131 visually confirmed as
the chumby logo. Plymouth's script engine exposes `Plymouth.SetRefreshRate`
(confirmed present in the installed libplymouth5/script.so, unused by any
shipped theme but real), so the theme sets it to exactly 12 Hz — one
callback tick per source frame, no time-based indexing or modulo frame-skip
math needed (supersedes the earlier "~50Hz tick, hold ~4 ticks" guess from
the initial investigation).

Built (this session, audio dropped from scope per Jan):
- pkg/chumby-player/plymouth-theme-chumby/chumby.plymouth + chumby.script —
  loads frames/frame-0.png..frame-131.png (not shipped), SetRefreshRate(12).
- pkg/chumby-player/chumby-download-firmware — opening.swf added to
  FIRMWARE_FILES (saved to $STATE like intro.swf); new install_boot_theme()
  runs the bundled ruffle-exporter, renames its zero-padded output to
  frame-N.png, and sudos the frames + `plymouth-set-default-theme chumby`
  into place (the one privileged step — decided over chat: the downloader
  sudos internally rather than splitting into a second root-only tool or
  requiring the whole script run as root).
- pkg/chumby-player/chumby-player-run — backgrounded, non-fatal `plymouth
  quit` right before `exec cage` in --kiosk, replacing wait_for_opening's
  external-kill role. This is a proxy for "cage is about to start," not a
  true window-mapped signal — DRM handoff timing (does cage reliably get
  DRM master right after plymouth releases it?) is unverified.
- pkg/build-debs.sh + DEBIAN/control — cross-builds & bundles
  ruffle-exporter alongside ruffle_desktop, installs the theme (script +
  config only, no frames), adds `plymouth` to Depends. Full build verified
  locally (dpkg-deb succeeds, contents checked with dpkg-deb -c).
- Docs updated: design.md §5, development.md §3 (exporter cross-build),
  requirements.md.

On-device pass (2026-07-19, second test Pi 192.168.210.159 — plain
Raspbian Lite trixie, HDMI 640x480 touchscreen WaveShare WS170120,
reset button GPIO3+GND, bend button GPIO5+GND):
- Step 0 suspicion CONFIRMED twice over: Lite ships without plymouth at
  all (our Depends pulls it in), AND two more pieces were missing —
  `splash` was absent from the kernel cmdline (Plymouth never draws
  without it; Full has `quiet splash plymouth.ignore-serial-consoles`),
  and the theme must be REBUILT INTO THE INITRAMFS (auto_initramfs=1;
  apt's rebuild predates the frames, leaving only the text fallback in
  early boot). Both fixes now automated in chumby-download-firmware:
  plymouth-set-default-theme gained -R, and a new ask-first
  enable_splash() appends the three cmdline args (backup kept).
- Full downloader pipeline verified e2e ON DEVICE: real 30 MB fetch →
  opening.swf extracted (NB: 1.7.3 image's copy is 31374 bytes vs
  31297 in Jan's backup — 77-byte variance, same 132-frame structure,
  count check passed) → ruffle-exporter rasterized 132 frames on the
  Pi → sudo install → Theme=chumby active → initramfs carries all
  frames after -u.
- BOOT ANIMATION CONFIRMED BY JAN AT THE SCREEN ("I saw plymouth
  showing the chumby"). Touch confirmed too (he tapped through the
  intro tour). Handoff-quality (black gap?), bend-button and
  power-button presses still awaiting his observation.
- TRAP found and hit: dtoverlay lines sed-inserted before `[all]`
  landed inside the preceding `[pi5]` filter section and silently do
  not load on non-Pi5 boxes — APPEND to config.txt instead. postinst
  guidance now says so explicitly.
- Power button: `dtoverlay=gpio-shutdown` (defaults = GPIO3,
  active-low, pull-up — exactly the button-to-GND wiring). Wake needs
  NO configuration: a halted Pi always wakes on a GPIO3 short
  (firmware). postinst now prints this guidance when the overlay is
  absent. Guidance only — the package never edits config.txt itself.
- Bend button: ZERO NEW CODE. FR3's designed gpio-key path composes
  with the fork's existing Home-key→tap_bend mapping:
  `dtoverlay=gpio-key,gpio=5,keycode=102,label=chumby-bend` makes the
  button a keyboard-class evdev device (name "button@5" — the label
  param does NOT name the device), cage forwards KEY_HOME (verified in
  the key-capability bitmap: 0x4000000000 in word 2 = bit 102) to the
  player like any keyboard. Pin is per-device config: FR3 says GPIO17,
  issue 2 proposes GPIO24/25, this box uses GPIO5 — docs should settle
  on one recommendation eventually.

- Bend press VERIFIED by Jan after one fix in the fork (dev 715a60f3c):
  the panel polls _bent per frame (~83 ms) and a crisp GPIO tap is
  shorter — level-only set_bent lost it ("sometimes does not fire,
  sometimes takes seconds" = re-pressing until a poll caught one). The
  Home-key handler now also latches tap_bend() on the press edge.
  No kernel-side debounce/hold was involved (gpio-keys default 5 ms).
- Power button RESOLVED (2026-07-19, later the same day): the wired
  chumby reset button never fired because the press never reached
  GPIO3 (debugfs pin watch: level never left hi). A direct short of Pi
  pins 5–6 shuts down and wakes cleanly — the whole software chain is
  good. Root cause was on the chumby side: the reset switch's
  chumbilical pair is physical 5↔6, not the guessed pins — found via
  the mainboard schematic ("reset switch on DC / reset pulls up",
  sheet 2) plus Jan's continuity measurement, which also exposed a
  one-row misread in the recorded pin table. Full corrected table
  (provenance-tagged, refined against pstrick2's 2019 forum survey —
  numbering convention comes from that post, not a board marking) and
  the P33VBKUP-role question: pcb-ideas branch, commit 7a1f3ef
  (hardware/chumby-hat/accelerometer.md §3). Wiring: chumbilical 5 →
  Pi GPIO3, chumbilical 6 → Pi GND; stock gpio-shutdown, wake
  included — the mainboard's active-high biasing was its own affair.

Still outstanding:
- Jan's at-screen observation of handoff quality (black gap?)
  (asked 2026-07-19).
- The original test Pi (192.168.42.51, SPI TFT ILI9486): does the DRM
  handoff behave with the SPI panel, where plymouth's drm renderer and
  cage contend for a much slower device? Untested — box offline.
- No-firmware-downloaded case: a box that never runs the downloader
  must still boot cleanly with no half-built theme (theme chrome ships
  but is never activated — believed safe by construction, unverified).
- pkg changes since the 0.9.1 deb build (postinst guidance, -R,
  enable_splash) need a rebuild + re-test of the deb itself; the
  on-device run used the 0.9.1 deb plus manual equivalents of the new
  steps.

---

Number: 4
Timestamp: 2026-07-26, 19:45
Title: Find a display: 3.5", 4:3, real brightness control, and 12 fps.
Status: open — the deciding criteria are now known; no candidate meets all four
Description: With the CPU renderer shipped (fork
claude/tiny-skia-backend-plan.md; CHUMBY_RENDERER=tiny-skia), the player is no
longer what limits the panel: it draws the control panel using 37 % of one core
and leaves ~90 % of that core idle, while the achieved rate stays ~6-7 fps. On
the current ILI9486 SPI TFT the *display* is the ceiling, so choosing a panel is
now the same decision as choosing a frame rate. Four criteria, all required:

1. **3.5"** — the enclosure and the original device's size (design.md §8).
2. **4:3** — the control panel and every widget were authored for 320x240.
   Note that the usual Pi 3.5" panels are 480x320, which is 3:2, not 4:3; the
   Waveshare 3.5" HDMI LCD (E) already on the second box is 640x480 and *is*
   4:3.
3. **Brightness control in hardware** — FR16 ships and is inert without it.
   The current clone ties its backlight rail to 3.3 V; on the (E) every
   software path was probed dead (no kernel backlight, vendor USB command
   accepted but ignored, no DDC/CI — design.md §8), leaving its PWM solder pad.
4. **12 fps** — the movie's own rate, and what the original hardware did.

Why the current panel cannot reach it: the piscreen overlay clocks SPI at
24 MHz (`speed=24000000`), and one 480x320 RGB565 frame is 307 200 bytes, i.e.
~2.46 Mbit. That caps the link at ~9.8 fps before any overhead, which matches
the measured ~6-7. So on SPI, 12 fps is a bandwidth question first: a 4:3 panel
at 320x240 needs only 153 600 bytes/frame (~19 fps at 24 MHz), while 640x480
needs 614 400 (~4.9 fps) and is out of reach over SPI entirely.

Cheapest experiments first, before buying anything:
- Raise the SPI clock on the panel in hand (`speed=32000000`, then 48) and
  re-measure fps the way development.md §6 counts it. Free, and it tests the
  bandwidth arithmetic directly; the controller may or may not tolerate it.
- Measure the (E) at 640x480 with tiny-skia on the second box (192.168.210.159,
  offline on 2026-07-26). It is the only 4:3 3.5" panel here, HDMI so no SPI
  bandwidth limit, and §6 measured ~11-12 fps on it with the *old* renderer;
  with the CPU renderer it should clear 12 fps with headroom. If it does, the
  whole question reduces to brightness — i.e. to the PWM solder-pad mod.
- Only then survey panels against all four criteria. design.md §8's existing
  candidates (Adafruit PiTFT Plus 3.5" 2441, Waveshare 3.5" (C)) were chosen for
  dimming and driver support, both are 480x320 3:2, and neither was assessed for
  throughput. DPI panels stay out: they consume the GPIO header, killing SPI and
  the bend button.

Decision to make once measured: keep 4:3 and accept the (E) plus a soldering
mod, take a 3:2 panel with clean dimming and letterbox the 4:3 content, or drop
to a 320x240-class panel where SPI has the bandwidth for 12 fps.
