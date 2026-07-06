# 04 — Ruffle gap analysis (Step 1.4)

Date: 2026-06-12. SWF under test: `chumby_backup/tmp/controlpanel.swf`
(2.8.87b3). Ruffle: built from source at commit
`91b61d405b13320842167171b8030b34a286db1e` (2026-06-12, master), **dev
profile**, on linux amd64 (X11, no audio device — ALSA errors in logs are
machine noise, not Ruffle gaps).

Method (agreed with user): a fixed set of scripted, non-interactive runs —
no UI walking. Logs in `logs/04/`, screenshots in `images/`. All runs used
`--proxy http://127.0.0.1:1` to block real network deterministically (and to
avoid leaking the device GUID to the live chumby.com).

Base command:
```sh
RUST_LOG=warn,avm_trace=trace ruffle_desktop --proxy http://127.0.0.1:1 \
  [--load-behavior blocking] [-Pvar=1 …] controlpanel.swf
```

## 1. Run matrix

| run | FlashVars / options | got to | blocked on | screenshot |
|-----|--------------------|--------|------------|------------|
| 1,2,3 | *(none — ground-truth invocation)* | "Authorizing…" spinner (frame `validate`) | GUID + network-status `exec://` fail → authorize skipped → no transition out | 04-validate-authorizing.png |
| 4 | `-Pbuiltin=1` | "Checking network status" | **Ruffle bug G3**: `gotoAndStop("builtin")` → label not found (streaming load) | 04-builtin-stuck-networkstatus.png |
| 5 | `-PfirstTime=1` | first-time wizard, touchscreen calibration ("TOUCH THE GREEN STAR"), fully rendered & interactive | next steps need touch input (not exercised) | 04-firsttime-calibration.png |
| 6 | `-PsafeMode=1` | same as run 4 | G3 again (`safeMode` label) | — |
| 7 | `-Pbuiltin=1 --load-behavior blocking` | "entering builtin mode" → network-adapter setup → **WiFi wizard "Choose a wireless connection"** incl. SKIP button (builtin-only), fully rendered | `exec://ap_scan`; SKIP click would continue (not exercised) | 04-builtin-wifi-wizard.png |
| 8 | `-Ptest=1 --load-behavior blocking` + fixture dir (`chumbyguid.xml`) | **full normal operation**: authorize failure tolerated → "Validating date" → "Entering normal operation" → `beAClock` → **clock on screen**, alarms loaded+processed, all music players instantiated, graceful degradation of FM/external-music | nothing — steady state with heartbeats | 04-testmode-clock.png |

## 2. Ruffle gaps found (the actual patch surface)

**G1 — `exec://` URL scheme unsupported.** `XML.load("exec://…")` fails as
`ruffle_core::loader: Error during LoadVars load … FetchError("builder error
for url …")`. Blocks *every* boot path at "Fetching Chumby GUID" /
"Checking network status". Observed: `exec://guidgen.sh`,
`exec://network_status.sh`, `exec://ap_scan` (runs 1–7). M2: intercept the
scheme in the loader/navigator → `ChumbyHost`.

**G2 — `ASnative(5,N)` returns `Undefined` silently.** Ruffle's
`core/src/avm1/globals/asnative.rs` dispatches by category number; category 5
is unmapped → `Undefined`; calling it is a silent no-op. **Zero log signal at
any level** — the plan's "triage unimplemented warnings" approach cannot see
natives; only our own logging table (M2 step 2.2.2) will reveal runtime
usage. Side effect: all `_putFile`/`_getFile`/`_backtick` silently no-op
(heartbeat writes, settings restore — panel tolerates all of it).
Hook point identified: the category match in `asnative.rs` is exactly where a
`5 => chumby::method` arm slots in, plus the same table-registration pattern
as other categories. (`ASnative(4,39)` and the Date helpers (5,176–178)
likewise silent.)

