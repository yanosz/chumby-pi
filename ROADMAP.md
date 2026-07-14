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

1. ~~**Widget channel: the deferred on-device pass.**~~ **Done 2026-07-14**
   (Jan closed it): the vanilla-card installs exercised it live — freshly
   built apt player, downloaded clock widget playing on the TFT
   (screenshot-verified), controls exercised during testing.

2. **Backup alarm: the on-device loudness check.** *Postponed until new
   hardware arrives (Jan, 2026-07-14).* The feature itself shipped
   2026-07-10 (see Done); what remains is confirming on the Pi that
   the Klaxon at mpv volume 100 through the 35 % hardware volume genuinely
   wakes someone. If not: escalate to sink/hw volume bumping (decision
   recorded in the fork's requirements.md FR13).

3. **Brightness & night mode (E2, B4).** *Software done 2026-07-13;
   postponed until new hardware arrives (Jan, 2026-07-14).* The fork maps
   the panel's `/proc/sys/sense1/brightness` writes
   onto any kernel backlight (sliders) and offers a `brightness_ctl`
   executable mode driving `_setLCDMute`'s discrete 0/1/2 (fork FR16); the
   `settings-brightness` rule lifts by itself when a backend exists, and
   the deb ships the backlight udev rule. Blocked on hardware: the current
   TFT's backlight rail is tied to 3.3 V and cannot dim. Remaining: buy a
   dimmable panel (candidates and driver risk: `claude-docs/design.md` §8),
   then the on-device pass.

4. **Intro widget: the on-device tour pass.** *Feature closed 2026-07-12;
   the remaining pass postponed until new hardware arrives (Jan,
   2026-07-14).* Since 0.8.x the INTRO button is gated on an owner-copied
   `intro.swf` (not downloadable — backup only), so the pass needs that
   file on the device: tour on the TFT, flag buttons, next-boot gating,
   in-panel INTRO.

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

6. ~~**Fork git housekeeping.**~~ **Done 2026-07-14** (force-pushed by
   Jan): `chumby` is now **two commits on pinned upstream master
   `8328af42d`** — pure additions (`c3afbec41`) first, the design §8 patch
   surface (`2ac3acc2e`) second, so rebase conflicts concentrate in the
   second commit. Old history (18 commits on `7f62f5dbf`) parked as
   `chumby-old`; `Cargo.lock` regenerated (upstream's + the three core
   deps), `chumby-ctl` moved to chumby-pi `pkg/chumby-player/`. The
   upstream move itself was a real merge and went cleanly: zero conflicts,
   hooks alive in all §8 files, 40/40 chumby unit tests, movie-start check
   green, screens walked on the desktop (`verify-screens.sh` gained the
   missing `windowraise` — the §7 trap). The on-device pass rides with the
   next deploy (items 2 and 4). Fork record: its `development.md` §6.

7. **End-user docs pass.** `docs/setup.md` rewritten 2026-07-14 around the
   real install story (apt repo, display overlay vs HDMI/DSI,
   `chumby-download-firmware`, backup copy table, `chumby-local-widgets`,
   the two config files; NFR1 kept — no direct links to copyrighted
   files). Remaining: the **asciidoctor site content** at the Pages root
   (`docs/index.adoc`, Jan writes it — skeleton and the README display
   section are the raw material), and a look at whether `docs/hardware.md`
   still matches 0.8.x.

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
- **Copyrighted material never enters git or any artifact** —
  `controlpanel.swf`, widget SWFs, thumbnails, alarm tones, the decompile.
  Since 0.8.0 (2026-07-13) there is no private deb at all: the former
  `chumby-player-data` was retired and owners copy those files from their
  chumby (or its backup) into `/var/lib/chumby`.

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
| 2026-07-14 | **Fresh-install hardening round 2, 0.8.4–0.8.6 + item 1 closed.** Display auto-detection (`--kiosk` glob, any Pi model; HDMI/DSI fallback) and live-state postinst guidance; helpers work before the first service start (`--seed`); exact copy-path table in the downloader; boot-to-panel fixed for the Lite image (`multi-user.target` — the unit had never started at boot); `merge_local_remote_widgets` (default 0) keeps local widgets out of curated chumby.com channels, live-verified both ways on the registered box. Full rm-rf → downloader → clock recovery verified remotely (screenshot). Item 1 (widget-channel on-device pass) closed by Jan on this evidence; `docs/setup.md` rewritten around the apt install story. |
| 2026-07-13 | **Fresh-install session (evening), 0.8.1–0.8.3.** Jan's first vanilla-card install from the apt repo surfaced and fixed: the missing/unreadable-SWF refusal now runs as `ExecStartPre` outside cage (visible in `systemctl status` even with a broken display; readability checked, chown/chmod hint); `/etc/default/chumby-player` shipped (DRM device per Pi model, audio device); `chumby-download-firmware` — user-run ask-first downloader for the control panel (2.8.87b3, md5-verified) and the Unsubscribed Clock via chumby.com (NFR1 amended; intro/alarm tones are firmware-only, not downloadable); INTRO button gated on intro.swf presence (fork `only_without_intro`, PR #24) after it black-screened without the file; launcher links the owner intro into the rootfs so the in-panel INTRO works at all. Also: asciidoctor docs-site skeleton at the Pages root. All deployed + verified on the vanilla Pi (clock widget renders — item 1's channel machinery seen live on-device). |
| 2026-07-13 | **Housekeeping (tasks 3 + 2 + apt repo), 0.8.0.** Channel machinery replaced by the panel-native `mergeLocalProfile` (empty static base fixture; `chumby-local-widgets` user helper writes `/psp/profile.xml` from `/var/lib/chumby/widgets`); `chumby-player-data` **retired** — the public deb ships the git-clean fixtures, owners copy `controlpanel.swf`/widgets/alarm tones/intro from their chumby backup into `/var/lib/chumby` (launcher refuses with instructions); signed flat **apt repo** on GitHub Pages under `/apt`, published by CI on every push to main (root kept free for a future site). All desktop-verified offline incl. an end-to-end fresh-install run from the real backup; on-device pass still owed (item 1). Task 1 of the housekeeping plan = open item 6. |
| 2026-07-12 | **Intro widget (item 4).** Fork `intro-widget`: `playIntro` + done-poll replaced at VM level (INTRO button live on the localCache path), in-panel `fscommand("quit")` swallowed, real `enable_intro`/`disable_intro` backticks, real `_accelerometer` for the ball page. Standalone ending (both flag buttons, quit exits the process) and same-session replay desktop-verified. Appliance: launcher runs the tour pre-panel every boot until `/psp/disable_intro` (rcS `start_intro`; USB override deliberately dropped), 0.5.0. On-device pass outstanding (rides with item 1). |
