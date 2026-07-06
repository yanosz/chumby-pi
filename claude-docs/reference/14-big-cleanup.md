# 14 — The Big Cleanup, BC1: repo separation (2026-07-06)

Milestone spec: `chumby-ruffle-plan.md` § "The Big Cleanup". This doc
records how the two public repos were constructed locally, the
copyright audit that shaped them, and the verification evidence.
Nothing has been pushed — BC1 CHECKPOINT gates that.

## 1. Decisions (Jan, 2026-07-06 evening)

- **chumby-ruffle**: regular fork layout — full upstream history,
  current `upstream/master` with our work applied **on top** as one
  squashed commit (linear; no merge commit).
- **chumby-pi**: **fresh public history** (single initial import
  commit). The internal repo (this one) keeps the full 16,885-commit
  history locally as the archive; it is never pushed.
- A `swf-assets/` folder exists in the public repo, empty and
  gitignored, for users to drop SWF assets into.
- Ruffle itself is Apache-2.0/MIT — no copyright concern; the
  copyright work is exclusively about chumby SWFs and derived assets.
- Screenshots and short decompiled-AS excerpts in the docs stay
  (customary for interoperability documentation) — final confirmation
  at the BC1 checkpoint.

## 2. Copyright audit

Method: `git ls-files` sweeps for `.swf/.mp3/.wav`, large-file scan
(>100 KB), diff of the ruffle fork against its upstream base, content
spot-checks.

**Must never be published (excluded from public tree AND history):**

| Path | What it is |
|---|---|
| `docs/reference/appendix/` (3,753 tracked files, 36 MB) | Full extracted export of controlpanel.swf + companion movies: decompiled ActionScript (919 KB `DoAction.as`), artwork SVGs, fonts (incl. Trebuchet MS), sounds, tag dumps |
| `fixtures/rootfs/usr/chumby/alarmtones/*.mp3` (7) | Alarm tones shipped with chumby firmware |
| `fixtures/widgets/*.swf` (2) | Downloaded chumby widget SWFs (builtinclock, unsubscribedclock) |
| `resources/zurks-offline-firmware-classic` | Embedded-repo gitlink; third-party firmware bundle containing chumby-copyrighted material |

**Audited and found clean:** `fixtures/http/**` (handwritten fixture
XML), `fixtures/rootfs/**` minus alarmtones (handwritten state files),
`fixtures/exec/**`, the entire chumby diff to ruffle (16 files,
+1,807 lines, no binaries), `pkg/`, scripts.

**Borderline — resolved at checkpoint (Jan, 2026-07-07):**
- `docs/reference/images/*.png` (panel screenshots show chumby
  artwork): **gitignored, not published** ("of no use"; if needed for
  tests they go to the CI Nextcloud instead). The ~21 image links in
  the public doc copies dangle; the internal repo keeps the images.
- Suspected decompiled-AS quotes in docs 03/04: **false alarm** — on
  inspection the docs contain only interface facts (function names,
  ASnative indices, call counts, URL patterns, behavior described in
  our own words) plus references like "F2:1449" into the private
  appendix. No copied code exists in the public docs; nothing to
  strip.

Consequence: the existing history cannot be pushed (old commits
contain all of the above), hence the fresh-history decision.

## 3. chumby-ruffle construction (`/home/jan/chumby-ruffle`)

The fork's history was already embedded in this repo by the 2026-07-02
subtree import (`059a229a9`, fork tip `68a46df0f`, upstream base
`91b61d405` = master of 2026-06-11), so no fresh 1-GB clone was
needed:

```sh
git clone --no-checkout /home/jan/chumby-pi /home/jan/chumby-ruffle
cd /home/jan/chumby-ruffle
# squashed chumby commit, byte-identical tree of the CURRENT ruffle/
# subdir (includes the three post-import commits), no CRLF filtering:
git checkout -b chumby 91b61d405
NEW=$(git commit-tree origin/master:ruffle -p 91b61d405 -m "chumby: ...")
git reset --hard $NEW
git remote add upstream https://github.com/ruffle-rs/ruffle.git
git fetch upstream master          # 7f62f5dbf, 146 commits ahead
# linear layout per Jan: our commit ON TOP of current upstream
git branch old-merge-layout        # (merge-layout variant, kept briefly)
git reset --hard upstream/master
git cherry-pick <squash-commit>    # → 544c7337e, applied clean
```

