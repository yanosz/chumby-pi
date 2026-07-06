# patch-notes — every upstream Ruffle file touched, and why

Rebase guide for the Ruffle fork living at `ruffle/` (imported 2026-07-02
via git subtree with full upstream history; previously a standalone clone
at `resources/ruffle`). Base: upstream master commit
`91b61d405b13320842167171b8030b34a286db1e` (2026-06-12). All chumby code is
compiled only with `--features chumby`; with the feature off the build is
upstream-identical. Hook numbering H1…H6 follows
`docs/design/chumby-host.md` §4; H7/H8 (simulated bend input) added in
step 2.2.4.

| Hook | File | Change | Step |
|------|------|--------|------|
| H6 | `core/Cargo.toml` | declare `chumby = []` feature | 2.2.1 |
| H6 | `desktop/Cargo.toml` | `chumby = ["ruffle_core/chumby"]` forward | 2.2.1 |
| H2 | `core/src/lib.rs` | `#[cfg(feature = "chumby")] pub mod chumby;` | 2.2.1 |
| — | `core/src/chumby/` | NEW module (no upstream file) | 2.2.1+ |

| H1 | `core/src/avm1/globals/asnative.rs` | cfg-gated `5 => chumby::avm::method` match arm (+ comment re ASnative(4,39) collision) | 2.2.2 |
| H5 | `desktop/src/cli.rs` | cfg-gated `--chumby-fixtures <PATH>` option | 2.2.2 |
| H5 | `desktop/src/main.rs` | cfg-gated host init after tracing setup | 2.2.2 |

| H4 | `desktop/src/player.rs` | cfg-gated `ChumbyNavigator` wrap before `.with_navigator(...)` | 2.2.3 |

| H7 | `desktop/src/app.rs` | cfg-gated Home-key → bend-sensor mapping in `KeyboardInput` (chumby's falconwing port used the same key) | 2.2.4 |
| H8 | `desktop/src/cli.rs` | cfg-gated `--chumby-control <FIFO>` option | 2.2.4 |
| H8 | `desktop/src/main.rs` | cfg-gated `chumby::input::spawn` next to host init | 2.2.4 |

| H9 | `core/src/player.rs` | cfg-gated click-target diagnostic in `run_mouse_pick` (silent unless `chumby_pick=debug`); body split into `run_mouse_pick_inner` | 2.2.5 |

| H10 | `desktop/src/app.rs` | cfg-gated pointer-queue drain at the top of `about_to_wait`: `click X Y` / `drag X1 Y1 X2 Y2` control commands become MouseMove/Down/Up `PlayerEvent`s, one action per loop iteration (sliders need the sequence spread over ticks); coordinates go through the same `window_to_movie_position` as real mouse input | 3.3 |

| H11 | `desktop/src/app.rs` | cfg-gated `WindowEvent::Touch` arm (upstream ignores touch entirely; Wayland touch is not a pointer): single-touch → left-button MouseMove/Down/Up; plus a `ChumbyTouch` field + `about_to_wait` check turning a stationary (≤12 px) ≥1 s hold into `tap_bend()` — the touchscreen stand-in for the bend squeeze | 3.3-TFT |

(3.5: no new hook. The kiosk mouse-pointer fix is a udev rule in the
chumby-player deb, not a ruffle patch — a briefly considered H12
`set_cursor_visible` hook was reverted as a dead end; see
11-perf-and-input-cleanup.md §2.)

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
a `PlayerBuilder` field — H4 reduced to the navigator wrap above; rationale
documented in `host.rs`.

Build verification 2.2.1: `cargo build -p ruffle_desktop` (feature off) and
`cargo build -p ruffle_desktop --features chumby` both succeed; feature-off
diff vs upstream is Cargo.toml/lib.rs lines above only (no behavioral code).
