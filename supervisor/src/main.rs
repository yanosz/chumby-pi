//! chumby-supervisor — owns the player process and restarts the panel when
//! the world changes under it (`claude/watchdog-plan.md`).
//!
//! Usage: `chumby-supervisor --ctl FIFO [--config player.toml] -- PLAYER ARGS…`
//!
//! Runs as cage's only client, so cage survives player restarts. Restarts
//! the player after a crash (R12) and asks it to restart when idle after a
//! wall-clock step (R9), when NetworkManager connectivity reaches full
//! after not being full (R3), and — with `access_chumby_com` — when
//! chumby.com reachability changes (R4). The player decides when it is idle
//! (fork `restart.rs`); a request is withdrawn when conditions are back to
//! their state at player start.

mod child;
mod nm;
mod policy;
mod probe;

use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use policy::{Backoff, Change, Policy, ProbeSchedule};

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => { eprintln!("chumby-supervisor: {}", format!($($t)*)) };
}

pub enum Event {
    PlayerExited(String),
    Nm(Option<u32>),
    Probe(Result<(), String>),
    Shutdown(i32),
}

const TICK: Duration = Duration::from_secs(1);
const STOP_GRACE: Duration = Duration::from_secs(10);
const DEFAULT_CONFIG: &str = "/etc/chumby-player/player.toml";

struct Args {
    ctl: PathBuf,
    config: PathBuf,
    player: Vec<OsString>,
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args_os().skip(1);
    let (mut ctl, mut config) = (None, PathBuf::from(DEFAULT_CONFIG));
    while let Some(a) = it.next() {
        match a.to_str() {
            Some("--ctl") => ctl = it.next().map(PathBuf::from),
            Some("--config") => config = it.next().map(PathBuf::from).ok_or("--config needs a path")?,
            Some("--") => {
                let player: Vec<OsString> = it.collect();
                if player.is_empty() {
                    return Err("no player command after --".into());
                }
                let ctl = ctl.ok_or("--ctl is required")?;
                return Ok(Args { ctl, config, player });
            }
            _ => return Err(format!("unexpected argument {a:?}")),
        }
    }
    Err("usage: chumby-supervisor --ctl FIFO [--config player.toml] -- PLAYER ARGS…".into())
}

/// `access_chumby_com` as the player reads it (fork `config.rs`: 0/1 or a
/// boolean); anything else, or no file, means off.
fn chumby_com_enabled(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    match text.parse::<toml::Table>().ok().and_then(|t| t.get("access_chumby_com").cloned()) {
        Some(toml::Value::Integer(1)) | Some(toml::Value::Boolean(true)) => true,
        _ => false,
    }
}

/// Wall clock minus monotonic clock: constant except when the clock is
/// set (or slewed, which stays far below the 15 s threshold).
fn clock_offset_ms() -> i64 {
    fn ms(clock: libc::clockid_t) -> i64 {
        let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe { libc::clock_gettime(clock, &mut ts) };
        ts.tv_sec * 1000 + ts.tv_nsec / 1_000_000
    }
    ms(libc::CLOCK_REALTIME) - ms(libc::CLOCK_MONOTONIC)
}

/// Blocks SIGTERM/SIGINT/SIGHUP in every thread and turns them into
/// `Event::Shutdown` from one waiting thread.
fn watch_signals(events: mpsc::Sender<Event>) {
    let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
    unsafe {
        libc::sigemptyset(&mut set);
        for s in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
            libc::sigaddset(&mut set, s);
        }
        libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
    }
    std::thread::Builder::new()
        .name("signals".into())
        .spawn(move || {
            let mut sig = 0;
            unsafe { libc::sigwait(&set, &mut sig) };
            let _ = events.send(Event::Shutdown(sig));
        })
        .expect("spawn signal thread");
}

/// One line to the player's control FIFO. Non-blocking: with no reader yet
/// (player still starting) the open fails and the caller retries.
fn send_ctl(ctl: &Path, line: &str) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(ctl)?;
    writeln!(f, "{line}")
}

/// What R8 wants on record when chumby.com keeps failing.
fn network_diagnostics(nm_state: Option<u32>) -> String {
    let route = std::fs::read_to_string("/proc/net/route")
        .ok()
        .and_then(|t| {
            t.lines().skip(1).find_map(|l| {
                let f: Vec<&str> = l.split_whitespace().collect();
                (f.len() > 2 && f[1] == "00000000").then(|| {
                    let g = u32::from_str_radix(f[2], 16).unwrap_or(0).to_le_bytes();
                    format!("{} via {}.{}.{}.{}", f[0], g[0], g[1], g[2], g[3])
                })
            })
        })
        .unwrap_or_else(|| "none".into());
    let nm = nm_state.map(nm::name).unwrap_or("unreadable");
    format!("NM connectivity {nm}, default route {route}")
}

