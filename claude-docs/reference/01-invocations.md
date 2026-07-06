# 01 — Artifact inventory & flash player invocations (Step 1.1)

Date: 2026-06-12. Sources: `/home/jan/chumby_backup` (device backup, read-only
ground truth; ironforge / Chumby Classic, software version 1.7.2) and
`resources/zurks-offline-firmware-classic` (zurk's offline firmware repo clone).

## 1. Control panel SWF variants

| # | Path | sha256 (first 12) | Size | SWF ver | Compressed | Notes |
|---|------|-------------------|------|---------|------------|-------|
| A | `chumby_backup/usr/widgets/controlpanel.swf` | `117e346e7ac0` | 510,837 | 6 | CWS (zlib) | **Shipped/built-in panel** (firmware 1.7.2 fallback, `DEFAULT_CP`) |
| B | `chumby_backup/tmp/controlpanel.swf` | `b458fc596127` | 593,693 | 6 | CWS (zlib) | **Downloaded panel, version 2.8.87b3** (`/tmp/controlpanelversion`); the file the device was actually running |
| C | `zurks…/www/controlpanel.swf` | `c4d7e544ca95` | 591,602 | 6 | CWS (zlib) | Offline-firmware panel; version unknown until decompile (≠ A, ≠ B) |

Byte-identical copies of C inside the firmware repo (same sha256):
- `lighty/html/xml/controlpanel.swf`
- `lighty/html/xml/c1/controlpanel.swf`
- `lighty/html/xml/controlpanel` — **no extension**: this is the raw SWF served
  by zurk's local lighty webserver at the URL path `/xml/controlpanel`, i.e. it
  impersonates chumby.com's download endpoint (see §6).

### Most recent official version — FOUND, already in hand
`download_cp` (see §5) fetches `http://www.chumby.com/xml/controlpanel?...`.
That URL is **still live** (checked 2026-06-12) and returns XML advertising
`falconwing-controlpanel-2.8.87b3.swf` with
md5 `21d54cd7f15b7671c8b661c16d53034f` — which is **byte-identical to variant B**
(`md5sum chumby_backup/tmp/controlpanel.swf` → same hash). So the backup's
`/tmp/controlpanel.swf` *is* the most recent officially served control panel;
no further hunting needed. (Server names it "falconwing-…" but it is what the
unparameterized endpoint serves; the Classic downloaded the same bytes.)
Community datapoints (forum id=8326): versions 2.8.57/2.8.72/2.8.83/2.8.84
existed; latest served is 2.8.87b3.

## 2. Related SWFs (potential `loadMovie` targets / companions)

All in `chumby_backup/usr/widgets/`:

| File | sha256 (first 12) | Size | SWF ver | Sig | Role (from launch scripts) |
|------|-------------------|------|---------|-----|----------------------------|
| `opening.swf` | `61f0eeb783fd` | 31,297 | 6 | FWS (uncompressed) | Boot logo animation (`start_opening_anim`) |
| `alt_opening.swf` | `6f10b51b08a8` | 20,275 | 6 | CWS | Alternate boot animation (if `/psp/alt_opening`) |
| `intro.swf` | `0e999fcb942f` | 939,224 | 7 | CWS | Intro movie (`start_intro`, skipped if `/psp/disable_intro`) |
| `preload.swf` | `36c54785345a` | 1,012 | 8 | CWS | Auto-loaded by the player itself unless `-U` (per `-h` usage text) |
| `builtinclock.swf` | `0624cfd6683f` | 14,751 | 6 | CWS | Built-in clock widget |
| `chumbradiod.swf` | `57ab7d2e3e5c` | 8,765 | 8 | CWS | Radio daemon helper movie |
| `temp_update.swf` | `0bfcc77a9629` | 18,242 | 8 | CWS | Firmware-update chooser UI (`select_update.sh`) |

(Which of these the control panel itself `loadMovie`s is a Step 1.3 question.)

## 3. The player binary

`chumby_backup/usr/bin/chumbyflashplayer.x` (1,851,940 bytes, ARM):
`Adobe FlashLite 3.1.5 Release (Version 9,1,120,0) - Chumby Industries`.
Full `-h` usage text extracted via `strings` → Appendix A. Config file search
order (from strings): `/mnt/usb/flashplayer.cfg`, `/psp/flashplayer.cfg`,
`/tmp/flashplayer.cfg` — **none exist in the backup**, so stock defaults apply.
Defaults worth noting: x=240, y=320 (portrait fb, rotated), 384 × 32K memory
blocks = 12 MB, log mask 383, pump delay 42–83 ms.

## 4. Invocations found in the backup (ground truth)

### 4.1 Control panel — `usr/chumby/scripts/start_control_panel`
```sh
/usr/bin/chumbyflashplayer.x -i $CP $CP_ARGS
```
- `$CP` resolution order:
  1. `/mnt/usb/controlpanel.swf` if present → adds `-dalternate=1` ("ALTERNATE")
  2. path in `/tmp/cp_path` if that file exists (downloaded panel; backup
     contains `/tmp/cp_path` → `/tmp/controlpanel.swf`)
  3. `/usr/widgets/controlpanel.swf` (`DEFAULT_CP`) → adds `-dbuiltin=1`
- First boot (`/psp/firsttime` = 1): forces `DEFAULT_CP`, adds `-dfirstTime=1`,
  treats as builtin.
- No network (per `network_status.sh`): uses `$CP_PATH` w/ `-dbuiltin=1`.
- Network + no prior download: runs `download_cp` (≤5 attempts), then uses
  `/tmp/cp_path`; on total failure writes an error to the framebuffer and exits.
- `-Q` (enable `fscommand quit`) is added whenever the builtin panel runs
  (`FP_QUIT=1`) so the panel can exit and the script re-runs itself to load the
  downloaded panel (tail recursion at the end of the script).
- Before exec: zeroes both framebuffers, resets `/tmp/flashheartbeat` and
  `/tmp/movieheartbeat` to `1`, creates `/tmp/flashplayer_started` (watchdog
  arm flag), and starts `/usr/sbin/btplayd` in the background.
- stdin redirection: if stdin is not a tty/null, all fds are redirected to
  /dev/null (busybox fd-sharing workaround).
- Caller: `etc/init.d/rcS` (`start_control_panel >/dev/null 2>&1 &`), except
  when `/psp/missed_alarm` exists, in which case `chumbalarmd` launches it.
- On player exit ≠ 0: code 255 (SIGILL) → `stop_control_panel --keepalive` and
  exit 1; otherwise restart via self-invocation.

**Effective ground-truth command lines:**
```sh
# normal boot, downloaded panel present:
/usr/bin/chumbyflashplayer.x -i /tmp/controlpanel.swf
# normal boot, no network / no download:
/usr/bin/chumbyflashplayer.x -i /usr/widgets/controlpanel.swf -dbuiltin=1 -Q
# first boot:
/usr/bin/chumbyflashplayer.x -i /usr/widgets/controlpanel.swf -dfirstTime=1 -dbuiltin=1 -Q
# USB override:
/usr/bin/chumbyflashplayer.x -i /mnt/usb/controlpanel.swf -dalternate=1 [-dbuiltin=1] [-Q]
```

### 4.2 Other movies
```sh
# start_opening_anim (rcS, backgrounded, unless /psp/UPDATE1):
/bin/nice -n -20 /usr/bin/chumbyflashplayer.x -i /usr/widgets/opening.swf -L 0 -Q
#   (-L 0 = no message output; movie can be /mnt/usb/opening.swf or alt_opening.swf)

# start_intro (rcS, foreground, unless /psp/disable_intro):
/usr/bin/chumbyflashplayer.x -i /usr/widgets/intro.swf -l 33 -Q -2 -b 512
#   (-l 33 = log errors+AS-trace only, -2 = sound hwsync, -b 512 samples/buffer)

# select_update.sh (firmware-update chooser):
chumbyflashplayer.x -Q -3 -i /usr/widgets/temp_update.swf
#   (-3 = do not start btplayd)
```

### 4.3 Signaling a running player (no new instance)
```sh
chumbyflashplayer.x -F15   # SIGTERM to instance in /var/run/chumbyflashplayer.pid
                           # (stop_control_panel, wait_for_opening)
chumbyflashplayer.x -F1    # SIGHUP → player reads events from /tmp/flashplayer.event
                           # (signal_soft_event.sh)
```
`signal_soft_event.sh` writes
`<event type="…" value="…" comment="…" destination="…"/>` lines to
`/tmp/flashplayer.event`, then `-F1`. **This XML-event channel is a core
environment touchpoint for Milestone 2** (hardware buttons, headphone, USB,
network events presumably arrive this way — to be confirmed in 1.3).

### 4.4 Supervision (cron, every minute — `psp/crontabs/root`)
```
* * * * * /usr/chumby/scripts/flashplayer_watchdog
* * * * * /usr/bin/renice 19 `cat /var/run/chumbyflashplayer.pid`
0 3 * * * /usr/chumby/scripts/sync_time.sh
```
Watchdog logic: if `/tmp/flashplayer_started` exists and either
`/tmp/movieheartbeat` or `/tmp/flashheartbeat` is older than 30 s →
`stop_control_panel --keepalive`, clear `/proc/sys/sense1|2/clear`, restart
`start_control_panel`. ⇒ *Something* must touch these heartbeat files
periodically (player and/or movie); the Ruffle replacement must replicate or
disable this contract. Also `/var/run/chumbyflashplayer.pid` is the
single-instance registration mechanism.

### 4.5 Misc
- `www/cgi-bin/memstats`: web CGI reading `/proc/<pid>` stats of the player pid.
- `flashplayer_is_running`: health check via pid file + `/proc/<pid>/stat`.

## 5. FlashVars and environment

FlashVars passed via `-d name=value` (complete set found in backup scripts):

| FlashVar | When | Meaning |
|----------|------|---------|
| `firstTime=1` | first boot | panel runs setup wizard path |
| `builtin=1` | shipped panel used | panel knows it's the fallback copy |
| `alternate=1` | USB override panel | panel loaded from /mnt/usb |

(The downloaded-panel invocation passes **no** FlashVars.)

Environment (from `etc/profile`; rcS itself only exports
`LD_LIBRARY_PATH=/lib` and `PATH` — note rcS does *not* source /etc/profile,
so treat these as "present in interactive shells, possibly compiled-in
defaults for the player"; verify in 1.4):
```
CNPLATFORM=ironforge  CONFIGNAME=ironforge
VIDEO_RES=320x240  VIDEO_X_RES=320  VIDEO_Y_RES=240
HAS_CP=1  LANGUAGE=en_US
```

## 6. download_cp & the offline firmware's impersonation

`usr/chumby/scripts/download_cp` (perl):
1. Builds `http://www.chumby.com/xml/controlpanel?id=<guid>&hw=<hw>&sw=<sw>&fw=<fw>&dcid_*=…&lang=<lang>`.
2. Expects XML: `<url> <compressed> <filename> <md5> <location> <launchname> <parameters>`.
3. Writes `<location>/<launchname>` to `/tmp/cp_path`, wgets the file, untars
   if `compressed=true`, verifies md5.

Zurk's offline firmware version of `download_cp` adds a `urlbase` override via
`/psp/urlbase_dlcp` (→ point it at the chumby's own lighty server) plus
`/psp/cp_software_ver` / `/psp/cp_firmware_ver` spoofing; lighty serves the SWF
at `lighty/html/xml/controlpanel`. **This is prior art for exactly our M2
approach** (environment impersonation instead of SWF modification). Note: the
served file is the raw SWF, not the XML envelope — how zurk's flow reconciles
that with download_cp's XML parsing is a Step 1.6 question.

## 7. Firmware-clone invocation differences (preview for 1.6)

The zurks repo `scripts/` are from a **later firmware generation** (references
to `VIDEO_RES`-suffixed bitmaps, `logo.brand` cmdline, `switch_fb.sh`,
falconwing-isms like bootstream/storage-partition scripts). Differences vs
backup, invocation-wise only:
- `start_control_panel` additionally: demo mode
  (`-i /mnt/usb/demo_mode.swf` or `/usr/widgets/demo_mode${BRAND}.swf`), and
  appends `--MaxLocalMemBlocksSlave=384` to `CP_ARGS`.
- `start_opening_anim` uses `-L 0 -Q -3` (adds -3).
- `signal_soft_event.sh` uses `killall -HUP chumbyflashplayer.x` instead of `-F1`.
- zurk's lighty CGIs (`zmote_*`, `chumote/event.cgi`, `custom/*.sh`,
  `event.sh`) inject events via `/tmp/flashplayer.event` + `-F1` — a web
  remote-control built on the same event channel.

Full script-level diff is deferred to Step 1.6.

## Appendix A — chumbyflashplayer.x usage text (extracted via `strings`)

```
usage: %s -i <filename> [
  -m <32K memory blocks (384 = 12mb)>
  -x [X] <pixels (240)>
  -y [X] <pixels (320)>
  -s [X] <stride (x*2)>
  -n [X] <sound buffers {4..32} (16)>
  -b <samples/buffer for master instance (512)>
  -p [X] <sample rate {5000|11025|22050|44100|8000|16000} (44100)>
  -e [X] <channels {1|2} (2)>
  -k [X] <bytes/channel {1|2} (2)>
  -r [X] (rotate additional 90 degrees - may be used multiple times)
  -q [X] <quality {0|1|2} (2)>
  -d name[=value]
  -o <network open debug level {0|1|2} (2)>
  -D [X] (disable screen updates)
  -A [X] (disable audio)
  -u <1K url cache blocks (4096)>
  -a <cache aggression level (5)>
  -S <cache stats write file>
  -I <cache stats write interval (60)>
  -L <message options (3)>
     message options may be a combination of
       1 write to stderr
       2 add timestamp prefix to stderr
       4 write to syslog
  -l <log mask (383)>
     bitmap of values allowed to be displayed
       1=errors, 2=warnings, 4=info, 8=trace, 16=debug,
       32=AS trace, 64=AS debug, 128=realtime debug, 256=video stats
  -M [X] <master_widget>
  -E <cert_list_file>
  -f <frame buffer options {0|1|2} (2)>
  -H <http and malloc options (9)>
     bitmap of options for http:
       1 use curl, 2 report memory usage, 4 verbose memory reporting,
       8 throttle http connection for large transfers,
       16 use localhost as alias for filesystem root (unsafe!)
       64 use verbose mode in libcurl
     additional bits for malloc debugging:
       32 report failed malloc() attempts,
       128 report all chunk allocations, 256 report chunk frees
  -P <preload widget (may repeat)>
  -U (do not load /usr/widgets/preload.swf)
  -T <timing stats file>
  -K <ts quadrant key mapping {0|1} (0)>
  -Y <minimum pump delay in ms (42)>
  -Z <maximum pump delay in ms (83)>
  -z (disregard existing instance)
  -X (turn on mcheck for heap debugging)
  -t <update timing collection run in seconds (0)>
  -R (backtick result display diagnostics)
  -N <asnative & ts debug flags (0)>
  -w <backtick long timeout (10)>
  -W <action when backtick long timeout reached (0=continue, 1=abort backtick, 2=exit flashplayer)>
  -F <signal to send running instance {SIGTERM=15, SIGHUP=1, SIGUSR1=10, SIGUSR2=12)>
     SIGHUP (-F1) triggers processing of events read from /tmp/flashplayer.event
  -g <global heap in mb (0)>
  -G <alternate cfg file or none>
  -j <cache manager options[:cache data dir] (0)>
     bitmap of options:
     1 write cache contents on -S stats file write
  -2 <enable sound hwsync for master instance>
  -3 <do not attempt btplayd start>
  -v (display version information)
  -h (print this message)
] ([X] marks development-only options)
where swf file specified by -i is the main movie to play (master instance)
```
Config-file option names also present in the binary (for `-G` files):
`Define NetOpenDebug HTTPCacheSize CacheStatsFile CacheStatsWriteInterval
MessageOpts LogMask EnableXDomain EnableQuit CertListFile FrameBufferOpts
HTTPMallocOpts PreloadWidget DisablePreload TimingStatsFile
TimingCollectionRunLength TSQuadKeyMapping MinPumpDelay MaxPumpDelay
IgnoreRunningInstance BacktickDebug …`
Note "backtick" options: AS code can apparently execute shell commands via a
backtick mechanism — must be confirmed in Step 1.3 (this is a chumby file://
/exec extension candidate). `-Q` does not appear in the usage text but is
accepted by every launch script; it corresponds to the `EnableQuit` cfg option
(enables `fscommand("quit")`).
