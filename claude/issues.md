# Issues

One block per open issue: Number, Timestamp, Title, Status, Description.
Appliance-side issues only; player issues live in ruffle/claude/issues.md.
Settled issues are one line each in the register at the end — the full blocks
are in git history, named there. Numbers are never reused.

---

Number: 1
Timestamp: 2026-07-17, 00:15 (moved 2026-07-17, 18:00)
Title: Check chumby_accel.c applicability to the Pi kernel.
Status: open — moved to the hardware-ideas branch with hardware/
Description: All PCB/hardware work (this issue, #2, and hardware/) now
lives on the hardware-ideas branch, kept out of dev's tree. This entry stays
here only as a pointer; the full text (unchanged) is on hardware-ideas.
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
Status: open — moved to the hardware-ideas branch with hardware/
Description: See issue 1's note — this issue and hardware/breakout-tryout/
now live on hardware-ideas, unchanged. Reframe hardware/chumby-hat/ from a Pi HAT to a passive breakout
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

Number: 22
Timestamp: 2026-09-24, 10:50
Title: The panel showed WLAN as good while the WLAN was not usable.
Status: open — cause unknown; mitigated by NetworkManager's connectivity check
(a real reachability test, not a default-route guess) plus the supervisor's
panel restart when connectivity returns, which refreshes the Info screen's
boot-time snapshot (`claude/watchdog-plan.md` R3/R7/R8, verified on
chumby-pi-3 2026-09-24)
Description: Jan, 2026-09-24: roughly 1-2 weeks earlier the panel's WLAN
information looked good while the WLAN could not be used. No log survived —
the journal on chumby-pi-3 is volatile. Why the display cannot tell the
difference: link quality is signal strength and nothing more
(`/proc/net/wireless` `link`, rescaled to percent), `connected="1"` means only
that the default route is on a wireless interface, and IP, netmask, gateway,
DNS and SSID are read once in the panel's boot chain and never refreshed.
Player side: fork issue 26.
Next time it happens, before anything is restarted: `ip addr`, `ip route`,
`cat /proc/net/wireless`, `iw dev wlan0 link`, ping the gateway and a name,
and `journalctl --since "<absolute time>"` while the lines still exist (this
box rejects relative timestamps).

---

Number: 23
Timestamp: 2026-09-30, 01:52
Title: Idea: rebuild the hardware chumby's control panel in Rust, without Flash.
Status: parked — idea only, Jan: "do not work on this for the time being"
Description: Target is the classic chumby (i.MX21, ARMv5TE, Linux 2.6.16), not
the Pi appliance. Nothing built or tested. The full scoping — the memory
estimate against Flash's 23 MB RSS, the toolkit survey (Slint recommended, no
GPU and no X, single-threaded because 2.6.16 spins on contended waits), the
panel's size from the decompile, and the three widget architectures — was last
present in baefaeb and stays in git history. Do not work on it until Jan says.

---

Number: 29
Timestamp: 2026-10-31, 23:55
Title: rust-remote's skin store must be a widget data store, not a Lys domain.
Status: open — documented only, not checked or implemented (Jan, 2026-10-31)
Description: The Lys widget's clock skin (lcars / deco / funkis, chosen by a
tap in the lower left corner) is kept by chumby-rust-remote so that it
survives widget changes, the control panel and reboots (`skin`,
`skin NAME`, file under /psp; chumby-ikea-lys PLAN.md, "Skins"). Jan's
note: if rust-remote saves skins, Lys must not corrupt its domain —
storing must be open to other widgets on the same interface, and for the
remote it is just data from a widget (an opaque key/value per widget, no
knowledge of what a skin is). To do: shape the store that way (a
widget-scoped key/value store on the XMLSocket interface, the skin one
entry of it) and move the skin names out of the service. Not scoped;
Jan decides when.

---

