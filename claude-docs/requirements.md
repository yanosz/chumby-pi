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
| Music: USB / local files | **wanted, not built** | needs `_getDirectoryEntry` in the player |
| Music: every other source | skip | SHOUTcast, iPod, FM, MP3tunes, NOAA, Internode, podcasts, Chumbcast, sleep sounds, Squeezebox, Pandora et al. — dead services or absent hardware |
| Widget channel | **delivered** | exactly one channel, generated at boot from the widgets we ship |
| Channel management (picker, info, add, reload) | **disabled** | needs remote download + registration |
| Widget preview picture on the dashboard | **delivered** | a static per-widget thumbnail |
| Delete / Send / Rate widget | **disabled** | delete can't persist; send and rate are chumby.com social features |
| Info / About | **delivered** | with real network diagnostics |
| Licenses | **delivered** | the original chumby's GPL and LGPL texts, verbatim |
| Geek panel, file browser | **disabled** | redundant next to a Raspberry Pi; the Info screen's π trigger is inert (rule `info-geek`) |
| Intro widget | deferred, **disabled** | cannot play on our widget path; rule `info-intro` dims the INTRO button and must be dropped when intro lands, see §3 |
| Brightness, night mode | **disabled** | blocked on display hardware, see §3 |
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
committed, never cached in CI, never attached to a release. The
`chumby-player-data` package contains them and is therefore **private-use
only** — it must never be published in an apt repository or a GitHub
release. Users obtain the SWF themselves; the documentation says so without
embedding a link.

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
| **Brightness + night mode** | Hardware. The current ILI9486 clone has its backlight LED rail tied straight to 3.3 V — GPIO22 is declared in the overlay but not routed on the board, and driving it does nothing (verified by watching the panel while toggling it). Dimming needs a different display; the candidates and their driver risk are in [design.md](design.md) §8. When it lands, it must also drop the `settings-brightness` rule from the player's UI policy. |
| **Intro widget** | The panel only loads `intro.swf` through the slave player, which the chosen widget architecture does not run. Needs interpreter-level work in the fork. |
| **Remote channels + registration** | Deliberately the project's last feature. |
| **USB / local music files** | `_getDirectoryEntry` in the player. |
| **Widget-channel on-device pass** | The channel, the preview picture and the disabled controls were verified on the desktop; the single combined on-device confirmation is still outstanding. Deploy a **freshly built** player — a stale binary has already produced one false "it doesn't work". |
| **Backup-alarm on-device pass** | Implemented in the player (fork's requirements.md FR13, 2026-07-10) and verified on the desktop; still owed on the Pi: that the Klaxon at mpv volume 100 through the 35 % hardware volume is genuinely loud enough to wake someone. If not, escalate to sink/hw volume bumping — a decision recorded in FR13. |
