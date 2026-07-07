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
- **CHECKPOINT BC4 — chumby-ruffle housekeeping** (added 2026-07-07,
  user; a LATER session, not the BC3 one) = milestone done. Details
  below.

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
the BC3 session):

1. **Every ASnative call documented.** Audit the ASnative table in the
   chumby module: each registered (a,b) index must have documentation
   (purpose, args, return, fixture behavior) in the repo's chumby doc.
2. **Remove the feature-toggle logic.** Drop the `chumby` cargo
   feature; the fork always builds with the chumby code (user decision
   2026-07-07 — no need to build the software without our feature).
   This supersedes the "behind a `chumby` cargo feature" clause of
   hard rule 2; the rest of rule 2 (one isolated module, minimal
   documented hooks in upstream files) stands. Same-step follow-ups:
   update `--features chumby` in both CI workflows, CHUMBY.md,
   chumby-pi docs/scripts (run-controlpanel.sh, build-debs.sh,
   docs/setup.md), and note the change for future upstream merges.
3. **README swap.** The repo-root README becomes the chumby doc
   (today's CHUMBY.md content); the original Ruffle README is renamed
   (e.g. `README.ruffle.md`) and linked from the new one.

After the cleanup: **widget channels** (user 2026-07-06, "one of the
next sessions").

## Future milestones (added at CHECKPOINT 2, 2026-06-12, by user decision)

- **Milestone: Widget channels & management** — channel switching, channel
  info, add widget (05-screens.md D1-D5, D7). Until then a single static
  fixture channel.
- **Milestone: Info & Licenses panels** (05-screens.md E6, E7).
- Also still open from M2, not Pi-specific: Music from USB / local files
  (C11, decided *needed*) — requires `_getDirectoryEntry` object-filling
  (5,320); clock/time/timezone panels (B3, E5, E12) unverified.

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

## Anti-patterns observed last time — explicit countermeasures
- **Running in circles:** every step above ends in a named written artifact.
  If an artifact can't be produced, that itself is the finding — write it down
  and ask the user.
- **Scope creep into screens nobody wants:** feature-decisions.md + rule 5.
- **Diffuse Ruffle edits:** rule 2; patch-notes.md is part of acceptance.
- **Guessing chumby behavior:** the backup at /home/jan/chumby_backup is
  ground truth; the wiki is the spec; the SWF export is the law. When all
  three disagree, ask the user.

