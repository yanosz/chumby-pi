# 08 — Pi build (Step 3.2)

Date: 2026-07-02. Target: aarch64 (Pi 3A+ interim / 3B+ final, arm64
Raspberry Pi OS trixie — see `07-pi-survey.md`). Build host: the amd64 dev
box (Debian 13). Fork commit: `68a46df0` (`chumby` branch tip, base
upstream `91b61d40`), now living at `ruffle/` in the monorepo (subtree
import 2026-07-02).

## 1. Strategy (CHECKPOINT 4 approved)

Cross-compile from the dev box. Native building on the Pi is out —
415 MiB RAM on the 3A+ (rustc alone needs GBs), and even the 1 GB 3B+
would need hours plus swap abuse.

## 2. One-time toolchain setup (dev box)

```sh
rustup target add aarch64-unknown-linux-gnu
sudo dpkg --add-architecture arm64
sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu \
    libasound2-dev:arm64 libudev-dev:arm64 \
    libssl-dev:arm64 libwayland-dev:arm64 libfontconfig-dev:arm64
```

Installed versions (2026-07-02): gcc-aarch64-linux-gnu 4:14.2.0-1,
libasound2-dev 1.2.14-1, libudev-dev 257.13-1~deb13u1, libssl-dev
3.5.6-1~deb13u2, libwayland-dev 1.23.1-3, libfontconfig-dev 2.15.0-2.3;
rustc 1.96.0.

The arm64 dev packages cover the `*-sys` crates in Ruffle's lock file that
link at build time (`alsa-sys`, `libudev-sys`, `openssl-sys`,
`wayland-sys`, `yeslogic-fontconfig-sys`); `x11-dl` and `ring` need no
arm64 packages (dlopen / self-contained), and the remaining `*-sys`
entries are vendored C or non-Linux. The only setup iteration needed was
fontconfig: the first build attempt failed in
`yeslogic-fontconfig-sys`'s build script until `libfontconfig-dev:arm64`
was added (it is in the list above).

## 3. Cargo cross configuration

Lives in `.cargo/config.toml` at the **monorepo root** — not inside
`ruffle/` — so the fork stays upstream-clean (rule 2). It sets the
aarch64 linker (`aarch64-linux-gnu-gcc`) and target-scoped
`PKG_CONFIG_*` variables pointing at `/usr/lib/aarch64-linux-gnu/pkgconfig`.
Host (amd64) builds are unaffected.

## 4. Build command

```sh
cd ruffle
cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu
```

(**BC4a 2026-07-07:** `--features chumby` removed from this command —
the fork now always builds the chumby code; the flag no longer exists.)

Output: `ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop`.

**Profile changed `release` → `dist` 2026-07-06 (Step 3.5):** `dist`
is upstream's shipping profile (release + fat LTO + codegen-units=1);
it measurably reduces the Pi's CPU-bound rasterization cost and shrinks
the binary ~37 MB → ~29 MB. Cost: the fat-LTO link makes even
incremental rebuilds take minutes (plain `--release` stays ~20 s warm —
fine for iteration, but the debs ship `dist`; `pkg/build-debs.sh`
refuses a missing dist binary). See 11-perf-and-input-cleanup.md.

Results (2026-07-02, dev box):
- Build time: ≈5–6 min total (≈3 min to the fontconfig failure, 2 m 13 s
  for the resumed rest). Fast because host-side artifacts (proc-macros,
  build scripts) were reused from the M2 amd64 builds in the same
  `target/` dir; expect ~15–25 min from a completely cold cache.
- Binary: 36 MB, `ELF 64-bit LSB pie executable, ARM aarch64`.
- Dynamic deps (`readelf -d` NEEDED): libfontconfig.so.1, libssl.so.3,
  libcrypto.so.3, libasound.so.2, libudev.so.1, libgcc_s, libm, libc —
  **all verified present on the Pi's stock Raspberry Pi OS** (2026-07-02).
  Wayland/X11 are dlopened, not linked, so no display libs in NEEDED.
- No source or config workarounds; the fork builds for aarch64 untouched.

## 5. Known risk carried into 3.3: the Pi graphics stack

From `07-pi-survey.md` §3: VideoCore IV has no GLES 3 and no Vulkan, so
wgpu is expected to pick **lavapipe (software Vulkan)**. To validate on
device before any packaging effort:

1. scp the binary + fixtures to the Pi, run inside the existing labwc
   session (`WAYLAND_DISPLAY` set) — smoke test.
2. Measure FPS / CPU at several output modes (1920×1200 native vs forced
   ~800×600) — decision input for the kiosk cage configuration.

Results → `09-pi-deploy.md` / `docs/progress.md` (M3 section).

## 6. Post-Big-Cleanup gotcha: stale `target/` from the renamed tree (2026-07-07)

First cross-build in the public working repo (`/home/jan/chumby-pi`,
submodule `ruffle/` at `6c99ebd1f`) failed in `core/build.rs`:

```
Error: Could not find or load main class macromedia.asc.embedding.ScriptCompiler
```

Cause: the gitignored `ruffle/target/` survived the BC1 directory
rename `/home/jan/chumby-pi-public` → `/home/jan/chumby-pi`. Host-side
build-script artifacts (`target/dist/build/…`) had the OLD absolute
path baked in via `env!("CARGO_MANIFEST_DIR")` — the `asc` wrapper
crate resolves `tools/asc/asc.jar` that way — and cargo fingerprints
do not notice a tree rename, so the stale build script was reused and
pointed java at a jar path that no longer exists. (`asc.jar` is
Adobe's ActionScript compiler; upstream Ruffle needs java at build
time, on the build host only, to compile its `playerglobal` AS
stdlib into the binary.)

`cargo clean -p asc -p build_playerglobal -p ruffle_core` did NOT fix
it (build-script artifacts survive `clean -p`). Fix that worked:

```sh
rm -rf ruffle/target/dist        # host-side dist artifacts only
cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu
```

Rebuild took 5m43s (aarch64 dep cache under
`target/aarch64-unknown-linux-gnu/` stayed valid). If other profiles
misbehave in this tree, the same applies to `target/debug` etc.
