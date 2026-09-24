# Panel watchdog — plan

Started 2026-09-24. Motivation: appliance issues 21 (alarms dead after a
clock step) and 22 (WLAN shown good, unusable). Status: **step 1
(requirements) done 2026-09-24; step 2, design, in progress.** Nothing
built.

## Ground truth: what the original firmware did

From `/home/jan/chumby_backup`:

- `psp/crontabs/root`: cron runs `usr/chumby/scripts/flashplayer_watchdog`
  every minute. It restarts the panel (`stop_control_panel --keepalive`,
  clear the sense1/sense2 drivers, `start_control_panel`) when
  `/tmp/movieheartbeat` or `/tmp/flashheartbeat` is older than 30 s, and
  restarts `chumbalarmd` on a stale `/tmp/chumbalarmd_heartbeat`. Armed by
  `/tmp/flashplayer_started`, which `network_status.sh` removes for its own
  duration.
- The panel writes `/tmp/movieheartbeat` every 15 s (`setupHeartbeat`,
  F2:149-178, write at F2:174). Our player still does: fixture rootfs
  `tmp/movieheartbeat` was fresh on chumby-pi-3 2026-09-24 10:15. Nothing
  reads it.
- `start_control_panel` restarts itself when the player exits non-zero and
  starts the panel `builtin=1` when `network_status.sh` reports an error.
- Clock: `restore_time` rebuilds the time from the crypto processor's
  uptime counter, `sync_time.sh -b` runs `ntpdate` in the background at boot
  (`etc/init.d/rcS:230`, racing the panel start), and cron repeats it daily
  at 03:00.
- Connectivity, in the panel: only when the network was up at start and
  lost later (`!hasNetwork && hadNetwork`, F2:5335) does `beAClock` arm a
  probe — `wget -T 10 http://www.chumby.com/crossdomain.xml` (F2:9696),
  first after 720 frames, then every random 5-20 min, `restart_network` on
  every 5th failure; success runs `WidgetPlayer.reload()` (F2:4110), not a
  panel restart. Booted offline, no probe is ever armed.

## Ground truth: the appliance today

- `chumby-player.service`: `Restart=on-failure`, `RestartSec=3`
  (`pkg/chumby-player/chumby-player.service:39-40`). Crashes only.
- Night mode survives a panel restart already: at start the panel enters
  night mode when `/tmp/nightmode` is `"1"` or `/psp/dimlevel` is `"2"`
  (F2:359); `NightMode` writes the file on enter/exit (F2:19751, 19757). Our
  `/tmp` is the fixture rootfs on disk, so it also survives a reboot. Alarms
  missed while the panel was down are not replayed.
- chumby-pi-3 runs NetworkManager with its connectivity check **off**
  (`ConnectivityCheckEnabled false`, `ConnectivityCheckUri ""`), so its
  `full` today is a guess from the default route — the shape of issue 22.
- `access_chumby_com` (`ruffle/fixtures/player.toml.example:21`, shipped 0)
  is the config switch for chumby.com.

## Requirements (interview answers, Jan)

- R1. Triggers: (b) the wall clock steps; (c) connectivity is restored.
  (a), the panel's heartbeat going stale, was chosen first and then
  dropped — see R11. (2026-09-24)
- R2. The watchdog is integrated in our own software and is the master for
  starting and restarting the control panel. (2026-09-24)
- R3. Network: record the state at panel start; a change to NetworkManager
  connectivity `full` restarts the panel; connection drops are ignored.
  (2026-09-24)
- R4. With `access_chumby_com` enabled, after a change to `full`, the
  supervisor probes chumby.com itself, the chumby way (the original's
  `wget` of `www.chumby.com/crossdomain.xml`, F2:9696). A loss of chumby.com
  restarts the panel, so it comes back in its offline form; the recovery
  after it restarts it again. Reading (a) of two offered — (b) would have
  armed on loss and restarted on recovery only. Cadence as the original:
  first probe ~1 min after the trigger (720 frames), then every random
  5-20 min, 10 s timeout each. (2026-09-24)
- R5. The watchdog is a new supervisor process that starts the player as
  its child. (2026-09-24)
