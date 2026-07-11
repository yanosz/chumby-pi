# Roadmap

What is left, what is done, and the decisions that still bind.

> **This file was compressed on 2026-07-09** from an 850-line plan that had
> accumulated since 2026-06-12. Completed milestones are now one line each,
> and the reasoning behind them lives in the two `claude-docs/` sets (this
> repo's, and the fork's). **It is therefore not complete, and it may be
> wrong.** The old plan asserted work as done that had never shipped — that
> is what prompted the compression, not a reason to trust the summary more.
>
> If you are an agent picking up work here: when this file, a doc, and the
> code disagree, **the code wins** — grep for the artifact before believing
> a claim about it. And when scope is ambiguous, **ask.** Jan does not mind
> answering a question twice; he minds a session that ends in confusion.
> Full history is in `git log` and in the docs deleted in commit `6cc2b17`.

Working rules — checkpoints, artifacts, branch policy, the read-only backup —
are in `CLAUDE.md`, in each repo. They are not repeated here.

---

## Open work

Roughly in order. The player-side detail lives in the fork's
`claude-docs/requirements.md` §3; the appliance-side in this repo's §3.

1. **Widget channel: the deferred on-device pass.** Channel, preview picture
   and disabled controls were verified on the desktop only. Deploy a
   **freshly built** player — a stale binary has already faked this failure.

2. **Configuration file support.** Read once at player start; no write
   support for now. Options as of today (Jan, 2026-07-11): **volume cap**
   (in %) and **access chumby.com** (0/1). The latter is the future switch
   for the remote-channels milestone (item 6) and must default to 0 —
   NFR6 ("nothing reaches chumby.com") stays the standing state until it
   is flipped deliberately. File location/format is design work in the
   fork.

3. **Music sources: reconsider scope.** USB/local files (C11) may have
   opened gates to other sources — go through the panel's source list
   (`MusicPlayer.musicSources`) and re-decide the blanket "skip every
   other source" row in the scope table (this repo's requirements §1 FR5).
   Candidates worth a look now that filesystem browsing and mpv playback
   are proven; dead services (MP3tunes, Chumbcast, CBS/NYT podcasts …)
   stay dead.

4. **Brightness & night mode (E2, B4).** Blocked on hardware: the current
   TFT's backlight rail is tied to 3.3 V and cannot dim. Needs a panel that
   can, then map the panel's `/proc/sys/sense1/brightness` writes (0–65535)
   and `_setLCDMute` (5,20) onto a real backlight. Must drop the
   `settings-brightness` ui-policy rule. Candidates and driver risk:
   `claude-docs/design.md` §8.

5. **Intro widget.** `playIntro` only reaches `intro.swf` through the slave
   player, which the `localCache` path does not run. We own the AVM1
   interpreter, so the option space is wider than "edit the SWF or revive the
   slave system" — VM-level interception is the intended route. Must drop
   the `info-intro` ui-policy rule.

6. **Remote channels + registration.** Deliberately the *last* feature.
   chumby.com is on life support — registration is possible, just not wanted
   yet. Its device-identity prerequisite already landed (fork PR #16): the
   GUID and HW# are real, derived from the SoC serial; a machine without one
   (dev box, CI) generates its own random GUID once and persists it (fork
   requirements FR10, 2026-07-10). Revisit that in-player generation here:
   a CI run must never present a registrable identity to chumby.com.

> Items 5 and 6 (intro widget, remote channels) were both once called "the
> very last" thing. Their order relative to each other was never actually
> settled. Ask.

## Standing decisions

These still bind. Changing one is a decision, not drift.

- **`controlpanel.swf` is never modified.** Everything is exerted from Rust.
- **Nothing reaches chumby.com** — not at boot, not from CI. The GUID must
  not leak.
- **Widget playback uses the panel's `localCache` in-movie path**, not the
  master/slave dual-instance system (confirmed at CHECKPOINT 3, 2026-06-12).
- **`--load-behavior blocking` is mandatory** — the panel jumps to frame
  labels a streaming load hasn't parsed.
- **Platform identity is `ironforge`, hw 3.8** — faithful to Jan's device.
- **The Pi OS owns timezone, NTP and network config**; the panel shows them
  read-only or not at all.
- **Screens are opt-in.** Before implementing one, ask whether it is in
  scope. The scope table is `claude-docs/requirements.md` §1 FR5.
- **Copyrighted material never enters git or a public artifact** —
  `controlpanel.swf`, widget SWFs, thumbnails, alarm tones, the decompile.
  The `chumby-player-data` deb must never be published.

## Done

| When | Milestone |
|------|-----------|
| 2026-06-12 | **M1 — understand the panel.** Decompiled 2.8.87b3; the environment contract, the gap analysis against stock Ruffle, the screen catalog, zurk's prior art. |
| 2026-06-13 | **M2 — stub the API.** `ChumbyHost` + `FixtureHost`; the panel boots to its main screen on fixtures. Alarms, My Streams, volume, bend sensor, mpv audio. |
| 2026-07-06 | **M3 — Raspberry Pi.** Cross-build, two debs, cage kiosk, boot straight to the panel on the SPI TFT. Then CPU halved (~215% → ~103%) and the phantom cursor killed. |
| 2026-07-07 | **The Big Cleanup.** Two repos, both CI green; the `chumby` cargo feature removed; every `ASnative(5,N)` index documented. |
| 2026-07-07 | **UI policy.** A general mechanism for disabling controls the Pi can't honour, plus the clock screen and four Settings icons. |
| 2026-07-08 | **Single local widget channel.** Boot-generated from the shipped widgets, static preview thumbnails, management controls disabled. Desktop only — see open item 1. |
| 2026-07-08 | **Info & Licenses.** The licenses viewer, the info screen, and the first real (non-fixture) host behaviour: live network diagnostics. Finished 2026-07-10 (I3 row below). |
| 2026-07-09 | **Docs & repo split.** Three docs per repo; the fork made self-contained for player work (fixtures, harness, decompile, its own `CLAUDE.md`); the UI policy compiled into the player. |
| 2026-07-10 | **The two owed ui-policy rules.** `info-geek` and `info-intro` disable the Info screen's π trigger and INTRO button; verified live (targets acquired, clicks inert). |
| 2026-07-10 | **I3 finished — real network diagnostics.** Type from the default-route interface, live SSID and signal, every `network_status.sh`/`signal_strength` field audited live (fork PR #15, FR10). |
| 2026-07-10 | **Backup alarm (FR13) + real device identity.** In-process dead-man beep on `/psp/ifalarm`, missed-alarm boot check; GUID/HW# derived from the SoC serial, `md5sum` computed for real (fork PR #16). |
| 2026-07-11 | **USB / local music (C11), closed same day.** Real `_getDirectoryEntry` (5,320) in the player (fork `usb-music`), My Music Files browse/play + alarm-from-USB desktop-verified; appliance automount (udev + `chumby-usb-mount@`, read-only `/media/chumby-usb`, seed-time symlink) deployed as 0.3.0. Physical-stick pass complete: hotplug automount, audible playback on the TFT, yank-while-playing cleaned up honestly. `externalmusic.xml` stays faithful. |
