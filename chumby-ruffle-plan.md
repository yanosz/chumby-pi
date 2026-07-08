# Plan: Chumby Control Panel on Raspberry Pi via Modified Ruffle

## Project context (read first)

**Overarching goal:** Run chumby widgets on a Raspberry Pi with the original chumby
look and feel. The original `controlpanel.swf` (Flash Lite 3, AVM1/AS2) is used
unmodified; we modify **Ruffle** to supply the chumby-specific environment (native
API extensions, script calls, frame control).

**Already done (Milestone 0):** Plain clock widgets run under unmodified Ruffle.

**Hard rules — do not violate:**
1. **Ask the user for feedback before iterating.** At every checkpoint marked
   `CHECKPOINT` below, stop, summarize findings, ask, and wait. Do not proceed
   past a checkpoint on your own.
2. **The Ruffle patch must be clean and maintainable.** All chumby-specific Rust
   lives in ONE isolated module/crate behind a `chumby` cargo feature. Upstream
   files get only minimal registration hooks. Document every hook point.
3. **`controlpanel.swf` is never modified.** Control is exerted only from the
   Rust side (API results, variables, frame navigation).
4. Any functionality living OUTSIDE the SWF (shell scripts, daemons, downloaded
   updates) must be cataloged so it can be re-implemented for the Pi.
5. The SWF has more screens than we want. **Before implementing support for any
   screen, present it to the user and ask whether it is in scope.** Maintain a
   `claude-docs/feature-decisions.md` with status: needed / skip / undecided.
6. Prefer small, verifiable steps with a written result over broad exploration.
   If you find yourself reading code for more than ~30 minutes without producing
   a documented finding, stop and write down what you have, then reassess.

**Local resources:**
- Offline firmware: `git clone https://github.com/francistheodorecatte/zurks-offline-firmware-classic`
- Device backup: `/home/jan/chumby_backup` (shows real invocations of the flash
  player with different parameters — treat as ground truth)
- Decompiler: `ffdec` (JPEXS Free Flash Decompiler). Use the CLI for
  reproducible exports, e.g.:
  `ffdec -export script,frame,image,shape OUTPUT_DIR controlpanel.swf`

