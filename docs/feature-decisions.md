# Feature decisions

Status values: **needed** / **skip** / **undecided** / **future-milestone**.
Screen IDs reference `docs/reference/05-screens.md`. Decisions only move out
of *undecided* with explicit user approval (project rule 5).

## Decided at CHECKPOINT 2 (user, 2026-06-12)

| Feature / screens | Status | Notes |
|-------------------|--------|-------|
| Boot path: startup A1, authorize-bypass A3, builtin clock A7, idle B1, main bar B2, play-trap A5 | **needed** | core |
| Volume (E1) | **needed** | standing decision |
| Brightness incl. night mode (E2, B4) | **future-milestone** | was *needed*; moved 2026-06-13 (user): no touchscreen at hand, heavily tied to the Pi environment — bundle with the Pi deployment milestone |
| Alarms (B5, B6) — list, editor, ring screen | **needed** | user: "In." |
| Clock panel / time / timezone (B3, E5, E12) | **needed** | implied by alarms + core settings |
| Music: My Streams (C2) | **needed** | user |
| Music: USB / local files (C11) | **needed** | user; "USB" maps to local dirs on Pi |
| Music: all other sources (C1, C3-C10, C12, C13) | **skip** | user: MyStreams + USB-local "is perfect" |
| Widget channels / management (D1-D5, D7) | **future-milestone** | ignore for now; single fixture channel in M2 |
| Info + Licenses (E6, E7) | **future-milestone** | ignore for now |
| Geek panel (E8, E9) | **skip** | user: leave untouched, redundant to RPi |
| First-time wizard (A2), activation (A4), safe mode (A6), network wizard (A8/E3), touchscreen calib (E4), rate (B7), send (B10), accept/decline (D6), microphone (E10), intercom (E11), firmware updates (B9, A6) | **skip** | user confirmed |

## Confirmed at CHECKPOINT 3 (user, 2026-06-12)

| Topic | Decision | Rationale |
|-------|----------|-----------|
| Widget playback architecture | **localCache in-movie path**, not master/slave | tiny Ruffle patch surface; proven in stock Ruffle (04 run 8); revisit if a widget misbehaves |
| Platform identity | report `ironforge`, hw `3.8`, sw/fw per backup (1.7.2/…) | faithful to the user's device (CNPLATFORM=ironforge); enables day/night brightness panel |
| Ruffle GoToLabel bug | no upstream report; rely on `--load-behavior blocking` | user prefers not to file Ruffle bugs |
| HTTP fixture transport | in-process interception (no local HTTP server) | user choice |
| Audio in M2 | silent state-machine stub | adopted as proposed (no objection) |
| Code placement | module `core/src/chumby/` behind `chumby` feature | adopted as proposed (no objection) |

**Verification aid:** a real Chumby Classic is reachable via SSH for
*harmless read-only* checks only (user actively uses it; ask before use).

## Confirmed at CHECKPOINT 4 (user, 2026-07-02)

| Topic | Decision | Rationale |
|-------|----------|-----------|
| My Streams (C2) network class | **real-network** on the Pi: mpv plays the user-configured stream URLs for real. Stream *list* stays a fixture; no chumby.com contact | first-exercise on real hardware per M3 definition; contract row updated from mock |
| Kiosk boot | systemd service + `cage` running ruffle fullscreen; no lightdm/desktop in player mode | fewest moving parts; `systemctl disable --now chumby-player` over SSH is the escape hatch |
| Packaging | two debs: `chumby-player` (binary, launcher, unit) + `chumby-player-data` (fixtures, SWFs) | keeps the copyrighted controlpanel.swf in a separate never-publish package |
| Monorepo | ruffle fork imported via **git subtree with full upstream history** at `ruffle/` | keeps the chumby patch series rebasable onto upstream (rule 2) |
| Bend button (Pi) | GPIO17 (phys pin 11) bridged to GND (phys pin 9), `gpio-key` overlay → key event; plus `chumby-ctl` CLI firing bend over a Unix socket | no keyboard/touch on the target; same IPC serves the "magic key" escape |

## Future milestones (requested at CHECKPOINT 2)

1. **Widget channels & management** — D1-D5, D7: channel switching, add
   widget, channel info; needs profile/catalog fixture design (zurk's
   profiles.sh is prior art, see 06 §3).
2. **Info & Licenses panels** — E6, E7.
3. **Brightness & night mode** — E2, B4 (moved out of M2 2026-06-13, user):
   needs the Pi display backlight; do together with the Pi deployment
   milestone.

(To be ordered against the pre-existing future milestone: Raspberry Pi
deployment/packaging.)
