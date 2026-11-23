//! Issue #393: `bashrs --version` must name the commit it was built from.

use assert_cmd::Command;

#[test]
fn test_version_names_commit_393() {
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let first = s.lines().next().unwrap_or("");
    assert!(
        first.starts_with(&format!("bashrs {}", env!("CARGO_PKG_VERSION"))),
        "version line lost: {first}"
    );
    let sha = first
        .split("commit: ")
        .nth(1)
        .map(|r| r.trim_end_matches(')').trim())
        .unwrap_or_else(|| panic!("no `commit: ` in {first:?}"));
    let hex: String = sha.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    assert!(hex.len() >= 7, "commit must be >= 7 hex chars, got {sha:?}");
}
