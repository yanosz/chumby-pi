//! When to restart the panel — pure decision logic, time injected.
//!
//! Requirements and design: `claude/watchdog-plan.md` (R3, R4, R9, R12,
//! R14; D3, D4, D7). Everything is judged against the state recorded at the
//! last player start, so a condition that returns to it before its restart
//! runs drops the restart.

use std::time::{Duration, Instant};

/// A wall-clock step beyond this leaves alarms outside the panel's
/// `RING_WINDOW` (F2:10186).
pub const CLOCK_STEP_MS: i64 = 15_000;
/// At most one triggered restart per this interval.
pub const TRIGGER_SPACING: Duration = Duration::from_secs(300);
/// After the first player start NM needs a few seconds for its first
/// connectivity check (`none` at start, `full` 5 s later on chumby-pi-3);
/// reaching full inside this window is its starting state, not a recovery.
pub const NM_SETTLE: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Clock,
    Network,
    ChumbyCom,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    /// A trigger appeared; the request goes out at `due`.
    Pending { reasons: Vec<Reason>, due: Instant },
    /// The player is asked to restart when idle.
    Requested(Vec<Reason>),
    /// Conditions are back to their state at player start; a request
    /// already sent is withdrawn.
    Dropped { was_requested: bool },
}

pub struct Policy {
    chumby_com: bool,
    base_offset_ms: i64,
    base_chumby: Option<bool>,
    offset_ms: i64,
    /// `None`: NetworkManager unavailable — the network trigger is off.
    nm_full: Option<bool>,
    seen_not_full: bool,
    settle_until: Option<Instant>,
    chumby: Option<bool>,
    pending: bool,
    wants_restart: bool,
    last_triggered: Option<Instant>,
}

impl Policy {
    pub fn new(chumby_com: bool, offset_ms: i64) -> Self {
        Self {
            chumby_com,
            base_offset_ms: offset_ms,
            base_chumby: None,
            offset_ms,
            nm_full: None,
            seen_not_full: false,
            settle_until: None,
            chumby: None,
            pending: false,
            wants_restart: false,
            last_triggered: None,
        }
    }

    pub fn player_started(&mut self, now: Instant, offset_ms: i64) {
        if self.settle_until.is_none() {
            self.settle_until = Some(now + NM_SETTLE);
        }
        self.base_offset_ms = offset_ms;
        self.offset_ms = offset_ms;
        self.base_chumby = self.chumby;
        self.seen_not_full = self.nm_full == Some(false);
        self.pending = false;
        self.wants_restart = false;
    }

    /// Returns whether the exit was the requested one (restart at once)
    /// rather than a crash (restart after backoff).
    pub fn player_exited(&mut self, now: Instant) -> bool {
        let requested = self.wants_restart;
        if requested {
            self.last_triggered = Some(now);
        }
        self.pending = false;
        self.wants_restart = false;
        requested
    }

    pub fn set_offset(&mut self, offset_ms: i64) {
        self.offset_ms = offset_ms;
    }

    pub fn set_nm(&mut self, now: Instant, full: Option<bool>) {
        match full {
            Some(false) => self.seen_not_full = true,
            Some(true) if self.settle_until.is_some_and(|t| now < t) => {
                self.seen_not_full = false;
            }
            _ => {}
        }
        self.nm_full = full;
    }

    pub fn set_chumby(&mut self, reachable: bool) {
        if self.base_chumby.is_none() {
            self.base_chumby = Some(reachable);
        }
        self.chumby = Some(reachable);
    }

    pub fn wants_restart(&self) -> bool {
        self.wants_restart
    }

    fn holding(&self) -> Vec<Reason> {
        let mut r = Vec::new();
        if (self.offset_ms - self.base_offset_ms).abs() > CLOCK_STEP_MS {
            r.push(Reason::Clock);
        }
        // R3: a drop alone never restarts; reaching full after one does.
        if self.nm_full == Some(true) && self.seen_not_full {
            r.push(Reason::Network);
        }
        if self.chumby_com && self.base_chumby.is_some() && self.chumby != self.base_chumby {
            r.push(Reason::ChumbyCom);
        }
        r
    }

