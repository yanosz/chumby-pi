# Development

How to build, run, deploy and verify — and the record of everything ever done
to the Raspberry Pi. Concepts: [design.md](design.md). What the appliance
must do: [requirements.md](requirements.md). The player's internals belong to
[chumby-ruffle](https://github.com/yanosz/chumby-ruffle)'s own `claude-docs/`.

`docs/setup.md` and `docs/hardware.md` are the end-user versions of §2–§4.
This document is the engineering record: it keeps the reasons and the traps.

---

## 1. Branch and commit policy

**One feature branch per working session, squashed on merge**, in both this
repository and the submodule.

**Finishing a session means opening the pull request** — push the branch and
create the PR yourself, in each repo the session touched. A pushed branch
with no PR is an unfinished session. Jan reviews and merges with GitHub's
*Squash and merge*.

```sh
git push -u origin <branch>
gh pr create --repo yanosz/chumby-pi --base main --head <branch> \
    --title "…" --body "…"
```

Both workflows run on pull requests, but **a PR only builds.** Everything
past the build needs `controlpanel.swf` — copyrighted, on a private share —
so the deb build, the install/run test and the fork's movie-start check run
on push to the default branch, after the squash-merge, and on manual
dispatch. Verify locally before opening a PR; CI will not catch it for you.
Merging is the user's call.

**Order matters when the session touched the player.** Bump the `ruffle/`
gitlink in the same change that needs it, but merge the **fork's PR first**:
squash-merging replaces its commits with a new one, orphaning whatever the
gitlink pinned. Then re-point the gitlink at the squashed commit, push, and
only then merge this repo's PR. Otherwise `git clone --recursive` of `main`
cannot fetch the submodule.

The submodule tracks the fork's `chumby` branch (`branch = chumby` in
`.gitmodules`), so re-pointing after a merge is:

```sh
git submodule update --remote ruffle    # fast-forward to origin/chumby
git commit -am "Bump ruffle to the squashed <topic> commit"
git push
```

## 2. Player work happens in the submodule

Running the panel, driving it, changing what it answers, reading the
decompiled SWF: all of it lives in `ruffle/`, which carries its own
`CLAUDE.md` and `claude-docs/`. Start there, not here:

```sh
cd ruffle && ./run-controlpanel.sh
```

Come back to this repo for packaging, the kiosk, the device, and CI.

## 3. Cross-build for the Pi

Native building on the Pi is not viable (rustc wants gigabytes; the 3B+ has
one). One-time setup on an amd64 Debian box:

```sh
rustup target add aarch64-unknown-linux-gnu
sudo dpkg --add-architecture arm64 && sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu \
    libasound2-dev:arm64 libudev-dev:arm64 \
    libssl-dev:arm64 libwayland-dev:arm64 libfontconfig-dev:arm64
```

Those arm64 `-dev` packages cover the `*-sys` crates that link at build time.
`fontconfig` is the one that is easy to forget — its build script fails
without it. `x11-dl` and `ring` need nothing (dlopen / self-contained).

`.cargo/config.toml` at this repo's root sets the linker and the
target-scoped `PKG_CONFIG_*`. Host builds are unaffected.

```sh
cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu --manifest-path ruffle/Cargo.toml
```

The binary, the fixtures, the SWF and the two helper scripts the debs
install all come out of `ruffle/`; `build-debs.sh` knows where.

`dist` (fat LTO, `codegen-units=1`) is what ships; `build-debs.sh` refuses a
missing `dist` binary. Expect ~6 min warm, ~9 min cold, and a multi-minute
link even on a one-line change. Use `--release` while iterating.

Upstream Ruffle needs a JVM at build time to compile its ActionScript stdlib.

## 4. Build, install, deploy

```sh
pkg/build-debs.sh                 # VERSION=x.y.z overrides
scp pkg/out/*.deb pi@<pi>:
# on the Pi
sudo apt install ./chumby-player_*.deb ./chumby-player-data_*.deb
sudo systemctl start chumby-player      # or reboot; postinst enabled it
```

Or all of it in one command: `pkg/deploy-pi.sh <pi>` rebuilds the dist
binary (§7: never trust a stale one), rebuilds the debs into a clean
`pkg/out/`, installs them on the Pi (`--reinstall`, so redeploying the
same version still replaces the files) and restarts the player.
`build-debs.sh` strips the build box's generated `rootfs/psp/guid` from
the data deb — a package must not ship one machine's dev identity.

Leaving player mode: `sudo systemctl stop chumby-player` (once), or
`sudo systemctl disable --now chumby-player`. State in `/var/lib/chumby`
survives removal *and* purge — `StateDirectory` contents are not tracked by
dpkg. Delete it by hand.

### Hot-replace (a build without a deb round trip)

