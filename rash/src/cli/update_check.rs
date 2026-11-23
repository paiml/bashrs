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
    fetch_latest_release_with_timeout("10")
}

fn fetch_latest_release_with_timeout(secs: &str) -> Result<String> {
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            secs,
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

/// Inputs for the startup notice; all environment is injected so tests are hermetic.
pub const STAMP_FILE: &str = "update-check-stamp";
const NOTICE_INTERVAL_SECS: u64 = 86_400;

pub struct NoticeCtx {
    pub opted_out: bool,
    pub is_tty: bool,
    pub ci: bool,
    pub cache_dir: Option<std::path::PathBuf>,
    pub now_secs: u64,
}

pub fn startup_notice_with(
    ctx: &NoticeCtx,
    current: &str,
    fetch: impl FnOnce() -> Result<String>,
) -> Option<String> {
    if ctx.opted_out || !ctx.is_tty || ctx.ci {
        return None;
    }
    let stamp = ctx.cache_dir.as_ref()?.join(STAMP_FILE);
    let last = std::fs::read_to_string(&stamp)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    if let Some(t) = last {
        if ctx.now_secs.saturating_sub(t) < NOTICE_INTERVAL_SECS {
            return None;
        }
    }
    // Stamp first so a failing or slow fetch is not retried on every command.
    if let Some(dir) = stamp.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&stamp, ctx.now_secs.to_string());
    let status = compare(current, &fetch().ok()?).ok()?;
    match status {
        UpdateStatus::Available { .. } => Some(render(&status)),
        UpdateStatus::UpToDate(_) => None,
    }
}

/// Startup notice for real runs: honours BASHRS_NO_UPDATE_CHECK, CI, non-tty;
/// cache under $XDG_CACHE_HOME or ~/.cache. Never fails or panics the command.
pub fn maybe_print_startup_notice() {
    use std::io::IsTerminal;
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache")))
        .map(|p| p.join("bashrs"));
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ctx = NoticeCtx {
        opted_out: std::env::var_os("BASHRS_NO_UPDATE_CHECK").is_some(),
        is_tty: std::io::stderr().is_terminal() && std::io::stdout().is_terminal(),
        ci: std::env::var_os("CI").is_some(),
        cache_dir,
        now_secs,
    };
    if let Some(line) = startup_notice_with(&ctx, CURRENT, fetch_latest_release_quick) {
        eprintln!("{line}");
    }
}

fn fetch_latest_release_quick() -> Result<String> {
    fetch_latest_release_with_timeout("3")
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

    fn ctx(dir: &std::path::Path, now: u64) -> NoticeCtx {
        NoticeCtx {
            opted_out: false,
            is_tty: true,
            ci: false,
            cache_dir: Some(dir.to_path_buf()),
            now_secs: now,
        }
    }

    fn ok(v: &'static str) -> impl FnOnce() -> Result<String> {
        move || Ok(v.to_string())
    }

    #[test]
    fn notice_when_newer_then_silent_for_a_day() {
        let d = tempfile::tempdir().unwrap();
        let n = startup_notice_with(&ctx(d.path(), 1000), "1.0.0", ok("v1.1.0")).unwrap();
        assert!(n.contains("1.1.0"));
        let again = startup_notice_with(&ctx(d.path(), 1000 + 3600), "1.0.0", || {
            panic!("must not fetch within a day")
        });
        assert!(again.is_none());
        let next = startup_notice_with(&ctx(d.path(), 1000 + 86_400), "1.0.0", ok("v1.1.0"));
        assert!(next.is_some());
    }

    #[test]
    fn up_to_date_is_silent_but_stamped() {
        let d = tempfile::tempdir().unwrap();
        assert!(startup_notice_with(&ctx(d.path(), 5), "1.1.0", ok("v1.1.0")).is_none());
        assert!(startup_notice_with(&ctx(d.path(), 6), "1.1.0", || panic!("fetched")).is_none());
    }

    #[test]
    fn fetch_failure_is_silent_and_stamped() {
        let d = tempfile::tempdir().unwrap();
        let fail = || Err(Error::Validation("offline".into()));
        assert!(startup_notice_with(&ctx(d.path(), 5), "1.0.0", fail).is_none());
        assert!(startup_notice_with(&ctx(d.path(), 6), "1.0.0", || panic!("fetched")).is_none());
    }

    #[test]
    fn opt_out_non_tty_ci_never_fetch() {
        let d = tempfile::tempdir().unwrap();
        for mutate in [
            (|c: &mut NoticeCtx| c.opted_out = true) as fn(&mut NoticeCtx),
            |c| c.is_tty = false,
            |c| c.ci = true,
        ] {
            let mut c = ctx(d.path(), 1);
            mutate(&mut c);
            let r = startup_notice_with(&c, "1.0.0", || panic!("fetched"));
            assert!(r.is_none());
        }
    }

    #[test]
    fn no_cache_dir_or_corrupt_stamp_never_fails() {
        let mut c = ctx(std::path::Path::new("/nonexistent/x/y"), 9);
        c.cache_dir = None;
        assert!(startup_notice_with(&c, "1.0.0", ok("v2.0.0")).is_none());
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join(STAMP_FILE), "garbage").unwrap();
        assert!(startup_notice_with(&ctx(d.path(), 9), "1.0.0", ok("v2.0.0")).is_some());
    }
}
