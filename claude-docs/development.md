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
also discards persisted volume and alarms — so for a **binary-only** change,
skip the fixture rsync and the wipe (done for the FR10 deploy, 2026-07-10:
volume/alarms survived, real network diagnostics verified on-screen).
`dpkg` still reports the old version afterwards — the on-disk files are
ahead of the package until the next install.

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
The current device is a **Pi 3B+**, arm64 Raspberry Pi OS trixie, on wifi
(SSID "LXC", brcmfmac; moved off wired ethernet before 2026-07-10 — eth0
stays cabled-off), with the ILI9486 480×320 SPI TFT and a USB sound card.
An earlier
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

**Kiosk**: `chumby-player.service`, shipped by the deb; `postinst` enables
it and reloads udev. (`chumby-widget-channel.service` also shipped through
0.6.0; dropped in 0.7.0 — design §4. On the first upgrade the stale oneshot
stays loaded-but-gone until the next reboot or a `systemctl daemon-reload`;
harmless, `RemainAfterExit` with nothing left to run.)

Version **0.6.0** (defined 2026-07-13; not yet built or deployed — the deb
must carry the brightness player, see below) adds `90-chumby-backlight.rules`
— write access for the player's brightness feature (fork FR16, design §8):
`chgrp video` + `g+w` on a backlight device's `brightness` file at add time,
plus a `udevadm trigger --subsystem-match=backlight` in postinst. Inert on
the current TFT (no backlight device); nothing was executed on the device
for this change. The `VERSION` default was bumped although the rule is the
only payload, because the fork's `player.toml.example` (a conffile) gained
`brightness_ctl` — rebuilding different content under the same version is
the 0.5.0 conffile trap below. Deploying 0.6.0 should follow the fork's
brightness PR merge so the player and its config template travel together.

Version **0.7.0** (defined 2026-07-13; not yet built or deployed) replaces
the widget-channel machinery (design §4): `chumby-widget-channel.service`
and the sidecar generator are gone — the fork's channel fixture is static
now — replaced by the user-run `chumby-local-widgets` helper: it scans
`/var/lib/chumby/widgets` and writes `/psp/profile.xml` after a `Y/n`
overwrite prompt; the launcher only creates the folder. Verified offline on
the desktop (`access_chumby_com = 0`, zero passthrough lines; merged widget
loaded and executed, including a percent-encoded path; the cache-salvage
help checked against the chumby backup). Needs the fork gitlink bumped
past the fork's housekeeping merge (generator + sidecars deleted there);
nothing executed on the device yet.

Version **0.8.0** (defined 2026-07-13; not yet built or deployed) —
housekeeping task 2, the SWF-free distributable. `chumby-player-data` is
**retired**: the git-clean fixtures tree ships in `chumby-player` (staged
with a `git ls-files -o` prune so no gitignored binary can leak), the
fork's base channel is empty, and every copyrighted file is owner-copied
into `/var/lib/chumby` — `controlpanel.swf` (launcher refuses to start
without it, printing the copy instructions; `CHUMBY_SWF` and the old
data-deb `/usr/share` path still win for existing installs), `widgets/`,
`alarmtones/` (fixture path becomes a symlink into the state dir unless a
data-deb-seeded tree still has tones), `intro.swf`. CI now builds and
install-tests the deb on every PR with zero SWF involvement, asserts the
no-SWF refusal and greps the deb for leaked `.swf`; the movie-start test
is skipped by default and runs only where the SWF is fetched (main /
dispatch), driving the real launcher under Xvfb. Desktop end-to-end
verified: staged deb tree + files copied from the chumby backup
(`/usr/widgets/controlpanel.swf`, `builtinclock.swf`,
`/usr/chumby/alarmtones/*.mp3`) → panel boots offline, backup clock
merges and plays. On Jan's existing Pi nothing needs copying: the
installed data deb keeps serving all four paths until removed.

