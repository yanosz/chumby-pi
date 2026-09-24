# Panel watchdog — plan

Started 2026-09-24. Motivation: appliance issues 21 (alarms dead after a
clock step) and 22 (WLAN shown good, unusable). Status: **steps 1
(requirements) and 2 (design) done and approved 2026-09-24; step 3 not
started.** Nothing built.

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
  (2026-09-24) Amended after test 1 (Jan, 2026-09-24): NM reaching `full`
  within 30 s of the *first* player start is its starting state, not a
  recovery — at boot NM reports `none` until its first check, `full` 5 s
  later on chumby-pi-3, which restarted the panel on every boot
  (`policy::NM_SETTLE`).
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
- R14. Every restart trigger waits while (a) an audio alarm (not a
  silent `type="none"` one) is ringing or snoozing, or (c) someone is
  using the screen — "in use" means a tap event less than 60 s ago —
  until that ends. Music playing alone does
  not hold a restart back. Triggers arriving during the wait merge into
  one restart when it ends. Reason for (a): a restart silences a ringing
  alarm, and the restarted panel does not ring it again
  (`lastAlarmRingTime`, F2:10910) — issue 13's symptom by another route;
  (c) keeps a restart from pulling the screen away mid-use. (2026-09-24)

## Open questions

- Moved to the design section below (D3 spacing, D4 alarm/tap state).

## Design (step 2, approved by Jan 2026-09-24)

Facts read for it: `pkg/chumby-player/chumby-player-run` (all 282 lines),
`chumby-player.service`, `chumby-ctl`; fork `input.rs`, `backup_alarm.rs`,
`fixture.rs:45-62`, `app.rs` chumby hooks, `claude/patch-surface.md`.

- D1. Process layout. `systemd → chumby-player-run --kiosk → cage →
  chumby-player-run` (seeding, boot intro — unchanged) `→ exec
  chumby-supervisor -- <the player command line the launcher builds
  today>`. The supervisor is cage's one client, so cage survives player
  restarts; each restart is a new Wayland client in the same cage (to be
  verified in step 3). The unit keeps `Restart=on-failure`, now for the
  supervisor only (R12).
- D2. Rust binary (Rust-over-shell principle; needs timerfd and D-Bus,
  which shell cannot do), in a new crate in chumby-pi — it is appliance
  code by the docs split; CI gains a second small cargo build. The
  player-side half of D4 is fork work. (Q-a, Jan, 2026-09-24)
- D3. Restart = the player runs in its own process group; the supervisor
  sends the group SIGTERM, waits up to 10 s (`stop_control_panel`'s
  budget), then SIGKILL. The group matters: mpv (music, alarm stream) and
  the backup-alarm Klaxon are the player's children and would otherwise
  outlive it. Crash restarts (R12): delay 3 s, doubling to 60 s, reset once
  the player has run 5 min. Triggered restarts: at most one per 5 min;
  a trigger inside that window merges into one restart at its end, as
  under R14 (Q-d, Jan, 2026-09-24).
  Re-evaluation (Jan, 2026-09-24): a pending restart — waiting for the
  5-min window or for the player (D4) — is dropped when conditions are
  back to good. The decision is taken when the restart would run, against
  the state recorded at the last panel start: clock — the accumulated
  offset change since then is back within 15 s; network — NM is no longer
  `full` (the next change to `full` triggers anew), while `full` after a
  drop still restarts, that recovery being R3's point; chumby.com —
  reachability equals the one at panel start. While the player holds a
  request, the supervisor withdraws it over the FIFO.
