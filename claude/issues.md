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

Outstanding, blocked on the test Pi (192.168.42.51, offline this session):
0. Whether Plymouth is installed/enabled at all on the target Lite image
   today — the observed white screen (vs. full Raspbian's rainbow splash)
   suggests it may not be; may need `plymouth` enabling steps beyond what
   `Depends: plymouth` alone gets you (apt install pulls it in, but Lite's
   cmdline.txt may lack `splash`/`quiet` and the relevant units may not be
   wired the way they are on the Full image).
6. Full on-device pass: fresh Lite install and an existing config — theme
   actually renders instead of white screen, duration/timing matches
   opening.swf, `plymouth quit`-to-cage handoff has no black gap/flicker/
   DRM-master race, and a box that never runs chumby-download-firmware
   still boots cleanly with no half-built theme active.
