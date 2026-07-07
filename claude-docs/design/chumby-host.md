# chumby-host — Ruffle integration architecture (Step 2.1)

Date: 2026-06-12. Inputs: 03-environment-contract, 04-ruffle-gap-analysis,
06-variant-diff (zurk prior art), feature-decisions.md. Ruffle pinned at
commit `91b61d405b13320842167171b8030b34a286db1e` (the 1.4 baseline).

## 0. Design goals

1. Rule 2: all chumby Rust in ONE module behind a `chumby` cargo feature;
   upstream files get only registration hooks (each listed in §4 and later
   in `docs/patch-notes.md`).
2. M1 finding-driven minimalism: the panel needs far less than the full
   contract — G1 (exec://), G2 (ASnative), file I/O, and a handful of HTTP
   fixtures put the main screen on screen (04 §4).
3. Everything mockable: `ChumbyHost` trait isolates *what the environment
   answers* from *where Ruffle asks*; `FixtureHost` (M2) answers from disk,
   `PiHost` (future) from the real system.

## 1. The `ChumbyHost` trait

One method per contract-row category (03), small enough to implement twice:

```rust
/// All errors are normal results — the panel handles failures (run 2-8).
pub trait ChumbyHost: Send + Sync {
    /// Category 1: ASnative(5,N) calls that need host state.
    /// Pure functions (md5, base64, blowfish) are implemented directly in
    /// the AVM table and never reach the host.
    /// `name` is the canonical wrapper name from 03 §1 (e.g. "_getPlatform").
    fn native(&self, index: u16, name: &str, args: &[HostValue]) -> HostValue;

    /// Category 2: shell execution — synchronous `_backtick` (5,52) and
    /// the async `exec://` URL scheme (loader wraps this in a future).
    /// Returns the command's stdout (the panel parses XML or raw text).
    fn exec(&self, command: &str) -> Result<Vec<u8>, HostError>;

    /// Category 3: URL fetch interception. `None` = not ours, pass through
    /// to the wrapped navigator (lets real internet radio streams flow
    /// later while chumby.com stays mocked — zurk's split, 06 §3).
    fn fetch(&self, url: &str) -> Option<Result<HostResponse, HostError>>;

    /// Category 4: persistence — the virtual rootfs behind _getFile (5,50),
    /// _putFile (5,51), _fileExists (5,53), _fileSize (5,54), _unlink
    /// (5,55), _getDirectoryEntry (5,320) and file:// directory listings.
    fn fs(&self) -> &dyn ChumbyFs;   // get/put/exists/size/unlink/list_dir
}
```

Every call is logged (`info!` target `chumby_host`) with args and result —
this is the G2 countermeasure: runtime visibility of native usage that
stock Ruffle cannot give us. The log replaces the gap-analysis loop in
step 2.2.2 ("run → read logs → add fixture → re-run").

### `FixtureHost` (M2)

- `native`: consults a defaults table (`fixtures/native.toml`: per-name
  return value), falling back to category-sensible defaults
  (getters → 0/"", setters → no-op). Stateful pairs that must round-trip
  (volume 180/181, balance, mute, brightness, touchclick, time/tz) are
  backed by files in the virtual rootfs — same place the panel itself
  persists them (e.g. `/psp/volume`), so panel and host agree.
- `exec`: longest-prefix match of the command string against
  `fixtures/exec/manifest.toml` entries `{ prefix | regex, response-file,
  dynamic? }`; a handful of commands (`chumby_set_volume N`, `md5sum`,
  `sync_time_state.sh`) get tiny built-in dynamic handlers instead of
  static files. Unknown command → logged loudly + empty response (panel
  tolerates this everywhere we observed).
- `fetch`: host-allowlist (`*.chumby.com`, `127.0.0.1:8080-8082`,
  mp3tunes, wunderground…) → static files under
  `fixtures/http/<host>/<path>` (zurk's stubs imported as the seed corpus,
  incl. `defaultUpdateTime/defaultProfileTime=9999` parameters and
  "no update" responses). Everything else → `None` (passthrough — M2
  desktop runs still see InvalidDomain from Ruffle's sandbox, which is
  fine until real-network-later features arrive).
- `fs`: rooted at `fixtures/rootfs/` (pre-seeded with the ★ paths from
  03 §3: `/psp/firsttime=0`, `/psp/nooverlay=1`, alarms XML, timezone,
  …). Writes confined to the root (reject `..`); `/tmp` and `/mnt` are
  plain subdirectories. The 03 §3 note about `externalmusic.xml` directing
  arbitrary paths is neutralized by this confinement.

### `PiHost` (future, named for completeness only — NOT designed now)

Same trait; real ALSA/backlight/clock/filesystem. Per the working
assumptions, nothing Pi-specific is designed in M2 beyond the trait split.

## 2. Widget playback: localCache path (per CHECKPOINT 2 Q4)

We do **not** implement the master/slave dual-instance system (natives
84-89, 110-119, 210/211, 220, 330-332, 360-364, 380-387 stay logging
stubs; `_getSlaveVar("_chumby_widget_done")` returns `"true"` so intro/
widget handoffs never hang — 03 §1a). The panel runs with `-PlocalCache=1`,
so widgets load in-movie via `loadMovie` (proven path, 04 run 8). Widget
SWFs come from `file://` hrefs in the fixture profile XML (zurk-proven,
06 §3) — no widget-server emulation. Revisit only if a wanted widget
misbehaves inside the shared VM.

## 3. Bootstrap: no Rust-side frame control needed

M1's key simplification (04 §3): with `builtin=1` the panel's own
dispatcher reaches `main` once (a) network status reports healthy and
(b) the clock is sane. Both are environment answers, not frame hacks:

```
ruffle_desktop \                        # chumby code always built (BC4a)
  --load-behavior blocking \            # G3 workaround (faithful anyway)
  -Pbuiltin=1 -PlocalCache=1 \
  --chumby-fixtures fixtures/ \
  controlpanel.swf
```

`network_status.sh` fixture returns a healthy `<network>` → wizard never
appears (A8 skip implemented as "panel believes network is up", per
feature-decisions). The plan's "bootstrap controller that sets _root vars
and jumps frames from Rust" is therefore **deferred**; if a later screen
needs it, the documented insertion point is `Player::update` with a
queued-action approach — but we start without it.

## 4. Integration points in upstream Ruffle (the entire patch surface)

> Correction 2026-07-07 (BC4a/BC4b): the `chumby` cargo feature and all
> cfg-gating were removed (doc 16), and the H1…H11 hook numbering this
> table introduced was retired — it carried no information (user
> decision). The current, file-keyed patch-surface table lives in
> `claude-docs/patch-notes.md`. The table below is the original design
> as written.

| # | File | Change | Size |
|---|------|--------|------|
| H1 | `core/src/avm1/globals/asnative.rs` | add `5 => Some(chumby::method)` arm behind `#[cfg(feature = "chumby")]` | 2 lines |
| H2 | `core/src/avm1/globals.rs` (or `globals/mod.rs`) | `mod chumby;` declaration behind cfg | 2 lines |
| H3 | `core/src/chumby/` | NEW module: `host.rs` (trait), `fixture.rs` (FixtureHost), `navigator.rs` (`ChumbyNavigator<T: NavigatorBackend>` decorator: `fetch()` matches `exec://`, chumby hosts, `file://`-directory → host; else `inner.fetch()`); `avm.rs` (the `method(activation, this, args, id: u16)` table — `TableNativeFunction` signature, same pattern as `globals::date::method`) | new code only |
| H4 | `core/src/player.rs` (PlayerBuilder) | optional `chumby_host` field + wrap navigator when set | ~6 lines |
| H5 | `desktop/src/cli.rs` + player setup | `--chumby-fixtures <dir>` flag → construct FixtureHost, pass to builder | ~10 lines |
| H6 | `core/Cargo.toml`, `desktop/Cargo.toml` | `chumby` feature decl (+ md5/blowfish deps, feature-gated) | few lines |

Notes: ASnative(4,39) collides with Ruffle's existing category 4
(`ASSetNative`) — chumby's `_batteryPower` is never called (03 §1b), so we
leave category 4 untouched and document it. `fscommand("quit")` already
maps to desktop quit handling (`desktop/src/backends/fscommand.rs`); `-Q`
gating is not replicated. Feature off ⇒ zero behavioral diff (verified in
step 2.2.1 by building both ways).

## 5. Fixture directory layout

```
fixtures/
  README.md            # how to change any mocked result (acceptance item)
  native.toml          # per-wrapper-name default returns + log level
  exec/
    manifest.toml      # command pattern → response file | builtin handler
    network_status.xml ap_scan.xml signal_strength.xml guid.txt mac.txt ...
  http/
    xml.chumby.com/xml/{authorize,chumbies,profiles,setprofile,...}
    update.chumby.com/update            # "no update"
    music.chumby.com/music_sources/...  # empty manifest (sources skipped)
  rootfs/
    psp/  {firsttime,nooverlay,clock_format,timezone_city,alarms,...}
    tmp/  mnt/usb/  usr/chumby/alarmtones/  LICENSES/
  widgets/
    profile.xml        # single fixture channel (D-milestone later)
    clock.swf ...      # file:// hrefs from profile.xml
```

Key principle: fixture *keys* are the panel's own request strings
(command line, URL path, filesystem path) — no translation layer, so the
chumby_host log line IS the name of the fixture file to create.

## 6. Step 2.2 work order (unchanged from plan, sharpened by M1)

1. Branch `chumby` at pinned commit; feature scaffolding (H2/H3/H6 empty);
   build with feature off == upstream (record in patch-notes.md).
2. H1 + logging-stub ASnative table + virtual rootfs (fs category) —
   run: settings restore + heartbeat work, log shows native traffic.
3. H3 navigator + exec fixtures (§2a set) — run with `-Pbuiltin=1`:
   expect date-check → **main screen** (B2). Screenshot, update 04.
4. HTTP fixtures (authorize/chumbies/profiles/update) — run without
   builtin: validate path reaches main too.
5. Iterate per remaining `needed` screens: volume (E1), brightness (E2),
   night mode (B4), clock (B3), alarms (B5/B6), music My-Streams (C2) +
   USB files (C11) — each = fixtures + occasionally a native moving from
   logging-stub to FixtureHost-backed. Screenshot each newly working
   screen into a running progress doc. Skip-screens: never navigated;
   their startup constructors already proven harmless (04 run 8).

## 7. Open points for CHECKPOINT 3

1. Confirm the two recommendation-decisions: **localCache** widget path
   (§2) and **ironforge/hw 3.8** identity (feature-decisions.md).
2. G3 upstream: file a Ruffle issue for the GoToLabel/streaming bug, or
   silently rely on `--load-behavior blocking`? (Proposal: file it —
   good citizenship, zero coupling.)
3. Fixture HTTP transport: pure in-process interception (proposed, §1)
   vs zurk-style local HTTP server. In-process is simpler and avoids
   ports; a local server would let an unmodified browser inspect
   fixtures. Proposal: in-process.
4. Audio (5,131-134/144 + btplay): M2 proposal is a state-machine stub
   (no sound) — real audio is a Pi/PiHost concern. OK?
5. Crate vs module: proposal is module `core/src/chumby/` (avoids a
   circular dependency with ruffle_core's AVM types). Acceptable under
   rule 2 as long as the module boundary stays clean?
