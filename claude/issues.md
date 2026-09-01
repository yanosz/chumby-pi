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
Status: open — boot animation confirmed on both boxes; only the
black-gap/handoff quality and the alt_opening audio question remain
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
- Same install repeated on the 5" DSI box 2026-08-24 (development.md §6,
  third test box) with nothing left to fix by hand: 132 frames rasterized
  on the Pi, theme activated (`/etc/plymouth/plymouthd.conf` →
  `Theme=chumby`), both initramfs images rebuilt (v8 and 2712 —
  `lsinitramfs` counts 132 frame files), and `enable_splash()` appended
  `quiet splash plymouth.ignore-serial-consoles` (backup
  `cmdline.txt.bak-chumby`). Watched the same day and it works: Jan
  confirmed the animation on the DSI panel ("animation loads"), and the
  handoff left no trace of trouble — `chumby-player` active 28.2 s into a
  30.9 s boot, with no DRM, cage or plymouth complaint in the journal. The
  animation runs ~11 s (132 frames at 12 fps), so Plymouth holds its end
  frame until the launcher's `plymouth quit`; the proxy signal is early
  enough in practice on this hardware. The initramfs hook's
  `label-pango.so` warning does not apply — `chumby.script` uses only
  Image/Sprite/SetRefreshRate, so `plymouth-themes` is not a missing
  dependency.
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
Timestamp: 2026-07-26, 19:45 (updated 2026-07-26, 22:40)
Title: Find a display: 3.5", 4:3, real brightness control, and 12 fps.
Status: open — criterion 4 is NOT met and cannot be tuned into being: every SPI
clock above the overlay's nominal 24 MHz visibly corrupts the picture. The
panel's honest clean ceiling is 8.4 fps, reached by pinning the core clock
alone. 4:3 and brightness remain unmet too. All three criteria now point at a
different display.
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

Why this looked like a purchase: one 480x320 RGB565 frame is 307 200 bytes,
i.e. ~2.46 Mbit, so at the overlay's original `speed=24000000` the link caps at
~9.8 fps before any overhead — below the movie's rate however good the player
gets. That framing was right about the arithmetic and wrong about the premise:
`speed=` is a free variable, and the panel tolerated being asked for more (see
"The SPI clock was simply set too low" below). For reference, the same
arithmetic on other geometries: 320x240 needs 153 600 bytes/frame, 640x480
needs 614 400.

Measured 2026-07-26, 21:40-21:50 (first test Pi, 3B+, ILI9486 SPI TFT, 0.9.1
deb, CHUMBY_RENDERER=tiny-skia; fps as cage's DRM_IOCTL_MODE_ATOMIC commits
over 8 s under strace, development.md §6's method — strace overhead is in every
number, so the ratios are the finding, not the absolutes):

**The panel was never running at 24 MHz.** No `core_freq` was pinned, and the
SPI baud divisor derives from the core clock, which the firmware scales down
when the SoC is idle: `vcgencmd measure_clock core` sampled 400, 287.5, 268.75,
250, 268.75 MHz over five seconds at idle, while the kernel clock tree reports
`vpu` at a flat 400 MHz (what the driver divides from). Shipping the CPU
renderer made the box idle, and the idle box clocked its own display link down
— part of the measured ~6-7 fps was self-inflicted.

| condition | core | ARM | commits/8 s | fps |
|---|---|---|---|---|
| stock, idle (`ondemand`) | 250-400, scaling | 1100 MHz | 52-53 | ~6.5 |
| stock + 2 busy cores | 400 MHz | — | 67 | ~8.4 |
| stock + `performance` governor | 400 MHz | 1400 MHz | 67 | ~8.4 |
| **`core_freq_min=400` + `ondemand`** | **400 MHz** | 1400 MHz | **67-68** | **~8.5** |
| `core_freq_min=400` + `powersave` | 400 MHz | 600 MHz | 66 | ~8.25 |

**+29 % for one config.txt line, and the cause is isolated.** The last row is
the control: forcing the ARM cores 2.3x slower (1400 -> 600 MHz) costs ~2 %, so
the gain is SPI baud, not the ili9486 driver's CPU-side XRGB8888->RGB565
conversion. Applied to the box as `core_freq_min=400`, appended inside `[all]`
(backup `/boot/firmware/config.txt.bak-corefreq`); after the change, 47.8 C,
`get_throttled` 0x0, box 88 % idle. Not packaged — whether this becomes postinst
guidance like the other config.txt lines is open.

The numbers fit the divisor model, though this is INFERENCE and was not checked
against spi-bcm2835's source: an even CDIV of 18 gives 400/18 = 22.2 MHz
(ceiling 9.0 fps, measured ~8.5) pinned, and ~280/18 = 15.6 MHz (ceiling 6.3,
measured ~6.5) at the pre-pin sampled average.

**The SPI clock was simply set too low** (measured 2026-07-26, 22:10-22:40,
same box and method, `core_freq_min=400` throughout, one reboot per step):

| `speed=` | implied effective SPI | predicted ceiling | commits/8 s | fps |
|---|---|---|---|---|
| 24 MHz | 22.2 MHz (CDIV 18) | 9.0 fps | 67-68 | ~8.5 |
| 32 MHz | 28.6 MHz (CDIV 14) | 11.6 fps | 85-86 | ~10.7 |
| **40 MHz** | **40 MHz (CDIV 10)** | 16.3 fps | **95-96** | **12.0** |

At 40 MHz the frame counter reads **exactly the movie's 12 fps** — 144 commits
over a 12 s window — at 49 % of one core, 120 MB RSS, 48.9 C, `get_throttled`
0x0 and no SPI or DRM errors in `dmesg` at any step.

**And it was measuring a corrupted picture.** Jan at the screen, 2026-07-26:

