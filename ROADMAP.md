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

2. **Backup alarm: the on-device loudness check.** The feature itself
   shipped 2026-07-10 (see Done); what remains is confirming on the Pi that
   the Klaxon at mpv volume 100 through the 35 % hardware volume genuinely
   wakes someone. If not: escalate to sink/hw volume bumping (decision
   recorded in the fork's requirements.md FR13). Rides along with item 1's
   on-device pass. (Restored 2026-07-11 — the roadmap sync `fa8087f` had
   silently dropped it.)

3. **Brightness & night mode (E2, B4).** Blocked on hardware: the current
   TFT's backlight rail is tied to 3.3 V and cannot dim. Needs a panel that
   can, then map the panel's `/proc/sys/sense1/brightness` writes (0–65535)
   and `_setLCDMute` (5,20) onto a real backlight. Must drop the
   `settings-brightness` ui-policy rule. Candidates and driver risk:
   `claude-docs/design.md` §8.

4. **Intro widget.** *Closed 2026-07-12, pending the on-device pass.*
   VM-level interception, as intended: the fork replaces `playIntro` (and
   its done-poll) on the prototype, so the INTRO button plays the tour on
   the `localCache` path; `info-intro` dropped; the `enable_intro` /
   `disable_intro` backticks are real. The launcher now also runs the tour
   before the panel, every boot until `/psp/disable_intro` — rcS
   `start_intro` semantics (appliance 0.5.0). Standalone ending and
   same-session replay desktop-verified; the on-device pass rides with
   item 1.

5. **Remote channels + registration.** *Registration and channel/widget
   download are done and verified live* (2026-07-11): DCID reverse-engineered
   on a real registered box (identity is the crypto-processor GUID; DCID
   `skin` is only branding, no signature to reproduce), register-the-Pi
   chosen over cloning, and the panel's own register wizard + the account's
   real widget channel made to work under `access_chumby_com` + a stable
   identity (hardware serial or `device_guid`), CI/dev structurally excluded
   (NFR6). End-to-end on the TFT: boot → wizard → tap ovals + claim on
   chumby.com → main; the account's widget downloaded, cached and rendered;
   CHANNEL/DELETE enabled. Fork branch `registration-phase2`, fork
   requirements §3 / design §12; device record in this repo's development.md.
   **The social surface** (add-widget browse, rating, send+mail) is
   **out of scope** (Jan, 2026-07-11) — `main-send`/`main-rate` stay
   disabled and those endpoints are never passed through. Item 5 is closed.

## Standing decisions

These still bind. Changing one is a decision, not drift.

- **`controlpanel.swf` is never modified.** Everything is exerted from Rust.
- **Nothing reaches chumby.com by default** — not at boot, not from CI.
  Since 2026-07-11 the *owner* may open two slices with
  `access_chumby_com=1` (`/etc/chumby-player/player.toml`): the music
  proxies (fork FR15, no identity), and the registration endpoints
  `/xml/authorize` + `/xml/registerchumby`, which carry the device GUID.
  The identity slice is additionally gated on a real hardware serial, so a
  CI/dev box can never present a registrable identity (fork NFR6). Default
  and CI stay fully offline; the GUID leaves only on the owner's opt-in
  from real hardware.
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
| 2026-07-11 | **Config file + music sources (items 3 & 4 of the 07-09 plan).** `player.toml` read once at start (fork FR14: volume cap as a scale, `access_chumby_com`, `enable_lyrion`; template ships as `/etc/chumby-player/player.toml`, a conffile the launcher links into the live fixtures). Music scope re-decided against live endpoints (fork FR15): SHOUTcast / blue octy radio / Sleep Sounds are *alive* on the revived chumby.com and opt-in via the switch — SHOUTcast browsed and played audibly on the desktop; iPod/NOAA/CBS hidden (NOAA+CBS confirmed dead on a real chumby); Squeezebox behind `enable_lyrion` (player side done, Lyrion unverified). Fork PR #19, appliance 0.4.0. |
| 2026-07-11 | **Registration + remote channels (item 5).** DCID reverse-engineered on a real registered box (identity = crypto-processor GUID; DCID `skin` = branding only, no signature); register-the-Pi chosen over cloning. `access_chumby_com` + a stable identity (serial or `device_guid`) opens the panel's own register wizard and the account's real widget channel — authorize/registerchumby/chumbies/profiles/widgets pass through to the revived chumby.com (`update.chumby.com` never), CI/dev structurally excluded (NFR6). Widget SWFs download via a Rust reimplementation of the panel's `curl` cache. Verified end-to-end on the TFT: boot → wizard → tap ovals + claim on chumby.com → main; account widget downloaded, cached, rendered; CHANNEL/DELETE enabled. Fork branch `registration-phase2`, fork design §12. Social surface (add-widget/rating/send+mail) de-scoped — item 5 closed. |
| 2026-07-11 | **USB / local music (C11), closed same day.** Real `_getDirectoryEntry` (5,320) in the player (fork `usb-music`), My Music Files browse/play + alarm-from-USB desktop-verified; appliance automount (udev + `chumby-usb-mount@`, read-only `/media/chumby-usb`, seed-time symlink) deployed as 0.3.0. Physical-stick pass complete: hotplug automount, audible playback on the TFT, yank-while-playing cleaned up honestly. `externalmusic.xml` stays faithful. |
| 2026-07-12 | **Intro widget (item 4).** Fork `intro-widget`: `playIntro` + done-poll replaced at VM level (INTRO button live on the localCache path), in-panel `fscommand("quit")` swallowed, real `enable_intro`/`disable_intro` backticks, real `_accelerometer` for the ball page. Standalone ending (both flag buttons, quit exits the process) and same-session replay desktop-verified. Appliance: launcher runs the tour pre-panel every boot until `/psp/disable_intro` (rcS `start_intro`; USB override deliberately dropped), 0.5.0. On-device pass outstanding (rides with item 1). |
