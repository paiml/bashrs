//! GH-264: SEC011 must recognise an early-return guard (`<test> || exit`, and
//! its negated `&&`-exit twin) as establishing the same precondition the
//! accepted `if` form does, and must keep reporting when there is no guard.

#![allow(non_snake_case)]

use super::*;

#[test]
fn test_PMAT251_gh264_or_exit_guard_is_recognised() {
    // Issue #264 flagged reproducer, verbatim.
    let script = r#"#!/bin/bash
WORK="$(mktemp -d)"
[ -n "$WORK" ] || exit 1
rm -rf "$WORK"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "`[ -n \"$WORK\" ] || exit 1` establishes WORK is non-empty before `rm -rf`"
    );
}

#[test]
fn test_PMAT251_gh264_test_n_or_exit_guard_is_recognised() {
    // `test` is a synonym for `[` and must be accepted the same way.
    let script = r#"#!/bin/bash
WORK="$(mktemp -d)"
test -n "$WORK" || exit 1
rm -rf "$WORK"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "`test -n \"$WORK\" || exit 1` is the `test` spelling of the same guard"
    );
}

/// Issue #264's guard table: `[ -d ] || exit` and the negated `&&`-exit form
/// must also be recognised - reachability, not syntax, is what SEC011 checks.
#[test]
fn test_PMAT251_gh264_dash_d_or_exit_and_negated_and_exit_are_recognised() {
    let dash_d = r#"#!/bin/bash
WORK="$(mktemp -d)"
[ -d "$WORK" ] || exit 1
rm -rf "$WORK"
"#;
    assert_eq!(
        check(dash_d).diagnostics.len(),
        0,
        "`[ -d \"$WORK\" ] || exit 1` also validates WORK"
    );

    let negated_and = r#"#!/bin/bash
WORK="$(mktemp -d)"
[[ -z "$WORK" ]] && exit 1
rm -rf "$WORK"
"#;
    assert_eq!(
        check(negated_and).diagnostics.len(),
        0,
        "`[[ -z \"$WORK\" ]] && exit 1` is the negated twin of the `||` guard"
    );
}

#[test]
fn test_PMAT251_gh264_unguarded_rm_rf_is_still_reported() {
    // The rule must not be defeated by simply removing the guard.
    let script = r#"#!/bin/bash
WORK="$(mktemp -d)"
rm -rf "$WORK"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "an unguarded `rm -rf \"$WORK\"` must still be reported"
    );
    assert_eq!(result.diagnostics[0].code, "SEC011");
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}
