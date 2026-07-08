# 19 — Single local widget channel (milestone; W1)

Date: 2026-07-08. Milestone "Single local widget channel"
(`chumby-ruffle-plan.md`). This doc accumulates across W1/W2/W3; below is
**W1 — the boot-generated channel** (CHECKPOINT W1).

Decision recap (user, 2026-07-08): one hardcoded channel = all installed
widgets, generated at boot; remote channels + registration deferred to the
project's last feature; the static preview picture is W2; disabling the
now-useless management UI is W3.

## 1. How the panel loads a channel (verified)

Startup chain, confirmed by a live desktop run (§5) and the decompiled
control panel (`appendix/.../frame_2/DoAction.as`):

1. Fetch `xml/chumbies` → `<chumby>` names one `<profile href="/xml/profiles" id="1">`.
2. `WidgetPlayer.fetchProfileXML()` fetches `xml/profiles?id=1&chumby_id=…`
   → a `<profile>` whose `<widget_instances>` list is the channel.
3. Per instance the panel reads only named nodes (`firstChildOfType` /
   `childrenOfType`, DoAction.as:4420, 4595–4611, 5009): the `<widget>`,
   its `<mode>`, the `<movie href>`, and (W2) `<widget><thumbnail href>`.
   Unknown elements/attributes are ignored; absent ones read as
   `undefined`, not a crash. So the profile schema is loose and
   forward-compatible.

Our fixtures serve both files from `fixtures/http/xml.chumby.com/xml/`.
Widget SWFs live in `fixtures/widgets/` and are referenced by
`file:///{FIXTURES}/widgets/<name>.swf`; the player host expands
`{FIXTURES}` to the absolute fixtures dir at serve time
(`ruffle/core/src/chumby/fixture.rs`), keeping the profile identical on the
dev box and the Pi.

## 2. Per-widget XML sidecars

The channel is not hand-maintained. Each widget carries a sidecar next to
its SWF:

    fixtures/widgets/
        unsubscribedclock.swf
        unsubscribedclock.widget.xml     <widget id="1">…</widget>
        builtinclock.swf
        builtinclock.widget.xml          <widget id="2">…</widget>

A `*.widget.xml` is exactly the `<widget>` element the panel consumes
(name, description, version, mode, access, movie href — and, from W2, a
`<thumbnail href>`). This mirrors how real chumby stores widget metadata
server-side; the backup has no local profile or thumbnails to reuse (only
scraped display names in `/tmp/widget_names` etc.), so we author these
ourselves. `file://` widget hrefs are accepted by the panel (zurk's
offline firmware relied on the same, doc 06 §3).

Channel order = ascending `id` attribute (tie-break: filename). The two
current ids (1 = Unsubscribed Clock, 2 = Clock) reproduce the prior
default order (Unsubscribed Clock is instance 1, shown first).

## 3. The generator: `chumby-widget-channel`

Python 3, repo root, installed to `/usr/bin/chumby-widget-channel`.
XML in, XML out (`xml.etree`); no JSON seam.

    chumby-widget-channel [--fixtures DIR] [--force] [--quiet]

- Enumerates `*.widget.xml`, wraps each `<widget>` in the
  `<widget_instance>`/`<profile>` envelope (instance name + mode mirror the
  widget's), assigns sequential instance ids, and writes:
  - `<fixtures>/http/xml.chumby.com/xml/profiles`
  - `<fixtures>/rootfs/tmp/currentProfileID` = `1`
  - `<fixtures>/rootfs/tmp/currentProfileName` = `chumbypi-channel`
- Change detection: a sha256 over the ordered sidecar set in
  `<fixtures>/widgets/.channel.sig` (gitignored, runtime cache). An
  unchanged boot skips the rewrite; `--force` overrides.
- `--fixtures` defaults to the tree beside the script (dev box); the Pi
  service passes `/var/lib/chumby/fixtures`.

Faithfulness proof: regenerating from the two sidecars is **canonical-XML
equal** to the previously hand-written, known-good `profiles` fixture
(only cosmetic empty-tag style differs). So the generator reproduces a
channel already proven to play.

## 4. Boot wiring & the starter check

Deliberately **not** folded into the panel launcher (user 2026-07-08): a
debug launch must work without regenerating first.

- `chumby-widget-channel.service` (oneshot) runs `Before=chumby-player.service`,
  as `pi` with `StateDirectory=chumby`. `ExecStartPre` seeds
  `/var/lib/chumby/fixtures` if absent (same first-run seed as
  `chumby-player-run`), then the generator refreshes the profile.
  `chumby-player.service` gains `Wants=`/`After=` it, so enabling the panel
  pulls it; a generator failure (Wants, not Requires) still lets the panel
  start and hit its own guard.
- Both launchers (`run-controlpanel.sh`, `chumby-player-run`) now **check**
  the profile exists and is non-empty before launching, and refuse with a
  hint otherwise — they do not run the generator. A committed/packaged
  profile ships in the tree, so debug launches pass the check.
- Packaging: `chumby-player` deb adds the script, the unit, and a `python3`
  dependency; `build-debs.sh` regenerates the profile in the staged data
  tree so the shipped `profiles` always matches the packaged sidecars.

## 5. Desktop verification (2026-07-08)

Live run, `ruffle/target/debug/ruffle_desktop` on `DISPLAY=:11`, generated
fixtures, `-PlocalCache=1`, 15 s timeout:

- `fetchProfileXML(): loading profile chumbypi-channel from …/xml/profiles?id=1` → fixture HIT.
- Panel wrote `/tmp/widget_names` = `Unsubscribed Clock\nClock` → both
  instances parsed, in order.
- `loading widget movie "Unsubscribed Clock" from file:////…/fixtures/widgets/unsubscribedclock.swf`
  → `{FIXTURES}` expanded, SWF loaded, `_chumby_widget_name` set → playback started.
- Exit 124 (alive at timeout), no panic. (ALSA/openh264/gamemode warnings
  are the headless shell's missing audio/H264, unrelated.)

Generator behaviours checked: unchanged→skip; add sidecar→3 instances;
remove→back to 2; missing profile→launcher refuses with hint.

CI `deb-install-test.sh` exercises the packaged path (installs both debs,
movie-start from packaged fixtures+SWF) — now over the build-time
generated profile.

## 6. Deferred / next

- **W2** — preview picture: add `<thumbnail href>` to each sidecar + ship
  an 80×60 JPEG per widget; the panel `loadMovie`s it into the B2 slot
  (`makeWidgetsURL`, DoAction.as:30612 → 8040; contract F2:4611). Real
  chumby *downloaded* this from `widgets.chumby.com`; we ship it locally.
- **W3** — disable the Channel button (D1–D7) + Delete (B8) via
  `fixtures/ui-policy.toml`.

## Pi operations performed

None yet — W1 is desktop + packaging only. On-device deploy and confirm
happen at CHECKPOINT W1 with the user; record the deploy commands here when
they run (per CLAUDE.md).