Number: 32
Timestamp: 2026-11-05, 09:00
Title: The supervisor's restart at 04:22 woke Jan — night mode lost, the
channel's first widget bright, and the reason gone with the journal.
Status: open — recorded only, no fix scoped (Jan, 2026-11-05)
Description: Jan woke between 5 and 6 on 2026-11-05 to chumby-pi-3 showing
the native clock widget, bright; Lys had left its night mode. No alarm was
involved: the only enabled alarms in
/var/lib/chumby/fixtures/rootfs/psp/alarms are "Daily at 7:55"
(action=nightmode off, time=475) and "Daily at 23:00" (nightmode on,
time=1380).
What happened: the supervisor restarted the player child at 04:22:22.
`ps -eo lstart` — ruffle_desktop pid 55418 started Thu Nov 5 04:22:22,
chumby-supervisor pid 19803 and cage pid 19789 since 2026-11-03 23:18:28,
systemd `NRestarts=0` for chumby-player; the panel's own startup writes
agree (/tmp/controlpanelversion and /psp/hostname mtime 04:22:23.8). A
fresh panel has night mode off, day brightness, and loads the channel's
first widget: chumbypi-channel is Chumby Analog Clock (white), RoboClock,
Space Clock, Lys (/tmp/widget_names) — the bright clock Jan saw. It stayed
there until the 7:55 alarm: `WARN tiny_skia::painter: path stroking failed`
runs at 4320 lines/minute (72/s) from 07:47 to 07:55:00, then stops.
Why the supervisor restarted is NOT established and cannot be recovered.
supervisor/src/policy.rs:119-132 has three triggers — Clock (step >15 s),
Network (NM connectivity back to full after a drop), ChumbyCom (off here:
access_chumby_com = 0 in /etc/chumby-player/player.toml). The reason is
logged (supervisor/src/main.rs:221-228) but journald on this box is
volatile (/var/log/journal empty, `--list-boots` retains only the current
boot from 07:47:58) and the 16.6 MB of stroking warnings rotated the night
away. A crash-restart (policy.rs Backoff) would look the same from the
outside; systemd-coredump is not installed, so there is no core either.
Circumstantial only: chumby-pi-3 is on wifi (wlan0, eth0 unavailable) and
its brcmfmac logged SDIO drops on other nights (dmesg: 2026-10-30 01:32 and
06:18, 2026-11-01 11:21) — a blip returning to `full` is exactly the
Network trigger, but nothing ties one to 04:22.
Follow-ups offered and declined for now (Jan, 2026-11-05): defer a
triggered restart while night mode or a widget's night claim is active;
land on Lys after a restart (channel order, or a rust-remote re-jump to the
claimant); silence or rate-limit the tiny-skia stroking warning and make
journald persistent, so the next occurrence is diagnosable.