**Online resources:**
- Chumby wiki (chumby's Flash extensions):
  - https://wiki.chumby.com/index.php?title=ChumbyNative  ← primary API reference
  - https://wiki.chumby.com/index.php?title=Developing_Widgets_for_Chumby:_Sensor_Access
  - https://wiki.chumby.com/index.php?title=Controlling_BTplay
  - https://wiki.chumby.com/index.php?title=Developing_Widgets_for_Chumby:_Testing_Widgets
    (documents `chumbyflashplayer.x -i` invocation and the watchdog /
    `/tmp/flashplayer_started` mechanism)
- The chumby hardware API is exposed to AS2 mostly as `ASnative(5, n)` calls
  (e.g. raw touchscreen coords 5,10/5,11; LCD mute 5,19/5,20; speaker mute
  5,17/5,18; DC power 5,16; accelerometer 5,60; bend sensor 5,25). A community
  list exists at:
  https://speakerdeck.com/scottjanousek/developing-flash-lite-widgets-for-the-chumby-platform
- chumbysphere forum (search for "controlpanel", "ASnative", "fscommand"):
  https://forum.chumby.com — note esp. thread id=9663 ("Success!", Sony dash),
  which confirms a custom/replacement control panel only needs the ChumbyNative
  call set.
- Ruffle:
  - Repo: https://github.com/ruffle-rs/ruffle
  - Wiki "Helpful Resources" (SWF19 spec, AS2 language reference, AMF0 spec —
    needed for SharedObject fixtures):
    https://github.com/ruffle-rs/ruffle/wiki/Helpful-Resources
  - AVM1 native function plumbing: `core/src/avm1/function.rs`
    (`NativeFunction` type), `core/src/avm1/globals.rs` (where globals like
    `ASSetPropFlags` are registered — model the chumby table on this),
    `core/src/avm1/activation.rs` (interpreter loop, relevant for frame
    control / call interception).
  - Search the repo for `fscommand` and `NavigatorBackend` to find where
    getURL/fscommand leave the VM — these are the hook points for script calls.
  - Linux ARM64 release builds exist; the desktop frontend (winit + wgpu) runs
    on Raspberry Pi.

**Working assumptions (clarified with the user 2026-06-12):**
- Device generation: Chumby Classic (320×240, AVM1) is the working basis, but
  this is NOT a hard commitment — the Pi display resolution may differ, so do
  not bake 320×240 into the Rust code as a constant; keep scaling/resolution
  configurable and note any place where the SWF itself assumes the resolution.
- Development & runtime target for M1/M2: **Linux amd64 desktop only.**
  Raspberry Pi deployment, Raspbian packaging, ARM builds etc. are a future
  milestone and must NOT be worked on or designed-for now beyond rule 2's
  general cleanliness. First prove M1/M2 are reachable.
- Network posture: **decided per use-case, with the user.** Default for M2 is
  fixtures/mocks for everything. Long-term: anything depending on chumby.com
  or device registration is unwanted; genuinely networked features the user
  wants (e.g. internet radio streaming) will get real network paths later.
  The environment contract (03) must therefore tag each touchpoint with a
  proposed network class: `mock-forever` / `real-network-later` / `discuss`.

**Feature scope decisions so far (seed for claude-docs/feature-decisions.md):**
- EXCLUDED: all social features (chum lists, sending widgets, etc.).
- EXCLUDED: hardware configuration screens — touchscreen calibration, WiFi /
  network setup — EXCEPT volume and brightness, which are wanted.
- UNDECIDED pending the M1 screen catalog: music sources (iPod/USB playback,
  btplay), alarms, anything else found. The catalog in 05-screens.md exists
  precisely so the user can decide these — present them, don't implement.

---

## Milestone 1 — Understand the Control Panel

**Deliverable:** a reference under `claude-docs/reference/` that makes Milestone 2
mechanical. Acceptance criteria (from the user):
- Documents the screens and functionality of `controlpanel.swf`, especially
  user-facing elements.
- Includes the complete ActionScript (raw export as appendix + curated
  walkthrough).
- Documents what the environment must provide ("environment contract").
- Documents differences between the offline-firmware panel, the backup panel,
  and (if obtainable) the most recent downloadable version.

### Step 1.1 — Inventory the artifacts (no decompiling yet)
- Locate every `controlpanel.swf` (and related SWFs it loads) in:
  (a) the offline firmware clone, (b) `/home/jan/chumby_backup`.
- Record for each: path, sha256, file size, SWF header version, compressed?
- Find how the flash player is invoked: grep the firmware and backup for
  `chumbyflashplayer.x`, collect every invocation with full arguments,
  environment variables, and FlashVars. Document in
  `claude-docs/reference/01-invocations.md`.
- Identify the scripts/daemons the panel ecosystem uses: list everything under
  `/usr/chumby/scripts/` (or equivalent) in firmware + backup, with a one-line
  purpose each → `claude-docs/reference/02-external-scripts.md`.
- Try to locate the most recent official control panel: check the offline
  firmware repo docs/README, archive.org snapshots of chumby update URLs, and
  the zurks repos. If not obtainable in ~30 min, note that and move on.

`CHECKPOINT 1: present the inventory to the user before decompiling.`

### Step 1.2 — Decompile and export
- Use ffdec CLI to export, reproducibly (record the exact command):
  scripts, frames (as images), frame labels, symbols, embedded assets — for
  each controlpanel variant.
- Store raw exports under `claude-docs/reference/appendix/<variant>/` (this satisfies
  the "complete code" criterion).
- Also export any secondary SWFs the panel loads (`loadMovie` targets).

### Step 1.3 — Build the environment contract (core artifact)
Statically scan the exported ActionScript of the **backup** variant first
(it is ground truth), then the offline variant, for every external touchpoint:
- `ASnative(` — record (a,b) indices, wrapper variable name, arguments, how the
  return value is used.
- `fscommand(`
- `getURL`, `loadVariables`, `loadMovie`, `XML.load`, `loadVars` — record URL
  patterns, esp. `file://` paths, localhost HTTP, and chumby's file://
  protocol extensions (chumby extended file:// to execute things — verify
  against the wiki).
- `SharedObject` usage (persistence — fixtures will need AMF0).
- FlashVars / `_root` variables read but never set inside the SWF (these come
  from the command line — cross-reference with 01-invocations.md).
- `System.capabilities` or version checks.

Produce `claude-docs/reference/03-environment-contract.md`: one table row per
touchpoint: *call site (script/frame) → screen → purpose → request format →
expected response format → network class (mock-forever / real-network-later /
discuss) → notes for Pi reimplementation*. This file is the single most
important output of M1.

### Step 1.4 — Dynamic discovery with unmodified Ruffle
- Run the backup controlpanel.swf in stock Ruffle desktop with
  `RUST_LOG=warn` (or `ruffle_core=debug`). Capture and triage every
  "unimplemented" / unknown-call warning into the contract table (mark rows
  observed dynamically vs only statically).
- Use Ruffle's debug UI to inspect `_root` variables and try jumping frames
  manually; note what the wizard checks before letting the main screen appear.
- Document how far the panel currently gets and what it blocks on →
  `claude-docs/reference/04-ruffle-gap-analysis.md`.

### Step 1.5 — Screen / frame catalog
- From ffdec frame labels + frame exports + the dynamic experiments, produce
  `claude-docs/reference/05-screens.md`: per screen — name/frame label, screenshot,
  user-facing elements, which contract rows it depends on, and a proposed
  status (needed / skip / undecided) **as a question for the user, not a
  decision**. Pre-apply the known exclusions (social features; hardware config
  except volume/brightness) as `skip`, but still catalog those screens — their
  wizard/init code may run regardless and must be understood to be hopped over.

### Step 1.6 — Variant diff
- Diff the offline-firmware panel vs the backup panel (script-level diff of
  the ffdec exports; ignore cosmetic recompilation noise), and vs the newest
  obtainable version if found. Focus on: removed network calls, changed
  ASnative usage, wizard differences → `claude-docs/reference/06-variant-diff.md`.
  Note in particular what zurk's offline firmware changed *outside* the SWF
  (server stubs, scripts) vs inside it — that is the existing prior art for
  exactly what we are doing.

`CHECKPOINT 2 (= end of Milestone 1): present 03/04/05/06 to the user.
Get explicit screen-scope decisions before starting Milestone 2.`

---

## Milestone 2 — Stub the API

**Goal:** The control panel main screen is visible; the setup wizard is hopped
over; the most relevant API extensions exist as mocks whose results we control
from Rust (fixtures), without touching the SWF.

### Step 2.1 — Architecture (write before coding)
Write `claude-docs/design/chumby-host.md` proposing:
- A `ChumbyHost` Rust trait: one method per contract row category
  (native call, script exec, url fetch, persistence). Implementations:
  `FixtureHost` (M2: returns canned data from a `fixtures/` directory keyed by
  call signature) and later `PiHost` (real implementations).
- Integration points in Ruffle (minimal, each documented):
  1. Registration of the `ASnative(5, n)` (and any other indices found) table
     into AVM1 globals — modeled on how `core/src/avm1/globals.rs` registers
     built-ins, but living in `chumby.rs` behind `#[cfg(feature = "chumby")]`.
  2. Interception of fscommand / getURL / loadVariables for chumby URL
     patterns (likely via the `NavigatorBackend` or equivalent) routing to
     `ChumbyHost`.
  3. A bootstrap controller that, after load, can set `_root` variables and
     issue `gotoAndStop`-equivalents from Rust to skip wizard frames —
     prefer setting the variables the wizard itself checks (found in M1)
     over brute-force frame jumping.
- Fixture format proposal (e.g. directory of files named by call signature,
  JSON or raw bodies; AMF0 blobs for SharedObject if needed).

`CHECKPOINT 3: get the design approved before writing Ruffle code.`

### Step 2.2 — Implement incrementally
Order of work (each step ends with the panel observably getting further):
1. Fork/branch Ruffle at a pinned commit (record it). Add the `chumby` feature
   and empty `chumby` module; verify build unchanged with feature off.
2. Implement the ASnative table with logging stubs (every call logged with
   args, returns a configurable default). Run the panel; update the gap
   analysis.
3. Implement URL/script interception with `FixtureHost`; add fixtures for the
   calls the wizard makes.
4. Implement the wizard skip (variables/frame control).
5. Iterate: run → read logs → add the next fixture/stub → re-run. After each
   newly reached screen, screenshot it and append to a running progress doc.
   **Do not implement functionality for screens marked "skip" or
   "undecided".** When an undecided screen blocks progress, ask the user.

### Step 2.3 — Acceptance
- `cargo run --features chumby -- controlpanel.swf` (or documented equivalent)
  shows the main control panel screen.
- A `fixtures/README.md` explains how to change any mocked result.
- A `claude-docs/patch-notes.md` lists every upstream Ruffle file touched and why
  (rebase guide).
- Demo to the user; collect feedback before defining Milestone 3.

---

## Milestone 3 — Raspberry Pi deployment (defined 2026-07-02, user decision)

**Goal:** the control panel runs on a Raspberry Pi with the same
fixture-backed behavior as on the desktop, plus the first real-hardware
bindings (display, audio out, backlight). The two flows deliberately left
untested on the desktop get their first exercise on the device (user
2026-07-02: prefer real-hardware testing over desktop verification):
**alarm ring-through (B6)** and **My Streams playback** (real network —
confirm the network-class change at CHECKPOINT 4 and record it in
feature-decisions.md).

**Access:** the Pi will be reachable via SSH (user is setting this up;
session paused until then). Note the M1/M2 assumption "desktop amd64 only"
ends here; rule 2 (clean patch) still applies unchanged.

### Step 3.1 — Device survey (needs SSH)
- Record: Pi model, SoC/architecture (armv7 vs aarch64), OS + version,
  display (resolution, connector, backlight sysfs path), audio devices,
  input devices (keyboard? touchscreen? GPIO button for bend?), storage,
  network, mpv availability → `claude-docs/reference/07-pi-survey.md`.
- Propose, per finding: build strategy (cross-compile from the dev box vs
  native build on the Pi), target resolution/scaling (NOT hardcoded —
  see working assumptions), audio route, bend-input mapping.

`CHECKPOINT 4: present survey + proposed build/deploy approach and the
stream-playback network decision. Wait for approval before building.`

### Step 3.2 — ARM build
- Build `ruffle_desktop --features chumby` for the Pi architecture at the
  pinned fork commit; record toolchain, exact commands, and workarounds →
  `claude-docs/reference/08-pi-build.md`. Known risk to document: the wgpu/
  graphics stack on the Pi (which renderer path works, at what FPS).

### Step 3.3 — Deploy & smoke test
- Deployment layout on the Pi (binary, `fixtures/`, `controlpanel.swf`
  copy — the Pi must NOT depend on `/home/jan/chumby_backup` —, mpv,
  launcher script). Document → `claude-docs/reference/09-pi-deploy.md`.
- Smoke test on device: boot to normal operation, widgets, bend summon
  via the chosen input, the five M2-verified screens; then first-time
  tests: alarm set → ring (visible + audible), My Streams playback.
  Results + photos/screenshots → `claude-docs/progress.md` (M3 section).

`CHECKPOINT 5: user tests on the device; collect feedback before 3.4.`

### Step 3.4 — Brightness & night mode (E2, B4) — MOVED TO END OF PLAN
- Moved here from M2 (2026-06-13); **moved to the very end of the plan
  2026-07-06 (user decision)**: the current TFT's backlight is
  hardwired (no dimming possible — findings and panel alternatives in
  `claude-docs/reference/13-brightness-hardware.md`), a dimmable replacement
  panel will be ordered but the model is not yet decided. See "Final
  milestone" at the bottom of this plan.

### Step 3.5 — Performance & input cleanup (added 2026-07-06, user request)
Bugfix/cleanup backlog, not gating 3.3/3.4 but tracked so it isn't lost.
Document findings in `claude-docs/reference/11-perf-and-input-cleanup.md`.

1. **CPU load higher than expected.** Ruffle runs at 200–300% CPU on the
   Pi 3B+ (see 09-pi-deploy.md / 10-tft-display.md measurements) — bearable,
   but the SoC gets hot. The original content (Chumby Classic controlpanel
   + widgets) was authored for a 350 MHz ARM9 (Freescale i.MX233), so this
   load is almost certainly Ruffle overhead, not workload. Investigate and
   mitigate: release/LTO build flags, wgpu/pixman renderer path and
   frame-rate/vsync behavior, whether Ruffle is busy-polling or redrawing
   unnecessarily on a static frame, and any chumby-fork-added per-frame work
   (input polling, avm_trace logging left on). Record cause(s) found and
   what was tried, including things that did NOT help.
2. **Mouse pointer visible on the touchscreen.** cage/wlroots draws a
   cursor even though the device has no mouse — meaningless on a
   touch-only kiosk. Investigate hiding it by default and only showing it
   if a real pointer device is plugged in (e.g. cage cursor-theme/hide
   options, wlroots seat capabilities, or a udev-triggered toggle).

## Milestone: "The Big Cleanup" — NEXT SESSION (user, 2026-07-06)

Publish the project as two GitHub repos. This supersedes the standing
"no pushable remotes / do not push" rule for exactly these two repos.

**1. https://github.com/yanosz/chumby-ruffle** — the patched Ruffle:
- Contains all chumby-relevant Ruffle changes; **squash** the fork's
  commits; **merge current upstream** Ruffle in.
- Document the architectural approach in-repo: how the chumby feature
  works — the ASnative table registration, navigator/URL interception,
  the virtual rootfs replacing chumby's file/exec surface (the
  "replaced syscalls"), mpv audio backend, control FIFO, touch/bend
  hooks. Today's `claude-docs/patch-notes.md` (hook map) is the seed.

