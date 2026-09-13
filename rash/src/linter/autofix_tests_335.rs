//! bashrs#335 — `apply_fixes` spliced a rule's replacement into the line and
//! wrote the result back with nothing checking it was still the same program.
//! 7.3.0 and published 7.4.0 turned a single-quoted trap into a double-quoted
//! one and merged four arguments into one, and `bash -n` accepted both.
//!
//! Two properties a fix may not break, whichever rule emitted it:
//!   1. the words of each simple command stay the same in number, and
//!   2. no expansion that was literal text becomes an UNQUOTED expansion.
//! The trap rewrite keeps ONE word, so (1) alone cannot see it; (2) can.

use super::*;
use crate::linter::{Diagnostic, Fix, Severity};

/// The exact file from bashrs#335.
const ISSUE_335: &str = r#"#!/usr/bin/env bash
set -euo pipefail
TD=$(mktemp -d)
trap 'rm -rf -- "${TD:?}"' EXIT
assert_row() { local label="$1" want="$2" file="$3" needle="$4"; grep -q "$needle" "$file" && echo "ok $label" || echo "FAIL $label ($want)"; }
printf 'added=1\n' > "$TD/append.yaml"
assert_row 'append one entry' PASS "$TD/append.yaml" 'added=1'
"#;

fn one_fix(source: &str, span: Span, replacement: &str) -> FixResult {
    let mut result = LintResult::new();
    result.add(
        Diagnostic::new("TEST335", Severity::Info, "injected".to_string(), span)
            .with_fix(Fix::new(replacement)),
    );
    apply_fixes(source, &result, &FixOptions::default()).unwrap()
}

#[test]
fn test_335_net_refuses_a_safe_fix_that_merges_words() {
    // 7.4.0's rewrite of the assert_row line: it pairs the CLOSING quote of one
    // string (col 29) with the OPENING quote of the next (col 54). Five words
    // become two.
    let source = "assert_row 'append one entry' PASS \"$TD/append.yaml\" 'added=1'\n";
    let fixed = one_fix(source, Span::new(1, 29, 1, 55), "\" PASS \"$TD/append.yaml\" \"");
    assert_eq!(fixed.fixes_applied, 0, "a word-merging fix was applied");
    assert_eq!(fixed.modified_source.as_deref(), Some(source));
}

#[test]
fn test_335_net_refuses_a_safe_fix_that_unquotes_an_expansion() {
    // 7.4.0's rewrite of the trap line. Still one word -- but ${TD:?} goes from
    // literal text to an unquoted expansion evaluated at definition time.
    let source = "trap 'rm -rf -- \"${TD:?}\"' EXIT\n";
    let fixed = one_fix(source, Span::new(1, 6, 1, 27), "\"rm -rf -- \"${TD:?}\"\"");
    assert_eq!(fixed.fixes_applied, 0, "an unquoting fix was applied");
    assert_eq!(fixed.modified_source.as_deref(), Some(source));
}

#[test]
fn test_335_net_still_applies_a_fix_that_quotes_an_expansion() {
    // CONTROL. SC2086's `$DIR` -> `"$DIR"`: same words, and the expansion goes
    // from unquoted to quoted. A net that refused this would be blocking fixes,
    // not checking them.
    let source = "ls $DIR\n";
    let fixed = one_fix(source, Span::new(1, 4, 1, 8), "\"$DIR\"");
    assert_eq!(fixed.fixes_applied, 1);
    assert_eq!(fixed.modified_source.as_deref(), Some("ls \"$DIR\"\n"));
}

#[test]
fn test_335_default_fix_leaves_the_issue_file_the_same_program() {
    let result = crate::linter::lint_shell(ISSUE_335);
    let fixed = apply_fixes(ISSUE_335, &result, &FixOptions::default()).unwrap();
    let out = fixed.modified_source.expect("not a dry run");
    let before: Vec<&str> = ISSUE_335.lines().collect();
    let after: Vec<&str> = out.lines().collect();
    assert_eq!(after[3], before[3], "the trap line was rewritten");
    assert_eq!(after[6], before[6], "the assert_row line was rewritten");
}

/// The span `test_fix_priority_sc2046_coverage` hand-wrote until #335 (end 22)
/// runs three bytes past `$(cat file.txt)`, so its "quoting" fix also deleted
/// the space before `/dest`: `cp "$(cat file.txt)"est`. That test asserted
/// `contains`, which the corrupted line satisfies. A word merge, refused
/// whichever rule sends it.
#[test]
fn test_335_net_refuses_a_span_that_swallows_the_next_separator() {
    let source = "cp $(cat file.txt) /dest\n";
    let fixed = one_fix(source, Span::new(1, 4, 1, 22), "\"$(cat file.txt)\"");
    assert_eq!(fixed.fixes_applied, 0, "a separator-eating fix was applied");
    assert_ne!(
        fixed.modified_source.as_deref(),
        Some("cp \"$(cat file.txt)\"est\n")
    );
}
