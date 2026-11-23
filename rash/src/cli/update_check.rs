//! `bashrs update --check`: report current vs latest release without installing.
//! First slice of sovereign-update adoption (issue #377). Network access is
//! behind a closure so tests never touch the network.

use crate::models::{Error, Result};
use std::cmp::Ordering;
use std::process::Command;

pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate(String),
    Available { current: String, latest: String },
}

/// Parse `v1.2.3` / `1.2.3[-pre]` into (major, minor, patch).
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim().trim_start_matches('v');
    let core = s.split(['-', '+']).next()?;
    let mut it = core.split('.');
    let v = (
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    );
    it.next().is_none().then_some(v)
}

/// Compare versions; never reports a downgrade as available.
pub fn compare(current: &str, latest: &str) -> Result<UpdateStatus> {
    let bad = |v: &str| Error::Validation(format!("unparseable version: {v:?}"));
    let c = parse_version(current).ok_or_else(|| bad(current))?;
    let l = parse_version(latest).ok_or_else(|| bad(latest))?;
    Ok(if l.cmp(&c) == Ordering::Greater {
        UpdateStatus::Available {
            current: current.to_string(),
            latest: latest.trim_start_matches('v').to_string(),
        }
    } else {
        UpdateStatus::UpToDate(current.to_string())
    })
}

pub fn render(status: &UpdateStatus) -> String {
    match status {
        UpdateStatus::UpToDate(v) => format!("bashrs {v} is up to date"),
        UpdateStatus::Available { current, latest } => {
            format!("bashrs {current} -> {latest} available (release)")
        }
    }
}

/// Extract `tag_name` from the GitHub releases/latest JSON.
pub fn latest_from_json(body: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| Error::Validation(format!("bad release JSON: {e}")))?;
    v["tag_name"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| Error::Validation("release JSON has no tag_name".into()))
}

fn fetch_latest_release() -> Result<String> {
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            "10",
            "https://api.github.com/repos/paiml/bashrs/releases/latest",
        ])
        .output()
        .map_err(|e| Error::Validation(format!("curl failed to start: {e}")))?;
    if !out.status.success() {
        return Err(Error::Validation("could not fetch latest release".into()));
    }
    latest_from_json(&String::from_utf8_lossy(&out.stdout))
}

/// Core of `update --check` with an injectable fetcher.
pub fn check_with(current: &str, fetch: impl FnOnce() -> Result<String>) -> Result<String> {
    Ok(render(&compare(current, &fetch()?)?))
}

pub fn handle_update_command(check: bool) -> Result<()> {
    if !check {
        return Err(Error::Validation(
            "installing is not implemented yet; use `bashrs update --check`".into(),
        ));
    }
    println!("{}", check_with(CURRENT, fetch_latest_release)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("v6.65.0"), Some((6, 65, 0)));
        assert_eq!(parse_version("1.2.3-rc1"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("x.y.z"), None);
    }

    #[test]
    fn newer_is_available() {
        assert_eq!(
            compare("6.65.0", "v6.66.0").unwrap(),
            UpdateStatus::Available {
                current: "6.65.0".into(),
                latest: "6.66.0".into()
            }
        );
    }

    #[test]
    fn same_or_older_is_up_to_date() {
        assert!(matches!(
            compare("6.65.0", "v6.65.0").unwrap(),
            UpdateStatus::UpToDate(_)
        ));
        assert!(matches!(
            compare("6.65.0", "v6.9.9").unwrap(),
            UpdateStatus::UpToDate(_)
        ));
    }

    #[test]
    fn bad_version_errors() {
        assert!(compare("6.65.0", "latest").is_err());
    }

    #[test]
    fn render_lines() {
        assert_eq!(
            render(&UpdateStatus::UpToDate("1.0.0".into())),
            "bashrs 1.0.0 is up to date"
        );
        assert_eq!(
            check_with("1.0.0", || Ok("v1.1.0".into())).unwrap(),
            "bashrs 1.0.0 -> 1.1.0 available (release)"
        );
    }

    #[test]
    fn fetch_failure_propagates() {
        assert!(check_with("1.0.0", || Err(Error::Validation("offline".into()))).is_err());
    }

    #[test]
    fn json_parsing() {
        assert_eq!(
            latest_from_json(r#"{"tag_name":"v2.0.0"}"#).unwrap(),
            "v2.0.0"
        );
        assert!(latest_from_json("{}").is_err());
        assert!(latest_from_json("nope").is_err());
    }
}
