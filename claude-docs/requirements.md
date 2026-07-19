# Requirements

What the appliance must do. A Raspberry Pi with a small touchscreen should
behave like a Chumby Classic: it boots into the original control panel,
shows widgets, rings alarms, plays audio, and is configured through the
panel's own screens — except where those screens are meaningless on a Pi.

This repository is the **appliance**: packaging, the kiosk, hardware,
end-user documentation, CI. The player is
[chumby-ruffle](https://github.com/yanosz/chumby-ruffle), a Ruffle fork
pinned here as the `ruffle/` submodule, and it carries its own environment —
the fixtures, the SWF, the run harness — so that player work needs only that
repository. What this one needs to know is how to build the player, how to
launch it, and where its data is; see [design.md](design.md) §2.

---

## 1. Functional requirements

### FR1 — Boot to the panel

Powering the Pi brings up `controlpanel.swf` fullscreen on the TFT with no
login, no desktop, and no manual step. Installing the packages is what turns
a stock Raspberry Pi OS into player mode.

### FR2 — An escape hatch that does not need the screen

SSH always stays up. `systemctl stop chumby-player` leaves player mode once;
`systemctl disable --now chumby-player` leaves it for good. A "magic key"
(bend held ≥5 s, or `chumby-ctl exit-player`) was proposed and is **not
implemented**.

### FR3 — Input without a keyboard or mouse

The device has a resistive touchscreen and nothing else. Touch must work.
The chumby's *bend* gesture — squeezing the body to summon the control-panel
button bar — is raised by a long press on the screen, and by
`chumby-ctl bend` over SSH. A GPIO button (GPIO17, physical pin 11, bridged
to GND on pin 9, via the `gpio-key` overlay) is the designed hardware path;
it is not wired up yet.

### FR4 — Audio out

Alarms ring audibly, streams play. A USB sound card is the intended route; a
different sink must be selectable without rebuilding anything.

### FR5 — The feature scope

The panel has far more screens than this appliance offers. Every screen is
either delivered, or reachable-but-disabled, or unreachable. Nothing is left
as a live control that silently does nothing.

| Area | Status | Notes |
|------|--------|-------|
| Boot path, idle, main button bar, play-trap | **delivered** | the core |
| Built-in offline clock | **delivered** | `builtinclock.swf`, one of the widgets in the generated channel |
| Volume | **delivered** | |
| Alarms — list, editor, ring screen | **delivered** | |
| Backup alarm (dead-man beep) | **delivered** | in-player watcher on `/psp/ifalarm`, missed-alarm boot path included — fork's requirements.md FR13; loudness check on the Pi still owed (§3) |
| Clock / time / timezone | **partly** | 12/24h toggle is live; the timezone picker and the NTP toggle are **disabled** — the Pi OS owns time and time sync |
| Music: My Streams | **delivered** | real network; the stream list is local data, no chumby.com |
| Music: USB / local files | **delivered** | `_getDirectoryEntry` in the player + read-only automount to `/media/chumby-usb`; alarm-from-USB included. Physical-stick pass done 2026-07-11 (hotplug, playback, yank-while-playing) |
| Music: SHOUTcast, blue octy radio, Sleep Sounds | **delivered**, opt-in | alive via the revived chumby.com music proxies (verified live 2026-07-11); hidden unless `access_chumby_com=1` in the owner config `/etc/chumby-player/player.toml`, which also passes exactly their two hosts through — fork's requirements.md FR15 |
| Music: Squeezebox Server | hidden, opt-in | player side complete; Lyrion server behavior unverified and out of scope (2026-07-11) — `enable_lyrion=1` shows it (fork FR15) |
| Music: every other source | hidden / skip | iPod, NOAA and CBS podcasts spliced out by the player (NOAA and CBS confirmed dead on a real chumby, 2026-07-11); FM, MP3tunes, Internode self-hide via their own failing probes |
| Widget channel | **delivered** | exactly one channel, generated at boot from the widgets we ship |
| Channel management (picker, info, add, reload) | **disabled** | needs remote download + registration |
| Widget preview picture on the dashboard | **delivered** | a static per-widget thumbnail |
| Delete / Send / Rate widget | **disabled** | delete can't persist; send and rate are chumby.com social features |
| Info / About | **delivered** | with real network diagnostics |
| Licenses | **delivered** | the original chumby's GPL and LGPL texts, verbatim |
| Geek panel, file browser | **disabled** | redundant next to a Raspberry Pi; the Info screen's π trigger is inert (rule `info-geek`) |
| Intro widget | **delivered** | both entry points: the Info screen's INTRO button (fork's VM-level `playIntro` replacement; rule `info-intro` dropped) and the boot-time tour before the panel (launcher, rcS `start_intro` semantics). On-device pass outstanding, see §3 |
| Brightness, night mode | **player-ready** | fork FR16 shipped 2026-07-13 (sliders onto a kernel backlight, or a `brightness_ctl` executable); on this TFT no backlight exists, so the Settings button stays honestly disabled — see §3 |
| Network setup wizard, touchscreen calibration | **disabled** | the OS owns these; both Settings icons carry a ui-policy rule |
| First-time wizard, activation, safe mode, firmware update | skip | chumby.com and chumby firmware machinery |
| Accept/decline sent widgets, intercom, microphone test | skip | social or pointless here |

