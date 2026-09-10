//! GH-265: `# shellcheck disable=` must honour only SC-numbered codes, and a
//! `# bashrs disable-line=` written on a comment-only line must be reported
//! instead of silently doing nothing.
use super::*;

fn codes(source: &str) -> Vec<String> {
    crate::linter::lint_shell(source)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

fn messages(source: &str) -> Vec<String> {
    crate::linter::lint_shell(source)
        .diagnostics
        .iter()
        .map(|d| d.message.clone())
        .collect()
}

/// s1: a bashrs-native code inside `# shellcheck disable=` is NOT honoured —
/// DET002 still fires — and a BASHRS001 diagnostic names the fix.
#[test]
fn test_PMAT255_gh265_shellcheck_directive_does_not_honour_bashrs_code() {
    let source = "#!/bin/bash\n# shellcheck disable=DET002\necho \"$(date)\" > VERSION\n";
    let found = codes(source);
    assert!(
        found.iter().filter(|c| *c == "DET002").count() >= 1,
        "DET002 must still fire: {found:?}"
    );
    let msgs = messages(source);
    assert!(
        msgs.iter()
            .any(|m| m.to_lowercase().contains("bashrs disable")),
        "expected a diagnostic pointing at the bashrs directive: {msgs:?}"
    );
}

/// s2: `# shellcheck disable=SC2086` (a real shellcheck code) keeps working.
#[test]
fn test_PMAT255_gh265_shellcheck_directive_still_honours_sc_code() {
    let source = "#!/bin/bash\nv=\"a b\"\n# shellcheck disable=SC2086\nls $v\n";
    let found = codes(source);
    assert!(
        !found.contains(&"SC2086".to_string()),
        "SC2086 must stay suppressed: {found:?}"
    );
}

/// s3: `# bashrs disable-line=DET002` on the line ABOVE the code does
/// nothing to DET002 (unchanged placement semantics) but must now be
/// reported as inert, mentioning `disable-line`.
#[test]
fn test_PMAT255_gh265_disable_line_on_preceding_line_is_reported() {
    let source = "#!/bin/bash\n# bashrs disable-line=DET002\necho \"$(date)\" > VERSION\n";
    let found = codes(source);
    assert!(
        found.iter().filter(|c| *c == "DET002").count() >= 1,
        "placement semantics must not change — DET002 still fires: {found:?}"
    );
    let msgs = messages(source);
    assert!(
        msgs.iter()
            .any(|m| m.to_lowercase().contains("disable-line")),
        "expected a diagnostic naming disable-line: {msgs:?}"
    );
}

/// s4: same-line `# bashrs disable-line=DET002` keeps working and is not
/// reported as inert.
#[test]
fn test_PMAT255_gh265_disable_line_same_line_unaffected() {
    let source = "#!/bin/bash\necho \"$(date)\" > VERSION # bashrs disable-line=DET002\n";
    let found = codes(source);
    assert!(
        !found.contains(&"DET002".to_string()),
        "same-line disable-line must keep suppressing: {found:?}"
    );
    assert!(
        !found.contains(&"BASHRS001".to_string()),
        "a working directive must not be reported as inert: {found:?}"
    );
}

/// s5: `# bashrs disable-file=DET002` keeps working.
#[test]
fn test_PMAT255_gh265_disable_file_unaffected() {
    let source = "#!/bin/bash\n# bashrs disable-file=DET002\necho \"$(date)\" > VERSION\n";
    let found = codes(source);
    assert!(
        !found.contains(&"DET002".to_string()),
        "disable-file must keep suppressing: {found:?}"
    );
}

/// A mixed `# shellcheck disable=SC2086,DET002` honours SC2086 and reports
/// DET002.
#[test]
fn test_PMAT255_gh265_mixed_shellcheck_directive_splits_codes() {
    let source =
        "#!/bin/bash\nv=\"a b\"\n# shellcheck disable=SC2086,DET002\nls $v\necho \"$(date)\" > VERSION\n";
    let found = codes(source);
    assert!(
        !found.contains(&"SC2086".to_string()),
        "SC2086 must stay suppressed in a mixed directive: {found:?}"
    );
    assert!(
        found.iter().filter(|c| *c == "DET002").count() >= 1,
        "DET002 must still fire in a mixed directive: {found:?}"
    );
}
