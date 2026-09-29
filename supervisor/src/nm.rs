//! NetworkManager's connectivity state (D6), polled over D-Bus.
//!
//! `Connectivity` is only meaningful with a connectivity check configured —
//! the appliance depends on network-manager-config-connectivity-debian
//! (R7); without one NM reports full whenever a default route exists.

use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::Event;

/// NM_CONNECTIVITY_FULL.
pub const FULL: u32 = 4;
const POLL: Duration = Duration::from_secs(5);

pub fn name(c: u32) -> &'static str {
    match c {
        1 => "none",
        2 => "portal",
        3 => "limited",
        4 => "full",
        _ => "unknown",
    }
}

/// Sends `Event::Nm` on every change; `None` while NM cannot be read.
pub fn watch(events: Sender<Event>) {
    let mut last: Option<Option<u32>> = None;
    let mut conn: Option<zbus::blocking::Connection> = None;
    loop {
        let now = read(&mut conn);
        if last != Some(now) {
            if now.is_none() {
                crate::log!("NetworkManager connectivity unreadable — network trigger off until it is");
            }
            last = Some(now);
            if events.send(Event::Nm(now)).is_err() {
                return;
            }
        }
        std::thread::sleep(POLL);
    }
}

fn read(conn: &mut Option<zbus::blocking::Connection>) -> Option<u32> {
    if conn.is_none() {
        *conn = zbus::blocking::Connection::system().ok();
    }
    let c = conn.as_ref()?;
    let value = zbus::blocking::Proxy::new(
        c,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .and_then(|p| p.get_property::<u32>("Connectivity"));
    match value {
        Ok(v) => Some(v),
        Err(_) => {
            *conn = None;
            None
        }
    }
}
