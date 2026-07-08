# patch-notes — every upstream Ruffle file touched, and why

Rebase guide for the Ruffle fork living at `ruffle/` (imported 2026-07-02
via git subtree with full upstream history; previously a standalone clone
at `resources/ruffle`). Base: upstream master commit
`91b61d405b13320842167171b8030b34a286db1e` (2026-06-12). Since BC4a
(2026-07-07) the chumby code is **always compiled** — the `chumby` cargo
feature and every `#[cfg(feature = "chumby")]` gate were removed (user
decision: the fork is never used without the chumby code). Do NOT
re-introduce gating during an upstream rebase; that would be a deliberate
re-decision, not drift resolution.

Since BC4b (2026-07-07) the hooks are no longer numbered (H1…H11 in older
docs and commit messages) — the numbering carried no information; the
table below is keyed by file. Every hook site in upstream code carries a
comment containing the word `chumby`, so `grep -rn chumby` over a touched
file finds the patch to re-apply. `core/src/chumby/` itself is a new
directory with no upstream counterpart and rebases clean.

| File | Change |
|------|--------|
| `core/src/lib.rs` | `pub mod chumby;` |
| `core/src/avm1/globals/asnative.rs` | `5 => chumby::avm::method` match arm routing the whole ASnative category 5 (+ comment re the ASnative(4,39) `_batteryPower` collision with Ruffle's category 4 — panel never calls it, category 4 stays upstream) |
| `core/src/player.rs` | click-target diagnostic in `run_mouse_pick` (silent unless `chumby_pick=debug`); body split into `run_mouse_pick_inner` |
| `desktop/src/player.rs` | `ChumbyNavigator` wrap before `.with_navigator(...)` |
| `desktop/src/cli.rs` | `--chumby-fixtures <PATH>` and `--chumby-control <FIFO>` options |
| `desktop/src/main.rs` | host init + `chumby::ui_policy::load` + `chumby::input::spawn` after tracing setup |
| `core/Cargo.toml` | `toml = { workspace = true }` dependency (ui-policy parsing, chumby module only) |
| `desktop/src/app.rs` | three input mappings: (a) Home key → bend sensor in `KeyboardInput` (chumby's falconwing port used the same key); (b) `WindowEvent::Touch` arm (upstream ignores touch entirely; Wayland touch is not a pointer): single-touch → left-button MouseMove/Down/Up, plus a `ChumbyTouch` field + `about_to_wait` check turning a stationary (≤12 px) ≥1 s hold into `tap_bend()`; (c) pointer-queue drain at the top of `about_to_wait`: `click X Y` / `drag X1 Y1 X2 Y2` control commands become mouse `PlayerEvent`s, one action per loop iteration (sliders need the sequence spread over ticks), through the same `window_to_movie_position` as real mouse input |

(3.5: the kiosk mouse-pointer fix is a udev rule in the chumby-player
deb, not a ruffle patch — a briefly considered `set_cursor_visible`
hook was reverted as a dead end; see 11-perf-and-input-cleanup.md §2.)

The fork's architecture — and the per-index reference of the whole
`ASnative(5,N)` table (purpose, args, return, fixture behavior) — lives
in the fork's own `README.md` (the upstream README is preserved as
`README.ruffle.md` there, swap done in BC4b).

Behavior note (UI-policy milestone, 2026-07-07, module-internal apart
from the `main.rs` load call and the `core/Cargo.toml` toml line above):
`chumby/ui_policy.rs` neutralizes panel controls the host platform does
not support — declarative rules from `<fixtures>/ui-policy.toml`
(selectors with name/depth segments → hide/disable/readonly), applied at
frame cadence from `avm::method` (rides the panel's per-frame `_bent`
poll). First use: clock screen TZ/NTP controls disabled on the Pi
(chumby-pi `claude-docs/reference/18-clock-screen-and-ui-policy.md`).

Behavior note (Info/Licenses milestone I3, 2026-07-08, module-internal +
one `main.rs` line): `chumby/real_net.rs` adds `RealNetHost`, which wraps
`FixtureHost` and answers the network `exec` touchpoints
(`network_status.sh`/`signal_strength`/`macgen.sh`) from live kernel state —
default-route interface + gateway from `/proc/net/route`, the interface's
IPv4 + netmask from `getifaddrs`, DNS from `/etc/resolv.conf`, MAC from
`/sys/class/net`. No shell. It is **always active** (wrapped unconditionally
in `main.rs`); a read that finds no connected interface returns `None` and
the call falls back to the inner fixture, so desktop/CI with no usable
network behave as before. `getifaddrs` needs `libc`, added target-gated to
`core/Cargo.toml` (`[target.'cfg(unix)'.dependencies]`) and used only under
`#[cfg(unix)]` (a `#[cfg(not(unix))]` stub keeps the wasm build clean).
`chumby/ui_policy.rs` gains a `Tint(0xRRGGBB)` action (AVM1 `Color.setRGB`
via `set_color_transform`) — used by the `wired-eth-bar` rule to repaint the
dashboard wifi meter blue.

Behavior note (Info/Licenses milestone I1, 2026-07-08, module-internal
in `chumby/navigator.rs`): `ChumbyNavigator::intercept` now serves
`file://` data loads from the virtual rootfs — panel-hardcoded chumby
paths (the licenses viewer's `file:////LICENSES/gpl.txt`) resolve against
`<fixtures>/rootfs/`, the same view the fs natives use. A rootfs miss
returns `None` (pass through to the real navigator), so widget/thumbnail
loads — which use `{FIXTURES}`-expanded real disk paths — are unaffected.

Behavior note (2.2.6, module-internal, no upstream hook): the mpv audio
backend (`chumby/audio.rs`, spawned process + Unix-socket IPC), ASnative
log deduplication (`chumby/avm.rs`), and their fixtures live entirely
inside the chumby module — the upstream-file table above is unchanged
since 2.2.5.

Behavior note (3.3 on-device fixes, 2026-07-06, module-internal): three
audio bugs found by ear on the Pi — (a) `fixture.rs`
`_getAudioPlayerState` now returns the SWF's constants (IDLE:-1 PAUSED:0
WAITING:1 PLAYING:2; we returned Playing=1=WAITING, so
`TrackedBTPlayer`'s watchdog killed every stream after 5 s); (b)
`audio.rs` `send_ipc` lazily reconnects to mpv's IPC socket — on a loaded
Pi mpv needs >1 s to create it, the old one-shot wait left volume stuck
at spawn level (alarms rang silently at fade-in volume 0); (c)
`stop_internal` now reaps the killed child (was leaving zombies).

Behavior note (2.2.5, module-internal, no upstream hook): `chumby/avm.rs`
deletes `WidgetPlayer.prototype.onPress` once the panel defines it. That
click-stats handler put the widget container into AS2 button mode and
swallowed all widget clicks in our localCache in-movie path (harmless on
real hardware, where widgets play in a slave player). The foreseen
"revisit if a widget misbehaves" case of the CHECKPOINT 3 decision.

Design deviation note (2.2.2): the host reaches AVM natives via a
process-global `OnceLock` registry in `core/src/chumby/host.rs` instead of
a `PlayerBuilder` field — the upstream patch in `desktop/src/player.rs`
is reduced to the navigator wrap above; rationale documented in `host.rs`.

Build verification 2.2.1 (historical, pre-BC4a): `cargo build -p
ruffle_desktop` (feature off) and `cargo build -p ruffle_desktop
--features chumby` both succeeded; feature-off diff vs upstream was
Cargo.toml/lib.rs lines only.

BC4a (2026-07-07): feature toggle removed. `cargo build -p ruffle_desktop`
is now the only build and always contains the chumby code; verification is
the CI movie-start test (chumby-ruffle `.github/workflows/chumby.yml`) plus
a local headless run (movie starts, `_getPlatform` answered, no panic).
