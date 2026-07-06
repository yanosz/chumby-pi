# 05 — Screen / frame catalog of controlpanel 2.8.87b3 (Step 1.5)

Date: 2026-06-12. Sources: ffdec exports (`appendix/controlpanel-2.8.87b3/`:
scripts, tag-dump.txt frame labels, sprite renders), the four Step 1.4 live
runs, and the Step 1.3 contract. Sprite renders referenced as
`sprites/DefineSprite_N/<frame>.png` (relative to the appendix dir); live
screenshots in `images/`.

**Status column = PROPOSAL, not decision.** Per project rule 5, every status
is a question to the user; answers go to `docs/feature-decisions.md` at
CHECKPOINT 2. Pre-applied from standing decisions: social features → `skip`;
hardware-config screens → `skip` EXCEPT volume & brightness → `needed`.
Even `skip` screens stay cataloged — their init code may run and must be
understood to be hopped over.

## Architecture in one paragraph

The main timeline has 10 frames: 2 `startup` (dispatcher + all class
definitions), 3 `firsttime`, 4 `validate`, 5 `register`, 6 **`main`**,
7 `playtrap`, 8 `safemode`, 9 (builtin date check), 10 `builtin`.
Frame 6 hosts two sprites: `widgetPlayer` (DS709, plays widgets) and
`controlPanel` (**DS1766**) — the master panel whose 13 labeled frames are
the screen registry: `idle, main, clock, music, send, rate, channel,
deleteWidget, settings, confirmUpdate, nightMode, alarms, tryNightMode`
(verified against tag dump). Navigation = `controlPanel.gotoAndStop(label)`;
each panel's `done()` returns to `main`/`idle`. Sub-panels are sprites with
their own labeled frames (music: DS1524, 20 frames; settings: DS1748,
13 frames; channel: DS1627, 12 frames).

## A. Boot & provisioning screens

