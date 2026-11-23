//! GH-336: a `SOURCE_DATE_EPOCH` read whose fallback is itself the wall clock
//! (`${SOURCE_DATE_EPOCH:-$(date +%s)}`) is exactly as non-reproducible as a
//! bare `date` whenever the variable is unset, so it must not clear DET002.

#![allow(non_snake_case)]

use super::check;

#[test]
fn test_GH336_wallclock_fallback_still_flagged() {
    let s = "NOW=$(date -u -d \"@${SOURCE_DATE_EPOCH:-$(date +%s)}\" +%Y-%m-%dT%H:%M:%SZ)\n\
cp build.log \"out/receipt-$NOW.json\"\n";
    assert!(
        check(s).diagnostics.iter().any(|d| d.code == "DET002"),
        "wall-clock fallback must still fire"
    );
}

#[test]
fn test_GH336_git_fallback_clean() {
    let s = "NOW=$(date -u -d \"@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}\" +%Y%m%d)\n\
cp build.log \"out/receipt-$NOW.json\"\n";
    assert!(check(s).diagnostics.is_empty());
}

#[test]
fn test_GH336_plain_sde_read_clean() {
    let s = "NOW=$(date -u -d \"@$SOURCE_DATE_EPOCH\" +%Y%m%d)\n\
cp build.log \"out/receipt-$NOW.json\"\n";
    assert!(check(s).diagnostics.is_empty());
}