```sh
cd ruffle && cargo build --profile dist -p ruffle_desktop \
    --target aarch64-unknown-linux-gnu

scp ruffle/target/aarch64-unknown-linux-gnu/dist/ruffle_desktop \
    pi@<pi>:/tmp/ruffle_desktop.new
rsync -a --delete --rsync-path="sudo rsync" ruffle/fixtures/ \
    pi@<pi>:/usr/share/chumby-player/fixtures/

ssh pi@<pi> '
  sudo systemctl stop chumby-player &&
  sudo install -m 755 -o root -g root /tmp/ruffle_desktop.new \
      /usr/lib/chumby-player/ruffle_desktop &&
  sudo rm -rf /var/lib/chumby/fixtures &&   # launcher re-seeds on start
  sudo systemctl start chumby-player'
```

Wiping the state directory is what makes new fixture content take effect; it
also discards persisted volume and alarms. `dpkg` still reports the old
version afterwards — the on-disk files are ahead of the package until the
next install.

Verify the binary's sha256 on both ends. Do not skip the rebuild: see §7.

## 5. Verify

**On the desktop**, from the submodule — see its `claude-docs/development.md`.

**On the device**, with `grim` against the cage session:

```sh
XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-0 grim /tmp/x.png
```

The main button bar auto-hides after a short inactivity timeout, so a
scripted `bend` followed by a separate `click` races it — do the summon and
the click in one command. Once inside Settings, which does not auto-hide,
further clicks are unhurried.

**In CI**, the movie-start assertion ([design.md](design.md) §9).

### CI secrets

Set once per repository (the `PASS` value must be rclone's *obscured* form,
not the plaintext password):

```sh
for repo in yanosz/chumby-ruffle yanosz/chumby-pi; do
  awk -F' = ' '$1=="url"{print $2}'  ~/.config/rclone/rclone.conf \
    | gh secret set RCLONE_CONFIG_RSHARE_URL  -R "$repo"
  awk -F' = ' '$1=="user"{print $2}' ~/.config/rclone/rclone.conf \
    | gh secret set RCLONE_CONFIG_RSHARE_USER -R "$repo"
done
```

Both workflows fetch only `controlpanel.swf`; the fork's fixtures are in its
own repo, so there is no fixtures tarball to keep in sync any more.

chumby-ruffle is public; this repo is private. Its Actions minutes are billed
against the account quota and the `dist` build is a long job.

## 6. The device record

Everything ever changed on the Pi, so the howto can be rebuilt from here.
The current device is a **Pi 3B+**, arm64 Raspberry Pi OS trixie, wired
Ethernet, with the ILI9486 480×320 SPI TFT and a USB sound card. An earlier
Pi 3A+ (512 MB, wifi-only, HDMI) was used for the first bring-up; findings
that were specific to it are not repeated here.