    fn due(&self, now: Instant) -> Instant {
        match self.last_triggered {
            Some(t) if t + TRIGGER_SPACING > now => t + TRIGGER_SPACING,
            _ => now,
        }
    }

    pub fn tick(&mut self, now: Instant) -> Option<Change> {
        let reasons = self.holding();
        if reasons.is_empty() {
            if !self.pending {
                return None;
            }
            let was_requested = self.wants_restart;
            self.pending = false;
            self.wants_restart = false;
            return Some(Change::Dropped { was_requested });
        }
        let due = self.due(now);
        if !self.pending {
            self.pending = true;
            if due > now {
                return Some(Change::Pending { reasons, due });
            }
        }
        if !self.wants_restart && now >= due {
            self.wants_restart = true;
            return Some(Change::Requested(reasons));
        }
        None
    }
}

/// Crash-restart spacing (R12): 3 s doubling to 60 s, reset once the
/// player has stayed up 5 min.
pub struct Backoff {
    delay: Duration,
}

impl Backoff {
    const FIRST: Duration = Duration::from_secs(3);
    const MAX: Duration = Duration::from_secs(60);
    const STABLE: Duration = Duration::from_secs(300);

    pub fn new() -> Self {
        Self { delay: Self::FIRST }
    }

    pub fn after_crash(&mut self, ran: Duration) -> Duration {
        if ran >= Self::STABLE {
            self.delay = Self::FIRST;
        }
        let d = self.delay;
        self.delay = (self.delay * 2).min(Self::MAX);
        d
    }
}

/// The original's chumby.com probe cadence (F2:5335-5390): first 60 s after
/// the network is full, then every random 5-20 min; paused while it is not.
pub struct ProbeSchedule {
    next: Option<Instant>,
    failures: u32,
    rng: u64,
}

impl ProbeSchedule {
    const FIRST: Duration = Duration::from_secs(60);
    const MIN: u64 = 5 * 60;
    const MAX: u64 = 20 * 60;
    /// The original ran `restart_network` every 5th failure; we log (R8).
    pub const WARN_EVERY: u32 = 5;

    pub fn new(seed: u64) -> Self {
        Self { next: None, failures: 0, rng: seed | 1 }
    }

    pub fn set_full(&mut self, now: Instant, full: bool) {
        if !full {
            self.next = None;
        } else if self.next.is_none() {
            self.next = Some(now + Self::FIRST);
        }
    }

    pub fn due(&self, now: Instant) -> bool {
        self.next.is_some_and(|t| now >= t)
    }

    /// Records a result; returns true when the failure streak warrants the
    /// diagnostic warning.
    pub fn done(&mut self, now: Instant, ok: bool) -> bool {
        self.failures = if ok { 0 } else { self.failures + 1 };
        if self.next.is_some() {
            self.next = Some(now + Duration::from_secs(self.pick()));
        }
        !ok && self.failures % Self::WARN_EVERY == 0
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }

