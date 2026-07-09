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

1. **Finish I3 — real network diagnostics.** `real_net.rs` reports a constant
   `type="lan"` and constant full signal, and the blue ethernet tint is an
   unconditional rule, so the status page lied after the Pi moved to wifi.
   Derive the type from the default-route interface (wireless iff
   `/sys/class/net/<if>/wireless/` exists); SSID needs nl80211, link quality
   comes from `/proc/net/wireless`. Then **audit every other field** on
   `network_status.sh` and `signal_strength` for anything else that isn't
   read from live state. Confirm on-device, wired *and* wifi.

2. **The two owed ui-policy rules.** The Info screen's `piButton` (the geek
   trigger) and `introButton` were recorded as disabled and never were.

3. **Widget channel: the deferred on-device pass.** Channel, preview picture
   and disabled controls were verified on the desktop only. Deploy a
   **freshly built** player — a stale binary has already faked this failure.

4. **USB / local music files (C11).** Wanted. Needs `_getDirectoryEntry`
   (5,320) to fill objects; `RootFs::dir_entry` exists, the native stubs.

5. **Backup alarm.** A topic Jan raised 2026-07-08. Not scoped — no
   interpretation implied. Scope it with him before touching anything.

6. **Brightness & night mode (E2, B4).** Blocked on hardware: the current
   TFT's backlight rail is tied to 3.3 V and cannot dim. Needs a panel that
   can, then map the panel's `/proc/sys/sense1/brightness` writes (0–65535)
   and `_setLCDMute` (5,20) onto a real backlight. Must drop the
   `settings-brightness` ui-policy rule. Candidates and driver risk:
   `claude-docs/design.md` §8.

7. **Intro widget.** `playIntro` only reaches `intro.swf` through the slave
   player, which the `localCache` path does not run. We own the AVM1
   interpreter, so the option space is wider than "edit the SWF or revive the
   slave system" — VM-level interception is the intended route. Must drop the
   `introButton` rule from (2).

8. **Remote channels + registration.** Deliberately the *last* feature.
   chumby.com is on life support — registration is possible, just not wanted
   yet. A real per-Pi device identity (reimplementing `guidgen.sh` in the
   host, today a fixed fixture GUID) rides with it.

> Items 7 and 8 were both once called "the very last" thing. Their order
> relative to each other was never actually settled. Ask.

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
| 2026-07-08 | **Single local widget channel.** Boot-generated from the shipped widgets, static preview thumbnails, management controls disabled. Desktop only — see open item 3. |
| 2026-07-08 | **Info & Licenses.** The licenses viewer, the info screen, and the first real (non-fixture) host behaviour: live network diagnostics. **Incomplete** — see open item 1. |
| 2026-07-09 | **Docs & repo split.** Three docs per repo; the fork made self-contained for player work (fixtures, harness, decompile, its own `CLAUDE.md`); the UI policy compiled into the player. |