**Packages installed:** `mpv`, `cage`, `grim`, `pipewire-alsa`. The last one
is not optional — without it ALSA clients (the player's `cpal`) have no route
into PipeWire and audio-device creation fails. mpv talks to PipeWire natively
and never needed it.

**`/boot/firmware/config.txt`** (backups `config.txt.bak`, `config.txt.bak-tft`):

```
dtparam=spi=on
dtoverlay=piscreen,speed=24000000,rotate=0,drm,swapxy=on,invy=on
```

`,drm` selects the mainline tiny driver over fbtft. Why `rotate=0` is the
landscape value, and why `swapxy=on` is an inverted boolean, are in
[design.md](design.md) §6. The touch values came from four physical taps:
screen-x was device-y, screen-y was inverted device-x.

The overlay can be swapped at runtime, reversibly, without a reboot:
`sudo dtoverlay -r piscreen && sudo dtoverlay piscreen speed=24000000 rotate=0 drm`.

**Kiosk**: `chumby-player.service` and `chumby-widget-channel.service`, both
shipped by the deb. `postinst` enables the player unit and reloads udev.
Installed version: **0.3.0** (deployed 2026-07-11 via `pkg/deploy-pi.sh`,
player at fork branch `usb-music` `b218502` — USB/local music C11). 0.2.0
(2026-07-10, fork tip `41fb650`) brought real network diagnostics, the
backup alarm and real device identity; binary sha256 verified on both ends.

**USB music automount** (2026-07-11, deployed with 0.3.0 — no manual device
surgery; everything below ships in `chumby-player`):

- `/usr/lib/udev/rules.d/99-chumby-usb-music.rules` — a USB block device
  carrying a filesystem (`ID_BUS=usb`, `ID_FS_USAGE=filesystem`) pulls in
  `chumby-usb-mount@<kernel-name>.service`.
- `/lib/systemd/system/chumby-usb-mount@.service` — oneshot, first-wins
  (`mountpoint -q` guard: extra partitions and second sticks are ignored),
  runs `systemd-mount --no-block --collect -o ro,nosuid,nodev,noexec` onto
  `/media/chumby-usb`. Read-only because the panel never writes to the
  stick (fork requirements FR5). `BindsTo=dev-%i.device` plus `--collect`
  unmount and garbage-collect on unplug.
- `/media/chumby-usb` is shipped by the deb, so the mountpoint — and the
  panel's `/mnt/usb` — always resolves: empty dir = no stick, which the
  panel answers with its own "No files available" screen.
- `chumby-player-run` replaces the seeded `fixtures/rootfs/mnt/usb`
  directory (desktop demo tones) with a symlink to `/media/chumby-usb` at
  every start (idempotent, survives an upgrade re-seed); `postinst` gained
  `udevadm trigger --subsystem-match=block --action=add` so a stick already
  inserted at install time mounts immediately.

Verified on the device 2026-07-11: the symlink conversion at restart; My
Music Files over an *empty* `/media/chumby-usb` shows "No files available"
(panel driven over the control FIFO; screenshots with
`XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-0 grim`); the mount
unit exercised with a loop-backed vfat image (`/usr/sbin/mkfs.vfat` on a
file, `losetup -f --show`, `systemctl start chumby-usb-mount@loop1`) —
the TFT browsed the image's folder and tracks, PLAY ALL had mpv playing
`/var/lib/chumby/fixtures/rootfs/mnt/usb/tone-a4.mp3` through the symlink,
and `findmnt` confirmed `ro,nosuid,nodev,noexec`; teardown
(`systemctl stop chumby-usb-mount@loop1 'media-chumby\x2dusb.mount'`,
`losetup -d`) returned the panel to the empty state. **Still outstanding:
the physical-stick pass** — the udev `ID_BUS=usb` match and
unplug-while-mounted are the two paths a loop device cannot exercise.
Alarm-from-USB was verified on the desktop (fork development.md §5) and
rides exactly the native + mpv path proven here; not re-run on device.

**udev**, shipped as `/usr/lib/udev/rules.d/90-chumby-ignore-cec-pointer.rules`:

```
SUBSYSTEM=="input", KERNEL=="event*", ATTRS{name}=="vc4-hdmi", ENV{LIBINPUT_IGNORE_DEVICE}="1"
```

This removes the phantom mouse cursor ([design.md](design.md) §7). It was
first tested from `/etc/udev/rules.d/` with
`udevadm control --reload && udevadm trigger /dev/input/event*`; that copy was
removed once the packaged rule shipped, so the device carries exactly one.

**`LP_NUM_THREADS=1`** is a launcher default, not a device file. It was
trialled through `/etc/default/chumby-player`, which was then removed — the
device carries no local override.

**Desktop session**: `systemctl disable --now lightdm`, with
`loginctl enable-linger pi` set **first** so the user manager (and PipeWire)
survive without a login session.

**Audio**: the USB sink is PipeWire's default; hardware volume was set to
35%. The panel's volume slider live-controls mpv and persists to
`psp/volume`; `alarm_volume` is a separate fixture value.

**Never written to `/home/jan/chumby_backup`** — it is read-only ground
truth. The Pi never reads it either; the SWF is copied at build time.

**Measurements** (idle on the clock widget, 480×320, packaged defaults):
~215% CPU before the two levers, ~103% after. Of the original 215%, roughly
170 was lavapipe's four raster worker threads. Temperature ~60 °C with the
soft-limit sticky bit set.

Things that did **not** help, recorded so nobody retries them: `--quality low`
(MSAA is not the cost driver); rebuilding Mesa or swapping lavapipe for a GL
path (the same llvmpipe rasterizer sits underneath); a client-side
`set_cursor_visible` hook (the cursor is server-drawn); `wlrctl`'s virtual
pointer under headless cage (never reaches the client).

## 7. Traps

The traps of working on the player itself — the stale `target/`, missed
clicks on the desktop — are in the fork's `claude-docs/development.md` §7.
These are the ones this repo owns.

- **The stale binary.** Rebuild before concluding anything. On the device
  the mistake is one `install` away, and the symptom is a feature that looks
  unimplemented.
- **`pkill -f ruffle_desktop`** over SSH also matches the session's own
  `bash -c` command line and kills the session, exit 255. Use `pkill -x`.
- **Orphaned mpv.** SIGTERM on the player skips destructors, so its mpv child
  survives and keeps playing. `pkill -x mpv` too.
- **DRM card numbers move between boots.** Always the `by-path` name.
- **A fixture change is not deployed** until `/var/lib/chumby/fixtures` is
  wiped and re-seeded.

## 8. Documentation

Three documents, this one included. Requirements state what must be true,
design states how and why, development states how to work on it and what was
done to the device. A note that fits none of the three is obsolete, or it
belongs to the player and goes to chumby-ruffle's `claude-docs/`.

Per-session or per-milestone records are not kept. What survives a milestone
is the decision and the reason, folded into the document it belongs in.
