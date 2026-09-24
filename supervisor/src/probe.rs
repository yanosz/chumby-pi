//! The chumby's own connectivity probe (F2:9696): `wget -T 10 -q -O -
//! http://www.chumby.com/crossdomain.xml`, success = wget exit 0, i.e. a
//! 2xx answer after following redirects (wget's limit: 20), HTTPS
//! included.

use std::time::Duration;

const URL: &str = "http://www.chumby.com/crossdomain.xml";
pub const TIMEOUT: Duration = Duration::from_secs(10);
const MAX_REDIRECTS: u32 = 20;

pub fn probe() -> Result<(), String> {
    probe_url(URL)
}

fn probe_url(url: &str) -> Result<(), String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .max_redirects(MAX_REDIRECTS)
        .user_agent("chumby-supervisor")
        .build()
        .into();
    match agent.get(url).call() {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(r) => Err(format!("answer: {}", r.status())),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Network, so ignored by default: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn test_chumby_com_answers() {
        assert_eq!(probe(), Ok(()));
    }

    #[test]
    #[ignore]
    fn test_redirect_to_https_is_followed() {
        assert_eq!(probe_url("http://github.com/"), Ok(()));
    }

    #[test]
    #[ignore]
    fn test_not_found_fails() {
        let e = probe_url("https://www.chumby.com/no-such-file-here").unwrap_err();
        assert!(e.contains("404"), "{e}");
    }
}