"Disabled" means the control renders dimmed and inert, via a rule in the
player's compiled-in UI policy (`ruffle/core/src/chumby/ui-policy.toml`).
"Skip" means the screen is simply never navigated to; its frames stay inert,
and its startup code — which does run — tolerates a failing environment.

### FR6 — Nothing reaches chumby.com

Not at boot, not from a widget, not from CI. The device GUID must not leak.
Registration is *possible* — chumby.com is on life support, not dead — but
is deliberately not attempted; it is the project's very last feature, and
the per-device UUID work rides with it.

### FR7 — Real network diagnostics

The Info screen and the dashboard signal meter exist so the user can find
out why the network is broken. Every field on them must be read from live
state at the moment it is shown. On a wired link the meter — the SWF has no
ethernet icon at all — is repurposed as an "ethernet up" indicator in a
distinct colour, with the Info screen stating `type: Ethernet` alongside so
the display stays honest.

> **Open defect.** The interface type is currently hardcoded to `lan`, so
> after moving the Pi to wifi the page still claimed Ethernet. The static
> full-signal reading and the unconditional blue tint have the same problem.
> This is a player-side fix; tracked in the fork's `claude-docs/`.

### FR8 — The panel's own settings persist

Volume, alarms, stream URLs, clock format and the rest are files the panel
writes. They must survive a reboot. They may be discarded by a package
upgrade — that is accepted, not desired.

---

## 2. Non-functional requirements

### NFR1 — Copyrighted material never enters a public artifact

`controlpanel.swf`, the widget SWFs, the widget thumbnails and the chumby
alarm tones are chumby firmware. They are gitignored, they are never
committed, never cached in CI, never attached to a release — and since
0.8.0 (2026-07-13) they never enter *any* package: the former
`chumby-player-data` deb was retired, so there is no private artifact left
to guard. Owners copy the files from their own chumby or its backup into
`/var/lib/chumby` (the launcher prints the exact paths); CI fetches the
SWF from the private share only to *run* the movie test, never to package
it. The documentation says where the files come from without embedding a
link.