- D4. Deferral (R14) is decided *in the player*, where the state lives, so
  there is no race between "alarm starts ringing" and "supervisor kills":
  1. The supervisor writes `restart-when-idle` to the control FIFO
     (`--chumby-control`; unknown commands are already ignored,
     `input.rs:15-17`, so the protocol grows without breaking chumby-ctl).
  2. The player sets a flag; merging falls out of it (R14).
  3. Each event-loop pass (next to the existing `take_pointer` hook,
     `app.rs:458`) it checks: no *audio* alarm (`_type != "none"`) with
     `_alarmRinging` or `_alarmSnoozing` in `AlarmSet.alarmSet._alarms`
     (F2:11779, 11230-11236) — a snooze lives only in panel memory, so a
     restart would silently drop it (Q-e, Jan, 2026-09-24); the audio
     layer cannot tell an alarm from music (`AudioPlayer::play(url,
     volume)`, `audio.rs:82`, no alarm flag), so the panel's flags are the
     signal — and no tap for 60 s (monotonic clock, stamped in the
     existing touch/mouse arms of `app.rs`). A tap is a touch or click on
     the screen; a bend is not (Q-c, Jan, 2026-09-24). When both hold it
     exits with a dedicated code; the supervisor restarts at once, no backoff.
  No forced timeout: an alarm that rings for its full duration delays the
  restart for that long. Rejected alternative: the player publishes a
  status file and the supervisor decides — leaves a window in which an
  alarm starts ringing after the last status write.
- D5. Clock (R9). A `timerfd` on `CLOCK_REALTIME` with
  `TFD_TIMER_CANCEL_ON_SET` wakes on every clock set; the step size is the
  change of (realtime − monotonic) across it. |step| > 15 s → restart
  request. Slews never wake it.
- D6. Network (R3, R7). NetworkManager's D-Bus property `Connectivity`
  (4 = full), read at each player start, then watched for changes. Any
  change from not-full to full → restart request; drops only logged. The
  appliance package gains `Depends:
  network-manager-config-connectivity-debian`. NM absent (a non-NM image)
  → watcher off, one log line. Recovery is seen at NM's check interval
  (default ~300 s, unverified); NM's `CheckConnectivity` method can force
  a check if that proves too slow.
