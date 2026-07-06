# 03 — Environment contract of controlpanel.swf 2.8.87b3 (Step 1.3)

Date: 2026-06-12. Source: static scan of the ffdec export under
`appendix/controlpanel-2.8.87b3/` (the backup's downloaded panel — ground
truth). The offline-firmware variant was descoped at CHECKPOINT 1.
Cross-referenced with wiki.chumby.com (ChumbyNative, Sensor Access,
Controlling BTplay). Line references are into
`appendix/controlpanel-2.8.87b3/scripts/frame_2/DoAction.as` (written `F2:n`)
unless a sprite path is given.

**Network classes** (proposed, per working assumptions — final say is the
user's): `mock-forever` = never give this a real backend on the Pi;
`real-network-later` = genuine network feature the user may want, M2 mocks it;
`local-reimpl` = no network involved, Pi needs a local implementation
(or a fixture in M2); `discuss` = needs a user decision.
A `dyn` column will be filled in Step 1.4 (✓ = observed at runtime in Ruffle).

**Dynamic observations (Step 1.4):** rather than editing every row, the rows
confirmed firing at runtime under stock Ruffle are listed in
`04-ruffle-gap-analysis.md` §5 — headline: `exec://guidgen.sh`,
`exec://network_status.sh`, `exec://ap_scan`, `{base}/xml/authorize`,
`music_sources/show`, FM `127.0.0.1:8081/radio/configure`, and the
`firstTime`/`builtin`/`safeMode`/`test` FlashVar gates all behave exactly as
this document predicts.

**Screen attribution** is best-effort from sprite structure; refined in 1.5.
Known sprite→screen anchors: DS612=network wizard, DS708=activation,
DS498=touchscreen calibration, DS686/DS1792=first-time wizard, DS1786=safe
mode/special options, DS1748=info/about, DS1766=night mode.

---

## 0. The six player extensions Ruffle must provide (mechanism level)

Everything below rides on these mechanisms. This is the actual "Ruffle patch
surface"; the per-call tables that follow are routing data for them.

| # | Mechanism | What the player provides | M2 plan |
|---|-----------|--------------------------|---------|
| M1 | **ASnative(5,N) table** (+ stray `ASnative(4,39)`) | ~140 native functions (§1). Registered before frame 1 runs. | Register table behind `chumby` feature; route to `ChumbyHost` |
| M2 | **`exec://` URL scheme** | `XML.load("exec://CMD")` runs CMD in a shell; stdout becomes the loaded document (parsed as XML or consumed raw via `onData`) | Intercept in navigator/loader; `FixtureHost` returns canned stdout keyed by command |
| M3 | **`_backtick(cmd)`** = ASnative(5,52) | Synchronous shell exec, returns stdout as string | Same fixture store as M2 (sync variant) |
| M4 | **`file://` directory listing** | `XML.load("file://DIR")` on a directory returns `<directory><file name=…/><directory name=…/></directory>` XML (used by music file finder + geek file browser). Plain file loads also used (e.g. `/LICENSES/gpl.txt`). Note quirky multi-slash forms: `file:///`, `file:////usr/...`, `////mnt/usb/...` | Implement listing in Rust; trivial on Pi |
| M5 | **FlashVars injection** (`-d name=value`) + player-injected `$version` | §4. Panel branches on `firstTime`/`builtin`/`safeMode`/`test`… | Ruffle already supports setting root vars; expose via CLI/bootstrap |
| M6 | **Master/slave dual-movie system** | Widgets/intro/ads run as a *separate player instance* ("slave") on a second framebuffer/overlay, controlled via natives 72–89, 110–119, 210–211, 330–332, 360–364, 380–387; variables exchanged with `_set/_getSlaveVar`; CP injects the `_chumby_*` var family into the slave (§4b) | Big design topic for `chumby-host.md`. M2 minimum: stub load-status so the panel doesn't hang; real widget playback can come later |

Supporting contracts outside the SWF (already documented in 01): heartbeat
files (`/tmp/movieheartbeat` written **by the panel itself** every 15 s via
`_putFile` — F2:174 — satisfying the cron watchdog; `/tmp/flashheartbeat`
presumably by the player binary), `/tmp/flashplayer.event` + SIGHUP event
injection (the panel even *generates* a script using it, F2:21844),
`fscommand("quit")` honored only with `-Q`.

---

## 1. ASnative(5,N) — full table

Complete inventory (every index bound in F2:1449–1595 or DS498). "Calls" =
static call-site count across all exported scripts (bindings excluded);
0-call wrappers still need to *exist* (the table is built unconditionally at
startup) but can be one shared logging stub in M2.

### 1a. Called by the panel — must behave sensibly in M2

| idx | wrapper | calls | screen/context | purpose / args | M2 stub proposal | class | dyn |
|-----|---------|-------|----------------|----------------|------------------|-------|-----|
| 5,50 | `_getFile(path)` (+`getFile` chomping helper) | 59 | everywhere | read file → string | back with fixtures dir mapped to fake rootfs | local-reimpl | |
| 5,51 | `_putFile(path,data)` (+helper) | 50 | everywhere | write string to file | write into fake rootfs (assert path allowlist §3) | local-reimpl | |
| 5,52 | `_backtick(cmd)` (+chomping helper) | 50 | everywhere | sync shell exec → stdout | fixture per command (§2) | local-reimpl | |
| 5,53 | `_fileExists(path)` | 63+8 | everywhere | 0/1 | fake rootfs | local-reimpl | |
| 5,54 | `_fileSize(path)` | 2 | widget cache | size in bytes | fake rootfs | local-reimpl | |
| 5,55 | `_unlink(path)` | 6 | settings reset, demo mode | delete file | fake rootfs | local-reimpl | |
| 5,320 | `_getDirectoryEntry(obj,path,idx)` | 2 | USB music browser | dir walker, fills obj; ret -1/0/1 | fake rootfs | local-reimpl | |
| 5,10/11 | `_rawX`/`_rawY` | 2+2 | DS498 calibration screen | raw touch coords | return synthetic coords; screen likely *skip* | discuss | |
| 5,12/13 | `_setCalibration`/`_writeCalibration` | 1+1 | DS498 | set/persist calibration | no-op | discuss | |
| 5,16 | `_dcVolts` | 2 | power handling F2:9376 | DC voltage | return nominal (e.g. 12.0) | local-reimpl | |
| 5,20 | `_setLCDMute(state)` | 2 | night mode F2:9026 | LCD on/dim/off | map to Pi backlight later; M2 log | local-reimpl | |
| 5,25 | `_bent` | 2 | F2:3395 (easter egg gate) | squeeze sensor 0/1 | return 0 | mock-forever | |
| 5,40 | `_powerDown(when)` | 3 | power-off button | 1=on-exit 2=now | M2: log/ignore; Pi: systemd poweroff | local-reimpl | |
| 5,42 | `_fscommand2(cmd,var)` | 3 | heartbeat F2:160–167 | GetTotal/FreePlayerMemory → timeline vars | return plausible numbers | local-reimpl | |
| 5,43/44 | `_get/_setTouchClick` | 1+2 | settings | click sound toggle | honor /psp/touchclick fixture | local-reimpl | |
| 5,72 / 5,74 | *(unnamed raw handles)* | 1+1 | `WidgetPlayer.prepareSlaveSettings` F2:4846-49, called `(1)`/`(0)` | undocumented slave settings | no-op accepting any args | local-reimpl | |
| 5,80/81 | `_setSlaveVar`/`_getSlaveVar` | 12+9 | widget player | exchange vars with slave; `_getSlaveVar("_chumby_widget_done")=="true"` ends intro/widgets | **critical**: `_chumby_widget_done` must eventually return "true" or the panel hangs on intro | local-reimpl | |
| 5,82 | `_routeUIEvents(mask)` | 4 | screen manager | input routing master/slave | no-op | local-reimpl | |
| 5,83 | `_setDisplay(i)` | 2 | screen manager | render target main/overlay | no-op (single display M2) | local-reimpl | |
| 5,84–87,89 | `_startSlave`, `_stopSlave`, `_pauseResumeSlave`, `_getDefaultSlaveInstance`, `_getSlaveLoadStatus` | 4,3,4,4,2 | widget player; intro `file:////usr/widgets/intro.swf` F2:5300; builtinclock F2:5333 | slave lifecycle | M2: pretend success, load-status=done, widget_done=true (skip actual widget render) | local-reimpl | |
| 5,100/101 | `_expireCache(Filtered)` | 2+1 | profile switch, intercom | flush player HTTP cache | no-op | local-reimpl | |
| 5,110 | `_setOverlayVisibility(a)` | 7 | screen manager | overlay alpha | no-op M2 | local-reimpl | |
| 5,112/114/116 | overlay blending/chroma setters | 1+5+1 | screen manager | overlay blending | no-op | local-reimpl | |
| 5,118/119 | `_enableMaster/SlaveUpdates` | 5+5 | screen manager | start/stop rendering of each instance | no-op (but never blank the only screen) | local-reimpl | |
| 5,120 | `_exitOpportunity` | 1 | idle | player may restart now | no-op | local-reimpl | |
| 5,131–134 | `_getAudioPlayerState`, `_pause/_resume/_stopAudioPlayer` | 2,1,1,1 | music/alarms | btplay control (state: -1 idle/0 paused/2 playing) | fixture state machine; Pi: real audio backend later | real-network-later (streams) / local-reimpl (control) | |
| 5,140 | `_isAudioPlayerAvailable` | 1 | music | btplay pipe open? | return 1 | local-reimpl | |
| 5,144 | `_playAudio(url,…)` | 1 | music/alarms | play URL/file | log; Pi: mpv/gstreamer backend | real-network-later | |
| 5,146 | `_playAudioLoopCount(n)` | 2 | alarms F2:12276 | loop count | no-op | local-reimpl | |
| 5,160/161 | `_base64En/Decode` | 1+1 | credentials | base64 | implement for real (pure function) | local-reimpl | |
| 5,162 | `_md5Sum(s)` | 2 | GUID hashing F2:1937 | md5 hex | implement for real | local-reimpl | |
| 5,163/164 | `_blowfishEn/Decrypt(s,key)` | 1+1 | mp3tunes creds /psp/mp3tunes | blowfish | implement for real (or stub if mp3tunes skipped) | discuss | |
| 5,176 | `_setSystemTime(secs)` | 2 | clock panel F2:16891 | set system clock | M2 log; Pi: timedatectl | local-reimpl | |
| 5,177/178 | `_get/_setTimeZone` | 1+1 | clock panel | tz get/set | fixture; Pi: zoneinfo | local-reimpl | |
| 5,180–185 | `_get/_setSystemVolume`, `…Balance`, `…Mute` | 2,1,2,1,2,1 | volume panel (WANTED feature) | 0-100 / -100..100 / 0-1 | M2 fixture; Pi: ALSA — **real implementation wanted** | local-reimpl | |
| 5,202 | `_getPlatform()` | 3 | startup F2:3396, widget cache, config | "ironforge"/"falconwing"/… | return "ironforge" (decide: emulate Classic) | discuss | |
| 5,205 | `_getEnvironment(name)` | 2 | startup F2:30568-70 | reads `LANGUAGE`, `CONFIGNAME` | fixture env ("en_US", "ironforge") | local-reimpl | |
| 4,39 | `_batteryPower` | 0 (bound F2:1471) | — | battery level (odd lib index 4) | shared stub | mock-forever | |

### 1b. Bound but never called (shared logging stub is enough for M2)

Grouped; all must *exist* (table built at startup, missing entries only fail
at call time — but several are probed by widgets, not the panel):

| group | indices | note |
|-------|---------|------|
| sensors | 14,15,21–24,26–28,38,39,41,60,61 | brightness get/setters unused because panel writes `/proc/sys/sense1/brightness` & sysfs **directly via _putFile** (F2:9119/9123)! |
| speaker/LCD legacy | 17,18,19 | speaker mute unused; volume done via `chumby_set_*` backticks + 180-185 |
| keyboard/mouse/gamepad/gesture | 90–99, 333 | USB-keyboard support; preload.swf binds them too |
| display/slave extras | 88, 200, 201, 203, 204, 207–211, 220, 300, 301, 330–332, 360–364, 370–373, 380–387, 420 | transform/privilege/mapping APIs |
| audio extras | 130, 135, 141–143, 145, 147–152, 340–342 | playlist mgmt unused by CP (used by music widgets) |
| network timeouts | 172–175 | |
| pipes | 190–195 | `_pipe*` family — wiki-documented, CP never calls |
| misc | 111, 113, 115, 117, 121, 122, 170 | |

Companion SWFs additionally bind: **5,70 `_csccd` / 5,71 `_gsccd`**
(preload.swf — undocumented, likely crypto-processor access), 5,60/5,25/5,16
etc. (preload's own table), `_backtick` in intro.swf
(`enable_intro`/`disable_intro` buttons) and temp_update.swf.

---

## 2. Shell execution (exec:// + backtick) — command catalog

Every distinct command the panel can run. "Response" = what the panel parses.
Class reflects the *feature*, not the mechanism.

### 2a. Wizard / startup / network (DS612 = network wizard; startup = F2 frame 2)

| cmd | via | call site | screen | response expected | class | dyn |
|-----|-----|-----------|--------|-------------------|-------|-----|
| `guidgen.sh` | exec:// onData | F2:204 | startup | GUID text (chomped, uppercased) | local-reimpl (fixture GUID) | |
| `macgen.sh` | exec:// onData | F2:245 | startup | MAC text | local-reimpl | |
| `network_status.sh` | exec:// | F2:286 | startup | `<network><configuration type ssid auth encryption/><interface ip netmask gateway nameserver1 nameserver2>[<error/>]</interface></network>` | local-reimpl (fixture: healthy wired) | |
| `ap_scan` / `/usr/chumby/scripts/ap_scan` | exec:// | DS612 frames 6,7,9 | wifi wizard | `<aps><ap hwaddr ssid auth encryption/>…</aps>` | mock-forever (wifi screens excluded) | |
| `start_network` | exec:// | DS612/frame_25, F2 frame_6:66 | wifi wizard | `<network>…` as above | mock-forever | |
| `restart_network; echo 1` / `ifconfig rausb0 down; rmmod rt73.ko;insmod /drivers/rt73.ko; ifconfig rausb0 up;start_network` | exec:// | F2:9753/9757 | network recovery | raw text | mock-forever | |
| `network_adapter_list.sh` | exec:// | F2:29196 | network wizard | `<network_adapters>…` | mock-forever | |
| `signal_strength` | exec:// + backtick | F2:8110, 27226 | main screen wifi icon | `<wifi connected linkquality signalstrength/>` | local-reimpl (fixture: full bars) | |
| `wget -T 10 -q -O - "http://www.chumby.com/crossdomain.xml" >/dev/null; echo $?` | exec:// onData | F2:9713 | connectivity probe | `0` first char = online | discuss (probe target should not be chumby.com on Pi) | |
| `dcid -o` | backtick | F2:9597 | device identity | DCID XML | mock-forever (registration adjunct) | |
| `chumby_version -h/-s/-f/-n` | backtick ×4 | F2:30551-66 | startup | version strings (fallback: /etc files) | local-reimpl (fixture versions) | |
| `md5sum /tmp/.guidhash` | backtick | F2:1946 | startup (legacy GUID md5) | `<md5>  <file>` | local-reimpl | |
| `killall headphone_mgr; ( headphone_mgr --no-monitor-network ) &` | _backtick | F2:30575 | startup (ironforge only) | — | mock-forever | |

### 2b. Settings / system control

| cmd | via | call site | screen | response | class | dyn |
|-----|-----|-----------|--------|----------|-------|-----|
| `chumby_set_volume [n]` | backtick | F2:9456/9475 | volume (WANTED) | none / current 0-100 | local-reimpl → real ALSA on Pi | |
| `chumby_set_pan [n]` | backtick | F2:9495/9511 | volume | none / -100..100 | local-reimpl | |
| `chumby_set_mute [0/1]` | backtick | F2:9533/9550 | volume | none / 0-1 | local-reimpl | |
| `sync_time_state.sh 0|1` | exec:// | F2:16755/17101 | clock panel | none | local-reimpl | |
| `reload_backup_alarm` / `rm /psp/ifalarm; reload_backup_alarm` | backtick | F2:11981-11994 | alarms | none | discuss (alarms undecided) | |
| `sync` | backtick | DS1786 | demo mode | none | local-reimpl | |
| `rm <file>` | backtick | F2:5263, 13006 | profile/music | none | local-reimpl | |
| `exec:///sbin/reboot` | exec:// | F2:4393, 27367 | geek panel / server-requested | none | local-reimpl | |
| `reboot_normal.sh`, `restore_factory_defaults.sh` | exec:// | DS1786 | safe mode | none | local-reimpl/discuss | |
| `start_sshd.sh` | exec:// | F2:27357 | geek panel | none | local-reimpl | |
| `fb_cgi.sh` | backtick | F2:27400 | geek panel | none | discuss | |

### 2c. Updates (likely *skip* on Pi — discuss)

`update_network.sh`, `update_usb.sh` (DS1786), `update.sh update1 REPAIR`
(F2:27385), `perl -e '… update_launch.sh <file>'` (F2:27389),
`update_now.sh` (F2:28543) — all fire-and-forget. Class: **mock-forever**
(chumby-specific firmware update machinery).

### 2d. Music service control

| cmd | via | call site | response | class | dyn |
|-----|-----|-----------|----------|-------|-----|
| `service_control inetradio start --output=… <url>` / `stop` | backtick | F2:12282/12293 | none (legacy stream path) | real-network-later | |
| `kill -USR1/-USR2 <btplay pid>` (pid from `/var/run/btplay.pid`) | backtick | F2:12306/12319 | none (pause/resume) | local-reimpl | |
| `service_control chumbipodd eject/status` | backtick | F2:13391/13403 | `<status><pids count="N"><pid running="1"/></pids></status>` | discuss (iPod) | |
| `service_control chumbradiod status` | backtick | F2:14318 | same status XML | discuss (FM radio hw) | |
| `curl '<url>' > <path>; echo $?` + `md5sum <path>` + `head -c 3 <path>` + `mkdir <cache>` | AsynchronousCommand/backtick | F2:30374/30333/30351/30203 | exit status / md5 / SWF magic | widget cache — local-reimpl | |

### 2e. Intercom (chumby-to-chumby; pre-marked **skip** candidate — social-ish)

mDNSResponder spawn, `mDNSPublish` (×2 forms), `arecord | nc <peer>:1337`,
name to `:1338`, `nc -ll -p 1337 -e aplay &`, several `kill … grep …` lines,
generated `/tmp/intercomnamed.sh` (writes `/tmp/flashplayer.event` + `-F1`!),
`service_control chumbhowld restart` (F2:21540–21850). Class: **discuss**
(suggest skip).

### 2f. Alarm easter egg

`/mnt/usb/post_alarm_action[_N]` executed via exec:// after alarm (F2:11175).
Class: discuss (alarms undecided).

---

## 3. Filesystem contract (complete path list)

The panel assumes this filesystem. M2's fake rootfs fixture tree should
provide the ★ paths at minimum (read on the boot path).

**/psp (persistent, survives reboot):** ★`firsttime` (W"0" ends wizard),
★`clock_format`, ★`touchclick`, ★`dimlevel` ("2"⇒boot into night mode),
`nightmode_brightness`, `daymode_brightness`, ★`nooverlay` ("1"⇒single
framebuffer — **set this on Pi/M2!**), `no_translation`, `translation.xml`,
`control_panel_translation.xml`, `widget_shuffle`, `widget_stats_time`,
`music_stats_time`, `ad_stats_time`, `alarm_volume`, `alarm_fade_duration`,
★`alarms` (XML, default written if missing), `ifalarm`, `music_order`,
`music_timer_duration`, `shoutcast_search`, `mp3files_order`,
`usb_max_files`, `fmradiostation`, `fmradioband`, `fmradio_presets`,
★`timezone_city` ("City\tCountry"), ★`use_ntp`, `mp3tunes` (blowfish),
`url_streams`, `npr_sources.xml`, `slimserver_ip`, `enable_intercom`,
★`hostname`, `network_configs`, `network_config`, `demo_mode`,
`profile.xml`, `hotspot` (E F2:377).

**/tmp (volatile):** ★`movieheartbeat` (panel W"1"/15 s), `nightmode`,
`.guidhash`, `channel_names`, `widget_names`, `currentProfileID`,
`currentProfileName`, `change_profile` (external profile-switch request,
polled), `controlpanelversion` (panel writes its version), `musicsource`,
`intercomnamed.sh`, `hidden_ssid`, `profile.xml`, `translation.xml`,
`widgetcache/` (ironforge w/o USB).

**Device/system files:** `/proc/sys/sense1/brightness` (W — brightness×655.35,
hw 3.8), `/sys/devices/platform/stmp3xxx-bl/.../brightness` (falconwing),
`/sys/class/power_supply/battery/{capacity,present,status}` (falconwing),
`/var/run/btplay.pid` (R), `/var/run/btplay.properties` (R),
`/etc/hardware_version`, `/etc/software_version`, `/etc/firmware_build`
(version fallbacks), `/LICENSES/gpl.txt`, `/LICENSES/lgpl.txt`
(file:// XML.load), `/usr/chumby/alarmtones/<name>.mp3`,
`/usr/widgets/intro.swf`, `/usr/widgets/builtinclock.swf` (slave-loaded).

**/mnt (USB/storage):** `usb`,`usb2..4`,`storage` existence;
`usb/profile.xml`, `usb/translation.xml`, `usb/alarmring.swf`,
`usb/alarm[N|<day>].mp3`, `usb/post_alarm_action[_N]`, `usb/update1`,
`usb/update.{fw,tgz,zip}` (+ `<CONFIGNAME>-update.*`), `usb/widgetcache/`,
`usb/externalmusic.xml` (**can direct arbitrary getFile/putFile via its XML**
— security note for Pi), `storage/widgetcache/` (+ `widgetcache.xml` config,
`<md5-of-guid>` entries).

Class: all local-reimpl (fixture tree in M2; real paths on Pi —
brightness/battery paths must be remapped, flagged `discuss` for hardware
mapping).

---

## 4. FlashVars, injected variables, capabilities

### 4a. Inbound (read by panel, set from outside)

| variable | set by | read at | gates | class |
|----------|--------|---------|-------|-------|
| `firstTime` | `-dfirstTime=1` (launcher) | F2:338 dispatch; DS1792 | out-of-box wizard frame | fixture | |
| `builtin` | `-dbuiltin=1` | F2:334/357; DS612/frame_7:117 | "builtin" offline clock+alarm frame; skip-button in wizard | fixture | |
| `safeMode`/`safemode` | (update flow) | F2:330/357, 8973 | safe-mode frame (DS1786) | fixture | |
| `alternate` | `-dalternate=1` | *(no read found in 2.8.87b3 — launcher legacy?)* | — | n/a | |
| `test` | `-dtest=1`; **auto-true when `$version` says MAC/WIN!** (F2:402-406) | 73 sites | desktop test mode: local XML fixtures instead of exec://, stub versions, no slave player | **decide: Ruffle must NOT report WIN/MAC, or set test deliberately — see note below** | |
| `localCache` | `-dlocalCache=1` | 10 sites | widgets via loadMovie instead of slave | useful for M2! | |
| `baseURL` | `-dbaseURL=…` | F2:30585 | overrides `http://xml.chumby.com` | **key M2 lever** | |
| `widgetsURL` | `-dwidgetsURL=…` | F2:30603 | overrides `http://widgets.chumby.com` | key M2 lever | |
| `adurl` | `-dadurl=…` | F2:29566 | ad manifest test URL | | |
| `forceUpdate` | `-dforceUpdate=1` | F2:4098 | force update screen | | |
| `alarmReloadInterval` | -d… | F2:11784 | alarm reload minutes | | |
| `_chumby_widget_test` | -d… | F2:4983/4686 | passed through to widgets | | |
| `wma` | dead — overwritten `false` F2:30533 | 26109/26296 | (nothing) | | |
| `__fileInfo` | internal/test | F2:1833 | FileFinder dir-walk object | | |
| `$version` | player | F2:404, 30530 | test-mode autodetect | see `test` | |
| `System.capabilities.screenResolutionX/Y` | player | F2:4684/4962 | forwarded to widgets | report real resolution | |

> **Important interplay:** the panel hardcodes `ScreenDimensions
> {WIDTH:320, HEIGHT:240}` (F2:2278) for its own layout, but forwards
> `screenResolutionX/Y` to widgets. And `_root.test` auto-enables on
> desktop-player version strings — under stock Ruffle the panel may already
> run in test mode (check in 1.4; possibly convenient, possibly misleading).

### 4b. Outbound — the `_chumby_*` contract the panel provides to widgets

Injected into slave SWFs via `_startSlave(url, initObj)` / `_setSlaveVar`:
`_chumby_widget_instance_id/index/count/href`, `_chumby_widget_id/name/state`,
`_chumby_user_name/user_id`, `_chumby_chumby_name/chumby_id`,
`_chumby_clock_format`, `_chumby_software/hardware/firmware_version`,
`_chumby_screen_width/height`, `_chumby_has_browser`, `_chumby_ad_*`
(+`_chumby_ad_email_href`), `_chumby_music_source_*`, `_chumby_param_…`,
`_chumby_profile_id/name`, `_chumby_system_volume/mute/balance`,
`_chumby_alarm`, `_chumby_timer_expires`. Read back: `_chumby_widget_done`,
`_chumby_controlpanel_event` (widget→CP event channel, F2:5179).
Also `CHUMBY_WIDGET_TEST`. (intro.swf reads `_chumby_widget_instance_id`,
sets `_chumby_widget_done` — confirms protocol.)

### 4c. fscommand

`fscommand("quit")` — DS1748/frame_2:33 (about-panel quit button),
DS1792/frame_10:2 (end of firsttime/builtin flow). Only honored with `-Q`.
M2: map to window close / controlled exit.

---

## 5. Network endpoints (HTTP)

`makeURL` prefixes `baseURL` (default `http://xml.chumby.com`) to
`/`-prefixed paths; `makeWidgetsURL` likewise (`http://widgets.chumby.com`).
**The `/xml/chumbies` response can rewrite both bases at runtime**
(attrs `baseurl`/`widgetsurl`, F2:4186-95) — zurk's firmware exploits exactly
this. Recurring params: `id`(GUID) `hw` `sw` `fw` `config`(platform)
`nocache`(timer) `<dcid…>` `ssi=Signature.stamp()`.

### 5a. Core boot path (panel blocks/branches on these)

| endpoint | mech | site | screen | response | class | dyn |
|----------|------|------|--------|----------|-------|-----|
| `{base}/xml/chumbies/?id=…` | XML.load | F2:4172 | startup | `<chumby>` w/ user/profile/name; may rewrite baseurl/widgetsurl | mock-forever (registration) | |
| `{base}/xml/authorize?hw=…` | XML.load 5 s poll | F2:29106; DS708/frame_3 | activation screen | `<chumby><name>…` once activated | mock-forever | |
| `{base}/xml/registerchumby?id=…&hash=…` | XML.load | DS708/frame_2 | activation | ack | mock-forever | |
| `{base}/xml/profiles?id=…` | XML.load | F2:4305 | widget channel load | `<profile>` w/ `<widget_instance>` list | discuss (needed if widgets wanted; can be static fixture) | |
| `{base}/xml/setprofile?profile_id=…` | XML.load | F2:3869 | channel switch | new `<profile>` | discuss | |
| `http://update.chumby.com/update?hw=…` | XML.load | F2:4076; DS1786/frame_2:76 | boot + safe mode | `<update><update1/>…` → triggers update.sh | mock-forever (return "no update") | |

### 5b. Widget/channel management (social-adjacent; many pre-marked skip)

| endpoint | site | purpose | class |
|----------|------|---------|-------|
| `{base}/xml/profileinfo/<id>` | DS1627/frame_4 | channel picker | discuss |
| `{base}/xml/categories` | F2:28038 | add-widget categories | discuss |
| `{base}/xml/ratewidgetinstance?…` | F2:28867 | widget rating | mock-forever (social) |
| `{base}/xml/accept/decline/removewidgetinstance/<id>` | DS1611/DS1609/DS1638 | sent-widget mgmt / delete | mock-forever (social) except remove=discuss |
| `{base}/xml/sendwidgetinstance?…` | DS1535/frame_3 | send to friend | mock-forever (EXCLUDED social) |
| `{base}+<instance href>` POST | F2:3824 | save widget params | discuss |
| `{widgets}+<movie href>` | F2:30297→`_startSlave` | widget SWF fetch | discuss |
| `{widgets}+<thumbnail href>` | F2:4611 etc. | thumbnails | discuss |

### 5c. Stats & ads (fire-and-forget POSTs / manifest)

`{xml}/duas/widgets`, `/duas/music`, `/duas/ads` (POST, ignored) — class
**mock-forever** (drop silently). Ad system: `manifest/show` + AdManifest
query, `files.chumby.com/ads/*` (image_ad.swf, manifest.xml, thumbnail) —
**mock-forever** (return empty manifest; nobody wants ads).

### 5d. Music sources (the big *discuss* cluster — most need a live directory service that no longer exists in original form)

| source | endpoints | playback | class |
|--------|-----------|----------|-------|
| SHOUTcast | `shoutcast.chumby.com/shoutcast/{list,search/<q>,show?id=}` → PLS | `BTPlayer.start(url)` (btplayd fetches stream) | real-network-later (user wants internet radio; needs replacement directory) |
| Chumbcast / Sleep sounds | `bor.chumby.com/chumcast/{list,show}` (+`type=AmbientStation`) | BTPlayer / loadClip sound SWF | discuss |
| CBS podcasts | `podcast.chumby.com/podcast/cbs/list` → third-party RSS → enclosure | BTPlayer | discuss |
| External music mfst | `music.chumby.com/music_sources/show/?…` → per-source player/panel/alarm SWFs (loadMovie!) | loaded SWFs (Pandora etc.) | discuss |
| My Streams | user URLs from `/psp/url_streams` | BTPlayer | real-network-later (simple, user-owned) |
| MP3tunes | `shop.mp3tunes.com/api/v1/{login,accountData}`, `ws.mp3tunes.com/api/v1/lockerData` (plaintext creds in URL) | BTPlayer | mock-forever (service dead) |
| NOAA weather radio | `wunderground.com/wxradio/requestxml.html?action=getallstations` | BTPlayer | discuss (endpoint likely dead) |
| Internode | `files.chumby.com/internode/stations.xml`, `media.on.net/radio/<id>.pls` | BTPlayer | discuss |
| NPR | local `nprstations.xml` + npr.org RSS ids | BTPlayer | discuss |
| iPod (chumbipodd) | `127.0.0.1:8080/{info,playlists,playlistIDs/<id>,track/<id>}` | BTPlayer | discuss (needs local daemon) |
| FM radio (chumbradiod) | `127.0.0.1:8081/radio/configure?…` | hardware tuner | mock-forever (no such hw on Pi) — unless USB tuner discuss |
| SlimServer | `<user-ip>:9000/stream.mp3` | BTPlayer | discuss |
| Intercom | `localhost:8082/_http._tcp` (mDNS gw) + nc/arecord (§2e) | native | discuss (suggest skip) |

### 5e. Misc

`System.security.allowDomain("*")` (F2:30532); `LocalConnection`
`_cp_connection` allows movies.chumby.com / s3.chumby.com / localhost
(F2:3493-3500) — widget→CP API channel. No `loadPolicyFile`; player ignores
crossdomain for its own loads (Ruffle: disable policy checks for chumby
feature). insignia.chumby.com appears in activation copy text only.

---

## 6. Persistence formats

No SharedObject / AMF anywhere — **all persistence is plain files** under
/psp (§3): mostly single-line values; XML for `alarms`, `network_config(s)`,
`url_streams`, `npr_sources.xml`, `translation.xml`, `musicsource`,
`profile.xml`, `widgetcache.xml`. Fixtures stay trivial (no AMF0 needed —
plan's AMF concern is moot).

---

## 7. Priority summary for M2 (proposed)

Must work for the main screen to appear with `-dbuiltin=1` (offline path,
per DS1792/builtin flow): M1 native table (logging defaults), file natives
over a fixture rootfs with ★ paths, backtick/exec fixtures for §2a rows
marked local-reimpl (guidgen/macgen/network_status/signal_strength/
chumby_version), `_getSlaveVar("_chumby_widget_done")→"true"`,
slave-lifecycle no-ops, `/psp/nooverlay=1`, and `-dbaseURL` pointed at a
fixture HTTP responder (or interception) returning: empty update, minimal
chumbies/profiles XML. Everything in 5b/5c/5d can return empty/error
fixtures initially.

Open questions for the user (carried to CHECKPOINT 2): platform string
choice (ironforge?), `test`-mode handling under desktop player, alarms
in/out of scope, which music sources to keep, intercom skip, update
machinery skip, touchscreen-calibration screen skip.