| `speed=` | core | effective SPI | fps | picture on the glass |
|---|---|---|---|---|
| 24 MHz | unpinned, 250-400 | ~14-22, wandering | 6.5 | clean |
| **24 MHz** | **pinned 400** | **22.2 constant** | **8.4** | **clean — shippable** |
| 32 MHz | pinned 400 | 28.6 | 10.6 | artifacts |
| 40 MHz | pinned 400 | 40 | 12.0 | artifacts |

The artifacts read as "super-high contrast, with a small bright border around
every symbol", "anti-aliasing going mad". They appear at 28.6 MHz and above and
vanish below, with the renderer held constant — so they are the SPI bus, not
tiny-skia. Note what this means for instrumentation: `dmesg` was clean at every
step, the atomic-commit counter was honest, and both were counting frames that
arrived at the panel damaged. Neither can see this class of fault, and neither
can `grim`, which captures the compositor's buffer rather than the glass. Only
eyes on the screen closed this.

**The clean win is the core-clock pin by itself.** `speed=24000000` with
`core_freq_min=400` holds a constant 22.2 MHz — not an overclock at all, since
it is *below* the 24 MHz the overlay has always requested and merely stops the
rate sagging to ~14 whenever the SoC idles. 6.5 -> 8.4 fps, verified clean at
the screen, one config.txt line, no hardware change. That is the configuration
to ship, and whether it becomes postinst guidance alongside the other
config.txt lines is the open packaging question.

**What it changes.** Criterion 4 is unreachable on this panel: the ILI9486
clone does not tolerate a faster bus, and 8.4 fps is its honest clean ceiling
against the movie's 12. Bandwidth is therefore still the reason to buy — but
for a panel with *fewer pixels*, not a faster link. 320x240 is half this
panel's bytes per frame, so it clears 12 fps at the same safe 22 MHz this one
tops out at. That makes the QVGA candidate class below the strongest option on
throughput as well as on aspect and fidelity.

Cheapest experiments first, before buying anything:
- DONE, and they closed criterion 4 negatively. Pinning the core clock is worth
  +29 % and is clean; every `speed=` above nominal is fast and corrupt. See the
  measurement blocks above.
- Measure the (E) at 640x480 with tiny-skia on the second box (192.168.210.159,
  offline on 2026-07-26). It is the only 4:3 3.5" panel here, HDMI so no SPI
  bandwidth limit, and §6 measured ~11-12 fps on it with the *old* renderer;
  with the CPU renderer it should clear 12 fps with headroom. If it does, the
  whole question reduces to brightness — i.e. to the PWM solder-pad mod.
- Only then survey panels, now against 4:3 and brightness rather than all four.
  design.md §8's existing candidates (Adafruit PiTFT Plus 3.5" 2441, Waveshare
  3.5" (C)) were chosen for dimming and driver support and are both 480x320
  3:2. DPI panels stay out: they consume the GPIO header, killing SPI and the
  bend button.

The class not yet considered, and the best fit on paper: a **3.5" 320x240
(QVGA) SPI TFT**. It is exactly 4:3 and exactly the content's pixel grid, so
the movie maps 1:1 with no scaling; it is period-correct, since the original
chumby's screen was 3.5" QVGA; it halves the bytes per frame, putting 12 fps
well inside budget even at 24 MHz; and ILI9341-class controllers have mainline
`drm/tiny` drivers, so the DRM+cage stack of design.md §5/§6 carries over the
way piscreen does — which is the part that took the most effort to get right.
Brightness then reduces to whether the module breaks its backlight LED pin out
separately instead of tying it to 3.3 V, which is usually readable from the
pinout. UNVERIFIED and the reason this is an idea and not a recommendation:
whether such a part is actually purchasable today. 3.5" is overwhelmingly sold
as 480x320, and most 320x240 modules are 2.4"/2.8"; no specific part has been
identified, priced, or checked for a mainline-supported controller.

Decision to make: keep 4:3 and accept the (E) plus a soldering mod, take a 3:2
panel with clean dimming and letterbox the 4:3 content, or find a 3.5" QVGA
panel and get aspect, fidelity and bandwidth in one part.

---