**2. https://github.com/yanosz/chumby-pi** — everything else:
scripts, Debian package definitions, documentation, fixtures.
- `chumby-ruffle` becomes a **git submodule** (replaces the vendored
  `ruffle/` tree).
- Rename the existing `docs/` → **`claude-docs/`** (internal
  engineering record, stays as-is); create a fresh `docs/` + `README`
  as **end-user documentation** with setup instructions.
- **gitignore all copyrighted material** — audit `fixtures/` etc.:
  `controlpanel.swf`, downloaded widget SWFs, chumby alarm tones.
  Users obtain `controlpanel.swf` via a Nextcloud link (Jan provides).
- End-user docs must cover **hardware variation**: how to use a
  different TFT (overlay choice, `WLR_DRM_DEVICES` override in
  `/etc/default/chumby-player`, touch calibration) and a different
  sound device (`CHUMBY_AUDIO_DEVICE`, PipeWire default sink).

**3. GitHub Actions:**
- `chumby-ruffle`: build passes and the player can **start the movie**
  (controlpanel.swf is gitignored — CI downloads it from the Nextcloud
  link, kept out of the repo).
- `chumby-pi`: the debs are **installable on a Pi** (all dependencies
  declared — note doc 12 currently documents deliberately undeclared
  library deps, which this milestone must fix) and the binaries run.