Update, 2026-11-05 (Jan's hypothesis, confirmed on the box): the night mode
lost at 04:22 was never the panel's — it was Lys's, and nothing on disk
records a widget-held night. Read-only off chumby-pi-3 (up 7 d 23 h, so the
tmpfs and its mtimes reach back to 2026-10-28):
- /tmp/chumby-panel-tmp/nightmode holds `0`, mtime 2026-11-04
  23:58:15.864 — 4 h 24 min before the restart, and no write after it (the
  day branch of F2:359 writes nothing).
- /psp/music_timer_duration holds `15`, mtime 2026-11-04 23:58:14.880 —
  0.98 s earlier: the sleep timer set from the Lys widget
  (rust-remote/src/main.rs:210-221).
- /psp/dimlevel does not exist on this box, so F2:359's second trigger
  (`== "2"`) is false too and restoreDimFromFile gives int("") =
  DIM_FULL_ON. (The `/psp/dimlevel` seed at fork fixture.rs:778 is a unit
  test's, not the shipped tree's.)
What is forced by the code, and what is not (Jan, 2026-11-05: "we
speculate a lot, because logs were not available" — correcting an earlier
version of this entry, which told the chain below as established):
- Forced: a `0` in that file means the panel's restoreDisplay ran
  (F2:19757 is its only writer of `0`), so at 23:58:15.864 the panel left,
  or believed it left, its night mode. On this box the only triggers
  available are a touch on the dim night screen (F2:19606-19628, which
  calls restoreDisplay and goes to `idle`) or a NightMode-off soft event
  (F6:119-122) — there is no bend sensor and no keyboard, and `bend down`
  only reaches the player over the control channel (fork
  core/src/chumby/input.rs:93).
- Forced: the sleep command at 23:58:14.880 came from the Lys widget, the
  only sender of `sleep` (chumby-ikea-lys widget/frame_1/08-night.as), so
  Lys was loaded and running then. A widget keeps running behind the
  panel's night screen, which is why its claim is renewed in every mode
  (08-night.as:53-62).
- NOT established: whether the `0` came from rust-remote's handover
  (watch() sees `night && claimed`, sends NightMode off, keeps
  HANDOFF_NIGHT — main.rs:566-575) or from a touch, and whether the "Daily
  at 23:00" alarm's night mode was still running at 23:58. The journal that
  would say is gone. Both readings end in the same state, which is the one
  that matters: `0` on disk, and the night held by Lys (its own night
  screen plus the 15 s `screen` lease) — the designed contract
  (main.rs:248-252; 08-night.as:1-7), under which a panel night mode cannot
  coexist with Lys on screen.
Why nothing restored it at 04:22:22: the panel read `0` → startupMode(),
day brightness, and widget mode's first widget (/tmp/widget_names line 1 =
Chumby Analog Clock (white)); Lys was unloaded, so no claim renewed;
rust-remote (pid 46495, never restarted, up since 2026-11-04 21:54) reads
`night()` from the panel's `0`, let the claim and the lease lapse and sent
bright. Its HANDOFF_NIGHT is deliverable only on a claimant's next reply
(main.rs:258-264) and is dropped as stale at the next fresh claim
(main.rs:254-256).
So the loss is structural, not a lost journal: while a widget holds the
night, the only night state in the system is in the widget's RAM. Why the
supervisor restarted is still unrecoverable — the journal now begins at
07:48:40, the stroking flood having rotated the rest away.
Constraint on the fix (Jan's direction is to persist it): the record must
NOT be `/tmp/nightmode` set to `1` while a widget owns the night. Frame 1
of DefineSprite_1766 is the `idle` label (tag-dump 7993-7995) and re-reads
that file on every return to widget mode — widgetMode()
(DefineSprite_1766/frame_1:1-4), the alarm path F2:11124, exitNightMode's
gotoAndStop("idle") F2:19622 — so a `1` there would evict Lys into the
panel's night screen mid-evening. The persistence belongs to rust-remote,
which re-establishes the night when it sees a fresh panel.
Status: cause of the night-mode loss established; the restart's own trigger
remains unknown. No fix scoped yet.

Fix plan (Jan, 2026-11-05 — semantics his, decided in this order):
The record is `/tmp/nightmode`, one byte, on the tmpfs as today, so a boot
still starts by day (R6):
- `0` no night. Written by the panel (F2:19757) and by rust-remote when
  Lys's night ends.
- `1` panel-owned night. The panel's own value (F2:19751), untouched by
  anyone else. It already survives a player restart (F2:359), so it needs
  no work.
- `2` Lys-owned night. rust-remote's value, and inert to the panel: both
  panel reads are `== "1"` string compares (F2:359,
  DefineSprite_1766/frame_1:149), with no int() anywhere — unlike
  /psp/dimlevel, which IS int()-parsed (F2:9060). A `1` could not be used
  for this: exitNightMode ends at gotoAndStop("idle") (F2:19622), the idle
  frame re-reads the file and a `1` would pull the panel back into its
  night screen, which rust-remote would hand over again — a flip-flop at
  POLL = 2 s.
rust-remote needs no widget change to keep it: Lys's showNight() sends
`screen dim` and endNight() sends `screen bright`
(chumby-ikea-lys widget/frame_1/08-night.as:76-94), and `claim` + `screen`
is Lys alone — yoga only ever sends `awake 2`
(yoga-widget widget/frame_1/07-screens.as:126).

Step 1 — rust-remote keeps the record. night() returns Day | Panel | Widget
instead of a bool; the handover branch (night && (awake || claimed), 
main.rs:566-575) stays as it is for Panel, and Widget behaves like Day for
the screen path, since Lys's own lease governs the backlight. Writes: `2`
when a claiming widget sends `screen dim`/`screen off`, re-asserted on each
renewal so a lost write heals within 5 s; `0` on `screen bright`, on a
handoff=day alarm, and when the claim lapses (which is what navigating away
from Lys does today — the lease expires and the screen goes bright).
rust-remote never writes `1` and never writes `0` over a `1` it did not
cause, so a panel night from the 23:00 alarm is never clobbered.

Step 2 — the panel start is pushed, not polled. The supervisor owns both
ends of a restart: it asks over the FIFO (supervisor/src/main.rs:233-239,
the player yields when idle — fork core/src/chumby/restart.rs) and it
spawns the replacement. After the spawn it connects to 127.0.0.1:8090 and
sends `panel-start <reason>`, the reason from its own state
(`requested:Clock,Network`, or `unexpected` for a crash or the panel's own
quit). rust-remote's handle() gains the command: record `2` → send
NightMode on, nothing otherwise. That is the whole restore (Jan: "it only
needs to know, if lys was in night mode before") — the panel's night screen
holds the dark, and if Lys is still the widget behind it, its claim
triggers the existing handover and the night is Lys's again within a poll.
A boot needs no case of its own: the tmpfs is empty, so the record is `0`.
Ordering: the notification arrives within a second of the spawn, well
inside the 15 s claim lapse that would otherwise clear the record.

Step 3 — a night holds a requested restart (Jan: "resetting the panel came
with good intentions, but did more harm than good"). The fork's restart.rs
already refuses to yield while an audio alarm rings or the screen was
pressed within TAP_QUIET — Hold::Alarm, Hold::Tap. A third hold,
Hold::Night (the record is `1` or `2`), puts the decision in the same
place, the process that holds the state. A triggered restart then waits for
the night to end instead of interrupting it; a crash still restarts at
once, which steps 1 and 2 are the backstop for.

Step 4 — DROPPED (Jan, 2026-11-05). It was to make the Pi's journal keep
every restart and its reason: an /etc/systemd/journald.conf.d/ override for
Storage=persistent against Raspberry Pi OS's own
40-rpi-volatile-storage.conf, an interim RUST_LOG mute of the tiny-skia
stroking flood, and a separate issue for the stroking defect itself. None
of it is wanted; no issue was opened for the stroking warning either. The
reason a restart happened therefore stays unrecoverable after the fact.

Verification: PC runs for steps 1-3 (fixtures, unit tests in the existing
style, the handover and the restore driven over the control channel), then
one device round on chumby-pi-3 at Jan's time — a real Lys night, a
SIGKILL of the player child for the crash path, and the journal surviving
it.

State spec: PARKED, and its two files deleted in the 2026-11-06
housekeeping (`claude/nightmode-spec.md`, `claude/nightmode-spec-broken.md`,
last present in c9142db). It was the automaton Jan asked for — three states
over `/tmp/nightmode`, mermaid in markdown — and he rejected the model on
2026-11-05: too much friction, and these three modelling questions come
first, probably with proper UML rather than an ad-hoc state diagram (his
words, condensed):
1. "Unload" is a technical event, not a domain event. The spec leaned on Lys
   being loaded or unloaded, which is a fact about the player, not about the
   appliance's behaviour; the domain concept behind it has to be named and
   defined before it can appear in a machine.
2. "The panel enters night mode" does not say which panel, or which night
   mode — the label collapses several different things.
3. There are probably more states than three. "Lys active" may well be a
   state of its own; the model carried it as an unstated condition, which is
   why edges kept needing guards, and why the restart out of Lys night
   behaved unlike every other edge.
Start from the domain, not from the file: what are the appliance's night
states as a user would name them, which events are domain events, and only
then which byte encodes what. Do not revive the automaton from git.

Where the code stands: nothing implemented, deployed or pushed. Step 1's
half-finished code was discarded at Jan's request (2026-11-05) — rust-remote
is back to `b627f0e` on main with a clean tree and the WIP branch deleted;
its reflog holds it as `a35e507` for about 90 days, but it implements the
rejected model and should be rewritten rather than revived.

Panel touchpoints found while specifying, kept here because they are
expensive to re-find and independent of the modelling:
- the panel enters night mode — a nightmode-on alarm (F2:11094-11108), the
  Night button (`DefineSprite_1766/frame_2:43-56`), or a NightMode soft
  event (F6:107-110; off at F6:119-122) — and writes `1` at F2:19751.
- the panel leaves night mode — a touch, key or bend on the night screen
  (F2:19606-19628, 19659, 19729) — and writes `0` at F2:19757.
- Lys announces its own night to the remote with `screen dim`/`screen off`
  and `screen bright` (chumby-ikea-lys `widget/frame_1/08-night.as:76-94`);
  a reloaded Lys starts `night = false` (`08-night.as:10`) and its claim
  handler is idempotent (`08-night.as:56-61`), so the next renewal
  re-establishes the night.

---

Number: 33
Timestamp: 2026-11-06, 09:30
Title: Housekeeping — fewer documents, so the project is easier to understand.
Status: closed 2026-11-06 — all four steps done
Description: Jan's ask: reduce the complexity of understanding this project.
Candidates he named: closed issues, loose ends, short tests, ditched features.
Decisions (interview, 2026-11-06):
- Delete whole files: the fork's three finished plan documents, both
  night-mode spec files, `rust-remote/research/` (the dropped
  third-framebuffer study and, on the second round, the never-adopted FastCGI
  experiment), and `docs/index.adoc` with CI's render step.
- Both `issues.md` files: **one line per settled issue, no exceptions.** The
  panel facts the closed blocks carried stay in git history, which Jan
  accepted when choosing this over keeping the fact-bearing bodies.
- `claude/watchdog-plan.md` stays as it is: it is the only requirements and
  design document for the shipped supervisor, and `supervisor/src/policy.rs:3`,
  `supervisor/src/main.rs:2` and the fork's `restart.rs:3` cite it by
  R-number.
- Adjudicated: close #21 and #31, keep #22 open as the unknown cause with its
  probe list, compress #18 and #23 to pointers.
- A dangling pointer in the frozen `claude-docs` is fixed rather than left
  (Jan): `development.md:815` named the deleted fork plan and now names
  `render/tiny_skia/`. `design.md`'s distribution paragraph got the same
  treatment for `docs/index.adoc` once the sweep found it.

Step 1 — whole files (chumby-pi baefaeb, fork 4bc40bd89, rust-remote 67685f2
and 297367a). 1012 lines of documents went in the first round, 843 of them in
the fork. Two facts that lived only in the deleted fork plan moved into the
code they describe, because the fork's frozen `claude-docs` mention tiny-skia
nowhere: the measured spike numbers and the viewport-vs-stage lever into
`render/tiny_skia/src/lib.rs`, and why the present path is shared memory
rather than a wgpu blit into `desktop/src/gui/controller.rs`. The night-mode
spec's surviving content — Jan's three modelling questions, where the
discarded code stands, and the panel touchpoints found while specifying — is
in #32. `cargo check` clean for `ruffle_render_tiny_skia` and
`ruffle_desktop`; nothing but comments changed there.
Checked before deleting `docs/index.adoc`: it was an empty asciidoc skeleton
(three headings, no content) published as the project's landing page on every
push to main, and every live reference — `docs/setup.md:82,84`,
`pkg/apt/index.html:20,22`, `claude-docs/design.md:255` — points at `/apt`,
never at the root. The site root will 404 after the next push; `/apt` is
untouched.

Step 2 — this file (30b1d43). 2117 lines → 508: the twenty settled issues
became the register at the end, #22 and #23 were compressed in place, and #1,
#2, #7, #8, #29 and #32 are unchanged — checked byte-identical against
baefaeb rather than assumed.

Step 3 — the fork's `claude/issues.md` (fork a77e42692, its issue 32):
896 lines → 443, same treatment, with 1, 2, 4, 5, 6 and 8 verified unchanged.
`ruffle/claude/` is now the register plus `patch-surface.md`.

Step 4 — this record, the gitlink bump, and a sweep for pointers into
deleted files: one was left, `claude-docs/design.md`'s docs-site paragraph,
fixed as above. `git grep` over both repos finds no other.

What the pass removed, in lines of document: 2019 here, 1418 in the fork, 610
in rust-remote — 4047 deleted against 351 written, and twelve files gone.
Nothing was pushed; main, chumby and the two release branches are untouched.

---

## The register of settled issues

One line each: the outcome, and where the result lives. The full blocks — with
their measurements, device traces and decompile line numbers — were last
present in baefaeb. Numbers 14-17 belong to the Dash (see #18) and are not
reused.

- **#3** Plymouth boot animation (closed 2026-10-20, good enough — Jan): the
  theme ships as `pkg/chumby-player/plymouth-theme-chumby/`, its 132 frames
  are rasterised on the box by `chumby-download-firmware`'s
  `install_boot_theme`, and a backgrounded `plymouth quit` in
  `chumby-player-run` hands over to cage. The handoff gap and alt_opening's
  audio are accepted as they are.
- **#4** Find a display — 3.5", 4:3, brightness, 12 fps (closed 2026-10-20 —
  Jan): the 5" DSI panel is the answer; brightness works there (#5), and 3.5"
  and 4:3 were given up with its 1024x600 (#12). On SPI the honest clean
  ceiling is 8.4 fps, reached by pinning `core_freq_min=400` alone — every
  clock above the overlay's nominal 24 MHz visibly corrupts the picture.
- **#5** Backlight brightness on the 5" DSI box (closed 2026-08-24): FR16 is
  satisfied on hardware; the deb's `90-chumby-backlight.rules` does fire on a
  bare install, so `pi` writes `/sys/class/backlight/10-0045` without sudo.
- **#6** CI shipped the player with Ruffle's mock clock (closed, device-
  verified 2026-08-24): cargo unified `exporter`'s `deterministic` and
  `default_font` into `ruffle_desktop`. CI now builds the two binaries in
  separate invocations, and a guard step fails the run if either feature
  resolves into the player again.
- **#9** Direct stream entries in `/psp/url_streams` (fixed 2026-08-23):
  `mimetype="audio/mpeg"` for a direct stream, `audio/x-mpegurl` only when the
  `url` really is a playlist.
- **#10** Playlist entries in an m3u (fixed 2026-08-24): the panel drops
  relative entries silently; absolute chumby paths (`/psp/birds.mp3`) survive
  and the host maps them through the virtual rootfs.
- **#11** The Tagesschau widget's video source (closed 2026-08-24): external —
  its hardcoded `welttheorie.de` FLV answers 502; the widget was deleted from
  the box.
- **#12** Black bars around a widget on the 1024x600 DSI panel (closed
  2026-08-24): Jan chose to leave the scaling alone. Two causes, both measured:
  320x240 under show-all scales 2.5x to 800x600, and the widget's own artwork
  carries 15 px of black at the top and 14 at the bottom.
- **#13** A "Birds + SWR3" alarm went silent 100 s in (won't fix 2026-11-03 —
  Jan: "this exists on the chumby, too. We do not need to deviate"): the
  silent 08:00 nightmode alarm cancels the still-ringing 07:59 alarm through
  the panel's own `stopAlarmsExcept`, traced to the millisecond on
  chumby-pi-3 2026-09-01. The guard that suppressed it is removed again in
  0.9.10 (fork #10). What remains is the configuration workaround — keep a
  nightmode alarm out of a wake alarm's ring window.
- **#18** Sony Dash panel support (stale 2026-10-20 — out of scope for the
  foreseeable future, Jan): everything Dash lives on the `sony-dash` branch of
  both repos and nowhere else, including appliance issues 14-17 and the fork's
  11-21. dev carries none of it. Anyone resuming reads the branch, not an entry.
- **#19** 0.9.6, the backlight cap on chumby-pi-3 (closed 2026-10-20):
  `brightness_cap = 25` in `player.toml`, accepted by Jan.
- **#20** 0.9.7, one volume ceiling (closed 2026-10-20): `volume_cap = 50`,
  accepted by Jan, with the backup-alarm Klaxon brought under it.
- **#21** After a cold boot the nightmode alarms never ring (closed
  2026-11-06): the panel computes each alarm's time once, from the wall clock
  at player start, and nothing in it notices a later NTP step — on 2026-09-22
  that clock was 5 d 15 h slow. Remedied by `chumby-supervisor`, which
  restarts the panel on a clock step > 15 s (`claude/watchdog-plan.md` R9);
  shipped in 0.9.8, device-verified on chumby-pi-3 2026-09-24 with a simulated
  boot on a slow clock. Player side: fork #25.