Number: 5
Timestamp: 2026-08-20, 22:05
Title: Implement backlight brightness on the 5" DSI box.
Status: closed — brightness confirmed working 2026-08-24; the udev rule
verified on a plain install
Description: The 5" Waveshare DSI LCD (C) (1024x600, overlay
`dtoverlay=vc4-kms-dsi-waveshare-panel,7_0_inchC` appended after `[all]`) on
the new 3B+ exposes a real kernel backlight: `/sys/class/backlight/10-0045`,
`max_brightness` 255. Writes are accepted with no I2C errors and Jan confirmed
visible dimming at the screen (sweep 255 -> 10 -> 255, held 8 s at the bottom).
This is the first display in the project with working brightness, so FR16 —
player-ready since 2026-07-13 and blocked on hardware ever since
(requirements.md §3, and issue 4's third criterion) — can be closed on a
device.
Already shipped, needs verifying end to end rather than building: the deb's
`90-chumby-backlight.rules` (chgrp video + g+w on `brightness`, re-run from
postinst), the fork's FR16 sliders, and the `settings-brightness` ui-policy
rule that lifts by itself once a backlight exists. Unverified on this box:
whether the pi user really gets write access through that rule (every write in
this session went through sudo), whether the Settings button un-dims, and how
the panel's slider range and night mode map onto 0-255.

Closed 2026-08-24 on the reflashed box (development.md §6, third test box):
Jan confirms brightness works. The write-access question is answered
independently — after a bare `apt install chumby-player` and a reboot,
`/sys/class/backlight/10-0045/brightness` is `root:video 0664` and `pi` can
write it with no sudo, so the shipped `90-chumby-backlight.rules` does fire on
a real install. `brightness_ctl` stays unset, so the player takes the
auto-detect path and `10-0045` is the lone backlight it finds. FR16 is
satisfied on hardware.

---

Number: 6
Timestamp: 2026-08-20, 22:45
Title: CI shipped the player with Ruffle's mock clock (deterministic feature).
Status: closed — CI split into two cargo invocations, guarded; verified on
the DSI box 2026-08-24
Description: `5c9b2dd` taught the workflow to cross-build the exporter the
deb bundles, and did it in the same cargo invocation as the player:
`cargo build -p ruffle_desktop -p exporter --profile dist --target ...`.
`exporter/Cargo.toml` asks `ruffle_core` for `features = ["deterministic"]`,
and cargo unifies features across packages built together, so
`ruffle_desktop` linked a `ruffle_core` whose
`locale::get_current_date_time()` is frozen at the test constant
2001-02-03 04:05:06 (`core/src/locale.rs`, `MOCK_TIME`). Every `new Date()`
in the panel returned that. Confirmed with
`cargo tree -e features -p ruffle_desktop -i ruffle_core`: clean alone,
`feature "deterministic"` present the moment `-p exporter` joins.
Visible as the built-in clock reporting February with no digits (ruffle
issue 3 — the frozen seconds make `BuiltinClock.update()` run exactly once,
before the digit strips are class-linked, so all six park blank forever).
The local cross-build in claude-docs/development.md §3 was always two
separate commands, which is why this only ever appeared on a CI-built deb.
Fix: the workflow builds the two binaries in separate invocations (costing
a second `ruffle_core` build) and a preceding guard step fails the run if
`ruffle_desktop` ever resolves `deterministic` again — the failure is
otherwise silent, since a wrong clock is the only outward sign.
`pkg/deploy-pi.sh` gained the missing exporter build as its own invocation
too — it built only the player and then called `build-debs.sh`, which
requires the exporter, so a clean tree could not deploy at all.

Corrected 2026-08-20, 23:10 — the first pass here named only
`deterministic`, which was the half of the leak that happened to be
visible. `exporter/Cargo.toml` asks for `features = ["deterministic",
"default_font"]` and cargo unified **both**. Full diff of the two
resolutions (`cargo tree -e features -p ruffle_desktop [-p exporter]`,
feature edges compared):

| feature | reaches the player when merged | consequence |
|---|---|---|
| `deterministic` | yes | mock clock, 2001-02-03 04:05:06 |
| `default_font` | yes | fallback font embedded in `ruffle_core` |

`ruffle_core` is the only *shared* crate that changes; everything else the
merged graph adds (`console`, `indicatif`, `rayon`, `portable-atomic`,
`unit-prefix`, `crossbeam-utils`) is exporter's own subtree and is not
linked into the player. The guard step now covers both names.

`default_font` caused nothing. It crossed on the same bad invocation, but
it has no symptom and no consequence, and an earlier revision of this entry
was wrong to raise one: it claimed that dropping the leak *takes a fallback
font away from the player*. It does not. Every build before `66cd1db` — so
every release the project ever shipped — ran `cargo build -p ruffle_desktop`
alone, with no exporter and therefore no `default_font`. The split restores
that exact configuration. The only build that ever carried the feature is
the broken one from 2026-08-20 18:06, which is not a baseline. Nothing about
fonts or `Depends` is open; the guard covers the second name only so the
leak cannot reopen unnoticed.

Verified on a device 2026-08-24: the apt repo's `chumby-player_0.9.3` — built
by CI run 32510824744 (main, 2026-08-21), whose "Guard against exporter's
features reaching the player" step passed and whose player and exporter built
in separate invocations — installed on a freshly bootstrapped 3B+ with the 5"
DSI panel. Jan confirmed at the screen that the built-in clock loads correctly:
no February 2001, all digit strips drawn. Nothing left open here.

---

Number: 7
Timestamp: 2026-08-22, 23:45
Title: A Pi reboot orphans the chumby's gadget link.
Status: open — known behaviour, no fix attempted
Description: When the Pi reboots, its USB gadget re-enumerates, which
destroys and recreates the netdev on the chumby side; any DHCP client the
chumby had running dies with it. Observed directly after a Pi reboot:
`usb0` on the Pi shows `RX: 0 bytes, 0 packets` against `TX: 7916` with
carrier up — the chumby is on the wire and silent. Nothing on the chumby
re-establishes the link outside its own boot path (mountmon's USB add event
plus `/psp/rfs1/userhook1`), so the chumby needs rebooting after the Pi does.
Fine in the intended steady state, where the Pi outlives the chumby. Fixing
it properly means a client on the chumby that reacts to the interface being
recreated, not just to boot; an earlier attempt at a polling watchdog was
rejected as too hacky, and the vendor path (`udhcpc -R -n`, one shot, skipped
whenever another `eth*` is RUNNING) cannot do it. See
pi.nic/README.md.

---

Number: 8
Timestamp: 2026-08-22, 23:45
Title: The control panel blocks forever on a no-timeout wget.
Status: open — hazard, not yet triggered by anything we control
Description: `/usr/chumby/scripts/network_status.sh` runs
`wget -q -O - http://www.chumby.com/crossdomain…` with no timeout. If the
chumby has a route whose gateway silently drops traffic, that fetch never
returns and the control panel never finishes starting — the screen sits
there looking like a boot hang, while `ps` shows `chumbyflashplayer.x`
running and a `wget` parked behind it. This is how the missing-NAT bug
presented, and it is worse than having no route at all: with no route the
fetch fails fast, which is why the box booted normally on a USB dongle and
only wedged once the Pi was its only NIC. Any future blackhole on that link
reproduces it. chumby.com itself is currently answering (232 bytes,
~1.2 MB/s), so this is latent rather than active.

---

Number: 9
Timestamp: 2026-08-23, 00:15
Title: Direct stream entries in /psp/url_streams need mimetype="audio/mpeg".
Status: fixed 2026-08-23 — SWR3 plays
Description: The standalone "SWR3" entry in `/psp/url_streams` carried
`mimetype="audio/x-mpegurl"` — the *playlist* type — while its `url` pointed
straight at an MP3 stream, so the player fetched it expecting an m3u and got
raw MP3 frames. Nothing played. The URL was never wrong: it is byte-identical
to the one in `/psp/list.m3u`, and the chumby pulled 1.52 MB from it in six
seconds over plain HTTP (no TLS, no redirect — an earlier theory that the 2006
player could not reach a modern endpoint was wrong).

The rule: `audio/mpeg` for a direct stream, `audio/x-mpegurl` only when the
`url` really is a playlist. Every working station on the list (1live, wdr-2,
wdr-3, wdr-5, NRK P3 Jazz, Radio Norge) uses `audio/mpeg`; the sole correct
`x-mpegurl` entry is "Birds + SWR3", which points at `file:////psp/list.m3u`.
The broken entry looks copy-pasted from that one with the mimetype left
behind — which is also why the playlist version played while the direct one
did not, a confusing pair of symptoms worth remembering when adding stations.

Fix: one attribute. Backup kept at `/psp/url_streams.bak-swr3`; `/psp` is
jffs2, so it persists.

---

Number: 10
Timestamp: 2026-08-24, 13:40
Title: Playlist entries in an m3u must be absolute chumby paths, not relative.
Status: fixed 2026-08-24 — birds.mp3 plays on the Pi
Description: The "Birds + SWR3" stream entry (`mimetype="audio/x-mpegurl"`,
issue 9) points at a local playlist. The panel — not the host — fetches and
parses that m3u and hands the host ONE chosen entry: `pgrep -a mpv` showed
mpv launched straight with the SWR3 URL, so a first line reading `birds.mp3`
had been discarded silently. Rewriting that line as the chumby's own absolute
form `/psp/birds.mp3` makes it play, birds first, then SWR3.

So relative entries are dropped and absolute chumby paths survive; the host's
`resolve_url` (fork `core/src/chumby/audio.rs`) then maps the leading `/`
through the virtual rootfs to
`/var/lib/chumby/fixtures/rootfs/psp/birds.mp3`. A real chumby's `list.m3u`
therefore needs no editing to work on the Pi — copy it verbatim, and copy the
media it names into the same fixtures `psp/` directory.

Ruled out on the way: mpv's unsafe-playlist filter. `mpv --load-unsafe-playlists`
changes nothing here, and mpv given the playlist directly resolves and plays
the relative entry fine — the filtering is the panel's, above the host.

---

Number: 11
Timestamp: 2026-08-24, 13:45
Title: The Tagesschau widget's video source is gone (502, not our stack).
Status: closed — external; widget deleted from the box 2026-08-24
Description: The widget (5.4 KB, fetched from chumby.com's guide, GUID
5D9DAB9E-D7E3-11DF-9EC6-0021288E6F90) hardcodes a single endpoint played
through `NetStream`: `http://welttheorie.de/tagesschau.flv` — a fan-made
widget pointing at a private server. The domain still resolves (IPv6 only,
2a00:17d8:100:1::1361) and the Pi reaches it, but the server answers 502 for
that file and for its root, so pressing "Wiedergabe" has nothing to play.
Untested, and now moot: whether our player decodes FLV video through
`NetStream` at all, and what that would cost on a 3B+.

---

Number: 12
Timestamp: 2026-08-24, 13:50
Title: Black bars around a widget on the 1024x600 DSI panel — accepted.
Status: closed — Jan chose to leave the scaling alone (2026-08-24)
Description: On the 5" DSI box, RoboClock reads as framed in black on all
four sides. Two independent causes, measured rather than guessed:
- The widget's own artwork. Rasterizing frame 0 with the shipped
  `ruffle-exporter` gives 15 px of black at the top and 14 px at the bottom
  of its 320x240 stage; the content band is 211 px tall. All four widgets
  fetched this session are exactly 320x240, the panel's native stage, so
  nothing is being letterboxed *into* the widget area.
- Our fit. 320x240 on 1024x600 under the default show-all scales 2.5x to
  800x600, leaving 112 px black to the left and right.

Options put to Jan: (A) `--scale no-border --force-scale`, which fills the
screen but crops 26 stage px off the top and bottom of *everything* — the
control-panel bar and clock edges included; (B) a fractional zoom lever in
the fork, tunable to crop just the ~15 px, which `StageScaleMode` cannot
express and would therefore be player work; (C) leave it. **Jan chose C.**
4:3 content on this panel letterboxes somewhere regardless; a 4:3 display
(issue 4) would remove the side bars but never the widget's own artwork.


---

Number: 13
Timestamp: 2026-08-28, 12:30
Title: A "Birds + SWR3" alarm went silent about 100 s in and stayed silent.
Status: cause identified on the device 2026-09-01 — the 08:00 nightmode alarm
cancels the still-ringing 07:59 alarm through the panel's own
`stopAlarmsExcept`. Nothing in the player, the stream or the network is
involved. Option B built and desktop-verified 2026-09-01 (fork issue 10);
device verification outstanding.
Description: Jan set a one-shot alarm on the "Birds + SWR3" My Streams entry
(the m3u of device issue 10) for 07:59 and left it to ring. The birds played,
SWR3 took over, and after roughly a minute and a quarter the sound stopped.
He did not snooze and did not dismiss — he wanted the radio. Nothing played
again until he started the same entry by hand at 08:09:35, which worked
normally. Box: chumby-pi-3 (192.168.42.24), 0.9.4, wlan0.

No player log exists for it. `chumby-player-run:229` defaults `RUST_LOG=warn`
and every trace that would settle this is at info, so the unit's journal holds
four systemd lines for the whole boot.

What the box does record, from rtkit's RT-thread grants — each new pid that
gets an RT audio thread is one mpv spawn:

    07:59:01  pid 2129        alarm rings, birds.mp3 (30.07 s, measured)
    07:59:33  pid 2148  +32 s SWR3 takes over
    08:00:48  pid 2168  +75 s
    08:00:53  pid 2186  +5 s
      (8 m 42 s with no mpv at all)
    08:09:35  pid 2208        Jan's manual replay: birds.mp3
    08:10:07  pid 2227  +32 s   and SWR3

**This evidence has a hole that matters:** an mpv that spawns but never opens
an audio device asks rtkit for nothing and leaves no line here. There may have
been further mpv processes in the silent stretch that this timeline cannot
show.

Read against the panel, the first four fit exactly. `/psp/list.m3u` ends with
a newline, so `M3U` (F2:15009, which pushes every line whose first character
is not `#`) yields **three** tracks — birds, SWR3, and an empty string. birds
ends at 30 s and is correctly treated as finished; SWR3 ends at 75 s, likewise;
the empty third track is spawned at 08:00:48, dies at once, and
`DirectURLPlayer.doStepTrack` (F2:15579) removes it under THRESHOLD = 5000 ms
and wraps the list — so 08:00:53 is birds starting over. The trailing newline
is the trap already recorded in fork issue 5; here it costs one dead spawn and
about five seconds, and is **not** why SWR3 stopped (without it the list would
have wrapped to birds anyway).

Two questions remain, and they are separate:

1. **Why did mpv exit 75 s into SWR3?** Not the panel's 2 s liveness gate:
   `poll_state` (fork `core/src/chumby/audio.rs:165`) reports Stopped only
   once the mpv *process* has exited, so a slow start reads as PLAYING here.
   Not the network either — no wpa_supplicant, dhcp, carrier or resolver event
   anywhere between 07:30 and 08:30. And not the stream: replayed from the
   same box at 12:15 with `--ao=null`, it ran the full 150 s asked of it with
   ICY titles updating, exiting only on our own `--length`. Our own spawn
   (`audio.rs:82`) sets no length, cache or timeout. The exit status is logged
   at info (`audio.rs:170`) and was therefore discarded.

2. **Why did nothing play for the next 8 m 42 s,** on an alarm with
   `duration="15"` and no dismissal? Leading hypothesis, unverified: birds
   replayed to its end at ~08:01:23, the panel advanced to SWR3 again, and
   that mpv stayed alive without ever producing audio — which our `poll_state`
   reports as PLAYING for as long as the process lives, so the panel's track
   supervision never fires and the alarm sits silent with the panel believing
   it is playing. That shape matches an eight-minute silence, and it would be
   ours, not the panel's. It also explains the missing rtkit line. Nothing
   confirms it yet.

Armed 2026-08-28 for the next occurrence: `RUST_LOG` set as an active line in
`/etc/default/chumby-player` (previous file kept at
`/etc/default/chumby-player.bak-preverbose`) to
`warn,chumby_host=info,chumby_audio=info,avm_trace=info`, and the box upgraded
to 0.9.5 and restarted. That captures `mpv pid=… url=…`, `mpv exited: …`,
`doStepTrack(): track ended at N secs`, `track dead, removing` and
`setTracks(): got N tracks` — between them enough to answer both questions.
Journald is at its defaults here (≈5.9 GB cap on a 59 G card, 2.9 MB used), so
the volume is safe; revert the line once this is closed.

Not fixed, deliberately: the trailing newline in `/psp/list.m3u`. It is a
one-byte change on a live box and would move the evidence under our feet
before the next ring is captured.

Update 2026-08-28, 18:30 — a control run, and the experiment it sets up.

With `RUST_LOG` raised and 0.9.5 installed, the identical alarm was armed for
18:10, deliberately far from the three enabled daily alarms (00:00, 08:00,
23:00). Every field but time and name was copied from the failing one — same
stream param, `duration="15"`, `snooze="5"`, `auto_dismiss="1"`,
`action="nightmode"`. It ran clean:

    18:10:00  playAsAlarm(): <stream url="/psp/list.m3u" … name="Birds + SWR3"/>
    18:10:00  TrackedPlayer.setTracks(): got 3 tracks
    18:10:00  mpv pid=3684 url="/psp/birds.mp3" vol=0 loops=1
    18:10:01  WARN IPC socket not ready — volume control limited to spawn-time
    18:10:01  IPC socket connected late
    18:10:31  mpv exited: exit status: 0
    18:10:31  doStepTrack(): track ended at 30.835 secs
    18:10:31  mpv pid=3714 url="http://liveradio.swr.de/…/play.mp3" vol=44

SWR3 then played unbroken from 18:10:31 until the service was stopped by hand
at 18:16:56 — 6 min 25 s, no `doStepTrack` intervention, no track death, no
`stopAlarmsExcept`. **Away from 08:00 this alarm does not fail**, which is the
first hard evidence that the morning's failure is not intrinsic to playing
this stream as an alarm.

Confirmed on the device, no longer inferred: `setTracks(): got 3 tracks` —
the trailing newline in `/psp/list.m3u` really does yield the phantom empty
third track.

Eliminated since the first pass:
- **The station.** Jan restarted the same entry by hand minutes after the
  failure and SWR3 played on. Same URL, same box, same build.
- **The periodic alarm reload.** `AlarmSet.step` (F2:12030) reloads
  `/psp/alarms` on a timer and calls `stopAlarmsExcept(undefined)` first,
  which would stop every ringing alarm — but the interval is
  `_root.alarmReloadInterval * ONE_MINUTE` **or `ONE_YEAR` when the FlashVar
  is absent** (F2:11784), and our launcher passes only `-PlocalCache=1`. Inert
  for us.

Leading hypothesis, Jan's: **the 08:00 nightmode alarm stopped it.**
`Alarm.ringAlarm` (F2:11182) opens with `_alarmSet.stopAlarmsExcept(this)`,
and `stopAlarmsExcept` (F2:12039) calls `stopAlarm(true)` on every *other*
ringing alarm. `"Daily at 8:00"` (`time="480"`, daily, enabled, `type="none"`,
`action="nightmode" action_param="off"`) would do exactly that to an alarm
still ringing from 07:59 — two alarms one minute apart. Unexplained detail,
recorded rather than argued away: `Alarm.step` (F2:10910) rings only when
`now - alarmTime` is within `RING_WINDOW = 15000` ms (F2:10186) and
`AlarmSet.step` takes a fresh `new Date()` every frame, so that alarm fired
between 08:00:00 and 08:00:15 or not at all — while SWR3's mpv was still alive
until ~08:00:47. The ~32 s gap is close to birds.mp3's length (30.07 s), which
may mean the spawn-to-track mapping is off by one somewhere. Not resolved.

Armed for 2026-08-29 07:59: the same one-shot alarm, with `"Daily at 8:00"`
still enabled, so the two ring a minute apart exactly as they did. The panel
traces `AlarmSet.stopAlarmsExcept(): cancelling <alarm>` (F2:12049) whenever
that path fires, so the outcome is binary — either that line appears at ~08:00
and settles it, or mpv's own exit status is captured at the moment of death.

Box state left behind on chumby-pi-3: 0.9.5, `RUST_LOG` active in
`/etc/default/chumby-player`, the 07:59 alarm armed, `/psp/alarms` backed up
at `/psp/alarms.bak-20260828`, `/psp/list.m3u` still carrying its trailing
newline on purpose.

Update 2026-08-29, 16:00 — session closed, retry moved to Monday
2026-08-31. Jan deleted the 07:59 test alarm before it could ring, so the
experiment did not run; the entry is gone from `/psp/alarms` (not merely
disabled) and Monday needs a fresh one. Its exact XML is preserved in the
backup `/psp/alarms.bak-20260828` and in the first block of this issue.

One measurement did come out of the morning, from a run where the test alarm
was not armed at all:

    Aug 29 08:00:00.065  avm_trace: Alarm.step(): ringing Daily at 8:00

The nightmode alarm fires within 65 ms of 08:00:00, on the device, exactly as
`RING_WINDOW` (F2:10186) predicts. **This strains the leading hypothesis
rather than supporting it.** `stopAlarm(true)` → `stopAlarmSoundContinuous`
→ `MusicPlayer.stopMusic(false)` reaches our `stop()` with no delay, so had
`stopAlarmsExcept` killed the 07:59 alarm on 2026-08-28 it would have died at
08:00:00, not at ~08:00:47. The 47 s is now measured to be unexplained, not
merely assumed to be. The hypothesis is not dead — nothing else found so far
stops a ringing alarm from outside — but it no longer accounts for the
timeline on its own.

To resume on Monday:
1. Re-arm the alarm for 07:59 with `"Daily at 8:00"` left enabled. `time` for
   a `when="once"` alarm is **minutes since the Unix epoch, local time**
   (verified: 29798279 = 2026-08-28 07:59 CEST); copy every other attribute
   from the block at the head of this issue. The panel reads `/psp/alarms`
   only at start, so restart `chumby-player` after writing it.
2. Confirm it is scheduled: the log prints
   `Alarm heartbeat <name> rings in N seconds, at:<time>` for each alarm.
3. After the ring, pull `journalctl --since "<date> 07:55:00"` and grep for
   `chumby_audio`, `setTracks`, `doStepTrack`, `stopAlarmsExcept`,
   `Alarm.step(): ringing`. Expect `got 3 tracks` — the phantom empty track
   is normal here and is not the failure.
4. Note journalctl on this box rejects relative timestamps ("today",
   "-1 min"); use absolute ones, and query without `-u chumby-player`, since
   the player's lines carry the `chumby-player-run` identifier.

Left armed on chumby-pi-3 for Monday: 0.9.5, `RUST_LOG` active in
`/etc/default/chumby-player`, `/psp/list.m3u` still ending in a newline on
purpose. Revert the `RUST_LOG` line once this issue closes.

Also seen and deliberately not pursued, worth its own issue later: the alarm
fade spawns mpv at `vol=0` and ramps over IPC, and on 2026-08-28 the socket
was not ready at spawn ("IPC socket not ready", then "connected late" 160 ms
after). It won the race that time. It is the same race behind the 2026-07-06
"alarm fade-in muted forever" note in `audio.rs:228`, and losing it means a
silent alarm.

Update 2026-09-01, 09:15 — **reproduced and fully traced. Jan's hypothesis
was right.** The alarm rang again at 07:59 and went silent as the 8 o'clock
news started; this time the box was still on 0.9.5 with `RUST_LOG` raised and
the player process running unbroken since 2026-08-28 18:17:20, so the whole
morning is in the journal. The decisive lines, verbatim:

    07:59:00.063  Alarm.step(): ringing Sep 01 2026,  7:59
    07:59:00.077  DirectURLPlayer.playAsAlarm(): <stream url="/psp/list.m3u" …/>
    07:59:00.091  TrackedPlayer.setTracks(): got 3 tracks
    07:59:00.094  mpv pid=9282 url="/psp/birds.mp3" vol=0 loops=1
    07:59:31.686  mpv exited: exit status: 0
    07:59:31.686  doStepTrack(): track ended at 30.69 secs
    07:59:31.689  mpv pid=9300 url="http://liveradio.swr.de/…/play.mp3" vol=44
    08:00:00.083  Alarm.step(): ringing Daily at 8:00
    08:00:00.084  Alarm.ringAlarm() Daily at 8:00
    08:00:00.085  AlarmSet.stopAlarmsExcept(): cancelling Sep 01 2026,  7:59
    08:00:00.086  Alarm.stopAlarm() Sep 01 2026,  7:59 isCancel:true
    08:00:00.087  Alarm.stopAlarmSoundContinuous(): Sep 01 2026,  7:59
    08:00:00.087  MusicPlayer.stopMusic() → doStopTrack → mpv killed
    08:00:00.108  Alarm.restoreSoundSettings(): restoring volume:16 mute:false

The chain is entirely the panel's own, and it is faithful to the original
firmware: `Alarm.ringAlarm` (F2:11182) opens with
`_alarmSet.stopAlarmsExcept(this)`, and `stopAlarmsExcept` (F2:12039) calls
`stopAlarm(true)` on every *other* ringing alarm. `"Daily at 8:00"` is
`type="none" arg="None" action="nightmode" action_param="off"
auto_dismiss="1" enabled="1" time="480"` — a **silent** alarm whose only job
is to leave night mode. It makes no sound of its own (its ringAlarm branch is
autoDismiss + TYPE_NONE → `doPreAction` → `stopAlarm`, F2:11184-11191), yet it
still runs `stopAlarmsExcept` first and so kills the audio alarm that has been
ringing since 07:59. Two alarms one minute apart, and the silent one wins.

Latency measured, no longer inferred: the cancel lands **85 ms** after
08:00:00.000, exactly as `RING_WINDOW` (F2:10186) predicts.

**Both open questions from the previous blocks dissolve.**

1. *Why did mpv exit 75 s into SWR3?* It did not exit — it was killed, at
   08:00:00.087, by our own `stop()` under `doStopTrack`. The 2026-08-28
   "+75 s" was reconstructed from rtkit RT-thread grants, and that
   reconstruction's own recorded hole (an mpv that never opens an audio device
   leaves no line) is what made it look like a 47 s discrepancy. Nothing was
   wrong with the exit.
