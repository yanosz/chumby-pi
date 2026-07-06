# Hardware variation: other displays, sound devices, Pi models

The packaged defaults target the reference device (Pi 3B+, ILI9486
SPI TFT, PipeWire default sink). Everything hardware-specific is
overridable in **`/etc/default/chumby-player`**, which the kiosk unit
reads at start (`sudo systemctl restart chumby-player` to apply):

| Variable | Default | Meaning |
|----------|---------|---------|
| `WLR_DRM_DEVICES` | `/dev/dri/by-path/platform-3f204000.spi-cs-0-card` | which DRM device (display) cage runs on |
| `WLR_RENDERER` | `pixman` | `pixman` = software rendering (right for GPU-less SPI panels); unset/`gles2` for HDMI on the Pi's GPU |
| `CHUMBY_AUDIO_DEVICE` | unset (PipeWire default sink) | mpv audio device for panel audio |
| `CHUMBY_STATE` | `/var/lib/chumby` | writable state dir (live fixtures) |
| `CHUMBY_CTL` | `/tmp/chumby-ctl` | control FIFO path |
| `RUST_LOG` | `warn` | set `warn,chumby_host=info` to log all panel↔host traffic |
| `LP_NUM_THREADS` | `1` | software-Vulkan worker threads; more threads ≈ more CPU for the same fps on a Pi 3 |

## A different TFT / display

The player needs a **DRM-capable** display driver — it renders through
cage (a Wayland compositor), not the legacy fbdev/fbtft framebuffer.

1. **Pick the overlay for your panel** in `/boot/firmware/config.txt`.
   For SPI panels use the vendor's DRM-mode overlay (e.g.
   `dtoverlay=piscreen,...,drm` for ILI9486 types; PiTFT/ILI9341
   panels have `mipi-dbi`/tinydrm overlays). An HDMI/DSI display needs
   no overlay at all — the standard `vc4-kms-v3d` KMS driver is the
   DRM device.
2. **Rotation gotcha:** DRM's base orientation is landscape. If you
   are porting settings from an fbtft setup, fbtft's `rotate=90`
   corresponds to DRM `rotate=0`.
3. **Point the kiosk at the right DRM device.** Card numbers
   (`/dev/dri/card0`, `card1`) are **not stable across boots** — use
   the by-path name:

   ```sh
   ls -l /dev/dri/by-path/
   # e.g. platform-3f204000.spi-cs-0-card  → SPI panel on a Pi 3
   #      platform-fe204000.spi-cs-0-card  → same panel on a Pi 4
   #      platform-gpu-card                → vc4 (HDMI/DSI)
   ```

   Then in `/etc/default/chumby-player`:

   ```sh
   WLR_DRM_DEVICES=/dev/dri/by-path/<your-card>
   ```

   (The default bakes in `3f204000.spi` — the Pi 3 SoC's SPI0 address.
   Any other Pi model has a different address, so this override is
   needed even for the same panel on a Pi 4/5.)
4. **Renderer:** keep `WLR_RENDERER=pixman` for SPI panels (no GPU
   path to them). On an HDMI display via vc4, unset it in
   `/etc/default/chumby-player` to let the GPU render.
5. **Resolution:** the panel movie is 320×240 design-size and scales;
   480×320 and 640×480 are both verified. Higher resolutions cost CPU
   (software rendering scales with pixels).

### Touch calibration

Resistive SPI panels wire their axes every which way. The overlay
flags fix axis mapping (values for the reference panel: `swapxy=on`,
`invy=on`). Symptoms → fix, for `piscreen`-style overlays:

- touch moves the wrong axis (finger left/right moves cursor
  up/down): toggle `swapxy`
- one axis mirrored: toggle `invx` / `invy`
- **Gotcha:** in the shipped `piscreen` overlay these are
  inverted-boolean device-tree hacks — `swapxy=on` *removes* the
  swapped-x-y property. If a flag seems to do the opposite of its
  name, that's why. Just try the four combinations until taps land
  under the finger.

Capacitive USB/DSI touchscreens usually need no calibration at all —
libinput picks them up; they just work through the same touch path
(including the long-press = squeeze gesture).

## A different sound device

Panel audio (alarm tones, internet radio) plays through **mpv** into
the `pi` user's PipeWire session; system sounds follow the panel's own
volume/mute natives.

Two ways to route it:

1. **Set the PipeWire default sink** (affects everything in the
   session):

   ```sh
   wpctl status          # list sinks
   wpctl set-default <sink-id>
   ```

2. **Pin mpv to a device** (affects only the panel audio), in
   `/etc/default/chumby-player`:

   ```sh
   CHUMBY_AUDIO_DEVICE=alsa/plughw:CARD=UACDemoV10   # example: USB dongle
   ```

   List valid names with `mpv --audio-device=help` (run as the `pi`
   user). Any mpv device spec works (`pipewire/...`, `alsa/...`).

No sound at all? mpv missing = silent-stub mode (the UI still works);
`apt install mpv`. Also check the panel isn't muted (night mode mutes).

## A different Pi

- Any Pi that runs 64-bit Raspberry Pi OS should work; the deb is
  arm64. The Pi 3B+ (4×1.4 GHz) runs the panel at roughly two busy
  cores with `LP_NUM_THREADS=1` — faster Pis have headroom, a Pi Zero
  2 W is untested but has the same architecture.
- The only model-specific default is the SPI controller address in
  `WLR_DRM_DEVICES` (see above).
- The kiosk unit assumes user `pi` exists (it runs the compositor as
  a real logind session for `pi` on tty1). A different username
  currently means editing `pkg/chumby-player/chumby-player.service`
  and rebuilding the deb.