| # | Screen | Sprite/frame | Visual | Elements | Contract deps (03 §) | Proposed status |
|---|--------|--------------|--------|----------|----------------------|-----------------|
| A1 | Startup "Initializing…" | main f2 `startup` | spinner | label + pinwheel | exec: guidgen/macgen/network_status; files: firsttime, clock_format, dimlevel, nightmode, hotspot; heartbeat | **needed** (it's the boot path) |
| A2 | First-time wizard | f3 `firsttime`; DS1792 (10 fr), calib DS686→498, date/time DS1790, network DS612 | images/04-firsttime-calibration.png | calibration → network → timezone → date/time steps; ends `fscommand("quit")` | putFile /psp/firsttime; delegates to A8/E2 | **skip?** (Pi has no touchscreen-calibration need; wizard only runs with `-dfirstTime=1` which we control) |
| A3 | Authorizing | f4 `validate` (sub: `authorize`, `timedout`) | images/04-validate-authorizing.png | spinner; timeout: retry/skip buttons | HTTP /xml/authorize (5 s poll) | **needed-but-bypassed?** (mock authorize success or enter via builtin; must not hang) |
| A4 | Activation / registration | f5 `register`; DS708 (`gotosite→dogrid→dopolling→dosuccess`) | sprites/DefineSprite_708/1.png | instructions, GUID display, grid-code entry, NEXT/LATER | HTTP registerchumby + authorize poll; unlinks stale /psp music creds | **skip** (chumby.com registration — mock as "already activated") |
| A5 | Play-trap error | f7 `playtrap` | — | error text, OK→main | none | needed (trivial; it's a guard) |
| A6 | Safe mode / special options | f8 `safemode`; DS1786 | sprites/DefineSprite_1786/1.png | Restart / Install updates / Restore factory / Demo mode; install-source + confirm sub-screens | exec: reboot_normal, update_network, update_usb, restore_factory_defaults; update.chumby.com; /mnt/usb update probes | **skip?** (chumby firmware machinery; Pi has its own update story) |
| A7 | Builtin offline clock | f9 (date check) + f10 `builtin`; DS381 bi_clock, DS1790 set date/time | — | analog clock; date/time picker if RTC unset; touch → retry network | none (local time) | **needed** (graceful offline mode; also our entry trick for M2) |
| A8 | Network setup wizard | DS612 (~28 labels: scan/choose AP, hidden SSID, WEP/WPA key entry w/ on-screen keyboard, DHCP/static, commit) | images/04-builtin-wifi-wizard.png, sprites/DefineSprite_612/7.png | AP list, keyboard, IP entry, SKIP (builtin only) | exec: ap_scan, start_network; putFile network_config(s), hidden_ssid | **skip** (excluded hw-config; Pi configures network at OS level; M2 must make panel believe network is up — wizard never appears) |

## B. Main control panel (DS1766)

| # | Screen | Sprite/frame | Visual | Elements | Contract deps | Proposed status |
|---|--------|--------------|--------|----------|----------------|-----------------|
| B1 | Idle / widget mode | 1766 `idle` | — | corner hotspot (forceControlPanel), bend sensor summon | getFile /tmp/nightmode | **needed** (default state) |
| B2 | Main button bar | 1766 `main`; DS784 | sprites/DefineSprite_784/1.png | prev/next widget, Clock, Music, Alarms, Night, Send, Rate, Channel, Delete, Stay, Settings, Mute; volume slider; channel/widget name; thumbnail; Activate (unregistered only) | thumbnail loadMovie; channel gated on hasNetwork | **needed** (THE control panel) |
| B3 | Clock panel | 1766 `clock`; DS1145 (main/setTime/setDate/timezone-map/alarms…) | — | time/date display+pickers, 12/24 toggle, zoomable timezone map | exec sync_time_state; /psp clock_format, timezone_city; _setSystemTime, _set/getTimeZone | **needed?** (time/date/tz on Pi — sensible to keep) |
| B4 | Night mode | 1766 `nightMode`; DS1758 (+`tryNightMode` DS1161 prompt) | sprites/DefineSprite_1766/1.png (dark) | big clock, next-alarm str, music-timer str, dark toggle, SKIP-alarm; touch/bend exits | /tmp/nightmode, /psp dimlevel + brightnesses; _setLCDMute or sysfs brightness | **needed** (brightness family is wanted) |
| B5 | Alarms panel | 1766 `alarms`; DS1765 (`mainPanel`/`setAlarmsPanel` DS1119/`setSimpleAlarmPanel` DS1144) | — | alarm list, day/time picker, sound source picker (beep/continuous/music), volume, snooze/stop, skip-next | /psp/alarms XML, alarm_volume, ifalarm + reload_backup_alarm backtick | **needed** (decided 2026-06-12, feature-decisions.md) |
| B6 | Alarm ring overlay | DS228 alarmRing in DS719 (always present, depth 364) | — | live clock, alarm name, OFF, SNOOZE (bend=snooze) | ScreenManager full-bright; plays source | follows B5 |
| B7 | Rate widget | 1766 `rate`; DS1555 | — | widget name/icon, 5 stars, done/cancel | HTTP ratewidgetinstance | **skip** (chumby.com social rating) |
| B8 | Delete widget | 1766 `deleteWidget`; DS1638 (`delete`/`cantdelete`) | — | thumbnail, yes/no | server instance removal | **undecided** (depends on channel mgmt scope) |
| B9 | Update confirm | 1766 `confirmUpdate`; DS1752 | — | "Update available", NOW/LATER | exec update_launch.sh | **skip** (firmware updates) |
| B10 | Send widget flow | 1766 `send`; DS1535 (send/sendBuddies/cantsend/nobuddies) | — | buddy multi-select list | HTTP sendwidgetinstance per buddy | **skip** (pre-decided: social) |

## C. Music panel (DS1524, 20 labeled frames)

Hub: **C0 source menu** (`main`, DS1154): source list, GO TO, stop/resume,
sleep-timer button (`setTimer`, DS1161), volume slider, now-playing.
Source registry is hardcoded (F2:12798) + ExternalMusicSources prepends
downloaded sources (Pandora et al. — panel SWFs fetched at runtime).
Status of C0 itself: **needed if any music source is kept**.

| # | Source screen | Sprite (frame label) | External deps | Proposed status |
|---|---------------|----------------------|---------------|-----------------|
| C1 | SHOUTcast Radio | DS1216 (`shoutcast`) | shoutcast.chumby.com directory → btplay | **undecided** — user wants internet radio; needs replacement directory |
| C2 | My Streams (user URLs) | DS1323 (`directurl`) | /psp/url_streams + btplay | **needed?** — simplest real internet-radio path |
| C3 | iPod | DS1206 (`ipod`) | chumbipodd daemon, 127.0.0.1:8080 | **skip?** (hardware/daemon unlikely on Pi) |
| C4 | FM Radio (+legacy `fmradio_old`) | DS1490, DS1262 | chumbradiod, 127.0.0.1:8081, tuner hw | **skip** (no tuner) |
| C5 | MP3tunes | DS1296 (`mp3tunes`) | mp3tunes.com API (dead service) | **skip** |
| C6 | NOAA Weather Radio | DS1361 (`noaa`) | wunderground.com directory | **skip?** (service likely dead) |
| C7 | Internode Radio | DS1369 (`internode`) | files.chumby.com + media.on.net | **skip?** |
| C8 | NPR / NYT / CBS / Mediafly podcasts | DS1523/1350/1358/1266 | RSS feeds (3rd party, partly dead) | **undecided** (podcasts generally?) |
| C9 | blue octy radio (Chumbcast) | DS1299 (`chumbcast`) | bor.chumby.com | **skip?** |
| C10 | Sleep Sounds | DS1372 (`sleepcast`) | bor.chumby.com (AmbientStation) | **undecided** (nice with alarms/night mode) |
| C11 | My Music Files (USB) | DS1520 (`mp3files`, +file browser DS154) | /mnt/usb scan via _getDirectoryEntry + btplay | **undecided** (local files on Pi could be nice) |
| C12 | Squeezebox/SlimServer | DS1233 (`slimserver`) | LAN server :9000 | **undecided** |
| C13 | External sources (Pandora…) | DS1363 (`external`) | music.chumby.com manifest + downloaded panel SWFs | **skip?** (depends on dead chumby.com manifest; Pandora API also changed) |

## D. Channel & widget management (DS1627)

| # | Screen | Frame/sprite | External deps | Proposed status |
|---|--------|--------------|---------------|-----------------|
| D1 | Channel hub | `main` | — | undecided (gateway for D2-D6) |
| D2 | Channel picker | `changeChannels` DS1577 | profiles XML, setprofile | **undecided** — depends on whether widget channels come from fixtures |
| D3 | Channel info | `profileInfo` DS1581 | profileinfo | undecided |
| D4 | Widget list / info | `widgetInfo` DS1593 | profileinfo XML | undecided |
| D5 | Add widget (categories→widgets) | `addWidget` DS1626 | categories + catalog XML | **undecided** (needs a server/fixture catalog) |
| D6 | Accept/decline sent widgets | `acceptWidgets` DS1604 / `declineWidget` DS1609 / `accepting` DS1611 | accept/decline endpoints | **skip** (social) |
| D7 | Reload channel | `reload` | profile re-fetch | follows D2 |
| — | Widget player (DS709) | not a screen | instance XML, _startSlave/loadMovie, thumbnails | **needed** (core widget playback; M2 decides slave-vs-localCache) |

## E. Settings (DS1748, 13 frames)

| # | Screen | Frame/sprite | Elements | External deps | Proposed status |
|---|--------|--------------|----------|---------------|-----------------|
| E0 | Settings menu | `main` | icons: Clock, Brightness, Network, Touchscreen, Volume, Info | — | **needed** (entry for the wanted items; unwanted icons: hide? → user) |
| E1 | Volume | `volume` DS1686 (VolumePanel) | slider, touch-click toggle | chumby_set_volume/pan/mute backticks, 180-185 natives, /psp/touchclick | **needed** (explicitly wanted) |
| E2 | Brightness | `brightness` DS1691 / `altBrightness` DS1747 (hw≥3.8) | level list / day+night sliders | /psp brightnesses, sysfs/proc backlight writes | **needed** (explicitly wanted) |
| E3 | Network | `network` → embeds DS612 | see A8 | see A8 | **skip** (pre-decided) |
| E4 | Touchscreen calibration | `touchscreen` → DS498 | green-star 2-point calibration | ASnative 5,10-13 | **skip** (pre-decided) |
| E5 | Clock | `clock` DS1736 (ClockOnlyPanel) | same as B3 | same | needed (= B3) |
| E6 | Info / About | `info` DS1665 (InfoPanel) | GUID, versions, owner, network info; buttons: Licenses, Geek, intro | backtick signal_strength | **needed?** (harmless, useful diagnostics) |
| E7 | Licenses | `licenses` DS1702 | GPL/LGPL viewer | file:// /LICENSES/*.txt | needed (trivial, legally nice) |
| E8 | Geek mode | `geek` DS1727 | SSHD, Files, Reboot, Repair, Power off, fb_cgi, Clear cache | exec start_sshd, reboot, update repair; _powerDown | **undecided** (power off/reboot useful on Pi; rest questionable) |
| E9 | File browser | `browser` DS1694+DS154 | dir listing | file:// directory XML | follows E8 |
| E10 | Microphone test | `microphone` DS1692/3 | mic level test | native audio | **skip?** |
| E11 | Intercom | `intercom` (geek-gated) | record/send | §2e intercom stack | **skip** (suggested earlier; social-ish) |
| E12 | Timezone picker | DS208 map (used by B3/E5/A2) | zoomable world map | /psp/timezone_city | needed (= B3) |

## F. Cross-cutting observations for M2

1. The **only screens on the forced boot path** are A1→(A3|A7) — everything
   else is user-navigated from B2. Making A1 complete and dispatch correctly
   is the whole battle for "main screen visible".
2. `skip` decisions mostly mean **hiding/never-navigating-to** a DS1766/
   DS1748/DS1524 frame, not patching it out — the frames are inert until
   `gotoAndStop`. Exceptions whose init runs regardless: alarm watcher DS719
   (always at depth 364), widgetPlayer DS709, NightMode restore-on-boot,
   ExternalMusicSources.load() and all music player constructors (run at
   startup — confirmed in run 8 trace; they tolerate failure).
3. The music panel's source list is built from `MusicPlayer.musicSources`
   minus sources whose probe fails — several `skip` decisions can be
   implemented by making probes fail cleanly (e.g. chumbipodd status →
   not running) rather than touching the UI.
4. Activation state (A4) gates the Activate button on B2 and nothing else —
   fixture "already authorized" makes the whole flow invisible.

## G. Questions for the user (CHECKPOINT 2, consolidated)

1. Alarms (B5/B6): in or out? (Affects C10 sleep sounds, night-mode alarm display.)
2. Which music sources survive: my-streams (C2)? USB/local files (C11)? SHOUTcast-style directory with a replacement backend (C1)? Podcasts (C8)? Squeezebox (C12)? Sleep sounds (C10)? — everything else proposed `skip`.
3. Widget channels (D1-D5, D7): static fixture channel only, or a real channel-management story (needs a profile/catalog fixture server)?
4. Widget playback architecture: implement the master/slave dual-instance system, or use the panel's own `localCache` in-movie path (run 8 evidence: works)?
5. Geek panel (E8): keep Reboot/Power-off only? Whole panel?
6. Info panel (E6) + Licenses (E7): keep as proposed?
7. Confirm the pre-applied skips: A2 first-time wizard, A4 activation, A6 safe mode, A8/E3 network wizard, E4 touchscreen calib, B7 rate, B10 send, D6 accept/decline, E10 microphone, E11 intercom, B9/firmware updates, C3-C5 iPod/FM/MP3tunes.
8. Platform string: keep reporting `ironforge` (hw 3.8 → BrightnessPanelAlt with day/night sliders) — OK?
