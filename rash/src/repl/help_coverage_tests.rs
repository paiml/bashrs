#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::help::show_help;

// =============================================================================
// Coverage tests for repl/help.rs
// Targets: show_explain_help (via show_help), show_debug_help (via show_help)
// =============================================================================

// =============================================================================
// show_explain_help coverage (called via show_help(Some("explain")))
// =============================================================================

#[test]
fn test_explain_help_returns_nonempty_string() {
    let help = show_help(Some("explain"));
    assert!(!help.is_empty());
}

#[test]
fn test_explain_help_contains_title() {
    let help = show_help(Some("explain"));
    assert!(help.contains("BASH EXPLANATIONS"));
}

#[test]
fn test_explain_help_describes_what_gets_explained() {
    let help = show_help(Some("explain"));
    assert!(help.contains("WHAT GETS EXPLAINED"));
    assert!(help.contains("Parameter expansions"));
    assert!(help.contains("Control flow"));
    assert!(help.contains("Redirections"));
    assert!(help.contains("Special variables"));
    assert!(help.contains("Test expressions"));
}

#[test]
fn test_explain_help_shows_usage() {
    let help = show_help(Some("explain"));
    assert!(help.contains("USAGE"));
    assert!(help.contains(":explain"));
    assert!(help.contains(":mode explain"));
}

#[test]
fn test_explain_help_contains_example_output() {
    let help = show_help(Some("explain"));
    assert!(help.contains("EXAMPLE OUTPUT"));
    assert!(help.contains("${version:-1.0.0}"));
    assert!(help.contains("Parameter Expansion"));
    assert!(help.contains("Use Default Value"));
}

#[test]
fn test_explain_help_lists_supported_constructs() {
    let help = show_help(Some("explain"));
    assert!(help.contains("SUPPORTED CONSTRUCTS"));
    assert!(help.contains("${var:-default}"));
    assert!(help.contains("${var:=value}"));
    assert!(help.contains("${var#pattern}"));
    assert!(help.contains("${var%pattern}"));
    assert!(help.contains("for i in"));
    assert!(help.contains("while"));
    assert!(help.contains("if ["));
    assert!(help.contains("case $x in"));
}

#[test]
fn test_explain_help_contains_related_expansions() {
    let help = show_help(Some("explain"));
    assert!(help.contains("${parameter:=word}"));
    assert!(help.contains("${parameter:?error}"));
    assert!(help.contains("${parameter:+word}"));
}

#[test]
fn test_explain_help_has_try_suggestion() {
    let help = show_help(Some("explain"));
    assert!(help.contains("Try:"));
    assert!(help.contains("${HOME:-/tmp}"));
}

// =============================================================================
// show_debug_help coverage (called via show_help(Some("debug")))
// =============================================================================

#[test]
fn test_debug_help_returns_nonempty_string() {
    let help = show_help(Some("debug"));
    assert!(!help.is_empty());
}

#[test]
fn test_debug_help_contains_title() {
    let help = show_help(Some("debug"));
    assert!(help.contains("BASH DEBUGGING"));
    assert!(help.contains("Coming Soon"));
}

#[test]
fn test_debug_help_lists_planned_features() {
    let help = show_help(Some("debug"));
    assert!(help.contains("PLANNED FEATURES"));
    assert!(help.contains("breakpoints"));
    assert!(help.contains("Step through code"));
    assert!(help.contains("Inspect variables"));
    assert!(help.contains("call stack"));
}

#[test]
fn test_debug_help_lists_planned_commands() {
    let help = show_help(Some("debug"));
    assert!(help.contains("COMMANDS (Planned)"));
    assert!(help.contains(":break"));
    assert!(help.contains(":step"));
    assert!(help.contains(":next"));
    assert!(help.contains(":continue"));
    assert!(help.contains(":vars"));
    assert!(help.contains(":backtrace"));
}

#[test]
fn test_debug_help_shows_status() {
    let help = show_help(Some("debug"));
    assert!(help.contains("STATUS"));
    assert!(help.contains("under development"));
}

#[test]
fn test_debug_help_lists_workarounds() {
    let help = show_help(Some("debug"));
    assert!(help.contains("WORKAROUNDS"));
    assert!(help.contains("explain mode"));
    assert!(help.contains("purify mode"));
    assert!(help.contains("lint mode"));
    assert!(help.contains(":vars"));
}

#[test]
fn test_debug_help_has_suggestion() {
    let help = show_help(Some("debug"));
    assert!(help.contains("try:"));
    assert!(help.contains(":mode explain"));
}

// =============================================================================
// Additional coverage: verify explain and debug are valid topics
// (ensuring the match arms are exercised)
// =============================================================================

#[test]
fn test_explain_topic_is_distinct_from_unknown() {
    let explain_help = show_help(Some("explain"));
    let unknown_help = show_help(Some("explain_typo"));
    // explain returns real help, not "Unknown help topic"
    assert!(!explain_help.contains("Unknown help topic"));
    assert!(unknown_help.contains("Unknown help topic"));
}

#[test]
fn test_debug_topic_is_distinct_from_unknown() {
    let debug_help = show_help(Some("debug"));
    let unknown_help = show_help(Some("debug_typo"));
    assert!(!debug_help.contains("Unknown help topic"));
    assert!(unknown_help.contains("Unknown help topic"));
}

#[test]
fn test_explain_and_debug_are_different() {
    let explain_help = show_help(Some("explain"));
    let debug_help = show_help(Some("debug"));
    // They should be meaningfully different
    assert_ne!(explain_help, debug_help);
    assert!(explain_help.contains("BASH EXPLANATIONS"));
    assert!(debug_help.contains("BASH DEBUGGING"));
}

#[test]
fn test_explain_help_length_reasonable() {
    let help = show_help(Some("explain"));
    // Should be a substantial help text (at least 500 chars)
    assert!(help.len() > 500);
    // But not excessively long (less than 5000 chars)
    assert!(help.len() < 5000);
}

#[test]
fn test_debug_help_length_reasonable() {
    let help = show_help(Some("debug"));
    assert!(help.len() > 300);
    assert!(help.len() < 5000);
}
