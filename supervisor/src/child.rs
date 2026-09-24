//! The player as a child in its own process group (D3): mpv — music, alarm
//! streams — and the backup-alarm tone are the player's children, and a
//! restart must take them down with it.

use std::ffi::OsString;
use std::io;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::Event;

pub struct Player {
    pgid: libc::pid_t,
    pub started: Instant,
}

pub fn spawn(argv: &[OsString], events: Sender<Event>) -> io::Result<Player> {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]).process_group(0);
    // The supervisor blocks its shutdown signals to sigwait on them; the
    // player must not inherit that mask.
    unsafe {
        cmd.pre_exec(|| {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            libc::pthread_sigmask(libc::SIG_SETMASK, &set, std::ptr::null_mut());
            Ok(())
        });
    }
    let mut child = cmd.spawn()?;
    let pgid = child.id() as libc::pid_t;
    std::thread::Builder::new()
        .name("player-wait".into())
        .spawn(move || {
            let status = child.wait().map(describe).unwrap_or_else(|e| e.to_string());
            let _ = events.send(Event::PlayerExited(status));
        })?;
    Ok(Player { pgid, started: Instant::now() })
}

fn describe(status: std::process::ExitStatus) -> String {
    match (status.code(), status.signal()) {
        (Some(c), _) => format!("exit status {c}"),
        (None, Some(s)) => format!("signal {s}"),
        _ => status.to_string(),
    }
}

impl Player {
    pub fn pid(&self) -> libc::pid_t {
        self.pgid
    }

    /// SIGTERM to the group, SIGKILL after `grace` (the original
    /// `stop_control_panel` waited 10 s). Also clears members left behind
    /// by a leader that already exited.
    pub fn stop_group(&self, grace: Duration) {
        if !self.group_alive() {
            return;
        }
        unsafe { libc::kill(-self.pgid, libc::SIGTERM) };
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if !self.group_alive() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        crate::log!("player group {} ignored SIGTERM for {} s — SIGKILL", self.pgid, grace.as_secs());
        unsafe { libc::kill(-self.pgid, libc::SIGKILL) };
    }

    fn group_alive(&self) -> bool {
        unsafe { libc::kill(-self.pgid, 0) == 0 }
    }
}