- D7. chumby.com (R4, R8), only with `access_chumby_com = 1` in
  `/etc/chumby-player/player.toml`: `GET
  http://www.chumby.com/crossdomain.xml`, 10 s timeout; first 60 s after
  the trigger, then every random 5-20 min. A change up→down or down→up →
  restart request. Every 5th consecutive failure → a warning with NM
  connectivity, default route and interface (R8's evidence). Probing runs
  whenever NM is `full` — from player start when it already is, else from
  the change to `full` — and stops while it is not. (Q-b, Jan, 2026-09-24)
- D8. Day mode after boot (R6). The launcher points
  `$STATE/fixtures/rootfs/tmp` at a directory under the real `/tmp` — a
  tmpfs on chumby-pi-3 (452 MB, verified) — seeding it from the shipped
  `fixtures/rootfs/tmp` when absent. Boot empties it, a restart keeps it:
  the original's ramdisk semantics, and it closes the gap `fixture.rs:52`
  records. The seed carries `nightmode=0`, so boot means day mode;
  `/psp/dimlevel` still applies. Precedent for a symlink inside the rootfs:
  `/mnt/usb` (`chumby-player-run`, USB_LINK). Consumer list for every
  panel `/tmp` path due before the change (step 3).
- D9. Logging (R13): every trigger, deferral, restart with its cause, the
  player's exit status, NM transitions — stderr, i.e. the journal.

Open design questions: none (Q-a to Q-e answered 2026-09-24).

Step-3 to-dos, not risks: confirm cage maps the new player window after
the old one exits (D1); the consumer list for every panel `/tmp` path
before D8's change (CLAUDE.md rule).

## Steps

1. Requirements interview — CHECKPOINT.
2. Design — CHECKPOINT.
3. Build, desktop verification, in sub-steps:
   - 3a. The two to-dos: cage maps a second player window; the `/tmp`
     consumer list — CHECKPOINT.
   - 3b. Fork: `restart-when-idle` (FIFO command, tap stamp, audio-alarm
     flag read, dedicated exit code), desktop-verified — CHECKPOINT.
   - 3c. chumby-pi: the supervisor crate (process group, spacing, clock,
     NetworkManager, chumby.com, re-evaluation, logging), unit-tested —
     CHECKPOINT.
   - 3d. Launcher and packaging: exec the supervisor, `rootfs/tmp` onto
     the tmpfs, the new Depends, CI's second cargo build — CHECKPOINT.
   - 3e. Desktop end-to-end run — CHECKPOINT.
4. Device verification on chumby-pi-3 — CHECKPOINT.

## Step 3a — consumer list for the panel's `/tmp` (D8), 2026-09-24

Question for each: does moving `$STATE/fixtures/rootfs/tmp` onto the real
tmpfs `/tmp` (emptied at boot, kept across restarts) change it? The
fixture fs joins paths under the rootfs and lets `std::fs` follow
symlinks (`fixture.rs:556-569`); `put_file` creates parents
(`fixture.rs:593`) — so the link target must exist, which the launcher
guarantees at each service start.

Panel (F2 = `frame_2/DoAction.as` of the decompile):

| path | use | verdict |
|---|---|---|
| `/tmp/nightmode` | read at start F2:359, `DefineSprite_1766/frame_1`:149; written F2:19751, 19757 | **the intended change**: absent after boot → day mode (R6) |
| `/tmp/musicsource` | resume banner F2:12797 | fork already deletes it at every start (`fixture.rs:52-58`); unaffected, the fork comment there goes stale ("ours persists") |
| `/tmp/widgetcache` | chumby.com widget download cache F2:30137, 3 MB cap | emptied at boot, as on the original; only matters with `access_chumby_com` |
| `/tmp/.guidhash` | written then `md5sum`'d at once F2:1945-1946 | unaffected |
| `/tmp/movieheartbeat` | written F2:174, no reader (R11) | unaffected |
| `/tmp/channel_names`, `/tmp/widget_names` | written F2:3738, 3750 | unaffected |
| `/tmp/currentProfileID`, `/tmp/currentProfileName` | written F2:4410-4411 | unaffected |
| `/tmp/controlpanelversion` | written F2:30531 | unaffected |
| `/tmp/translation.xml` | read only, ALT2 settings F2:3102/3237; nothing writes it | unaffected (absent either way) |
| `/tmp/profile.xml` | read only, first of the MultipathFile F2:4344; nothing writes it | unaffected |
| `/tmp/change_profile` | easter egg, read + `rm` F2:5255 | unaffected |
| `/tmp/hidden_ssid` | written `DefineSprite_612/frame_9`:21 | unaffected |
| `/tmp/intercomnamed.sh` | written + exec'd F2:21844-21849 | unaffected by location |

Fork:

| site | verdict |
|---|---|
| `fixture.rs:52-58` musicsource removal | still needed (restarts keep `/tmp`); comment to update |
| `fixture.rs:823-836` test | own temp rootfs; unaffected |
| `navigator.rs:51, 296-307` widgetcache curl mapping | resolves through the link; unaffected |
| `real_ident.rs` `md5sum` via the fixture fs | through the link; unaffected |
| `audio.rs:42` `/tmp/chumby-mpv.sock` | the real `/tmp`, not the rootfs; unaffected |
| `fixtures/rootfs/tmp/*`, 8 tracked files | panel-written artifacts; none has to pre-exist (`nightmode` absent = day), so the device needs no seeding; desktop runs keep using them |

chumby-pi:

| site | verdict |
|---|---|
| `pkg/chumby-player/chumby-local-widgets:81, 113` (`HOW_TO` text, "The download cache is …/rootfs/tmp/widgetcache") | path still resolves; text must add that the cache is emptied at every boot |
| `pkg/chumby-player/chumby-player-run` | gains the link block (D8), USB_LINK-style |

## Step 3a — cage maps a second player window (D1), 2026-09-24

Desktop, cage 0.2.0-2 (chumby-pi-3 has 0.2.0-2+rpt1+b1, same upstream
version), nested in the X11 session: `DISPLAY=:10 WLR_BACKENDS=x11
WLR_RENDERER=pixman cage -- client.sh`, where the client script started
`ruffle_desktop` (release build of 2026-09-09, tiny-skia, a scratch copy
of `fixtures/`, `controlpanel.swf`), killed it after 25 s, waited 2 s and
started it again. Screenshots: run 1 showed the panel clock (11:16), the
gap a black cage output, run 2 the clock again (11:17); both players
exited on SIGTERM (143), cage exited 0 only after the client script
ended. **Cage keeps running across a player restart and maps the new
window** — D1 holds as designed.

## Step 3b — fork change list and consumers (before coding), 2026-09-24

New module `core/src/chumby/restart.rs`: a requested flag, the last tap
(`Instant`), and `apply(activation)` which, when requested, no audio
alarm rings or snoozes and the last tap is ≥ 60 s old, clears the flag,
logs, and quits the player through the panel's own path.

| touchpoint | change | consumers → verdict |
|---|---|---|
| `input.rs:80` `handle()` | new verbs `restart-when-idle`, `restart-cancel` | `chumby-ctl` sends only `bend`/`tap` → unaffected; launcher `mkfifo` → unaffected; unknown verbs were already ignored (`input.rs:15-17`); new consumer: the supervisor |
| `avm.rs:60` `method` | one more `restart::apply(activation)` after `alarm_guard::apply` | runs on every native call — every frame via the `_bent` poll (`avm.rs:75`); read-only until it quits |
| `core/src/player.rs:3181` chumby block in `run_mouse_pick` | stamp the tap time when the left button is down | the `chumby_pick` debug trace is unchanged; covers mouse, touch (mapped to left button, `app.rs:178-205`) and FIFO `click`/`drag` |
| quit | `external_interface.invoke_fs_command("quit", "")` | `DesktopFSCommandProvider` → `RuffleEvent::ExitRequested` → `event_loop.exit()` (`desktop/src/backends/fscommand.rs:13`, `app.rs:769`); bypasses the intro's quit guard, which sits in `avm1/fscommand.rs:32`, not in the provider |
| panel state read | `AlarmSet.alarmSet` (F2:11779) → `_alarms[i]` → `_type`, `_alarmRinging`, `_alarmSnoozing` | read-only; `_alarmRinging` set F2:11191-11223, cleared 10973, 10993, 11022, 12906; `_alarmSnoozing` set 10980, cleared 10894, 11015, 11034, 11226; `TYPE_NONE = "none"` F2:10170 |
| `mod.rs` | `pub mod restart;` | — |

Both accepted by Jan, 2026-09-24. Deviation from D4: the quit path ends the process with exit
code 0, not a dedicated one (a dedicated code would need a hook in
`desktop/src/main.rs` after `run_app`). The supervisor can tell an idle
restart from a crash by whether it had a request pending.

Semantics note, for Jan: the tap stamp counts any press on the screen,
so a long-press — the touch stand-in for a bend (`app.rs:441-452`) —
also counts as a tap. A real bend (Home key, `chumby-ctl bend`) does not.

## Step 3b — result, 2026-09-24

Fork commit `cd1dc5c12`, fork issue 27 (full record there). The player
honours `restart-when-idle` / `restart-cancel` on the control FIFO and
quits (exit 0) once no audio alarm rings or snoozes and the screen has
been untouched for 60 s. One change against the change list above: the
check runs only at the per-frame `_bent` poll — inside other natives the
alarm flags are half-updated (`snoozeAlarm`, F2:10973-10980), which made
the first alarm run quit at the snooze. Desktop-verified: idle quit,
press hold + cancel, ring → snooze → re-ring → turn-off → quit 60 s after
the press.

## Step 3c — result, 2026-09-24

`supervisor/` (crate `chumby-supervisor`): `policy.rs` holds every
decision with time injected — clock trigger, network trigger, chumby.com
trigger, 5-min spacing, re-evaluation against the state at player start,
crash backoff, probe cadence; `child.rs` the process group; `nm.rs`
NetworkManager over D-Bus (zbus, polled every 5 s); `probe.rs` the chumby.com
GET; `main.rs` the event loop. 14 unit tests pass, no warnings. Deps: libc,
toml (the fork's major, same `access_chumby_com` parsing as `config.rs`),
zbus, ureq.

Two implementation choices against the design text:
- D5: the clock is read as (realtime − monotonic) on every 1 s tick instead
  of a `timerfd` with `TFD_TIMER_CANCEL_ON_SET`. The loop ticks anyway; a
  step is seen within 1 s, with no extra fd or unsafe code.
- D7: the probe follows redirects like wget (up to 20, HTTPS included) —
  Jan, 2026-09-24, replacing a first bare HTTP GET. `ureq` 3 with rustls
  (no gzip); `Cargo.lock` grows from 89 to 122 crates. Network tests,
  `#[ignore]`d so CI never runs them (`cargo test -- --ignored`): chumby.com
  answers; `http://github.com/` (301 → https) is followed to success; a
  404 fails.

Desktop smoke run of the binary with a stand-in player (`sh -c 'sleep …
& sleep 2; exit N'`): crash spacing 3/6/12/24/48 s; NM read as `full`;
chumby.com probed 60 s after `full` (probe on via a scratch player.toml) →
reachable; after the leader exits its orphan child in the same group is
gone; SIGTERM → "signal 15 — stopping the player", player group stopped,
nothing left. Clock and network triggers against a real player are 3e.

## Step 3d — result, 2026-09-24

- `chumby-player-run`: its last line execs `chumby-supervisor --ctl
  "$CTL" --config /etc/chumby-player/player.toml -- <the player command
  as before>`; `CHUMBY_SUPERVISOR` overrides the binary like
  `CHUMBY_RUFFLE`. The fixture `rootfs/tmp` is replaced by a link to
  `/tmp/chumby-panel-tmp` (mode 700), created at every start; the root-run
  `--seed` path hands that directory to `pi` along with `$STATE` — found
  while writing it: `chown -R` does not follow the link, and a root-owned
  700 directory would have locked the panel out of its `/tmp` until reboot.
- Package 0.9.8 (`build-debs.sh:21`): ships
  `/usr/lib/chumby-player/chumby-supervisor` (cross-built release; NEEDED
  only `libc.so.6`, `libgcc_s.so.1`, both already declared); Depends gains
  `network-manager-config-connectivity-debian` (R7), with a line in the
  description saying why. Kept as Depends, not Recommends (Jan,
  2026-09-24), knowing it pulls `network-manager` onto an image that uses
  another network manager — a no-op on Raspberry Pi OS, where NM already
  owns the interfaces.
- `deploy-pi.sh` and CI build it (CI also runs its unit tests; the network
  tests stay `#[ignore]`d); the install test checks it runs and refuses a
  bare call with its usage line (exit 2). CI's movie-start test still
  expects 124: `timeout` reports 124 whatever the supervisor's own exit.
- Text: `chumby-local-widgets` says the download cache is emptied at every
  boot; `docs/setup.md` "Operating it" describes the self-restarts and day
  mode after boot; fork `fixture.rs:52` comment updated (fork `66ebb6bd2`).
- Checked locally: `sh -n` on the launcher; `build-debs.sh` with the
  existing (Sep 10) dist binaries produced a 0.9.8 deb with the right
  Depends and the supervisor in place — packaging mechanics only, that
  deb was deleted, not deployed. Not yet run: the launcher end to end
  (3e) and CI.

## Step 3e — desktop end-to-end, 2026-09-24

The real launcher (`pkg/chumby-player/chumby-player-run`, dev overrides
`CHUMBY_STATE/SHARE/SWF/RUFFLE/SUPERVISOR/CTL`, X11 `:10`), release
builds of player (fork `cd1dc5c12`) and supervisor. The clock was moved
with a test-only preload shim (scratchpad `fakeclock.c`: `CLOCK_REALTIME`
+ the seconds in a file) inherited by supervisor and player alike, as
both see a real step. Fixtures: a daily beep alarm 5 min after the real
time.

1. Start with the clock 3 days slow: the launcher execs the supervisor;
   the player runs in its own group; `rootfs/tmp` is a link into
   `/tmp/chumby-panel-tmp`, no `nightmode` (day mode). The panel schedules
   the alarm for **Mon Sep 21 17:51** — issue 21's shape.
2. Clock stepped to real time at 17:47:29.026 → 0.72 s later `restart
   wanted ([Clock])`, `restart-when-idle` sent → 69 ms later the player
   quits (exit 0) → restarted at once; the new panel schedules the alarm
   for **Thu Sep 24 17:51**.
3. The alarm rang at 17:51:00.042. `kill -9` of the player → `exited
   unasked (signal 9) after 211 s — restarting in 3 s` → new player.
4. SIGTERM to the supervisor → `signal 15 — stopping the player`, launcher
   exit 0, no process left.

No panic. The only error, `Unable to create audio device`, appears once
per player start and equally in the 3b runs that started the player
directly — this desktop session's audio, not the supervisor.

Not covered on the desktop, left for the device (step 4): the network
trigger (would need NetworkManager on this machine to change state), a
real boot (tmpfs emptied → day mode), group cleanup with real mpv
children, `access_chumby_com` against a real chumby.com loss, and CI
(runs on a push, Jan's call).

## Step 4 — device, chumby-pi-3 (192.168.42.24)

Deployed 2026-09-24 18:02 with `pkg/deploy-pi.sh` (fork `66ebb6bd2`,
chumby-pi `76643e8`). Read off the box afterwards:
- `chumby-player` 0.9.8 active, `NRestarts=0`; `cage` (5551) →
  `chumby-supervisor` (5574) → `ruffle_desktop` (5585) in its own process
  group. Supervisor lines carry the `chumby-player-run[5574]` identifier
  (exec'd from the launcher).
- `rootfs/tmp` → `/tmp/chumby-panel-tmp` (pi, 700); the old on-disk
  `rootfs/tmp` contents are gone, as intended.
- apt pulled `network-manager-config-connectivity-debian` 1.52.1-1+rpt4
  from archive.raspberrypi.com; NM reloaded its config (SIGHUP,
  `op="reload"`, same NM pid 636 — no restart, SSH unaffected) and now
  reports `ConnectivityCheckEnabled true`, URI
  `http://network-test.debian.org/nm`, `Connectivity 4` (full).
- The supervisor's first lines: started, player pid 5585, NM `full`.

### Test 1 — reboot, 2026-09-24 18:21, and the regression it exposed

**Regression, fixed:** 0.9.8 froze the panel from the deploy (18:02)
until the redeploy (18:27). `ps` showed the player in state `T`. The
service's stdin is `tty1` (`TTYPath=/dev/tty1`, `StandardInput=tty-fail`);
the player inherited it and the fork's stdin control channel reads it; in
its own process group — not the tty's foreground group — that read earns
SIGTTIN. Invisible on the desktop, where stdin was no tty. Fix: the
supervisor gives the player `/dev/null` as stdin (`child.rs`); reproduced
and verified under `script`'s pty (old: stand-in stopped, `T`; new: `S`).
Symptoms on the box, all explained by it: `/tmp/chumby-panel-tmp` stayed
empty, and the boot's `restart-when-idle` never reached the player.
Second fix from the same episode: `stop_group` follows SIGTERM with
SIGCONT — the old supervisor could not end the stopped player and needed
the 10 s SIGKILL (`player group 1153 ignored SIGTERM for 10 s`); a stopped
stand-in now ends 0.1 s after SIGTERM. After the redeploy (18:27:12): the
player runs (`R`), `/tmp/chumby-panel-tmp` fills, `stdin control channel
closed`.

**Reboot findings** (journal of boot `26fdfe1d…`):
- The clock step of a warm reboot is small: timesyncd restored 18:21:18
  from its clock file and its first answer (18:22:08.093) moved the clock
  by a few seconds — under 15 s, so no clock trigger. R10's "nearly every
  cold boot" holds for a box that was off for a while, not for a reboot.
- **Every boot triggers the network restart.** At player start
  (18:21:44.944) NM connectivity was `none` — its check had not run yet —
  and `full` 5 s later, so R3 asked for a restart at 18:21:49. With the
  fix in place that means a second panel start seconds after every boot.
- Day mode after boot: `/tmp/chumby-panel-tmp` was empty at boot (tmpfs),
  no `nightmode`.

### Test 1 repeated with the 30 s settling, 2026-09-24 18:33

Boot at ~18:33: NM `none` at player start (18:33:42.233), `full` at
18:33:47.237 — no restart requested; player running since boot, day mode.
Pass.

### Test 3 — mpv dies with the player, 2026-09-24 18:35-18:38

A one-shot alarm on the "Birds + SWR3" stream was added to
`/psp/alarms` (backup `alarms.bak-watchdog`); `kill -9` of the player →
`exited unasked (signal 9) after 103 s — restarting in 3 s`, and the new
panel read the alarm. It rang at 18:38:00 with `mpv pid=1459`, a child of
the player in its group (pgid 1292). `kill -9 1292` at 18:38:09.904 → no
mpv 1 s later; restarted after 6 s, the doubled delay for a second crash.
Pass. `/psp/alarms` restored from the backup (`cmp` identical) and the
service restarted so the panel reloads it; the test alarm is gone.

### Regression 2 — `systemctl stop`/`restart` hangs 90 s

Found during that restart. The unit's processes live in the logind
session scope (`PAMName=login`, pam_systemd moves them), not in the
service cgroup: `/proc/<pid>/cgroup` of cage, supervisor and player =
`/user.slice/user-1000.slice/session-12.scope`. So stopping the unit
signals cage alone. cage closes its Wayland clients — the player exits
(status 1) — and then waits for its child, which used to be ruffle and is
now the supervisor, which took the exit for a crash and started another
player 12 s later. Journal: `Stopping chumby-player.service` 18:38:37.125,
`State 'stop-sigterm' timed out. Killing.` 18:40:07.340, cage SIGKILLed,
the supervisor then logs `signal 1` (the tty hangup), `Failed with result
'timeout'`. Before 0.9.8 ruffle died with the Wayland connection and cage
followed at once. The same pattern explains the `signal 1` of the first
redeploy (18:27:11). Not fixed yet.

### Regression 2 fixed, 2026-09-24 19:49

Fork unchanged; chumby-pi `b05dbad`: the unit's `ExecStop=/usr/bin/pkill
-TERM -x -u pi chumby-supervis` stops the supervisor before systemd
signals cage (Depends gains `procps`), and the supervisor sets
`PR_SET_PDEATHSIG = SIGTERM`, so it ends if cage dies (desktop: SIGKILL of
a stand-in parent → `signal 15 — stopping the player`, nothing left).
On the box, `systemctl restart`: `Stopping` 19:49:35.847 → `Stopped`
19:49:36.169 (0.32 s), `Result=success`, new instance up 19:49:37. The
supervisor logged `signal 1`: the tty hangup and `ExecStop`'s SIGTERM
arrive together and the lower number is delivered first.

### Test 2 — network trigger, 2026-09-24 19:50

Correction first: `nftables` 1.1.3-1 was already installed — the earlier
"no firewall tool" came from `which` as `pi`, whose PATH lacks
`/usr/sbin`. Nothing was installed; its service is disabled, the ruleset
was empty. The check host resolved to 151.101.238.132 and
2a04:4e42:38::644; a separate table `inet chumbytest` dropped both on
output, `nmcli networking connectivity check` → `limited` (19:50:41);
the supervisor logged `limited` and asked for nothing (R3 ignores drops).
Table deleted 19:51:00, check → `full`; 19:51:02.104 `restart wanted
([Network])`, `restart-when-idle` received 1 ms later, `panel idle —
quitting` 89 ms after that, player restarted at once. Ruleset empty
again. Pass. The alarm-file backup of test 3 was removed after `cmp`.

### Step 4 — state

Passed on the device: day mode after boot, no restart from NM's first
check at boot, mpv dies with the player, crash spacing, the network
trigger, stop/restart in under a second. Found and fixed on the device:
the tty/SIGTTIN freeze and the 90 s stop. Not exercised on the device:
the clock trigger — a warm reboot steps the clock by a few seconds only;
it needs the box off for more than 15 s — and `access_chumby_com` (off on
this box). CI has not run: nothing is pushed.