- R6. After a boot the panel starts in day mode, as on the original, whose
  `/tmp` was a ramdisk. Implied by "as on the original", stated by me for
  Jan to correct: `/psp/dimlevel == "2"` still forces night mode at start
  (F2:359), and a watchdog restart — not a boot — keeps `/tmp/nightmode`,
  as `stop_control_panel --keepalive` did. Related: `fixture.rs:52` already
  records full `/tmp` volatility as a gap. (2026-09-24)
  Why this is a boot question, not a watchdog one: a restart at the clock
  step recomputes every alarm from the correct time, so only transitions
  due while no panel ran on a correct clock are lost — chiefly power-off.
- R7. NetworkManager's `full` is made real with Debian's own check:
  package `network-manager-config-connectivity-debian` (1.52.1-1+rpt4 on
  trixie/rpt, not installed on chumby-pi-3), which ships only
  `/usr/lib/NetworkManager/conf.d/20-connectivity-debian.conf` =
  `[connectivity] uri=http://network-test.debian.org/nm`; the endpoint
  answers 200 `NetworkManager is online` (2026-09-24). No interval is set,
  so NM's default applies — 300 s by my recollection, unverified. Means a
  new Depends of the appliance package. (2026-09-24)
- R8. The supervisor never touches networking — NetworkManager owns the
  interfaces. Where the original would have run `restart_network` (every
  5th failed probe, F2:5377) it logs instead, so the next issue-22 case
  leaves evidence. Jan first chose "leave it alone", then "log only".
  (2026-09-24)
- R9. A wall-clock step of more than 15 s, either direction, restarts the
  panel. 15 s = `RING_WINDOW` (F2:10186): a smaller forward step is
  absorbed by the ring window; a larger one skips any alarm inside it
  (issue 21); a backward step can leave an alarm already computed for
  tomorrow. DST is not a step (the kernel clock is UTC). (2026-09-24)
- R10. No wait at boot: the supervisor starts the panel at once, and the
  first NTP step restarts it through R9. With no RTC nearly every cold boot
  ends in a large forward step, so a cold boot normally means one extra
  restart shortly after start. Jan first accepted a bounded 60 s wait for
  the first sync, then reversed it. (2026-09-24)
- R11. No hang detection. The supervisor does not watch
  `/tmp/movieheartbeat` (or any player heartbeat): Jan has never seen the
  panel hang. Reverses trigger (a) of R1; the original's
  heartbeat mechanism is recorded above in case it is needed later.
  (2026-09-24)
- R12. The supervisor itself restarts a player that exits (systemd only
  restarts the supervisor). No cap on the number of restarts; every restart
  is logged with its cause. Starts are spaced out so a crash loop cannot
  overload the box — values settled in design (step 2). (2026-09-24)
- R13. The supervisor logs to the journal only — no log file of its own,
  no change to journald. The journal on chumby-pi-3 is volatile by
  Raspberry Pi OS design (`/usr/lib/systemd/journald.conf.d/
  40-rpi-volatile-storage.conf`, `Storage=volatile`, package
  `raspberrypi-sys-mods`): a 182 MB `/run` tmpfs, capped near 18 MB, lost
  at every reboot. Accepted. Reverting the verbose `RUST_LOG` (issue 13)
  is what stretches its reach. (2026-09-24)
- R14. Every restart trigger waits while (a) an alarm is ringing or
  snoozing, or (c) someone is using the screen — "in use" means a tap
  event less than 60 s ago — until that ends. Music playing alone does
  not hold a restart back. Triggers arriving during the wait merge into
  one restart when it ends. Reason for (a): a restart silences a ringing
  alarm, and the restarted panel does not ring it again
  (`lastAlarmRingTime`, F2:10910) — issue 13's symptom by another route;
  (c) keeps a restart from pulling the screen away mid-use. (2026-09-24)

## Open questions

- R12: the start-spacing values (design; my proposal: a delay growing from
  a few seconds to about a minute, reset once the player has stayed up).
- R14 (design): how the supervisor learns the alarm state and the last
  tap time from the player process.

## Steps

1. Requirements interview — CHECKPOINT.
2. Design — CHECKPOINT.
3. Build, desktop verification — CHECKPOINT.
4. Device verification on chumby-pi-3 — CHECKPOINT.