Both the intermediate upstream merge and the final cherry-pick applied
**without conflicts** — 3.5 weeks of upstream drift did not touch our
hook sites. Result: `chumby` branch = `upstream/master` (7f62f5dbf) +
`544c7337e` "chumby: control panel host integration (squashed fork,
steps 2.2-3.x)" (16 files, +1,807).

Pitfall recorded: assembling the squash commit via
`git archive | tar | git add` produced 81 spuriously-changed files
(CRLF normalization of upstream test fixtures). `git commit-tree`
against the existing tree object avoids the working-tree roundtrip
entirely.

Before push: delete the scaffolding branch/remote
(`old-merge-layout`, `origin` → the local clone source).

## 4. chumby-ruffle verification (BC1 acceptance: movie RUNS)

- `cargo build -p ruffle_desktop --features chumby` — success.
  (Note: `chumby` is NOT a default feature; a plain build compiles
  but contains none of our code.)
- Ran controlpanel.swf (from the read-only backup) against the new
  binary with the chumby-pi fixtures, flags as in
  `run-controlpanel.sh`: panel boots to the widget clock (correct
  date/time), bend via control FIFO registers (`pressBendSensor` in
  avm_trace), B2 control panel renders fully (chumbypi-channel,
  2.8.87b3, volume/mute/night/music/settings/alarms). Screenshots
  captured; 104 chumby_host log lines; the only error-level log lines
  (11) are environmental ALSA/OpenH264/gamemode noise, byte-identical
  to runs of the pre-merge build.

## 5. chumby-pi-public construction (`/home/jan/chumby-pi-public`)

```sh
git init -b main
git -C /home/jan/chumby-pi archive HEAD \
  ':(exclude)ruffle' ':(exclude)docs/reference/appendix' \
  ':(exclude)fixtures/rootfs/usr/chumby/alarmtones' \
  ':(exclude)fixtures/widgets/*.swf' ':(exclude)resources' \
  ':(exclude).gitignore' | tar -x       # tracked files only
# swf-assets/ (empty, self-gitignored: '*' + '!.gitignore');
# .gitkeep in fixtures/widgets/ and .../alarmtones/
# root .gitignore: *.swf *.mp3 *.wav, appendix, zurk, pkg/build|out
git -c protocol.file.allow=always submodule add /home/jan/chumby-ruffle ruffle
git config -f .gitmodules submodule.ruffle.url \
    https://github.com/yanosz/chumby-ruffle.git   # committed URL
git config submodule.ruffle.url /home/jan/chumby-ruffle  # local override
```

Initial commit: 108 files, +5,443 lines; index verified free of
`.swf/.mp3/.wav/appendix/zurk`. Submodule pins `544c7337e`.

Layout notes:
- `.cargo/config.toml` (aarch64 cross config, doc 08) is at the repo
  root as before; cargo picks it up when building inside `ruffle/`.
- `docs/` keeps its name in BC1; the `docs/` → `claude-docs/` rename
  plus new end-user docs happen in BC2.
- `run-controlpanel.sh` / deb data package expect the user-supplied
  SWF/tones; wiring `swf-assets/` into the scripts is BC2 material.

## 6. chumby-pi verification (deb build from the public tree)

`cargo build --profile dist -p ruffle_desktop --features chumby
--target aarch64-unknown-linux-gnu` inside the public repo's
submodule, then `pkg/build-debs.sh`. Result: recorded below after the
build (fat-LTO cross build runs ~30 min).

- arm64 dist cross-build: success (6m22s, one pre-existing
  ruffle_core warning), binary at
  `ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop`.
- `pkg/build-debs.sh`: success — `chumby-player_0.1.2_arm64.deb`
  (7.8 MB) + `chumby-player-data_0.1.2_all.deb` (586 KB, without
  tones/widget SWFs as intended).
- Note: `chumby-player-data` deb built from the public tree lacks
  alarmtones/widget SWFs by design; it must never be released
  publicly anyway (plan, standing constraint).

## 7. BC1 checkpoint resolution (2026-07-07)

1. GitHub repos created by Jan; SSH push access verified (deploy-key
   identity). `gh` remains uninstalled — plain `git push` suffices.
2. Screenshots excluded (see §2); no decompiled code in public docs.
   Public initial commit amended → 88 files.
3. CI infrastructure (BC3 groundwork, set up by Jan): a dedicated
   Nextcloud CI user with an app token; rclone configured purely via
   `RCLONE_CONFIG_RSHARE_*` env vars (these are the GitHub Actions
   secret names). `testdata/` holds controlpanel.swf — verified
   byte-identical (md5) to the read-only backup copy — and `deploy/`
   receives artifacts. Credentials live only in Claude's private
   session notes and GitHub secrets, never in either repo. This
   supersedes the earlier share-link plan for CI downloads.
