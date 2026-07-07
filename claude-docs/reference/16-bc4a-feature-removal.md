# 16 — BC4a: the `chumby` cargo feature is gone (2026-07-07)

> Note (2026-07-07, later): the fork's incremental commits were
> squashed into the single commit `7f4d8031b` at Jan's request
> (doc 17 §7); chumby-ruffle hashes below are pre-squash and no longer
> resolve on GitHub.

User decision (plan §BC4): the fork is never used without its chumby
code, so the feature toggle was pure ceremony — every build command,
script, and CI job passed `--features chumby`. Removed in chumby-ruffle
commit `7e7d8fa65` ("chumby: BC4a — remove the chumby cargo feature").

This supersedes the "behind a `chumby` cargo feature" clause of hard
rule 2. The rest of rule 2 stands: one isolated module
(`core/src/chumby/`), minimal documented hooks in upstream files.

## 1. What changed in chumby-ruffle

- `core/Cargo.toml`: `chumby = []` deleted; `desktop/Cargo.toml`:
  `chumby = ["ruffle_core/chumby"]` deleted. Both files are
  upstream-identical again — hook H6 no longer exists.
- All 12 `#[cfg(feature = "chumby")]` attributes removed (lib.rs,
  player.rs, asnative.rs in core; main.rs, cli.rs, player.rs, app.rs ×6
  in desktop). The hook code itself is unchanged, just un-gated.
- Doc comments in `chumby/mod.rs` + `chumby/audio.rs` updated;
  CHUMBY.md rewritten where it described the gating (incl. an
  upstream-merge note: the hooks are intentionally un-gated — do not
  re-gate as "drift resolution" during a rebase).
- `.github/workflows/chumby.yml`: build step is now plain
  `cargo build -p ruffle_desktop`.

Consequences worth knowing:

- `--chumby-fixtures` / `--chumby-control` and the Touch/Home-key/
  long-press input mappings are now in every build of the fork. Without
  `--chumby-fixtures` no host is registered and ASnative(5,N) still
  returns undefined, so a plain `ruffle_desktop foo.swf` behaves like
  stock Ruffle for normal movies; it is no longer *bit-identical* to
  upstream, which was the old feature-off guarantee. That guarantee had
  no consumer.
- Future upstream merges: one fewer patched file pair (the Cargo.tomls
  drop out of the squashed commit), but `cfg`-related conflicts in
  app.rs etc. are also gone since the attributes no longer exist.

## 2. What changed in chumby-pi

`--features chumby` dropped from every current-state command:

- `.github/workflows/ci.yml` (dist build step)
- `run-controlpanel.sh` (build-hint message)
- `README.md`, `docs/setup.md` (§3 cross-build, §7 desktop run),
  `fixtures/README.md`
- `claude-docs/patch-notes.md` (header + H6 rows + verification note —
  it is the rebase guide, so it now describes the un-gated surface)
- `claude-docs/design/chumby-host.md` §3 example command
- `claude-docs/reference/08-pi-build.md` §4 and `15-ci.md` §2/§3 got
  dated correction notes (they are as-it-happened records)
- `pkg/build-debs.sh` needed no change (it packages a prebuilt binary)
- `ruffle/` submodule pin bumped to the BC4a commit

Historical mentions in 07/11/12/14-*.md and old plan steps were left
as written — they describe what was true at the time.

## 3. Verification (acceptance = CI green on both repos)

- Local: `cargo build -p ruffle_desktop` clean; 30 s headless run of
  controlpanel.swf against `fixtures/` — movie starts, `_getPlatform`
  answered, `/tmp/movieheartbeat` written, no panic (same criteria as
  the CI movie-start test).
- chumby-ruffle CI run: see §4 below.
- chumby-pi CI run: after Jan pushes `main` (auto mode blocks that
  push).

## 4. CI run results

- chumby-ruffle: run **28864269056** (push of `7e7d8fa65`) — success,
  incl. the movie-start + `_getPlatform` assertions, 2026-07-07.
- chumby-pi: pending — commit `d3fb1ab` is local until Jan pushes
  `main` (auto mode blocks that push); record the run id here when it
  is green.
