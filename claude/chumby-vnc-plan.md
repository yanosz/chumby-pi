# A VNC viewer on the real chumby, beside the control panel

Draft 2026-09-09, approved. Idea stage — **nothing measured on the device
yet.** Step 1 waits for the chumby, offline as of 2026-09-09 and back in a few
days; the step order stands as written.
Sources: `chumby_backup` (read-only), the chumby wiki, `pi.nic/README.md`.

## Goal

A Pi is attached to a chumby as its NIC (`pi.nic/`: USB ethernet gadget, the
Pi NATs to wifi). Show a **conventional Unix GUI app** running on that Pi on
the chumby's own 3.5" screen, and switch back to the control panel at will.
The device stays a chumby; this adds a second thing it can show.

Route: a native VNC client on the chumby's **second framebuffer plane**, with
the control panel left running on the first — so alarms keep working while the
viewer is up.

## What we know

The box: Linux **2.6.16-csb**, ARMv5TE (i.MX21, "Ironforge"), root read-only
cramfs, only `/psp` (jffs2) persists. Firmware **1.7.2**
(`chumby_backup/etc/software_version`; note `etc/version` is an absolute
symlink to `/etc/issue` and resolves against the *host*, not the chumby).

The player, `chumby_backup/usr/bin/chumbyflashplayer.x`: **Adobe FlashLite
3.1.5 (Version 9,1,120,0) — Chumby Industries**, ARM **EABI5**, dynamically
linked against glibc (`/lib/ld-linux.so.3`), built for Linux 2.6.14 with
**GCC 4.3.2**.

**The LCD controller composites two planes, and the player drives them:**

| String in the player binary                                                                                         | What it means                                                                    |
|---------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| `Using master fb=%d (/dev/fb%d)`, `Using client fb=%d (/dev/fb%d)`                                                  | the player picks a framebuffer *index* and knows two roles                       |
| `FrameBuffer::SetDriverValue(%s)` / `GetDriverValue(%s)`, `/proc/driver/imxfb/`, values `enable`, `alpha`, `key_en` | userspace control of the i.MX21 LCDC graphic window — alpha blend and colour key |
| usage `-f <frame buffer options {0\|1\|2} (2)>`, cfg name `FrameBufferOpts`                                         | the selection is a documented option                                             |

So the screen can be shared in **hardware** — no virtual framebuffer device,
no compositor copy loop, no `LD_PRELOAD` shim. `/drivers` ships `fbcon.ko` but
**no `vfb.ko`**, so a software fallback would mean building a kernel module.

Two levers from the same binary: `/psp/flashplayer.cfg` is in the player's
config search path (with `/tmp/` and `/mnt/usb/`), so the panel's options can
be set persistently without touching the read-only root; and
`-D (disable screen updates)` makes a hidden panel cheap, at the cost of a
restart to apply.

Input devices named by the player: `/dev/ts`, `/dev/input/event0`,
`/dev/bend`, `/dev/switch`. Its config carries both *"use of deprecated
touchscreen interface"* and *"touchscreen event interface"*. Calibration lives
in `/psp/ts_settings`.

**Toolchain** (wiki: *GNU Toolchain*): firmware ≥ 1.7 targets **GCC 4.3.2 /
glibc 2.8**, published toolchain CodeSourcery **`arm-2008q3`**
(`arm-none-linux-gnueabi`) — exactly what the player was built with.

    http://files.chumby.com/toolchain/arm-2008q3-72-arm-none-linux-gnueabi-i686-pc-linux-gnu.tar.bz2

Still served: HTTP 200, 77 137 543 bytes, modified 2013-02-03 (checked
2026-09-09). It is an **i686 host** build, so amd64 Debian 13 needs 32-bit
runtime support or a container.

## Steps

### 1. Survey the device (read-only)

