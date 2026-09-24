//! The chumby's own connectivity probe (F2:9696): `wget -T 10 -q -O -
//! http://www.chumby.com/crossdomain.xml`, success = a 2xx answer. The URL
//! answers plain HTTP without a redirect (checked 2026-09-24), so a bare
//! HTTP/1.1 GET does; wget's redirect following is not reproduced.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const HOST: &str = "www.chumby.com";
const PATH: &str = "/crossdomain.xml";
pub const TIMEOUT: Duration = Duration::from_secs(10);

pub fn probe() -> Result<(), String> {
    let deadline = Instant::now() + TIMEOUT;
    let addrs: Vec<_> = (HOST, 80)
        .to_socket_addrs()
        .map_err(|e| format!("resolve {HOST}: {e}"))?
        .collect();
    let left = |what: &str| {
        deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| format!("{what}: timed out after {} s", TIMEOUT.as_secs()))
    };
    let mut last_err = format!("{HOST}: no address");
    let mut stream = None;
    for a in &addrs {
        match TcpStream::connect_timeout(a, left("connect")?) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => last_err = format!("connect {a}: {e}"),
        }
    }
    let mut s = stream.ok_or(last_err)?;
    s.set_write_timeout(Some(left("send")?)).map_err(|e| e.to_string())?;
    write!(
        s,
        "GET {PATH} HTTP/1.1\r\nHost: {HOST}\r\nUser-Agent: chumby-supervisor\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| format!("send: {e}"))?;
    s.set_read_timeout(Some(left("receive")?)).map_err(|e| e.to_string())?;
    let mut head = [0u8; 64];
    let n = s.read(&mut head).map_err(|e| format!("receive: {e}"))?;
    status_ok(&head[..n])
}

fn status_ok(head: &[u8]) -> Result<(), String> {
    let line = String::from_utf8_lossy(head);
    let line = line.lines().next().unwrap_or("");
    match line.split_whitespace().nth(1) {
        Some(code) if code.starts_with('2') && code.len() == 3 => Ok(()),
        _ => Err(format!("answer: {line:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line() {
        assert!(status_ok(b"HTTP/1.1 200 OK\r\nContent-Type: text/xml").is_ok());
        assert!(status_ok(b"HTTP/1.0 204 No Content\r\n").is_ok());
        assert!(status_ok(b"HTTP/1.1 301 Moved\r\n").is_err());
        assert!(status_ok(b"HTTP/1.1 503 Busy\r\n").is_err());
        assert!(status_ok(b"").is_err());
    }
}