    fn pick(&mut self) -> u64 {
        // xorshift64: spreads probes of many boxes; no crypto needed.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        Self::MIN + self.rng % (Self::MAX - Self::MIN + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A player started long enough ago that NM has settled.
    fn started(chumby_com: bool, nm: Option<bool>) -> Policy {
        let mut p = Policy::new(chumby_com, 1_000);
        let long_ago = Instant::now() - 2 * NM_SETTLE;
        p.set_nm(long_ago, nm);
        p.player_started(long_ago, 1_000);
        p
    }

    #[test]
    fn test_clock_step_beyond_15s_requests() {
        let t = Instant::now();
        let mut p = started(false, Some(true));
        p.set_offset(1_000 + 15_000);
        assert_eq!(p.tick(t), None, "exactly 15 s is inside the ring window");
        p.set_offset(1_000 - 15_001);
        assert_eq!(p.tick(t), Some(Change::Requested(vec![Reason::Clock])));
        assert!(p.wants_restart());
    }

    #[test]
    fn test_clock_stepping_back_withdraws() {
        let t = Instant::now();
        let mut p = started(false, Some(true));
        p.set_offset(500_000);
        assert!(matches!(p.tick(t), Some(Change::Requested(_))));
        p.set_offset(2_000);
        assert_eq!(p.tick(t), Some(Change::Dropped { was_requested: true }));
        assert!(!p.wants_restart());
    }

    #[test]
    fn test_network_drop_ignored_recovery_requests() {
        let t = Instant::now();
        let mut p = started(false, Some(true));
        p.set_nm(t, Some(false));
        assert_eq!(p.tick(t), None, "a drop alone never restarts");
        p.set_nm(t, Some(true));
        assert_eq!(p.tick(t), Some(Change::Requested(vec![Reason::Network])));
    }

    #[test]
    fn test_network_not_full_at_start_then_full_requests() {
        let t = Instant::now();
        let mut p = started(false, Some(false));
        p.set_nm(t, Some(true));
        assert_eq!(p.tick(t), Some(Change::Requested(vec![Reason::Network])));
    }

    #[test]
    fn test_network_drop_again_before_restart_withdraws() {
        let t = Instant::now();
        let mut p = started(false, Some(false));
        p.set_nm(t, Some(true));
        assert!(matches!(p.tick(t), Some(Change::Requested(_))));
        p.set_nm(t, Some(false));
        assert_eq!(p.tick(t), Some(Change::Dropped { was_requested: true }));
    }

    #[test]
    fn test_no_network_manager_no_network_trigger() {
        let t = Instant::now();
        let mut p = started(false, None);
        p.set_nm(t, None);
        assert_eq!(p.tick(t), None);
    }

    #[test]
    fn test_spacing_and_drop_inside_window() {
        let t0 = Instant::now();
        let mut p = started(false, Some(false));
        p.set_nm(t0, Some(true));
        assert!(matches!(p.tick(t0), Some(Change::Requested(_))));
        assert!(p.player_exited(t0), "exit after a request is the requested one");
        p.player_started(t0, 1_000);

        let t1 = t0 + Duration::from_secs(60);
        p.set_offset(100_000);
        assert_eq!(
            p.tick(t1),
            Some(Change::Pending { reasons: vec![Reason::Clock], due: t0 + TRIGGER_SPACING })
        );
        assert_eq!(p.tick(t0 + Duration::from_secs(299)), None);
        // Back to good inside the window: no restart at its end.
        p.set_offset(1_000);
        assert_eq!(
            p.tick(t0 + Duration::from_secs(200)),
            Some(Change::Dropped { was_requested: false })
        );
        assert_eq!(p.tick(t0 + TRIGGER_SPACING), None);

        p.set_offset(100_000);
        let t2 = t0 + Duration::from_secs(250);
        assert!(matches!(p.tick(t2), Some(Change::Pending { .. })));
        assert_eq!(
            p.tick(t0 + TRIGGER_SPACING),
            Some(Change::Requested(vec![Reason::Clock]))
        );
    }

    #[test]
    fn test_nm_reaching_full_while_settling_is_the_start_state() {
        let t0 = Instant::now();
        let mut p = Policy::new(false, 1_000);
        p.player_started(t0, 1_000);
        p.set_nm(t0, Some(false));
        p.set_nm(t0 + Duration::from_secs(5), Some(true));
        assert_eq!(p.tick(t0 + Duration::from_secs(5)), None, "boot: NM's first check");
        p.set_nm(t0 + NM_SETTLE, Some(false));
        p.set_nm(t0 + NM_SETTLE + Duration::from_secs(1), Some(true));
        let t = t0 + NM_SETTLE + Duration::from_secs(1);
        assert_eq!(p.tick(t), Some(Change::Requested(vec![Reason::Network])));
    }

    #[test]
    fn test_settling_is_only_after_the_first_start() {
        let t0 = Instant::now();
        let mut p = Policy::new(false, 1_000);
        p.player_started(t0, 1_000);
        p.set_nm(t0 + NM_SETTLE, Some(true));
        assert!(!p.player_exited(t0 + NM_SETTLE));
        let t1 = t0 + NM_SETTLE + Duration::from_secs(1);
        p.set_nm(t1, Some(false));
        p.player_started(t1, 1_000);
        p.set_nm(t1 + Duration::from_secs(2), Some(true));
        assert_eq!(p.tick(t1 + Duration::from_secs(2)), Some(Change::Requested(vec![Reason::Network])));
    }

    #[test]
    fn test_crash_is_not_a_requested_exit() {
        let mut p = started(false, Some(true));
        assert!(!p.player_exited(Instant::now()));
    }

    #[test]
    fn test_chumby_com_off_never_triggers() {
        let t = Instant::now();
        let mut p = started(false, Some(true));
        p.set_chumby(true);
        p.set_chumby(false);
        assert_eq!(p.tick(t), None);
    }

    #[test]
    fn test_chumby_com_loss_and_recovery_each_restart() {
        let t = Instant::now();
        let mut p = started(true, Some(true));
        p.set_chumby(true);
        assert_eq!(p.tick(t), None, "first probe only sets the baseline");
        p.set_chumby(false);
        assert_eq!(p.tick(t), Some(Change::Requested(vec![Reason::ChumbyCom])));
        assert!(p.player_exited(t));
        p.player_started(t, 1_000);
        let later = t + TRIGGER_SPACING;
        assert_eq!(p.tick(later), None, "started while down: down is the baseline");
        p.set_chumby(true);
        assert_eq!(p.tick(later), Some(Change::Requested(vec![Reason::ChumbyCom])));
    }

    #[test]
    fn test_backoff_doubles_to_a_minute_and_resets() {
        let mut b = Backoff::new();
        let short = Duration::from_secs(1);
        let got: Vec<u64> = (0..7).map(|_| b.after_crash(short).as_secs()).collect();
        assert_eq!(got, [3, 6, 12, 24, 48, 60, 60]);
        assert_eq!(b.after_crash(Duration::from_secs(300)).as_secs(), 3);
    }

    #[test]
    fn test_probe_schedule_cadence_and_pause() {
        let t = Instant::now();
        let mut s = ProbeSchedule::new(42);
        assert!(!s.due(t + Duration::from_secs(3600)), "never before full");
        s.set_full(t, true);
        assert!(!s.due(t + Duration::from_secs(59)));
        assert!(s.due(t + Duration::from_secs(60)));
        let t1 = t + Duration::from_secs(60);
        s.done(t1, true);
        assert!(!s.due(t1 + Duration::from_secs(299)));
        assert!(s.due(t1 + Duration::from_secs(1200)));
        s.set_full(t1, false);
        assert!(!s.due(t1 + Duration::from_secs(3600)), "paused while not full");
        s.set_full(t1, true);
        assert!(s.due(t1 + Duration::from_secs(60)), "restarts with the 60 s first probe");
    }

    #[test]
    fn test_probe_interval_stays_in_range() {
        let mut s = ProbeSchedule::new(7);
        for _ in 0..1000 {
            let v = s.pick();
            assert!((300..=1200).contains(&v), "{v}");
        }
    }

    #[test]
    fn test_probe_warns_every_fifth_failure() {
        let t = Instant::now();
        let mut s = ProbeSchedule::new(1);
        let warns: Vec<bool> = (0..10).map(|_| s.done(t, false)).collect();
        assert_eq!(warns.iter().filter(|w| **w).count(), 2);
        assert!(warns[4] && warns[9]);
        assert!(!s.done(t, true));
        assert_eq!(s.failures(), 0);
    }
}