Amendment (Jan, 2026-07-13): `chumby-download-firmware` may fetch
firmware *from Chumby's own servers* — a user-invoked, ask-first script
(design §5); the files still never enter a package, and nothing runs
without the owner confirming. Amended again (Jan, 2026-07-14): the
script's source is now the classic **1.7.3 firmware image**
(`update.zip` on files.chumby.com, md5-pinned) — its CRAMFS rootfs
carries intro.swf and the alarm tones, all byte-identical to Jan's
backup — plus the `download_cp` protocol for the newest control panel.
(The image's boot openings were extracted in 0.9.0 only; the boot
animation was dropped in 0.9.1, then reattempted as a Plymouth theme —
animation confirmed on-device 2026-07-19, remaining verification in
claude/issues.md #3 — design §5.) Nothing is backup-only any more except extra
widgets. The script embeds the image URL and the protocol endpoint;
the *documentation* still names no file URLs. The former stock-clock
widget download was dropped outright: a widgetless panel shows its
built-in clock (fork FR17), so no widget needs to be fetched at all.

The GPL and LGPL texts shipped in the virtual rootfs are freely
distributable and *are* committed.

### NFR2 — The debs install on a stock Raspberry Pi OS

Every dependency is declared. This is verified, not assumed: CI installs
both packages into a clean arm64 `debian:trixie` container and runs the
player from the packaged files.

### NFR3 — Hardware variation is a first-class concern

Another TFT, another sound card, another Pi model. The display device, the
renderer, and the audio sink are all overridable through
`/etc/default/chumby-player` without rebuilding. The DRM device is named
by its stable `by-path` name, because card numbers are **not** stable across
boots — the panel's TFT was `card1` one boot and `card0` the next.

The SPI controller address is part of that by-path name
(`platform-3f204000.spi-cs-0-card` on a Pi 3), so a different Pi model needs
that line changed. `docs/hardware.md` is the user-facing version of this.

### NFR4 — It has to be quiet and cool enough

The content was authored for a 350 MHz ARM9. On a Pi 3B+ at 480×320 and
12 fps the player sits at roughly one core; the remainder is software
rasterization, which is the floor for this renderer. Two levers got it
there: `LP_NUM_THREADS=1` and the fat-LTO `dist` profile — measurements and
the lavapipe explanation in [development.md](development.md) §6. A heatsink
is recommended; the SoC brushes its soft thermal limit under sustained load.

### NFR5 — Every device change is written down as it happens

Command, reason, result. Packages installed, `config.txt` edits, overlays,
systemd units, sysfs writes, udev rules. The goal is that the "make your
Raspberry Pi into a Chumby" howto can be assembled from
[development.md](development.md) afterwards without re-deriving anything
from memory or shell history.

---

## 3. Deferred, and why

| Item | Blocked on |
|------|-----------|
| **Brightness + night mode** | Hardware only, since 2026-07-13. The player side shipped (fork FR16, desktop-verified) and the deb ships the backlight udev rule (design §8) — both inert on the current ILI9486 clone, whose backlight LED rail is tied straight to 3.3 V (GPIO22 is declared in the overlay but not routed; verified by watching the panel while toggling it). The `settings-brightness` ui-policy rule now lifts by itself when a backlight exists. Remaining: buy a dimmable display (candidates: design §8), then the on-device pass. |
| **Intro on-device pass** | Wired 2026-07-12, desktop-verified only. The fork plays the INTRO button on the localCache path (its requirements §3 "Boot-time intro"); the launcher plays `intro.swf` standalone before the panel unless `/psp/disable_intro` exists — rcS `start_intro` semantics, and the tour always quits itself. Owed on the Pi with the 0.5.0 debs: tour on the TFT at boot, both flag buttons, next boot honoring the flag, INTRO button in-panel. Rides with the widget-channel pass below. |
| **Remote channels + registration** | Deliberately the project's last feature. |
| **Widget-channel on-device pass** | The channel, the preview picture and the disabled controls were verified on the desktop; the single combined on-device confirmation is still outstanding. Deploy a **freshly built** player — a stale binary has already produced one false "it doesn't work". |
| **Backup-alarm on-device pass** | Implemented in the player (fork's requirements.md FR13, 2026-07-10) and verified on the desktop; still owed on the Pi: that the Klaxon at mpv volume 100 through the 35 % hardware volume is genuinely loud enough to wake someone. If not, escalate to sink/hw volume bumping — a decision recorded in FR13. |