Standing constraint: released artifacts (debs, CI caches) must never
ship the copyrighted SWFs/tones — the `-data` package cannot go into
GitHub releases as-built.

### Checkpoints (agreed with user, 2026-07-06)

**Status: BC1 PASSED and executed 2026-07-07 (~00:05) — both repos
live (chumby-ruffle: `chumby` branch = upstream + squashed commit;
chumby-pi: fresh-history `main`), working repo swapped to the public
tree, internal archive at `/home/jan/chumby-pi-internal`. Full record:
`claude-docs/reference/14-big-cleanup.md`. BC2 work done 2026-07-07
(architecture doc CHUMBY.md in chumby-ruffle — pushed; docs/ →
claude-docs/; end-user README + docs/setup.md + docs/hardware.md;
swf-assets/ wired into run-controlpanel.sh and build-debs.sh) —
record in doc 14 §9, awaiting CHECKPOINT BC2 review by Jan (session
ended here 2026-07-07). NOTE: chumby-pi commits are local only — the
Claude Code auto-mode permission classifier blocks `git push origin
main`; Jan pushes manually or approves the push interactively.
Still open: deb VERSION bump 0.1.2 → 0.2.0 before the next deb
release. BC3 (CI green on both repos, declare doc-12 library deps)
starts NEXT SESSION.**