2. *Why did nothing play for the next 8 m 42 s?* Because the alarm had been
   cancelled and nothing was supposed to play. There is no stuck-but-silent
   mpv and no missed track supervision; the leading hypothesis recorded on
   2026-08-28 (a live mpv that our `poll_state` reports as PLAYING forever) is
   **refuted** — it never happened.

Also settled, and both exonerated:

- **The stream and the 8 o'clock news are innocent.** After Jan restarted the
  entry by hand, SWR3 played unbroken from 08:00:51 to 08:17:15 — 16 min 24 s
  straight through the entire news bulletin, no `doStepTrack` intervention, no
  track death, ending only when he stopped it.
- **The 08:00:19 "replay" is Jan's own.** `_bent() -> 1` at 08:00:14
  (`BendTapper.onBend`), the control panel opens, Music, and at 08:00:19
  `MusicPlayer.resume(): resuming from <stream …Birds + SWR3/>`. Nothing
  restarted itself.

Side finding, same trace, worth knowing: the cancel runs
`Alarm.restoreSoundSettings()`, which puts the system volume back to its
pre-alarm value — 44 → 16 here. So the manual resume at 08:00:19 played at
volume 16 while the alarm had been at 44, which is why a restart after a
cancelled alarm sounds quiet.

The full consumer list for `stopAlarmsExcept`, since any remedy touches it:

| site | caller | argument | reachable on our stack |
|------|--------|----------|------------------------|
| F2:11182 | `Alarm.ringAlarm` | `this` | **yes — this is the path that bites** |
| F2:12033 | `AlarmSet.step`, periodic reload | `undefined` | no — the interval is `ONE_YEAR` without the `alarmReloadInterval` FlashVar (F2:11784) and our launcher passes only `-PlocalCache=1` |
| F2:12238 | `AlarmSet.gotEvent("reload")` | `undefined` | fed by `ExtendedEvents.AlarmPlayer` (F2:12182); we drive no such event |
| F2:12244 | `AlarmSet.gotEvent("load")` | `undefined` | same |

Options for a remedy, none chosen (Jan's call):

- **A — configuration only, no code.** Move `"Daily at 8:00"` out of the way
  (before the wake alarm, or after its `duration="20"` window ends at 08:19),
  or disable it. Zero risk, faithful, but the owner has to remember the
  collision every time a wake alarm is set near a nightmode alarm.
- **B — fork fix: a silent alarm must not cancel a sounding one.** Prototype
  surgery on `AlarmSet.stopAlarmsExcept` (or on the `ringAlarm` call site) so
  an alarm whose `_type == Alarm.TYPE_NONE` (F2:10170) skips the cancel. The
  guard exists to keep two *sounding* alarms from overlapping; an alarm that
  makes no sound has nothing to protect. Same class of one-shot VM surgery as
  `empty_channel.rs` and `intro.rs`, and it is a deliberate deviation from
  stock behaviour around alarms, so it needs saying out loud in the docs.
- **C — leave it.** It is what a real chumby did.

Box state unchanged by this session: chumby-pi-3, 0.9.5, `RUST_LOG` still
active in `/etc/default/chumby-player`, `/psp/list.m3u` still ending in a
newline (`setTracks(): got 3 tracks` again today — the phantom empty track is
present and is *not* implicated in the failure). Nothing was written to the
device. Revert the `RUST_LOG` line once a remedy is settled.

Update 2026-09-01, 09:55 — remedy CHOSEN (option B), fix deferred to the next
session for a clean context. Handoff:

**The fix.** One-shot AVM prototype surgery, same pattern as
`empty_channel.rs`/`intro.rs`. Recommended hook site: **`Alarm.ringAlarm`**
(F2:11178). Wrap it so that when `this._type == Alarm.TYPE_NONE` (F2:10170,
value `"none"`) it does **not** run the opening `_alarmSet.stopAlarmsExcept(this)`
(F2:11182) — a silent nightmode/none alarm then does its night-mode side
effect without silencing a sounding alarm. Keep every other effect of the
TYPE_NONE branch intact (autoDismiss → doPreAction → stopAlarm,
F2:11184-11191); only that one cancel line is skipped. This site is preferred
over guarding `stopAlarmsExcept` itself because the canceller's identity is
not passed to it (`anAlarm` is the survivor), and because `ringAlarm`/F2:11182
is the single reachable caller — the other three `stopAlarmsExcept` callers
(F2:12033, 12238, 12244) pass `undefined` and are unreachable on our stack
(`ExtendedEvents.AlarmPlayer` drives none of them, and the periodic reload's
interval is ONE_YEAR without the `alarmReloadInterval` FlashVar).

**Where it goes.** New file in the fork under `core/src/chumby/` (e.g.
`alarm_guard.rs`), wired in `mod.rs`, one-shot retry until frame 2 defines
`Alarm.prototype.ringAlarm` — mirror `empty_channel.rs`. Before coding, grep
every consumer of `ringAlarm`, `stopAlarmsExcept` and `TYPE_NONE` and list
them with a verdict (CLAUDE.md consumer-list rule). It is a deliberate
deviation from stock alarm behaviour, so record it in the fork's
requirements.md (amendment near FR13) and design.md (the alarm chain / §9),
per the docs-split rules.

**Verify.** Desktop first: two alarms one minute apart — the later
`type="none" action="nightmode"`, the earlier an audio stream — the audio must
survive the silent alarm's ring. Then on chumby-pi-3, the real 07:59+08:00
pairing this issue reproduced. Success signal: **no** `AlarmSet.stopAlarmsExcept():
cancelling` at ~08:00 while the audio alarm keeps its mpv alive.

**Recorded, not part of this fix:** the cancel also runs `restoreSoundSettings()`
(volume 44→16 on 2026-09-01), so a manual resume after any cancel plays quiet.
Moot once the cancel no longer fires for a silent alarm.

Box left for the next session: chumby-pi-3 (192.168.42.24), 0.9.5, verbose
`RUST_LOG` active in `/etc/default/chumby-player`
(backup `.bak-preverbose`), `/psp/list.m3u` still ends in a newline. When the
fix lands and is verified on the device, revert the `RUST_LOG` line and close
this issue.

Update 2026-09-01, 11:10 — option B built and verified on the desktop; the
device run is what remains.

Jan confirmed option B this session. The fix is fork commit 75f3acf28,
`core/src/chumby/alarm_guard.rs`: a one-shot wrapper on
`AlarmSet.prototype.stopAlarmsExcept` (F2:12039) that returns without
cancelling when the argument's `_type` is `Alarm.TYPE_NONE` (F2:10170), and
delegates to the parked original otherwise. Full rationale, consumer list and
evidence are in the fork's `claude/issues.md` #10 — the player record, not
repeated here.

Two decisions differ from the 09:55 handoff, both deliberate:

- **Hook site.** The handoff recommended wrapping `Alarm.ringAlarm` because
  "the canceller's identity is not passed to `stopAlarmsExcept`". That is
  wrong: at F2:11182 the argument is `this`, the ringing alarm, so canceller
  and survivor are the same object on the only reachable path. Wrapping
  `stopAlarmsExcept` is stateless; wrapping `ringAlarm` would require either
  reimplementing its 50-line body or shadowing `stopAlarmsExcept` with a
  no-op and restoring it, where a missed restore disables cancelling for every
  alarm. Jan chose the `stopAlarmsExcept` site.
- **Where it is documented.** The handoff said to amend the fork's
  `requirements.md` (FR13) and `design.md` (§9). Those live in
  `claude-docs/`, which fork commit 85d3447bc froze: *"claude-docs/ is the
  frozen historical record. Live notes go in claude/."* The deviation is
  recorded in the fork's `claude/issues.md` #10 instead.

Desktop verification, same binary with the guard compiled out and in, two
`when="once"` alarms a minute apart — earlier `type="beep" auto_dismiss="0"`,
later `type="none" action="nightmode" auto_dismiss="1"`, the shape of the
device failure. Guard off reproduced it off-device for the first time
(`stopAlarmsExcept(): cancelling`, `isCancel:true`, `restoreSoundSettings():
volume:60`); guard on shows none of those, the surviving alarm is never
mentioned again after it rings, and the silent alarm still does its own work
(`night mode off`, widget mode, `post_alarm_action` probes, `saveAlarms`).

Still to do, and the reason this issue stays open:
1. Build and deploy to chumby-pi-3 (192.168.42.24), gitlink bumped to
   75f3acf28 on this repo's dev.
2. Re-run the real pairing: a `when="once"` audio alarm at 07:59 with
   `"Daily at 8:00"` enabled. Success signal: `silent alarm "Daily at 8:00"
   rang — not cancelling ringing alarms` at 08:00, **no**
   `stopAlarmsExcept(): cancelling`, and mpv still alive afterwards.
3. Then revert the `RUST_LOG` line in `/etc/default/chumby-player` (backup at
   `.bak-preverbose`) and close this issue.

Box state unchanged: 0.9.5, verbose `RUST_LOG` active, `/psp/list.m3u` still
ends in a newline.
