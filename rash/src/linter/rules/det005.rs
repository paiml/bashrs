//! DET005: Time-dependent control flow (#232, split from DET002)
//!
//! **Rule**: A wall-clock value is a scheduling hazard - not a reproducibility
//! defect - when it reaches a *branch condition* (`if`/`elif`/`while`/`until`,
//! a `case` selector, or a bare `[ ]`/`[[ ]]`/`((` test) rather than an
//! artifact. #230/DET002 correctly stopped reporting a timestamp that is only
//! ever compared, because nothing there reaches a build artifact. But a
//! script that branches on the wall clock behaves differently depending on
//! *when* it runs - not testable by replay, and a classic source of "works on
//! my machine, fails at 00:00 UTC". That is a distinct defect from
//! reproducibility, so it gets its own rule and its own remedy.
//!
//! The destination analysis lives in [`crate::linter::timestamp_flow`];
//! DET005 fires on exactly `SinkClass::Conditional`, the polarity DET002
//! excludes, so the two rules never double-report the same line.
//!
//! **Not flagged**: a duration - the arithmetic difference of two timestamp
//! captures (`elapsed=$(( end - start ))`) - never reaches a branch
//! condition, so it is `SinkClass::Benign` here too. Measuring elapsed time
//! is the point of measuring elapsed time.
//!
//! **Decision (#232's "debatable" case)**: a timeout/backoff loop that
//! compares against a deadline computed earlier in the same script (`while [
//! "$(date +%s)" -lt "$deadline" ]; do ... done`) still reports DET005. The
//! loop's *termination* depends on wall-clock time, which is exactly the
//! hazard this rule names (a test can hang or race depending on when it
//! runs); a duration computation never branches at all, so it stays exempt.
//! Documented in `book/src/linting/determinism.md`.
//!
//! Suggested severity: **Warning**, not Error - this is often deliberate
//! (cron-style scripts, backoff loops), and an unactionable Error gets the
//! rule disabled rather than the code fixed (#227/#230's lesson).
//!
//! **Auto-fix**: none - the remedy is design-level (inject the deadline as a
//! parameter, or accept the non-determinism deliberately).

use crate::linter::timestamp_flow::{analyze, SinkClass, TimestampUse};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for a wall-clock value reaching a branch condition (#232).
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for u in analyze(source) {
        if u.class == SinkClass::Conditional {
            result.add(build_diagnostic(&u));
        }
    }
    result
}

/// Build the diagnostic for one reportable timestamp.
///
/// The span stays on the `date` occurrence, matching DET002's convention:
/// `# bashrs disable-line=DET005` and `.bashrsignore` are keyed on
/// `span.start_line`.
fn build_diagnostic(u: &TimestampUse) -> Diagnostic {
    let span = Span::new(u.line, u.col, u.line, u.col + u.len);
    Diagnostic::new("DET005", Severity::Warning, message_for(u), span)
}

/// Message text, naming the captured variable when we have one.
fn message_for(u: &TimestampUse) -> String {
    let captured = match u.var.as_deref() {
        Some(v) => format!("Timestamp captured in `${v}`"),
        None => "Timestamp".to_string(),
    };
    format!(
        "{captured} reaches a branch condition; this script's behaviour depends on when it \
         runs, not just its input (#232). If that is deliberate (cron-style scheduling, a \
         backoff loop), suppress with `# bashrs disable-line=DET005`; otherwise inject the \
         deadline as a parameter."
    )
}

#[cfg(test)]
#[path = "det005_tests.rs"]
mod det005_tests;