Ask Jan first; he uses the box. Read: `ls -l /dev/fb*`;
`cat /proc/driver/imxfb/*`; `fbset -fb /dev/fbN -i` per plane; which plane the
panel took (its own `master fb`/`client fb` log line); `cat
/proc/bus/input/devices`; free RAM; current `/psp/flashplayer.cfg`.

**Artifact:** a survey section appended to this file with the raw readings.

**CHECKPOINT 1 — does a usable second plane exist at full 320×240, and what
are its geometry and bpp constraints?**

### 2. Toolchain

Install `arm-2008q3` and build a hello-world that runs on the device against
its own `/lib`.

**Artifact:** a working cross-toolchain, a verified binary on the device, and
a build recipe in this file (host setup, invocation, sysroot).

**CHECKPOINT 2 — vendor toolchain on a 32-bit-capable host or a container,
and dynamic against glibc 2.8 or static?**

### 3. Prove the plane

~50 lines of C: open the overlay framebuffer, fill it, toggle
`enable`/`alpha`/`key_en` with the control panel running underneath
throughout.

**Artifact:** `planetest.c` plus measurements — does the panel survive, is it
still correct when uncovered, how fast is the switch, what does it cost in
CPU.

**CHECKPOINT 3 — is coexistence real, or does the panel corrupt or stall?**

### 4. Input arbitration

The plane switch does not move touches: while the viewer is on top, the panel
underneath still receives every tap. Test `EVIOCGRAB` from a second process on
the panel's input device; confirm the panel goes quiet and recovers cleanly on
ungrab. If the panel is on `/dev/ts` rather than evdev, find the equivalent —
or move it to the evdev interface via `/psp/flashplayer.cfg`.

**Artifact:** a grab test program and a verdict on the mechanism.

**CHECKPOINT 4 — is there a clean way to take input away and give it back?**

### 5. The viewer

Port `fbvnc` or a small `libvncclient` front end onto the overlay plane. Touch
→ pointer; apply `/psp/ts_settings` if the client reads raw coordinates.

**Artifact:** a VNC client binary for the chumby, its build recipe, and the
patch carrying the plane and touch handling.

### 6. The server side on the Pi

One conventional Unix GUI app on a small X server sized to the panel, plus a
VNC server — not a scaled-down desktop. Source resolution matters more than
transport (`claude-docs/design.md` §6): a 1080p desktop squeezed into 320×240
is unreadable, a 320×240 X session is not.

**Artifact:** a script or systemd unit on the Pi that starts the X server, the
app, and the VNC server at the panel's resolution.

### 7. The switch, and surviving a reboot

Pick the trigger — a watcher on `/dev/bend` (unknown whether that char device
tolerates a second reader while the panel holds it), a trigger from the Pi
over the gadget link (needs the sshd the chumby ships but does not start: one
line in `userhook1`, `pi.nic/README.md`), or a local widget that shells out,
since the panel executes backticks (`-R`, `-w`, `-W` in the usage). Then
persist everything in `/psp` — binary, hooks, `/psp/flashplayer.cfg` — and
make sure the panel can never be left hidden with no way back.

**Artifact:** the switch script, its trigger, and the `/psp` install step.

**CHECKPOINT 5 — which trigger.**

## Risks and open questions

- **CPU.** Both processes run at once on a 2006 SoC, and Flash repaints even
  when invisible. `-D` may help, at the cost of a panel restart.
- **The overlay plane** may be smaller than the screen, fixed-bpp, or need its
  own reserved buffer. Unknown until step 1.
- **Link ceiling.** The gadget link measured ~1.2 MB/s in issue 8's fetch.
  Probably ample for one mostly-static app window; unmeasured for this use.
- **Issue 7** — a Pi reboot orphans the chumby's link until the chumby
  reboots. Cosmetic today; visible the moment the screen depends on it.
- **No keyboard.** Single-touch resistive gives tap and drag only. Either pick
  apps that need no typing, or put an on-screen keyboard in the X session on
  the Pi.
- **Scope.** Appliance/device work on the *real* chumby. Touches neither the
  fork nor the Pi-side player.