fn seed() -> u64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    t ^ u64::from(std::process::id()) << 32
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            log!("{e}");
            std::process::exit(2);
        }
    };
    let chumby_com = chumby_com_enabled(&args.config);
    log!(
        "starting; control FIFO {}, chumby.com probe {}",
        args.ctl.display(),
        if chumby_com { "on" } else { "off" }
    );

    let (tx, rx) = mpsc::channel();
    watch_signals(tx.clone());
    {
        let tx = tx.clone();
        std::thread::Builder::new()
            .name("nm".into())
            .spawn(move || nm::watch(tx))
            .expect("spawn nm thread");
    }

    let mut policy = Policy::new(chumby_com, clock_offset_ms());
    let mut schedule = ProbeSchedule::new(seed());
    let mut backoff = Backoff::new();
    let mut player: Option<child::Player> = None;
    let mut respawn_at = Some(Instant::now());
    let mut sent = false;
    let mut probing = false;
    let mut nm_state: Option<u32> = None;
    let mut chumby_up: Option<bool> = None;

    loop {
        let now = Instant::now();

        if player.is_none() && respawn_at.is_some_and(|t| now >= t) {
            respawn_at = None;
            match child::spawn(&args.player, tx.clone()) {
                Ok(p) => {
                    log!("player started, pid {}", p.pid());
                    policy.player_started(now, clock_offset_ms());
                    sent = false;
                    player = Some(p);
                }
                Err(e) => {
                    let d = backoff.after_crash(Duration::ZERO);
                    log!("cannot start the player: {e} — retrying in {} s", d.as_secs());
                    respawn_at = Some(now + d);
                }
            }
        }

        policy.set_offset(clock_offset_ms());
        if chumby_com && !probing && schedule.due(now) {
            probing = true;
            let tx = tx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(Event::Probe(probe::probe()));
            });
        }

        match policy.tick(now) {
            Some(Change::Pending { reasons, due }) => log!(
                "restart wanted ({reasons:?}); spacing holds it for {} s",
                due.saturating_duration_since(now).as_secs()
            ),
            Some(Change::Requested(reasons)) => {
                log!("restart wanted ({reasons:?}); asking the player to restart when idle")
            }
            Some(Change::Dropped { was_requested }) => log!(
                "conditions back to their state at player start — restart {}",
                if was_requested { "request withdrawn" } else { "dropped" }
            ),
            None => {}
        }
        if player.is_some() && policy.wants_restart() != sent {
            let line = if policy.wants_restart() { "restart-when-idle" } else { "restart-cancel" };
            if send_ctl(&args.ctl, line).is_ok() {
                sent = policy.wants_restart();
            }
        }

        match rx.recv_timeout(TICK) {
            Ok(Event::PlayerExited(status)) => {
                let now = Instant::now();
                if let Some(p) = player.take() {
                    p.stop_group(STOP_GRACE);
                    let ran = p.started.elapsed();
                    if policy.player_exited(now) {
                        log!("player quit for the requested restart ({status}) — restarting");
                        respawn_at = Some(now);
                    } else {
                        let d = backoff.after_crash(ran);
                        log!(
                            "player exited unasked ({status}) after {} s — restarting in {} s",
                            ran.as_secs(),
                            d.as_secs()
                        );
                        respawn_at = Some(now + d);
                    }
                }
                sent = false;
            }
            Ok(Event::Nm(state)) => {
                log!("NetworkManager connectivity: {}", state.map(nm::name).unwrap_or("unreadable"));
                nm_state = state;
                let full = state.map(|c| c == nm::FULL);
                policy.set_nm(Instant::now(), full);
                schedule.set_full(Instant::now(), full == Some(true));
            }
            Ok(Event::Probe(result)) => {
                probing = false;
                let ok = result.is_ok();
                if chumby_up != Some(ok) {
                    match &result {
                        Ok(()) => log!("chumby.com reachable"),
                        Err(e) => log!("chumby.com unreachable: {e}"),
                    }
                    chumby_up = Some(ok);
                }
                policy.set_chumby(ok);
                if schedule.done(Instant::now(), ok) {
                    log!(
                        "chumby.com failed {} probes in a row ({}); {} — the original would restart the network here, we only log",
                        schedule.failures(),
                        result.err().unwrap_or_default(),
                        network_diagnostics(nm_state)
                    );
                }
            }
            Ok(Event::Shutdown(sig)) => {
                log!("signal {sig} — stopping the player");
                if let Some(p) = player.take() {
                    p.stop_group(STOP_GRACE);
                }
                return;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
