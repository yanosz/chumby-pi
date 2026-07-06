# 02 — External scripts & daemons of the control-panel ecosystem (Step 1.1)

Date: 2026-06-12. One-line purposes from script headers/contents; no decompiling
done. ★ marks scripts directly relevant to the control-panel runtime contract
(launch, supervise, or be called *by* the panel — the latter to be confirmed in
Step 1.3 when we see which scripts the SWF invokes).

## A. Backup: `/home/jan/chumby_backup/usr/chumby/scripts/` (62 entries, ironforge fw 1.7.2 — ground truth)

| Script | Purpose |
|--------|---------|
| `add.sh` | mountmon "add" event hook (device plugged in) |
| `ap_scan` ★ | scan for wireless networks, output XML (panel's WiFi setup screen) |
| `check_for_update.sh` | check `/etc/firmware_url` for firmware updates |
| `chumby_set_mute` ★ | wrapper: `/usr/bin/chumby_set_volume -master_mute $@` |
| `chumby_set_pan` ★ | wrapper: `chumby_set_volume -master_panning` |
| `chumby_set_volume` ★ | wrapper: `chumby_set_volume -master_volume` (binary in /usr/bin) |
| `chumby_version` ★ | print software/firmware/hardware versions (used by download_cp) |
| `cpi.sh` ★ | interface to the `cpi` command line (Control Panel Interface?) — persistent; inspect in 1.3 |
| `disable_intro` / `enable_intro` | touch/rm `/psp/disable_intro` |
| `download_cp` ★ | download control panel from chumby.com (see 01-invocations §6) |
| `fb_cgi.sh` | expose framebuffer screenshot via HTTP CGI (imgtool capture) |
| `flashplayer_is_running` ★ | health check via pid file + /proc state |
| `flashplayer_watchdog` ★ | cron watchdog: restart panel on stale heartbeats (30 s) |
| `get_voltage.sh` | report `/proc/chumby/battery-voltage` raw values |
| `guidgen.sh` ★ | extract device GUID from crypto processor (download_cp, registration) |
| `log-rollover`, `log-rollover-enable` | rotate /var/log/messages; enable via crontab |
| `macgen.sh` | print wireless NIC MAC address |
| `mkdevs` | populate /dev (boot plumbing) |
| `mountmon_utils.sh`, `mount.sh`, `umount.sh`, `umount_repeated.sh`, `remove.sh` | mountmon helpers for USB storage events |
| `network_adapter_list.sh` | enumerate network adapters |
| `network_interface` | resolve which interface to use (respects /psp overrides) |
| `network_running.sh` | is the network up? |
| `network_status.sh` ★ | poll network status → XML at `/tmp/chumby/network_status.xml` (start_control_panel gates on its "error" output) |
| `prepare_updates.sh` | stage firmware updates in /mnt/cache & /mnt/storage |
| `reboot_normal.sh` | set paranoid-boot flag and reboot |
| `reload_backup_alarm` ★ | `kill -HUP chumbalarmd` (panel alarm changes → daemon reload) |
| `restore_time` / `save_time` ★ | restore/store clock via crypto processor uptime (panel clock) |
| `script_locations.sh` | set default script dir locations |
| `select_update.sh` | offer firmware updates via `temp_update.swf` UI |
| `service_control`, `service_getpid`, `service_list` | generic service start/stop wrappers |
| `signal_soft_event.sh` ★ | append XML event to `/tmp/flashplayer.event`, signal player with `-F1` |
| `signal_strength` ★ | output wifi signal strength XML (panel status display) |
| `start_control_panel` ★ | main panel launcher (full logic in 01-invocations §4.1) |
| `start_intro` ★ | play intro.swf unless disabled |
| `start_network`, `start_network.sh` | bring up network from /psp/network_config |
| `start_opening_anim` ★ | play opening.swf at boot (nice -20) |
| `start_sshd.sh` | start sshd service |
| `stop_control_panel` ★ | kill player (`-F15`), launcher, btplayd; zombie cleanup |
| `stop_dhcp` | kill udhcpc instances |
| `stop_getty` | stop getty on /dev/ttyUSB0 |
| `switch_fb.sh` ★ | switch visible framebuffer 0/1 (player uses dual fb) |
| `sync_time.sh` ★ | NTP/rdate time sync (cron 03:00; panel time settings) |
| `sync_time_state.sh` ★ | get/set `/psp/use_ntp` (panel toggle) |
| `time_zone.sh` ★ | get/set timezone → `/psp/timezone`, `/psp/localtime` (panel setting) |
| `update_now.sh`, `update.sh` | trigger/perform firmware update |
| `volume_info.sh`, `volume_name.sh` | XML info / name of a mounted volume (panel music/USB screens) |
| `wait_for_link` | wait for net link |
| `wait_for_opening` ★ | wait for opening anim completion, or kill the player |
| `wait_for_usb` | wait for USB subsystem |
| `weppasswd.pl` ★ | WEP password→key derivation (WiFi setup screen) |

### Daemons referenced by the launch path (binaries, not scripts)
| Daemon | Started by | Role |
|--------|-----------|------|
| `/usr/sbin/btplayd` ★ | `start_control_panel` | resident audio/stream player ("btplay"); controlled via `btplay --passthru=…` |
| `/bin/chumbalarmd` ★ | `rcS` | backup alarm daemon; can itself launch `start_control_panel` (missed-alarm path); heartbeat `/tmp/chumbalarmd_heartbeat` |
| `headphone_mgr` | `rcS` | monitors headphone jack / battery / network |
| `/usr/bin/chumby_set_volume` ★ | wrappers above | mixer control binary |
| `dcid` ★ | `download_cp` | prints DCID node/value XML (device config id) |
| `imgtool` | fb_cgi.sh, fw scripts | framebuffer image capture/draw |
| `fbwrite` | start_control_panel error path | write text to framebuffer |

## B. Firmware clone: `resources/zurks-offline-firmware-classic/scripts/` (110 entries)

⚠ Generation caveat: several scripts here are for **later/other hardware**
(falconwing/silvermoon: `bootstream_make_bootable`, `create_storage_partition.sh`,
`power_state_changed.sh` with sysfs backlight, `logo.brand` parsing, stormwind
references in `temp_update.sh`). The repo is the *offline-firmware overlay*,
not a 1:1 ironforge script set. Detailed reconciliation belongs to Step 1.6.
Scripts identical in name to section A are listed only with deltas/notes.

| Script | Purpose / delta vs backup |
|--------|---------------------------|
| `add-mount.sh` / `remove-mount.sh` | mount/unmount a drive, report mountpoint (newer mountmon style) |
| `add.sh`, `mount.sh`, `umount.sh`, `umount_repeated.sh`, `remove.sh`, `mountmon_utils.sh` | as in A |
| `ap_scan` | newer "3rd generation" network script |
| `BDXML.pm` | "brain-dead XML parser" perl module used by other scripts |
| `bootstream_make_bootable` | falconwing boot-image writer (not ironforge) |
| `check_for_update.sh`, `check_update.sh` | firmware update checks (+legacy OTA variant) |
| `chumby_3g_connect.pl` | 3G modem connect |
| `chumby_set_mute/pan/volume`, `chumby_set_volume.sh` | as in A (+.sh variant) |
| `chumby_version`, `cpi.sh` | as in A |
| `cpp`, `gcc`, `g++`, `lex`, `make` | toolchain shims (zurk dev convenience) |
| `create_storage_partition.sh` | repartition /psp (falconwing) |
| `dcid_getparms` | reformat `dcid` output as URL params (split out of download_cp) |
| `disable_intro` / `enable_intro` | as in A |
| `download_cp` ★ | as in A **plus `/psp/urlbase_dlcp` URL override + sw/fw version spoofing** — the offline-redirect mechanism |
| `enable_audio.sh`, `enable_regulator.sh`, `enable_switch.sh`, `enable_timer.sh`, `enable_touchscreen.sh`, `enable_usb.sh`, `enable_watchdog.sh` | hardware bring-up (later-gen sysfs/gpio) |
| `fb_cgi.sh` | as in A |
| `fbwriteln` | write text line to framebuffer (script version of fbwrite) |
| `flashplayer_is_running`, `flashplayer_watchdog` | as in A (watchdog reworked) |
| `get_input_by_bus` / `get_input_by_name` | resolve /dev/input/event* devices |
| `guidgen.sh`, `macgen.sh`, `mkdevs` | as in A |
| `headphone_present` | detect headphone state |
| `is_valid_boot_image`, `printlabel.sh`, `printpass.pl` | factory/boot tooling |
| `list_mounts` | XML summary of mounts |
| `log-rollover*` | as in A |
| `mount_storage` | mount yaffs2 cache/storage |
| `mute_leds` | LED control |
| `network_*`, `restart_network`, `start_ap.sh`, `stop_dhcp`, `stop_getty` | network plumbing (newer generation) |
| `power_state_changed.sh` | backlight/power events (later-gen) |
| `prepare_ota_update.sh`, `prepare_updates.sh`, `update_launch.sh`, `update_network.sh`, `update_now.sh`, `update_usb.sh`, `temp_update.sh`, `temp_update_sub.sh`, `wait_for_update.sh` | firmware update machinery |
| `reboot_normal.sh`, `reload_backup_alarm` | as in A |
| `reset_usb.sh` | GPIO USB reset |
| `restore_factory_defaults.sh` | restore /psp from install source |
| `restore_time`, `save_time` | as in A |
| `script_locations.sh` | as in A |
| `select_input` | input device selection |
| `service_control/getpid/list` | as in A |
| `shutdown` | chumby-specific shutdown |
| `signal_daemon_event.sh` | like signal_soft_event but for daemons |
| `signal_soft_event.sh` ★ | as in A but signals via `killall -HUP` |
| `signal_usb_event.sh` ★ | raise USB event to player (`/tmp/flashplayer.event` + `-F1`) |
| `signal_strength` | as in A |
| `start_control_panel` ★ | as in A **plus demo-mode movies and `--MaxLocalMemBlocksSlave=384`** |
| `start_intro`, `start_opening_anim`, `stop_control_panel`, `wait_for_opening` ★ | as in A, minor deltas (see 01 §7) |
| `start_network*`, `start_sshd.sh` | as in A |
| `switch_fb.sh`, `sync_time.sh`, `sync_time_state.sh`, `time_zone.sh` | as in A |
| `usb_keyboard_egg.sh` | easter egg when USB keyboard present & player running |
| `volume_info.sh`-equivalents absent; `wait_for_link`, `wait_for_usb`, `weppasswd.pl` | as in A |
| `ytextract.pl` | extract mp4 URL from a YouTube page (zurk extra) |

### Zurk's web-control CGIs that talk to the player (`lighty/cgi-bin/`)
`event.sh`, `chumote/event.cgi`, `chumote/streams`, `custom/{setmute,setvol,
changewidget,shoutcast,somafm,multistreams}.sh`, `zmote_{on,off,play,playloop}.sh`
— all inject XML into `/tmp/flashplayer.event` and signal with
`chumbyflashplayer.x -F1`; `widget_refresh.sh` simply `killall`s the player.
This confirms the event-file channel is the universal external control path.

## C. Open questions carried into Step 1.3
1. Which of these scripts does the **SWF itself** invoke (via the backtick/
   exec mechanism hinted at in the player binary, or chumby's file:// exec
   extension)? The ★ set is the candidate list.
2. What writes the `/tmp/movieheartbeat` / `/tmp/flashheartbeat` files — the
   player binary or ActionScript inside the movies?
3. What exactly does `cpi` (Control Panel Interface) do — binary at
   `/usr/bin/cpi`? The backup `/tmp/query_*.xml` files (hwvr/snum/time) look
   like its request format.
