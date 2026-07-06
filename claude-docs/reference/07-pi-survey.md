# 07 — Pi device survey (Step 3.1)

Date: 2026-07-02. Access: `ssh pi@192.168.210.159` (key auth, hostname
`chumby-pi`). Survey was read-only; nothing on the device was changed.

**This is an interim device, not the final target** (see §1). Findings that
depend on the 3A+ specifically are marked; most transfer to the 3B+
(same SoC family, same VideoCore IV GPU).

## 1. User-provided context (Jan, 2026-07-02)

- Interim device: Pi **3 Model A+** with an **HDMI display**. The 3A+
  reportedly does not support the final display; **2× Pi 3B+ are ordered**
  as the final targets.
- Final display: touch TFT with **XPT2046** resistive-touch controller.
  (Jan wrote "XPT4046"; assuming the ubiquitous XPT2046 SPI touch chip —
  *confirm at CHECKPOINT 4*, and the panel model/resolution once known.)
- Audio: **USB sound is the final route.** A USB audio dongle (Jieli
  "UACDemoV1.0") is already attached to the interim Pi.
- Bend button: **not attached.** Jan can bridge GPIO pins if told which
  (proposal in §4.4). Additionally wants a **CLI utility that fires the
  bend event via IPC** into the running player.
- Milestone-3 goals confirmed by Jan (extends the plan's 3.3):
  1. Port to the Pi **as a Debian package**; reorganize the git repo as a
     **monorepo** (debian packaging incl. resources, plus the ruffle fork).
  2. The Pi **boots straight into the player, fullscreen**.
  3. Installing the package turns a stock RPi into "player mode";
     **SSH or a magic key must be able to disable it**.

## 2. Device findings

| item | finding |
|------|---------|
| model | Raspberry Pi 3 Model A Plus Rev 1.1 |
| SoC / arch | BCM2837B0, 4× Cortex-A53 @ 1.4 GHz, **aarch64** (arm64 userland) |
| RAM | 512 MB physical: 448 MB ARM / 64 MB GPU split → **415 MiB usable**, zram swap 415 MiB active. *3A+-specific: the 3B+ has 1 GB.* |
| OS | Raspberry Pi OS (Debian 13 "trixie") arm64, pi-gen stage4 (desktop) 2026-06-18, kernel 6.18.34+rpt-rpi-v8 |
| storage | 512 GB SD card, 445 GB free on `/` |
| display | HDMI-A-1 connected; preferred mode **1920×1200**, 1080p et al. available. KMS driver (`dtoverlay=vc4-kms-v3d`) |
| backlight | `/sys/class/backlight/` is **empty** (HDMI panel). → Step 3.4 (real backlight control) is **untestable on the interim device**; revisit with the final TFT |
| audio | card0 `vc4hdmi` (HDMI), card1 `bcm2835 Headphones` (3.5 mm jack), **card2 `UACDemoV1.0` (USB, the final route)** |
| input | **none attached** — no keyboard, mouse, or touch. Only HDMI-CEC and the USB dongle's HID volume keys. All interaction must come via SSH / IPC / GPIO |
| network | WiFi only (3A+ has no Ethernet), 192.168.210.159/24, regdom DE |
| GPIO | `/dev/gpiochip{0,1,4}`; `pinctrl` and `gpioset` (libgpiod) installed |
| session | `graphical.target`; lightdm **autologin** user `pi` → session `rpd-labwc` (**Wayland**, labwc + wf-panel-pi + pcmanfm desktop) |
| mpv | **not installed**; apt candidate 0.40.0-3+deb13u1 (fine — package dependency) |
| cage | apt candidate 0.2.0-2+rpt1+b1 (relevant for the kiosk proposal, §4.5) |

## 3. Graphics stack — the step-3.2 "known risk", now concrete

- The Pi 3 GPU is **VideoCore IV**; mesa's `vc4` driver provides **GLES 2.0
  only**. Ruffle's wgpu **GL backend requires GLES 3.0** → the GPU GL path
  is unavailable.
- Vulkan ICDs present: `broadcom` (v3dv — VideoCore VI, Pi 4+; will not
  enumerate on VC4) and `lvp` (**lavapipe, software Vulkan**). wgpu will
  therefore almost certainly select **lavapipe = CPU rendering**.
- Consequence: at the panel's native 1920×1200 software rendering is likely
  too slow. The SWF is 320×240 @ 12 fps; at a reduced HDMI mode (e.g.
  800×600) or the final TFT's class (~480×320) it is plausibly fine.
  **Must be measured in 3.2/3.3** — FPS at several output modes.
- This transfers 1:1 to the 3B+ (same VC4 GPU). RAM pressure does not:
  the 3B+ has double the memory.

## 4. Proposals (decide at CHECKPOINT 4)

### 4.1 Build strategy
Native build on the Pi is impossible (415 MiB RAM; rustc needs GBs).
Propose **cross-compiling `aarch64-unknown-linux-gnu` from the dev box**
(amd64): rust target via rustup, `aarch64-linux-gnu-gcc` as linker, arm64
sysroot for the few C deps (ALSA/libudev). Fallback if sysroot wrangling
gets ugly: the `cross` tool (Docker). Documented either way in
`08-pi-build.md`.

### 4.2 Debian packaging
Binary `.deb`(s) built on the dev box with the cross-compiled binary —
**not** a full Debian source package (vendoring a Rust workspace into
sbuild is heavy and serves no goal here). Proposed split, open to debate:
- `chumby-player` — ruffle binary (`--features chumby`), launcher,
  systemd/session integration, `Depends: mpv, cage` (or labwc, §4.5).
- `chumby-player-data` — fixtures, `controlpanel.swf` copy (private use —
  the SWF is copyrighted chumby firmware; fine for Jan's own devices, must
  never be published in an apt repo), widget SWFs.
Installing both = player mode; SSH always stays on; disable =
`systemctl disable --now chumby-player` or the magic key (§4.5).

### 4.3 Monorepo reorganization
Current repo + the local-only ruffle fork under `resources/ruffle` (branch
`chumby`, base `91b61d40`) become one repo, e.g.:
`ruffle/` (fork), `debian/` or `pkg/` (packaging), `fixtures/`, `docs/`,
`resources/` (SWFs). Open question: keep the fork's upstream history
(subtree merge — big but rebase-friendly) vs. squash-import with the base
commit recorded in `docs/patch-notes.md`. Recommendation: **subtree with
full history**, since rule 2 (rebasable clean patch) is easier to honor.

### 4.4 Bend input mapping
Propose **GPIO17 (physical pin 11) bridged to GND (physical pin 9)** —
adjacent pins, no special alternate functions, safe to short to ground.
Software side, two layers:
1. `dtoverlay=gpio-key,gpio=17,active_low=1,gpio_pull=up,keycode=…` — the
   kernel turns the button into a normal input key event; PiHost/Ruffle
   maps that key to the bend action. Debounce for free, no polling daemon.
2. The requested **CLI trigger**: a small control socket (Unix domain) in
   the chumby module + `chumby-ctl bend` (also useful later for the magic
   key, screenshots, volume, and scripted tests — same IPC path).

### 4.5 Fullscreen boot ("player mode")
Replace the stock desktop autologin with a dedicated kiosk session:
**`cage` (single-app Wayland compositor) launching ruffle fullscreen**,
run either as the lightdm autologin session or as a systemd service on
`tty1` (dropping lightdm entirely). Recommendation: systemd service +
cage — fewer moving parts, `systemctl disable` is the SSH escape hatch.
"Magic key": with no keyboard on the final device, propose the bend
button held ≥5 s (or `chumby-ctl exit-player`) as the escape.

### 4.6 Target resolution / scaling
Stays configurable (working assumption from 2026-06-12; nothing hardcoded).
For interim perf tests: force a reduced HDMI mode via kernel cmdline
(`video=HDMI-A-1:800x600@60`) and let Ruffle scale. Final resolution is
decided by the real TFT once the 3B+ arrives.

### 4.7 Audio route
USB card (card2) for everything: Ruffle/cpal and mpv both pointed at it
explicitly by the launcher/package config (ALSA device by *name*, not
index — USB enumeration order isn't stable). HDMI and the 3.5 mm jack
remain manual fallbacks. First-exercise flows on device (per M3
definition): **alarm ring-through (B6)** and **My Streams playback** —
the latter needs the network-class change to *real-network*, to be
confirmed at CHECKPOINT 4 and recorded in `feature-decisions.md`.

## 5. CHECKPOINT 4 outcome (user, 2026-07-02)

1. Build: cross-compile from dev box — **approved** (§4.1).
2. Packaging split `chumby-player` + `chumby-player-data` — **approved** (§4.2).
3. Monorepo: **subtree with full history** (§4.3).
4. Bend: GPIO17/pin 11 ↔ GND/pin 9 proposed to user (§4.4).
5. Kiosk: **systemd + cage**, drop lightdm in player mode (§4.5).
6. My Streams → **real-network confirmed** (§4.7; recorded in
   `feature-decisions.md`).
7. **XPT2046 confirmed** (user, 2026-07-02). Final TFT panel model +
   resolution still open until the 3B+ hardware arrives. Bend pins not
   bridged yet; until then the bend button is fired via the `chumby-ctl`
   shell script (repo root) → control FIFO (user request 2026-07-02). (§1)