**Apt repo** (2026-07-13, same branch): `apt-repo` CI job publishes the
deb as a signed flat repo on GitHub Pages (design §5). Signing key
generated on the dev box (`gpg --quick-gen-key … ed25519 sign never`),
public half committed (`pkg/apt/chumby-archive.gpg`), private half handed
to Jan for the `CHUMBY_APT_GPG_KEY` secret — never in git. Dress-rehearsed
locally: `apt-ftparchive` + signing, `gpgv` good on both signatures, and a
real `apt-get update` + `apt-cache policy` against the `file://` repo
(arch forced to arm64) offered 0.8.0. Jan-side setup (both done
2026-07-13): set the secret, and — as repo admin — set Pages
"Build and deployment → Source" to **GitHub Actions**. Creating the
Pages *site* needs administration:write, which neither the workflow
token (configure-pages enablement fails: "Resource not accessible by
integration") nor the dev box's gh login (jluehr-4711, push but no
admin on yanosz/chumby-pi) has; the first deploy failed on exactly
that, and a gh-pages-branch workaround was briefly on main (aec65dd,
reverted in 0416e2f) before Jan flipped the setting.

**Fresh-card install exercise → 0.8.1** (2026-07-13). Jan reinstalled
the Pi 3B+ from a vanilla Raspberry Pi OS card and installed
`chumby-player` 0.8.0 from the apt repo — the first real end-to-end
install. Three findings, all reproduced and fixed:

- **The vendor display overlay cannot work.** The Waveshare wiki's
  `dtoverlay=waveshare35b-v2` (plus its `hdmi_*` block) is an fbtft
  driver: it creates only `/dev/fb1`, never a DRM device, and cage needs
  KMS — hence `Found 0 GPUs, cannot create backend`. The working
  config.txt lines are exactly the two recorded above (piscreen with
  `,drm`); Jan restored them from the old card's backup and the by-path
  device appeared (`[drm] Initialized ili9486`, connector `connected`,
  mode 480x320). The `Failed to parse EDID` error that remains at every
  cage start is harmless — the SPI panel has no EDID.
- **A sudo-copied `controlpanel.swf` is unreadable by the service.**
  `sudo cp` kept `-rwx------ root:root`; the launcher's old existence
  check (`[ -f ]`) passed, the player started and showed a **blank white
  screen** — journal: `Async error: Could not fetch: "Permission denied
  (os error 13)"`. Device fixed with `sudo chown pi:pi
  /var/lib/chumby/controlpanel.swf && sudo chmod 644 …`; panel rendered
  (~101 % CPU, the known NFR4 signature — it idles near 4 % when stuck).
- **The launcher's refusal was invisible** on the broken overlay: it
  runs *inside* cage, so when cage dies first the copy-instructions
  never print.

Version **0.8.1** (built and deployed 2026-07-13 via `pkg/deploy-pi.sh`,
player unchanged at fork `8772232a0`) addresses ii/iii: the launcher's
`check_swf` tests readability (with a chown/chmod hint naming the exact
file), an unreadable owner `intro.swf` warns and is skipped, and the same
check runs as `ExecStartPre=/usr/bin/chumby-player-run --check` — so a
missing/unreadable SWF fails the unit with the instructions in
`systemctl status chumby-player` even when the compositor cannot start
at all. The deb also ships `/etc/default/chumby-player` (conffile, all
comments) documenting the `WLR_DRM_DEVICES` by-path name per Pi model
(`3f204000.spi` is Pi 3 SPI0; Pi 4 is `fe204000.spi`), `WLR_RENDERER`
and `CHUMBY_AUDIO_DEVICE`. Verified on the device: chmod 000 + restart →
ExecStartPre exits 1 and the journal carries the hint; chmod 644 +
restart → panel up. (CI's install test still matches — it greps for
`controlpanel.swf not found`.)

Version **0.8.2** (built and deployed 2026-07-13, player still
`8772232a0`) adds `chumby-download-firmware` (design §5, NFR1
amendment). Run on the device as `pi`: control panel skipped (already
present — and md5-identical to what the script would fetch), the
Unsubscribed Clock SWF + thumbnail downloaded into
`/var/lib/chumby/widgets` (0644, owner `pi`), `chumby-local-widgets`
generated `/psp/profile.xml` (instance 1000), player restarted clean.
This also populated the previously empty widget channel — the suspected
cause of the post-boot black screen (panel slides away to play an empty
channel; a `chumby-ctl bend` summons it back).

