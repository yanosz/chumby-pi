# 17 — BC4b: ASnative reference, README swap, comment audit (2026-07-07)

Plan §BC4, second checkpoint (closes the Big Cleanup milestone once Jan
reviews). Originally chumby-ruffle commit `4800581a5`; later the same
day Jan asked for the fork's history to be squashed per branch
discipline, so the whole fork — BC1 squash through BC4b plus the wiki
cross-check below — is now the **single commit `7f4d8031b`** on top of
upstream `7f62f5dbf`. Pre-squash hashes in this doc and doc 16 no
longer resolve on GitHub (they survive in the local clone at
/home/jan/chumby-ruffle).

User guidance received during the session (recorded here because it
shaped the deliverable beyond the plan text):

- Verify the source-code comments while documenting ASnative; code
  should speak for itself (minor clarity refactorings OK), and comments
  should refer to ASnative calls where helpful — "one cannot assume to
  know ASnative indexes by heart".
- Drop the hook numbering (H1…H11): no valid point in numbering all
  hooks; focus on ASnative, put less focus on hooks.

## 1. ASnative(5,N) reference (the audit)

Every index in `avm::wrapper_name` — all ~140 the panel or a companion
SWF binds — is now documented in the fork's `README.md` §"ASnative(5,N)
reference": purpose, arguments, return value, and what `FixtureHost`
currently answers, grouped by function family. Sources: the environment
contract (03 §1), the panel's call sites, the chumby wiki, and the
fixture code itself. Audit findings worth knowing:

