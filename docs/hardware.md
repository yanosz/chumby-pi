# Hardware variation: other displays, sound devices, Pi models

The packaged defaults target the reference device (Pi 3B+, ILI9486
SPI TFT, PipeWire default sink), but the kiosk detects the display by
itself and every hardware-specific value has an override in
**`/etc/default/chumby-player`**, which the kiosk unit reads at start
(`sudo systemctl restart chumby-player` to apply). The file documents
each variable in place; in short:

| Variable | Default | Meaning |
|----------|---------|---------|
| `WLR_DRM_DEVICES` | auto-detected | which DRM device (display) cage runs on: the first SPI panel if one exists, otherwise cage's own choice (HDMI/DSI). Set it only to pin a specific device |
| `WLR_RENDERER` | `pixman` | software rendering in the compositor (right for GPU-less SPI panels) |
| `CHUMBY_RENDERER` | `tiny-skia` | the player's rasterizer: `tiny-skia` draws on the CPU and loads no Vulkan at all; `wgpu` goes through software Vulkan (lavapipe) and costs three times the CPU |
| `CHUMBY_QUALITY` | `low` | Flash stage quality, `wgpu` only; `low` skips anti-aliasing, `high` = stock Flash look |
| `LP_NUM_THREADS` | `2` | lavapipe worker threads, `wgpu` only |
| `CHUMBY_AUDIO_SINK` | unset (leave PipeWire's default) | PipeWire sink made the default output: moves the player and mpv alike |
| `CHUMBY_AUDIO_DEVICE` | unset (PipeWire default sink) | mpv audio device for music and alarms only |
| `CHUMBY_STATE` | `/var/lib/chumby` | writable state dir (live fixtures) |
| `CHUMBY_CTL` | `/tmp/chumby-ctl` | control FIFO path |
| `RUST_LOG` | `warn` | set `warn,chumby_host=info` to log all panel↔host traffic |

## A different TFT / display

The player needs a **DRM-capable** display driver — it renders through
cage (a Wayland compositor), not the legacy fbdev/fbtft framebuffer.

1. **Pick the overlay for your panel** in `/boot/firmware/config.txt`.
   For SPI panels use the vendor's DRM-mode overlay (e.g.
   `dtoverlay=piscreen,...,drm` for ILI9486 types; PiTFT/ILI9341
   panels have `mipi-dbi`/tinydrm overlays). An HDMI display needs no
   overlay at all — the standard `vc4-kms-v3d` KMS driver is the DRM
   device. DSI panels vary: the official Touch Display is detected by
   the firmware, while third-party ones need their own overlay on top
   of `vc4-kms-v3d` (e.g.
   `dtoverlay=vc4-kms-dsi-waveshare-panel,7_0_inchC` for the Waveshare
   5″/7″ DSI LCD (C)) and stay dark without it.
2. **Rotation gotcha:** DRM's base orientation is landscape. If you
   are porting settings from an fbtft setup, fbtft's `rotate=90`
   corresponds to DRM `rotate=0`.
3. **The kiosk finds the display by itself.** It pins an SPI panel by
   its stable by-path name (card numbers like `/dev/dri/card0` shuffle
   between boots) on any Pi model, and otherwise lets cage pick the
   HDMI or DSI output. Only a machine with several candidates needs a
   pin, in `/etc/default/chumby-player`:

   ```sh
   ls -l /dev/dri/by-path/
   # e.g. platform-3f204000.spi-cs-0-card  → SPI panel on a Pi 3
   #      platform-fe204000.spi-cs-0-card  → same panel on a Pi 4
   #      platform-gpu-card                → vc4 (HDMI/DSI)
   WLR_DRM_DEVICES=/dev/dri/by-path/<your-card>
   ```
4. **Resolution:** the panel movie is 320×240 design-size and scales to
   the screen, keeping its 4:3 shape; 480×320, 640×480 and a 1024×600
   DSI panel (black bars at the sides) have all run. Higher resolutions
   cost CPU, since rendering is done in software and scales with pixels.

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

There are **two** audio paths, which is why one knob is not enough: mpv
plays music and alarm tones, while the player emits the panel's own
sounds directly. Only the first follows `CHUMBY_AUDIO_DEVICE`; the
player follows the PipeWire **default sink**.

Two ways to route it:

1. **Choose the default sink** (affects everything — player and mpv),
   in `/etc/default/chumby-player`:

   ```sh
   CHUMBY_AUDIO_SINK=alsa_output.platform-3f902000.hdmi.hdmi-stereo
   ```

   Find the name with `wpctl status` for the id, then `wpctl inspect
   <id> | grep node.name`. The launcher applies it at every start;
   unset leaves the default alone.

   **Worth setting on an HDMI panel.** PipeWire's own pick is
   arbitrary, and on a Pi 3A+ driving the Waveshare HDMI display it
   chose the Pi's analog jack — so HDMI carried no audio at all and the
   panel's speakers stayed silent, with nothing in any log to say so.

2. **Pin mpv to a device** (affects only music and alarms), in
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
  arm64. With the shipped defaults a Pi 3B+ drives the SPI TFT on about
  a third of one core — the display, not the player, caps the frame
  rate there at 6-7 fps. Faster Pis have headroom; a Pi Zero 2 W has the
  same architecture but is untested. The original Pi Zero W is too slow.
- Nothing in the defaults is tied to a Pi model: the display is
  detected, the overlay line in `config.txt` is the same on every model.
- The kiosk unit assumes user `pi` exists (it runs the compositor as
  a real logind session for `pi` on tty1). A different username
  currently means editing `pkg/chumby-player/chumby-player.service`
  and rebuilding the deb.