Installed version: **0.8.2** (deployed 2026-07-13, above). Earlier:
0.8.1 (2026-07-13, above); 
0.5.0 (2026-07-12 via `pkg/deploy-pi.sh`,
player at fork branch `intro-widget` — boot-time intro in the launcher,
see below); 0.4.0 (2026-07-11, fork `config/player-toml`
`2e8343fc9`, PR #19 — player.toml config FR14 + music source policy FR15);
0.3.0 (2026-07-11, fork `usb-music` `b218502` — USB/local music C11);
0.2.0 (2026-07-10, fork tip `41fb650`) brought real network diagnostics,
the backup alarm and real device identity; binary sha256 verified on both
ends.

**Boot-time intro deploy** (2026-07-12, appliance 0.5.0). Two snags, both
now handled:

- `apt install` hit an interactive **conffile prompt** on
  `/etc/chumby-player/player.toml` — the shipped default changed between
  0.4.0 and 0.5.0 (PR #20 rewrote the `access_chumby_com` comments and
  added `device_guid`) and the device copy carries Jan's edits — and dpkg
  died on EOF over the non-interactive ssh. Recovered with
  `sudo dpkg --force-confold --configure -a`: local file kept
  (`access_chumby_com = 1` preserved), the new default landed as
  `player.toml.dpkg-dist`. `deploy-pi.sh` now passes
  `-o Dpkg::Options::=--force-confold` so this cannot recur.
- The device's live fixtures predate `intro.swf`, and the launcher seeds
  only when `$STATE/fixtures` is absent. A re-seed
  (`rm -rf /var/lib/chumby/fixtures`) would have discarded alarms, volume
  and the cached account channel, so instead the one new file was copied
  into the live tree:
  `mkdir -p /var/lib/chumby/fixtures/rootfs/usr/widgets && cp -a
  /usr/share/chumby-player/fixtures/rootfs/usr/widgets/intro.swf` there
  (as `pi`, over ssh). A fresh install needs no such step.

After `systemctl restart chumby-player` the kiosk came up with the tour:
`ps` shows `ruffle_desktop … /var/lib/chumby/fixtures/rootfs/usr/widgets/intro.swf`
— the launcher's pre-panel run, exactly `start_intro`'s behavior. On-device
checks (tour on the TFT, flag buttons, next-boot gating, in-panel INTRO)
are Jan's pass.

**Owner config** (2026-07-11, shipped with 0.4.0): the player reads
`<fixtures>/player.toml` once at start (fork FR14). On the device the real
file is `/etc/chumby-player/player.toml` — a dpkg **conffile** (default
content = the fork's `player.toml.example`), which `chumby-player-run`
links into the live fixtures root at every start
(`ln -sf … /var/lib/chumby/fixtures/player.toml`), so the setting survives
both deb upgrades and fixture re-seed wipes. On this device
`access_chumby_com` was set to 1 (Jan's decision, 2026-07-11):

```sh
sudo sed -i 's/^access_chumby_com = 0/access_chumby_com = 1/' \
    /etc/chumby-player/player.toml && sudo systemctl restart chumby-player
```

Result verified on the TFT: the Music source list shows SHOUTcast Radio /
blue octy radio / Sleep Sounds / My Streams / My Music Files (Squeezebox
stays hidden, `enable_lyrion = 0`); the SHOUTcast directory loaded live
through the revived chumby.com proxy, and station 1 (ANTENNE BAYERN)
played through mpv (`http://stream.antenne.de:80/antenne`, panel volume
60). Playback stopped and the kiosk restarted to its idle state afterwards.

**chumby.com registration + remote channels** (2026-07-11, roadmap item 5,
fork branch `registration-phase2`; mechanism in the fork's design.md §12).
The player passes the "using chumby.com" surface through to the revived
service under `access_chumby_com` (already 1 here) + a stable identity, so
the panel's own register wizard and the account's real widget channel work.
(Do **not** record this box's serial or GUID anywhere tracked — the salt is
public, so the serial reproduces the GUID and the GUID impersonates the
device.) Deployed by **binary-only hot-replace** on top of 0.4.0 (no deb, no
fixture wipe, so volume/alarms survive the deploy):

```sh
# from the submodule, dist cross-build (§3; use --release while iterating), then:
scp target/aarch64-unknown-linux-gnu/dist/ruffle_desktop pi@<pi>:/tmp/ruffle_desktop.new
ssh pi@<pi> 'sudo systemctl stop chumby-player &&
  sudo install -m755 -o root -g root /tmp/ruffle_desktop.new /usr/lib/chumby-player/ruffle_desktop &&
  sudo systemctl start chumby-player'   # sha256 verified equal both ends
```

Verified live on the TFT end-to-end: with the flag on, an unregistered box
boots into the panel's **register wizard** (authorize passes through →
`<unauthorized/>` → `register()`, GUID + oval pad shown). Jan tapped the
pattern and claimed the GUID on chumby.com; the 5 s poll flipped to `main`.
Then the real account channel loaded — chumbies → the account's profile →
one widget instance ("12 Hour Flip Clock") — the widget SWF **downloaded,
cached and rendered on the TFT** (see design §12 for the widget-cache
download-in-Rust and the scheme-less rootfs path), and the main bar's
**CHANNEL/DELETE became enabled** (SEND/RATE stay disabled, Phase 3). A
`systemctl restart chumby-player` re-authorised straight to `main`;
identity is recomputed each boot, so nothing is persisted for registration.
Caveats (design §12): the wizard's success OK button wipes a set of `/psp`
alarm/music prefs (faithful clean-slate reset); widget SWFs cache into the
persistent rootfs `/tmp/widgetcache`. To take a box offline set
`access_chumby_com = 0`. A box **without** a hardware serial (not this Pi)
can still register by setting `device_guid` in `/etc/chumby-player/player.toml`.

**Temporary debug aids used during this work, since reverted to pristine:**
the player's own logs don't reach journald (it's a cage wayland client), so
to read them set `RUST_LOG` and redirect — either add `RUST_LOG=…,chumby_host=info`
to `/etc/default/chumby-player` (the unit's `EnvironmentFile`) and/or append
`> /tmp/ruffle.log 2>&1` to the `exec` line of `/usr/bin/chumby-player-run`.
Both were removed after verification; the launcher and env are back to the
deb's shipped state.

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

Packaging audit for a *plain* Pi (2026-07-11, after the milestone closed):
deb contents and control verified from the built artifacts —
rule/unit/`/media/chumby-usb`/updated scripts all present, `Depends`
already carries `cage, mpv, pipewire-alsa, python3, chumby-player-data` +
libs, and the guards' tools (`findmnt`, `mountpoint`) are util-linux
(essential). One real gap found and fixed: on a Pi **booting from a USB
disk**, the root/boot partitions match the udev rule, and the panel would
have been offered the rootfs as USB music (also triggered at install time
by postinst's block trigger). The unit now short-circuits for any device
`findmnt` shows as already mounted; boot-time fstab mounts are visible to
that check because default unit dependencies order the service after
`local-fs.target`. Two non-issues, checked and left alone: simultaneous
partition events serialize on the single transient mount-unit name, and
dpkg's rmdir of a busy `/media/chumby-usb` on package removal fails
silently and harmlessly. Redeployed (0.3.0 rebuilt) and the guard proven
on-device: `systemctl start chumby-usb-mount@mmcblk0p2` (the mounted root
partition) exits success and mounts nothing.

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
`losetup -d`) returned the panel to the empty state.

**Physical-stick pass** (2026-07-11, later the same day, Jan's 114.6 GB
SanDisk with one vfat partition): plug-in fired the udev match with no
manual step — `chumby-usb-mount@sda1` active within the minute, vfat ro at
`/media/chumby-usb`. The TFT browsed it correctly: four MP3s listed, the
stick's `.exe`/`.dmg`/`.pdf` cruft filtered by the panel's own extension
match, `System Volume Information` shown as an ordinary folder. Tapping a
track played it audibly through the USB sound card (mpv on the resolved
rootfs path). **Yank while playing**: the service went inactive via its
device binding, the `systemd-mount` unit garbage-collected, the mountpoint
returned to an empty dir, mpv exited, no player errors or panic; the
browser showed its stale listing until screen re-entry (it re-lists only
on entry — known), then "No files available". Milestone closed.
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