- **#24** Look-and-feel check, the chumby 8 panel under Ruffle (done
  2026-10-13 — Jan has seen enough): 800x600, SWF v8, 12 fps; flat and light,
  nearer the Dash than the classic; nothing built. Where its panel comes from:
  `http://files.chumby.com/resources/chumby8/1-8-2/update.zip` (firmware 1.8
  build 3883, panel at `usr/widgets/controlpanel.swf`), and the newest,
  5.0.38b2, from `https://www.chumby.com/xml/controlpanel?id=…&hw=10.6&sw=1.8.2&fw=1.8.2&lang=en-us&cnplatform=silvermoon&configname=opus`
  — the server needs `cnplatform`/`configname`, `hw` alone is ignored.
- **#25** Housekeeping before ikea-lys (closed 2026-10-20): all four steps
  done; `ROADMAP.md`, `CLAUDE.old.md` and the VNC plan went, the launcher
  resolves everything from `$STATE`, and CI lost its dead PR triggers. The
  fork half is fork #30.
- **#26** Local widget services on the Pi — the stick's debugchumby hook and
  the socket allow-list (closed 2026-11-03): superseded by #31. The hook
  emulation shipped in 0.9.9 and was replaced in 0.9.10 by
  `CHUMBY_SOCKET_ALLOW`, `CHUMBY_LETTERBOX` and the `conf.d` / `profile.d`
  drop-ins. One finding worth keeping: the panel takes the FIRST `profile.xml`
  of its search order, so a stick tree with one hides the fixtures' own.