4. Post-approval (Jan approved): push both repos, swap
   `/home/jan/chumby-pi-public` into place as the working repo
   (internal repo archived locally), decide deb VERSION bump
   (0.1.2 → 0.2.0?) since the player now carries the 146 merged
   upstream commits.

## 8. BC1 completed (2026-07-07, 00:00–00:05 CEST)

- Pushed `chumby` → `yanosz/chumby-ruffle` (default branch). SSH
  deploy-key identity had no access to these repos; pushes go over
  HTTPS with gh credentials (`gh auth setup-git`), and pushing the
  upstream history required adding the `workflow` OAuth scope
  (`gh auth refresh -s workflow`) because upstream ships
  `.github/workflows/*`.
- Pushed `main` → `yanosz/chumby-pi` (initial import, 89 files —
  doc 14 included, screenshots excluded).
- Swap done: `/home/jan/chumby-pi` = public working repo,
  `/home/jan/chumby-pi-internal` = archived full-history repo (never
  push). Untracked gitignored assets restored into the working tree:
  alarmtones, widget SWFs, appendix, images.
- Post-swap smoke test PASS (panel boot + bend + B2 from the swapped
  tree).
- Deb VERSION bump: still open, decide before the next deb release.

## 9. BC2: documentation (2026-07-07)

All four BC2 items done; work recorded per item. Only BC1 gated
pushing, so BC2 commits are pushed as they land.

1. **chumby-ruffle architecture doc**: `CHUMBY.md` at the fork's repo
   root (root-level file = discoverable and collision-free for future
   upstream merges; upstream `docs/` only holds fuzzing.md). Seeded
   from patch-notes.md + design/chumby-host.md, checked against the
   actual module source. Covers: why a fork (the three non-standard
   channels), hook table H1–H11, `ChumbyHost` trait + `OnceLock`
   registry deviation, ASnative table incl. the (4,39) collision and
   the `WidgetPlayer.onPress` surgery, `ChumbyNavigator`, virtual
   rootfs confinement + `{FIXTURES}` token, mpv audio (incl. the
   state-constant mapping), control FIFO protocol, touch/bend mapping,
   build/run commands, linear-branch discipline. Pushed as
   `00decada1` on `chumby`.
2. **`docs/` → `claude-docs/`**: `git mv` (untracked appendix/images
   moved with the directory). Path references updated in CLAUDE.md,
   the plan, `.gitignore`, `verify-screens.sh`, pkg comments,
   `fixtures/README.md`. Doc content itself unchanged ("stays
   as-is"); their internal `docs/...` self-references are historical
   text and were left alone. CLAUDE.md now states artifacts go under
   `claude-docs/` and `docs/` is end-user documentation.
3. **End-user docs**: new root `README.md` (what/why, repo map, quick
   start, SWF-assets copyright notice, status) + `docs/setup.md`
   (bare Pi → chumby walkthrough: sources, SWF asset placement,
   cross-build, debs, config.txt overlay, install/operate, desktop
   run) + `docs/hardware.md` (the `/etc/default/chumby-player`
   override table; other TFT: DRM requirement, overlay choice,
   rotation gotcha, by-path `WLR_DRM_DEVICES` incl. per-model SPI
   address, renderer choice, touch calibration flags incl. the
   inverted-boolean gotcha; other sound device: PipeWire default sink
   vs `CHUMBY_AUDIO_DEVICE`; other Pi models). Facts drawn from
   claude-docs 08/10/12 and fixtures/README.
4. **swf-assets/ wired into scripts** (BC1 §5 leftover):
   `run-controlpanel.sh` and `pkg/build-debs.sh` now default to
   `swf-assets/controlpanel.swf` (`CHUMBY_SWF` still overrides), with
   pointing-to-docs error messages. Locally `swf-assets/
   controlpanel.swf` is an untracked symlink to the read-only backup
   copy, so Jan's workflow is unchanged. Verified: `pkg/build-debs.sh`
   runs clean end-to-end from the new default (0.1.2 debs rebuilt);
   desktop run against the new path verified with a freshly built
   debug binary.

Open (unchanged from §7): deb VERSION bump 0.1.2 → 0.2.0 before the
next deb release; BC3 (CI on both repos, declare the doc-12 library
deps) is next.