- **CHECKPOINT BC1 — repos separated & verified locally, BEFORE any
  push.** Fork squashed, current upstream Ruffle merged, chumby-pi
  restructured with chumby-ruffle as submodule, copyrighted material
  audited and gitignored. Acceptance is not "builds" but **builds and
  controlpanel.swf runs** (movie starts, panel usable) — the upstream
  merge can compile clean and still break the ASnative hooks. User
  reviews, approves, and only then are the repos pushed. This
  checkpoint guards the irreversible action: nothing copyrighted may
  ever enter public git history.
- **CHECKPOINT BC2 — documentation.** chumby-ruffle architecture doc
  (seeded from patch-notes.md), `docs/` → `claude-docs/`, new end-user
  `docs/` + README including other-TFT and other-sound-device
  instructions. After BC1, normal commits/pushes flow freely; only BC1
  gates pushing.
- **CHECKPOINT BC3 — CI green on both repos.**
  Includes fixing the doc-12 undeclared library deps.
  **DONE 2026-07-07** (claude-docs/reference/15-ci.md): chumby-ruffle
  run 28862399194 and chumby-pi run 28862410493 both green; deps
  declared, debs at 0.2.0. Repo visibility per user decision:
  chumby-ruffle public, chumby-pi private.
- **CHECKPOINT BC4a — chumby-ruffle source refactoring** (added
  2026-07-07, user; a LATER session, not the BC3 one): remove the
  feature-toggle logic — chumby features are always needed/built.
  **DONE 2026-07-07** (claude-docs/reference/16-bc4a-feature-removal.md):
  chumby-ruffle CI run 28864269056 green; chumby-pi CI pending Jan's
  push of `main`.
- **CHECKPOINT BC4b — chumby-ruffle docs** = milestone done: every
  ASnative instruction properly documented; README swap. Details
  below. **Work done 2026-07-07**
  (claude-docs/reference/17-bc4b-asnative-docs.md): ASnative(5,N)
  per-index reference in the fork's README, README.ruffle.md swap,
  source-comment audit (indices ↔ names annotated both ways, hook
  numbering dropped per user 2026-07-07); local movie-start check
  green. **CHECKPOINT PASSED 2026-07-07: Jan accepted BC4b as done**
  (detailed review still pending on his side; corrections may come
  later). **This closes the Big Cleanup milestone.**

### Decisions for BC3 (recorded 2026-07-06)

- **controlpanel.swf source for CI:** a secret Nextcloud share URL,
  provided by Jan. The URL is SECRET — never committed anywhere, in
  either repo; it is configured as a GitHub Actions secret (e.g.
  `CONTROLPANEL_SWF_URL`) and referenced only via
  `${{ secrets.CONTROLPANEL_SWF_URL }}`. End-user docs say "obtain
  controlpanel.swf" without embedding the link.
- **arm64 strategy:** GitHub-hosted arm64 runners
  (`ubuntu-24.04-arm`, free for public repos since Jan 2025); the deb
  install/run test executes inside an arm64 Debian container matching
  Raspberry Pi OS. Fallback if unavailable: qemu via
  `qemu-user-static`. A self-hosted runner on the Pi is rejected
  (security exposure on a public repo).

### BC4: chumby-ruffle housekeeping (added 2026-07-07, user — next session)

After BC3, on the chumby-ruffle repo (explicitly NOT to be started in
the BC3 session). Two checkpoints (user 2026-07-07):

**CHECKPOINT BC4a — source-code refactoring:**
- **Remove the feature-toggle logic.** Drop the `chumby` cargo
  feature; the fork always builds with the chumby code (user decision
  2026-07-07 — assume chumby features are always needed). This
  supersedes the "behind a `chumby` cargo feature" clause of hard
  rule 2; the rest of rule 2 (one isolated module, minimal documented
  hooks in upstream files) stands. Same-step follow-ups: update
  `--features chumby` in both CI workflows, CHUMBY.md, chumby-pi
  docs/scripts (run-controlpanel.sh, build-debs.sh, docs/setup.md),
  and note the change for future upstream merges. Acceptance: CI on
  both repos still green (incl. movie-start) after the refactor.

