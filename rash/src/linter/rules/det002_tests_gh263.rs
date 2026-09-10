//! GH-263: DET002 must detect a bare `date` command word - not only `date`
//! inside `$( )`/backticks - when its value reaches a reproducible sink
//! (a truncating/appending redirect, or a pipe into `tee`), while a bare
//! `date` that only prints to the terminal stays unreported (that's
//! `#232`/DET005 territory, not this rule).

#![allow(non_snake_case)]

use super::check;

#[test]
fn test_PMAT251_gh263_bare_date_redirected_to_file_is_detected() {
    // Issue #263 reproducer A, verbatim.
    let script = "#!/bin/bash\ndate > VERSION\n";
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "`date > VERSION` writes the current time into a tracked file"
    );
    assert_eq!(result.diagnostics[0].code, "DET002");

    // The `-u +%Y` spelling from the issue's scope table must also be caught.
    let script2 = "#!/bin/bash\ndate -u +%Y > VERSION\n";
    assert_eq!(
        check(script2).diagnostics.len(),
        1,
        "`date -u +%Y > VERSION` is the same defect with a format flag"
    );
}

#[test]
fn test_PMAT251_gh263_bare_date_appended_or_teed_is_detected() {
    let appended = "#!/bin/bash\ndate >> VERSION\n";
    assert_eq!(
        check(appended).diagnostics.len(),
        1,
        "appending a bare `date` into a non-log artifact is still a reproducibility defect"
    );

    let teed = "#!/bin/bash\ndate | tee VERSION\n";
    let result = check(teed);
    assert_eq!(
        result.diagnostics.len(),
        1,
        "`date | tee VERSION` pipes the timestamp into a reproducible sink"
    );
    assert_eq!(result.diagnostics[0].code, "DET002");
}

#[test]
fn test_PMAT251_gh263_date_to_terminal_is_not_reported() {
    // Issue #263 reproducer C: a bare `date` with no redirect or pipe just
    // prints to the terminal. No sink, nothing to report.
    let script = "#!/bin/bash\ndate\n";
    assert_eq!(
        check(script).diagnostics.len(),
        0,
        "a bare `date` printed to the terminal is not a DET002 defect"
    );
}

#[test]
fn test_PMAT251_gh263_substituted_date_still_detected() {
    // Issue #263 reproducer B, verbatim - the previously-working spelling
    // must keep working after generalising bare-command detection.
    let script = "#!/bin/bash\necho \"$(date)\" > VERSION\n";
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "`echo \"$(date)\" > VERSION` must still be detected"
    );
    assert_eq!(result.diagnostics[0].code, "DET002");
}