- **#27** Widget navigation on the Pi (closed 2026-11-03): the fork gained
  `ExtendedEvents` and a lossless control FIFO (fork #31),
  chumby-rust-remote writes `event KIND VALUE` to `CHUMBY_CTL`, and it runs on
  aarch64 — so the drop-down, "Nacht" and the Lys radio work there.
- **#28** The Pi's side bars (closed 2026-11-03): superseded — no host-drawn
  panel. The player runs `--letterbox off --background-color 000000` and the
  Lys panel skin lays itself out 410 stage px wide; taps in the margins reach
  the widget.
- **#30** The Lys panel skin on the Pi (closed 2026-11-03): deployed to
  chumby-pi-3, its radio plays through the panel's own player over soft
  events, and Jan's end-user test passed ("test fine").
- **#31** The widget services become private debs from a LAN apt repo (closed
  2026-11-06): steps 0-4 done. `pkg/publish-private-apt.sh` publishes
  chumby-rust-remote and chumby-ikea-lys to the private repo, reading HOST,
  USER, DIR, URL and KEY_ID from `~/.config/chumby-apt/private.env` — no value
  of it is in this repo. The public launcher lost the hook and gained the two
  switches plus the two drop-in directories; 0.9.10 shipped 2026-11-04 (main
  `e017c92`, fork pair `d5237b1c1`) and chumby-pi-3 runs it with both private
  debs installed. The chumby keeps its stick and `debugchumby`.