- Rows are honest about ignorance: 5,70/71 (`_csccd`/`_gsccd`),
  5,72/74 (nameless bindings — our `_slaveSetting72/74` names are
  project-invented) and 5,420 (`_SetOnLocationCallback`) are marked
  *undocumented*. (5,170 `_getstalt` was initially marked so too,
  until the wiki cross-check below identified it as chumby's Gestalt.)
- Fixture behavior is described by three shorthands (rootfs / store /
  stub) matching the actual dispatch tiers in `avm.rs` + `fixture.rs`,
  with explicit rows where `FixtureHost` does something specific
  (platform, env, bend, audio family, volume/mute, timezone).
- Two findings surfaced by the audit: **a known bug** — `_setTimeZone`
  (5,178) lands in the generic state store and is NOT reflected by
  `_getTimeZone` (5,177), which reads `/psp/timezone` from the rootfs;
  Jan confirmed set → get round-trips on a real chumby, so this is a
  bug, documented in the fork README (5,178 row) and in a `KNOWN BUG`
  comment at the `_getTimeZone` arm in `fixture.rs`, to be fixed with
  the clock-panel verification (plan, future milestones).
  **FIXED 2026-07-07** (fork commit `6c99ebd1f`, amended per branch
  discipline): `_setTimeZone` got an explicit arm writing the trimmed
  value to `/psp/timezone` via the rootfs; round-trip covered by
  `fixture.rs::tests::test_timezone_set_get_round_trip` (passes);
  `KNOWN BUG` comment removed, README 5,178 row rewritten; local 45 s
  movie-start check green (exit 124, `_getPlatform` seen, no panic);
  fork CI on the re-squashed commit green (run 28884211332).
  And a quirk:
  `_SetOnLocationCallback`'s capital S bypasses the `_set…` store
  prefix match, so it stubs.
- `RootFs::dir_entry` exists but nothing calls it yet —
  `_getDirectoryEntry` (5,320) stubs "end of listing" until the
  USB-music milestone; noted in code and README.

## 1a. Wiki cross-check (same day, after Jan's question)

Jan asked whether the table had been verified against the chumby wiki
(doc 03 was, in 2026-06; the BC4b table initially was not re-checked).
**wiki.chumby.com is still online** — the README's reference section
now links its two relevant pages and was cross-checked against both:

- `ChumbyNative` (`index.php?title=ChumbyNative`) — most of the table.
- `Controlling_BTplay` (`index.php?title=Controlling_BTplay`) — the
  audio family; confirms the −1/0/2 state values (the SWF additionally
  defines WAITING:1, which the wiki does not document).
- `Sensor_Access`, cited by doc 03 in June, is now a 404.

Corrections adopted from the wiki: `_getstalt(key)` is chumby's
Gestalt (system-property query), not undocumented; concrete signatures
for `_setCalibration(xoffset,xscale,yoffset,yscale)`,
`_powerDown(when[,secondsToPowerUp])`, `_accelerometer(index)`, the
pipe family (handle-based), blowfish's optional mode arg,
`_playAudioAddPlaylist(mimeType, paths…)`, `_startSlave → instance
id`, `_stopSlave(id)`, `_pauseResumeSlave(id, mode)`, overlay opacity
range 0–255, `_getAudioPlayerTrackAttributes → object`.

Discrepancies found (SWF is the law, per project ground rules):

- The wiki's ChumbyNative page lists the time family under **category
  103** (`_getTimeZone` 103,320 / `_setTimeZone` 103,321 /
  `_setSystemTime` 103,322); the SWF binds them at **5,177 / 5,178 /
  5,176** (verified in the frame_2 export, F2:1552–1554). Category 103
  is `Date` in Ruffle/Flash. Noted in the README's 5,176 row.
- The BTplay page numbers a plain "PlayAudio" at 5,151; the SWF has
  `_playAudio` at 5,144 (what the panel calls) and `_playAudioNow` at
  5,151. Both noted in the 5,151 row.
- The wiki lists `_keyboardGetString` also at 5,93 (colliding with
  `_keyboardGetScanCode` there) — ambiguous, not adopted.

## 2. README swap

- `README.md` (upstream Ruffle's) → `README.ruffle.md` (git mv,
  history preserved).
- `CHUMBY.md` → `README.md`, restructured: the H-numbered hook table
  became a plain file list ("grep for `chumby`"), the ASnative section
  gained the full reference, stale `docs/…` paths now say
  `claude-docs/…`, and the new README links README.ruffle.md at the
  top. `CHUMBY.md` no longer exists.

## 3. Source-comment audit (same commit)

- `avm.rs` dispatch: every bare index got its wrapper name in a
  comment (`// (5,50) _getFile(path) -> contents`, …); misleading
  "Category 4/2" group labels (environment-contract jargon) replaced
  with plain descriptions.
- `fixture.rs` native(): every name-keyed arm got its (5,N) index;
  `default_for_getter` documented per entry.
- `mod.rs` header no longer lists `navigator` as "(planned)" (it has
  existed since 2.2.3) and now lists `audio`.
- All "hook H<n>" comments in upstream files replaced by plain
  descriptions still containing `chumby` (grep-ability for rebases is
  unchanged).
- Doc-path references in code updated (`docs/…` → `claude-docs/…`, or
  to the fork's own README where that is now the right target).

## 4. chumby-pi side (this repo)

- `claude-docs/patch-notes.md` rewritten file-keyed, hook numbers
  gone; points at the fork README for architecture + ASnative
  reference.
- `claude-docs/design/chumby-host.md` §4 got a dated correction note
  (design record stays as written).
- `README.md` link fixed (CHUMBY.md → fork README).
- Plan: BC4a marked DONE (was missing), BC4b marked work-done awaiting
  checkpoint review.
- `ruffle/` submodule pin bumped to `4800581a5`.

Historical mentions of CHUMBY.md and hook numbers in docs 14/16, old
plan text, and progress entries were left as written — they describe
what was true at the time.

## 5. Verification

- `cargo build -p ruffle_desktop` clean.
- Local 30 s movie-start run (desktop, DISPLAY :10 — xvfb not
  installed on the dev box): player alive at timeout (exit 124),
  `_getPlatform` answered, `/tmp/movieheartbeat` written in the
  fixture rootfs, no panic. Same criteria as the CI movie-start test.
- chumby-ruffle CI: run 28867051379 (push of `4800581a5`) — see §6.
- chumby-pi CI: pending — commits are local until Jan pushes `main`
  (auto mode blocks that push). Also still pending from BC4a: doc 16
  §4 wants the first green chumby-pi run id recorded after Jan's push.

## 6. CI run results

- chumby-ruffle, pre-squash: run **28867051379** (push of `4800581a5`)
  — success, incl. the movie-start + `_getPlatform` assertions,
  2026-07-07.
- chumby-ruffle, post-squash: run **28870908351** (force-push of
  `7f4d8031b`) — success, incl. movie-start + `_getPlatform`
  assertions, 2026-07-07.
- chumby-pi: pending Jan's push of `main`; record the run id here when
  it is green.

## 7. Squash (Jan, 2026-07-07)

Jan requested the fork's commits be squashed for online review. Done:
`git reset --soft 7f62f5dbf` (upstream base) + one commit
`7f4d8031b`, force-pushed with `--force-with-lease`. The local
verification (build + 30 s movie-start) was re-run on the exact
squashed tree before pushing. chumby-pi's two local commits (`e7db9bb`
BC4a, `6d7c5c1` BC4b) pinned pre-squash submodule hashes that no
longer exist on GitHub, so — both being unpushed — they were rebuilt
as a single local commit pinning `7f4d8031b`; `git submodule update`
works at every public-history commit.