**G3 — `GoToLabel` fails for labels on not-yet-loaded frames.**
`WARN ruffle_core::avm1::activation: GoToLabel: Frame label '"builtin"' not
found` (same for `"safeMode"`). Labels demonstrably exist (tag dump: `builtin`
tag #1996/frame 9, `safeMode` frame 8). Cause: default
`--load-behavior streaming` — `dispatch()` runs from frame 2 before later
frames are parsed; early labels (`firsttime` #576, `validate` #829) work.
**Workaround verified: `--load-behavior blocking` fixes it** (run 7). The
real chumbyflashplayer loads the whole file first, so blocking is also the
faithful behavior. M2: always run with blocking (or fix upstream — candidate
for an upstream issue, arguably a Ruffle bug since Flash queues gotos to
unloaded frames).

**G4 — desktop network sandbox blocks http from file: movies.**
`InvalidDomain("http://xml.chumby.com/xml/authorize?…")` (run 8; also
music.chumby.com, 127.0.0.1:8081). Stock desktop Ruffle refuses cross-domain
HTTP for local SWFs regardless of proxy. Irrelevant for M2 (we intercept
chumby URLs anyway) but explains behavior during testing; localhost daemon
URLs would also need either interception or sandbox relaxation.

**G5 — `System.security.allowDomain()` is a stub** (warn once). Harmless.

**Not gaps (works well in stock Ruffle):** AVM1 execution of this big AS2
codebase (30k+ lines in frame 2 alone), rendering (all screens pixel-faithful
at 2× window scale incl. embedded fonts), XML parsing, 15 s heartbeat timers,
`setInterval` machinery, alarm engine (fired the midnight defaults in run 8),
`_root.$version`-based test autodetect behaves device-like under Linux
(`LNX 32,0,0,0` → `test=false`).

## 3. What gates the main screen (dynamic confirmation of 03 §7)

Boot chain on every path: GUID → (DCID → MAC, only if GUID succeeded) →
network status → `dispatch()`:

- *(no vars)* → `validate` frame: needs network + authorize against
  `{base}/xml/authorize` (poll) — then date validation → `main`. Without
  network it parks on "Authorizing…" forever. **On the device this is the
  downloaded-panel path; it hard-requires the server** (or a fixture base URL).
- `builtin=1` → `builtin` frame: network-adapter check → WiFi wizard (has
  SKIP because builtin) → date validation (`frame 9: year > 2000`) →
  `main`. **Shortest path to the main screen without any server.**
- `firstTime=1` → calibration → (rest of wizard untested).
- `test=1` → all exec:// replaced by local fixture loads
  (`chumbyguid.xml`, `dcid.xml`, …), MAC stubbed, `hasLocalNetwork=true`
  hardcoded, authorize failure tolerated, versions stubbed (hw=3.8 sw=0.9.2
  fw=100), **and crucially `test||localCache` renders the clock/widgets
  in-movie via library clips instead of the native slave-player system** —
  which is why run 8 shows a working clock without any ASnative support.

## 4. Implications for Milestone 2 (proposed re-prioritization)

1. `--load-behavior blocking` from day one (G3) — zero code.
2. `exec://` interception + fixture store (G1) — unblocks GUID/network
   status/ap_scan; with `builtin=1` this plausibly reaches the WiFi wizard
   SKIP → date check → **main screen** with *no ASnative work at all*.
3. ASnative logging table (G2) — not to make things work, but to *see* what
   the panel actually calls at runtime (static counts in 03 §1 are the prior).
4. `_getFile`/`_putFile`/`_fileExists` over fixture rootfs — settings
   restore, heartbeats, alarms persistence.
5. The `localCache=1` path (in-movie widget rendering, no slave system) looks
   like a legitimate long-term simplification on the Pi, not just a test
   crutch — **worth a user decision in M2 design** (vs. implementing the
   dual-instance master/slave system).
6. URL interception for `{base}=xml.chumby.com` etc. comes after the main
   screen works (only needed for channels/widgets/music directories).

## 5. Dynamically observed contract rows (fills 03's `dyn` column)

Observed firing at runtime: `exec://guidgen.sh`, `exec://network_status.sh`,
`exec://ap_scan` (and the network-adapter step), test-mode fixture loads
`chumbyguid.xml` + `dcid.xml`, `{base}/xml/authorize` (exact query observed:
`?hw=&sw=&fw=&id=&nocache=&config=ironforge`),
`music.chumby.com/music_sources/show/?…`,
`127.0.0.1:8081/radio/configure?country=us`, FlashVars
`firstTime`/`builtin`/`safeMode`/`test` (all four gate as predicted),
heartbeat loop + `_putFile("/tmp/movieheartbeat")` (silent no-op),
`_getFile` settings restore (silent), alarm default-write attempt
(`/psp/alarms`), `_setSlaveVar`-family not reached (no widgets).
Not yet observed (need deeper navigation or network): `macgen.sh`
(only runs after a *successful* GUID fetch on the non-test path),
`signal_strength`, update.chumby.com, all music-source directories,
volume/brightness backticks, `fscommand("quit")`.