**CHECKPOINT BC4b — docs (= Big Cleanup milestone done):**
- **Every ASnative instruction properly documented.** Audit the
  ASnative table in the chumby module: each registered (a,b) index
  must have documentation (purpose, args, return, fixture behavior)
  in the repo's chumby doc.
- **README swap.** The repo-root README becomes the chumby doc
  (today's CHUMBY.md content); the original Ruffle README is renamed
  (e.g. `README.ruffle.md`) and linked from the new one.

After the cleanup: **widget channels** (user 2026-07-06, "one of the
next sessions").

### NEXT SESSION first: fix the _setTimeZone bug (user, 2026-07-07) — DONE 2026-07-07

**Fixed 2026-07-07** (fork commit `6c99ebd1f`, amended +
`--force-with-lease` per branch discipline below): explicit
`_setTimeZone` arm writes the trimmed value to `/psp/timezone`;
round-trip unit test `test_timezone_set_get_round_trip` passes; local
45 s movie-start check green; fork CI green on the re-squashed commit
(run 28884211332); doc 17 §1 has the dated fixed note.

Small, before or alongside the widget-channels start. The bug (found in
the BC4b audit, confirmed by Jan against real hardware; fork README
5,178 row, doc 17 §1):

- **Bug:** `_setTimeZone` (5,178) falls into `FixtureHost`'s generic
  `_set…` store, while `_getTimeZone` (5,177) reads `/psp/timezone`
  from the virtual rootfs — so set → get does not round-trip. On a
  real chumby it does.
- **Fix:** give `_setTimeZone` an explicit arm in
  `fixture.rs::native()` that writes the value to `/psp/timezone` via
  the rootfs (mirroring the 5,177 read; trim/format to match the
  seeded fixture file). Remove the `KNOWN BUG` comment at the
  `_getTimeZone` arm and the bug wording in the README 5,178 row +
  plan future-milestones line; doc 17 gets a dated "fixed" note.
- **Acceptance:** set → get round-trips (exercise the natives — e.g. a
  unit test in `fixture.rs` like the audio test, or a scripted run);
  `/psp/timezone` in the state dir contains the set value; CI
  movie-start still green on the (re-squashed) fork commit. Full
  clock/timezone panel verification (B3, E5, E12) stays a separate
  later item.
- Branch discipline reminder: amend into the single fork commit and
  `--force-with-lease`, do not stack a second commit.

### Milestone: UI policy — disable unsupported panel controls (direction confirmed by user 2026-07-07)

**Context/decision (2026-07-07):** the real chumby runs in the user's
LOCAL timezone (verified via SSH: `date` → CEST, `/psp/timezone` =
`Europe/Oslo`, `/etc/localtime` → `/psp/localtime`). On the Pi the OS
owns timezone + NTP, so the panel must not set them: the clock screen
shows the timezone read-only, TZ picker and NTP selection are
disabled, only the 12/24h checkbox stays functional (recorded in
feature-decisions.md). This needs a **general mechanism** for
disabling UI elements the Pi doesn't support — expected to be reused
by later features.

**Chosen approach (user confirmed direction; alternatives weighed for
simplicity/performance/maintainability/robustness, user 2026-07-07):**
host-side **property-based disabling** driven by a declarative policy
config, applied by the existing bootstrap controller on frame entry
(re-applied if the SWF resets it). Explicitly REJECTED alternatives:
bytecode patching at load (fragile across panel variants, against the
spirit of rule 3), renderer patching (cosmetic only — input stays
live), native no-ops (UI looks functional but silently ignores input
= worst UX for alarm-safety). Runtime property setting is the same
proven mechanism as the wizard skip, costs nothing per frame, keeps
the SWF untouched, and each policy entry is data, not code.

Requirements from the user (2026-07-07):
- **Stable element identification.** Instance paths may be unstable
  (auto-named `instanceN` depends on load order). The policy config
  therefore supports MULTIPLE selector alternatives per element —
  e.g. dotted instance path, parent-path + depth, symbol/export ID —
  tried in order, first match wins. A selector that resolves to
  nothing is a logged WARNING (visible failure), never silent.
- **All form-input kinds, no format assumptions.** Actions operate at
  the DisplayObject level so they are type-generic: `hide`
  (`_visible = false`), `disable` (`enabled = false` + lowered
  `_alpha` for a grayed-out look — no SWF art needed), `readonly`
  (input TextField → dynamic / unselectable). Per-object-class
  mapping lives once in the chumby module. Fallback per element: if a
  control is script-hit-tested and can't be disabled, hide it.
- Policy config is a versioned file in chumby-pi (fixtures-style):
  rows = screen/frame label → selector alternatives → action.

Steps:
1. Catalog the clock-settings screen(s) (B3, E5, E12 — currently
   unverified) from the ffdec export + a live fixture run: every
   control, its instance path/depth/symbol, which natives/psp keys it
   touches. Real chumby = reference oracle (read-only, ask first).
   → `claude-docs/reference/18-clock-screen-and-ui-policy.md`.
   Known issue to catalog (found by Jan on-device 2026-07-07, decision:
   no fix, record only): the 12/24h toggle persists (`/psp/clock_format`)
   but the running clock only picks it up on restart/reboot — on real
   hardware the panel pushes `_setSlaveVar("_chumby_clock_format", …)`
   every heartbeat to the slave clock instance; our in-movie widget
   path (localCache, CHECKPOINT 3) has no slave-var → running-widget
   bridge, so the value is only injected at widget start.
2. `CHECKPOINT UI1:` present catalog + concrete policy entries
   (what gets hidden vs disabled) + config format before wiring.
   **PASSED 2026-07-07** (doc 18 §8): disable = dim + dead input for
   ntpButton + setTimezoneButton, ntpLabel dims too (3rd rule),
   config = fixtures/ui-policy.toml, no belt-and-braces rules.
   Also decided there: B3 orphaned in 2.8.87b3 → policy covers E5
   only; E12 unreachable once the globe is disabled.
3. Implement the policy mechanism in the chumby module + the clock
   screen entries. Acceptance: TZ/NTP controls visibly disabled and
   inert; 12/24h round-trips (with the `_setTimeZone` fix in place,
   set → get consistent); desktop fixture run + on-device check; CI
   movie-start green; patch-notes/CHUMBY docs updated.
   **Implemented 2026-07-07** (fork `adeb6058d`, amended; doc 18 §9):
   `ui_policy.rs` + `fixtures/ui-policy.toml` (3 rules); desktop
   verification complete incl. pick-traced inertness proof and live
   12/24h round-trip; docs updated (fork README section + hook map,
   patch-notes, fixtures README).
   **DONE 2026-07-07:** fork CI (`Build and start controlpanel.swf`)
   green on `adeb6058d`; hot-replace deployed to the Pi (doc 09) and
   Jan confirmed on-device that the NTP + SET TIME ZONE controls render
   disabled on Settings → TIME/DATE. UI-policy milestone (clock screen)
   complete.
4. **Extension 2026-07-07 (user request):** disabled four Settings-menu
   entries — Network (E3), Chumby Info (E6), Touchscreen (E4),
   Brightness (E2) — via the same disable mechanism at the menu-icon
   level (doc 18 §10). Fixtures-only change (4 rules in
   `fixtures/ui-policy.toml`), deployed to the Pi; **Jan confirmed
   on-device 2026-07-07** that all four icons render disabled/inert.
   NB: `settings-brightness` must be dropped by the final brightness
   milestone (noted there).

Ordering: AFTER the `_setTimeZone` fix above; relation to widget
channels start = user's call at CHECKPOINT UI1.

## Milestone: Single local widget channel (scoped 2026-07-08, user)

Supersedes the old one-line "Widget channels & management" future-milestone
entry. Scope narrowed by the user 2026-07-08 after discussion:

**Direction (user decisions 2026-07-08):**
- **One hardcoded channel = all available widgets**, generated at boot (a
  boot step enumerates the widgets we ship and emits the single
  `profile.xml`, replacing today's hand-written
  `fixtures/http/xml.chumby.com/xml/profiles`).
- **Remote channels + registration are deferred to the very last feature**
  of the project. chumby.com is on life support (registration is *possible*
  but not desirable now) — do NOT treat it as dead, and do NOT build a
  server story here.
- **Dashboard preview stays**, backed by a **static** thumbnail via
  `loadMovie` (contract row F2:4611, `{widgets}+<thumbnail href>` — a
  per-widget image/SWF the profile points at, NOT a live second render).
  This is compatible with the chosen `localCache` single-instance playback
  path — no revival of the rejected master/slave dual-instance system.
- **Channel-management UI that is useless without remote/download gets
  disabled** via the existing `fixtures/ui-policy.toml` mechanism (same as
  the Settings-menu icons).

**Widget playback architecture** stays the decided `localCache` in-movie
path (feature-decisions.md). Widget-channel decisions come from
`06-variant-diff.md §3` (zurk's offline `profiles.sh` multi-channel is the
prior art) and `03-environment-contract.md §5a/5b` (profiles/setprofile/
thumbnail rows).

Three working increments, each ending in a demonstrable result and a
`CHECKPOINT` where work STOPS for the user's feedback. Investigation folds
into whichever step needs it (no separate catalog step); the written record
accumulates in `claude-docs/reference/19-widget-channel.md`. Real chumby =
read-only oracle (ask first) for confirming on-device behavior.

**On-device testing deferred (user, 2026-07-08):** verify W1/W2/W3 on the
desktop at their checkpoints; do the Pi deploy + on-device confirm ONCE,
after W3, when the full widget featureset is in place. So the per-step
"on-device" acceptance below is satisfied at that single later pass, not
per step.

- **W1 — Boot-generated channel, used correctly.** DONE on desktop
  2026-07-08 (commit `e4b1e00`); on-device deferred per the note above. A boot step enumerates
  the shipped widgets and emits the single `profile.xml` ("all available
  widgets"); the panel loads and plays it. Investigate/confirm: how the
  channel is assembled and played, what holds "current channel" state, how
  the generator replaces the static fixture on both desktop and Pi.
  Acceptance: desktop fixture run + on-device check show the generated
  channel playing its widgets; CI movie-start green; doc 19 + fixtures
  README updated.
  `CHECKPOINT W1: present the generated channel working; wait for feedback.`
- **W2 — Preview picture.** DONE on desktop 2026-07-08 (commit `d73932b`);
  on-device deferred. The dashboard main-bar (B2) thumbnail shows the
  current widget's preview via `loadMovie` of a fixture-provided static
  thumbnail referenced from the profile XML. Delivered with no Ruffle/host
  code change: an optional `<thumbnail href>` sidecar child (the generator
  already deepcopies it into the profile) + a gitignored 80×60 JPEG per
  widget. Desktop-verified: preview renders and swaps per widget (doc 19
  §7). Acceptance: thumbnail renders on desktop + on-device; doc 19 updated.
  `CHECKPOINT W2: present the preview working; wait for feedback.`
- **W3 — Disable controls not needed.** DONE on desktop 2026-07-08 (commit
  `28e9a22`); on-device deferred. `fixtures/ui-policy.toml` rules disable
  the main-bar CHANNEL (D1-D7 unreachable), DELETE (B8), and — per user
  2026-07-08 — SEND (B10) / RATE (B7), which `updateButtons` keeps live
  despite being scope=skip (so they aren't clickable dead-ends). All are
  named children of the `mainButtons` sprite → single named selectors
  (no depth fallback). Desktop-verified: all four dim; CHANNEL click-proven
  inert (doc 19 §8; feature-decisions D-rows + B7/B8/B10 resolved).
  Acceptance: buttons render disabled/inert on desktop + on-device.
  `CHECKPOINT W3 (= milestone done): present the disabled controls; wait.`

Ordering note: W1 first means the Channel button is briefly reachable and
dead-ends until W3 disables it — acceptable, since each step is verified on
its own.

**Milestone status (2026-07-08):** W1/W2/W3 all DONE and desktop-verified.
Remaining: the single deferred **on-device pass** (deploy a freshly built
player + fixtures to the Pi; confirm channel + preview + disabled controls
together), then this milestone is fully closed.

## Future milestones (added at CHECKPOINT 2, 2026-06-12, by user decision)

- **Milestone: Info & Licenses panels** (05-screens.md E6, E7).
- **Remote channels + registration** (05-screens.md D2-D5, D7 remote path;
  activation A4): the **very last feature** of the project (user
  2026-07-08). chumby.com on life support — revisit reviving registration/
  remote channel download only then. A real per-Pi device-identity (UUID)
  hook — reimplementing `guidgen.sh` in `PiHost` off Pi hardware, currently
  a fixed fixture GUID — belongs with this, if wanted.
- Also still open from M2, not Pi-specific: Music from USB / local files
  (C11, decided *needed*) — requires `_getDirectoryEntry` object-filling
  (5,320); clock/time/timezone panels (B3, E5, E12) unverified (now
  covered by the "UI policy" milestone above: TZ shown read-only,
  TZ/NTP selection disabled, 12/24h functional). The `_setTimeZone`
  round-trip bug found in BC4b was FIXED 2026-07-07 (see the
  "_setTimeZone" section above; doc 17 §1).

Scope decisions for all screens: `claude-docs/feature-decisions.md`.

## Final milestone — Brightness & night mode (E2, B4) (moved 2026-07-06)

Deliberately LAST (user decision 2026-07-06): blocked on new display
hardware — the current TFT clone cannot dim (13-brightness-hardware.md
has the evidence and researched alternatives), and which panel gets
ordered is still open. When the new panel is in hand:
- New display bring-up first: overlay swap, DRM device name in the
  kiosk unit, touch recalibration (doc 13 lists per-candidate driver
  caveats — the Waveshare (C) route needs driver work, the PiTFT
  route is drop-in).
- Then the original 3.4 content: real backlight control as the first
  real (non-fixture) `PiHost` behavior — map the panel's
  `/proc/sys/sense1/brightness` writes (0–65535, via `_putFile`) and
  `_setLCDMute` (5,20) onto the real backlight; keep fixture fallback
  for desktop runs. Night mode (B4) on-device test closes it out.
- **RE-ENABLE the brightness Settings entry**: the UI-policy rule
  `settings-brightness` in `fixtures/ui-policy.toml` currently disables
  the E2 menu icon (it was unwired). This milestone must drop that rule
  so the panel's brightness screen is reachable again (doc 18 §10).

## Anti-patterns observed last time — explicit countermeasures
- **Running in circles:** every step above ends in a named written artifact.
  If an artifact can't be produced, that itself is the finding — write it down
  and ask the user.
- **Scope creep into screens nobody wants:** feature-decisions.md + rule 5.
- **Diffuse Ruffle edits:** rule 2; patch-notes.md is part of acceptance.
- **Guessing chumby behavior:** the backup at /home/jan/chumby_backup is
  ground truth; the wiki is the spec; the SWF export is the law. When all
  three disagree, ask the user.

