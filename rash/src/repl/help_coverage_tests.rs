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

// =============================================================================
// show_history_help coverage (called via show_help(Some("history")))
// =============================================================================

#[test]
fn test_history_help_returns_nonempty_string() {
    let help = show_help(Some("history"));
    assert!(!help.is_empty());
}

#[test]
fn test_history_help_contains_title() {
    let help = show_help(Some("history"));
    assert!(help.contains("COMMAND HISTORY"));
}

#[test]
fn test_history_help_describes_features() {
    let help = show_help(Some("history"));
    assert!(help.contains("HISTORY FEATURES"));
    assert!(help.contains("Persistent across sessions"));
    assert!(help.contains("1000 commands"));
    assert!(help.contains("Duplicate commands filtered"));
}

#[test]
fn test_history_help_lists_keyboard_shortcuts() {
    let help = show_help(Some("history"));
    assert!(help.contains("KEYBOARD SHORTCUTS"));
    assert!(help.contains("Up arrow"));
    assert!(help.contains("Down arrow"));
    assert!(help.contains("Ctrl-R"));
    assert!(help.contains("Ctrl-S"));
}

#[test]
fn test_history_help_lists_commands() {
    let help = show_help(Some("history"));
    assert!(help.contains("COMMANDS"));
    assert!(help.contains(":history"));
}

#[test]
fn test_history_help_has_examples() {
    let help = show_help(Some("history"));
    assert!(help.contains("EXAMPLES"));
    assert!(help.contains("Browsing history"));
    assert!(help.contains("Searching history"));
    assert!(help.contains("Private commands"));
}

#[test]
fn test_history_help_describes_configuration() {
    let help = show_help(Some("history"));
    assert!(help.contains("CONFIGURATION"));
    assert!(help.contains("max_history"));
    assert!(help.contains("history_ignore_dups"));
    assert!(help.contains("history_ignore_space"));
}

#[test]
fn test_history_help_has_try_suggestion() {
    let help = show_help(Some("history"));
    assert!(help.contains("Try:"));
}

// =============================================================================
// show_variables_help coverage (called via show_help(Some("variables")))
// =============================================================================

#[test]
fn test_variables_help_returns_nonempty_string() {
    let help = show_help(Some("variables"));
    assert!(!help.is_empty());
}

#[test]
fn test_variables_help_contains_title() {
    let help = show_help(Some("variables"));
    assert!(help.contains("SESSION VARIABLES"));
}

#[test]
fn test_variables_help_describes_setting() {
    let help = show_help(Some("variables"));
    assert!(help.contains("SETTING VARIABLES"));
    assert!(help.contains("x=5"));
    assert!(help.contains("name=\"bashrs\""));
}

#[test]
fn test_variables_help_describes_using() {
    let help = show_help(Some("variables"));
    assert!(help.contains("USING VARIABLES"));
    assert!(help.contains("echo $x"));
    assert!(help.contains("${x}"));
    assert!(help.contains("${x:-default}"));
}

#[test]
fn test_variables_help_describes_viewing() {
    let help = show_help(Some("variables"));
    assert!(help.contains("VIEWING VARIABLES"));
    assert!(help.contains(":vars"));
}

#[test]
fn test_variables_help_describes_expansion() {
    let help = show_help(Some("variables"));
    assert!(help.contains("VARIABLE EXPANSION"));
}

#[test]
fn test_variables_help_lists_special_variables() {
    let help = show_help(Some("variables"));
    assert!(help.contains("SPECIAL VARIABLES"));
    assert!(help.contains("$?"));
    assert!(help.contains("$$"));
    assert!(help.contains("$!"));
    assert!(help.contains("$@"));
    assert!(help.contains("$#"));
}

#[test]
fn test_variables_help_lists_parameter_expansions() {
    let help = show_help(Some("variables"));
    assert!(help.contains("PARAMETER EXPANSIONS"));
    assert!(help.contains("${var:-default}"));
    assert!(help.contains("${var:=value}"));
    assert!(help.contains("${var#prefix}"));
    assert!(help.contains("${var##prefix}"));
    assert!(help.contains("${var%suffix}"));
    assert!(help.contains("${var%%suffix}"));
}

#[test]
fn test_variables_help_has_examples() {
    let help = show_help(Some("variables"));
    assert!(help.contains("EXAMPLES"));
    assert!(help.contains("version=1.0.0"));
}

#[test]
fn test_variables_help_via_alias() {
    let help = show_help(Some("vars"));
    assert!(help.contains("SESSION VARIABLES"));
}

// =============================================================================
// show_shortcuts_help coverage (called via show_help(Some("shortcuts")))
// =============================================================================

#[test]
fn test_shortcuts_help_returns_nonempty_string() {
    let help = show_help(Some("shortcuts"));
    assert!(!help.is_empty());
}

#[test]
fn test_shortcuts_help_contains_title() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("KEYBOARD SHORTCUTS"));
}

#[test]
fn test_shortcuts_help_lists_history_navigation() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("HISTORY NAVIGATION"));
    assert!(help.contains("Up arrow"));
    assert!(help.contains("Down arrow"));
    assert!(help.contains("Ctrl-R"));
}

#[test]
fn test_shortcuts_help_lists_line_editing() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("LINE EDITING"));
    assert!(help.contains("Ctrl-A"));
    assert!(help.contains("Ctrl-E"));
    assert!(help.contains("Ctrl-B"));
    assert!(help.contains("Ctrl-F"));
    assert!(help.contains("Alt-B"));
    assert!(help.contains("Alt-F"));
}

#[test]
fn test_shortcuts_help_lists_deleting() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("DELETING TEXT"));
    assert!(help.contains("Ctrl-H"));
    assert!(help.contains("Ctrl-D"));
    assert!(help.contains("Ctrl-K"));
    assert!(help.contains("Ctrl-U"));
    assert!(help.contains("Ctrl-W"));
    assert!(help.contains("Alt-D"));
}

#[test]
fn test_shortcuts_help_lists_multiline() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("MULTI-LINE INPUT"));
}

#[test]
fn test_shortcuts_help_lists_completion() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("COMPLETION"));
    assert!(help.contains("Tab"));
}

#[test]
fn test_shortcuts_help_lists_control() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("CONTROL"));
    assert!(help.contains("Ctrl-C"));
    assert!(help.contains("Ctrl-L"));
}

#[test]
fn test_shortcuts_help_has_examples() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("EXAMPLES"));
    assert!(help.contains("Editing a long command"));
    assert!(help.contains("Searching history"));
    assert!(help.contains("Multi-line function"));
}

#[test]
fn test_shortcuts_help_mentions_readline() {
    let help = show_help(Some("shortcuts"));
    assert!(help.contains("rustyline") || help.contains("Readline"));
}

#[test]
fn test_shortcuts_help_via_alias() {
    let help = show_help(Some("keys"));
    assert!(help.contains("KEYBOARD SHORTCUTS"));
}

// =============================================================================
// All topics produce distinct output
// =============================================================================

#[test]
fn test_all_help_topics_are_distinct() {
    let topics = vec![
        None,
        Some("commands"),
        Some("modes"),
        Some("purify"),
        Some("lint"),
        Some("parse"),
        Some("explain"),
        Some("debug"),
        Some("history"),
        Some("variables"),
        Some("shortcuts"),
    ];
    let outputs: Vec<String> = topics.iter().map(|t| show_help(t.as_deref())).collect();
    for i in 0..outputs.len() {
        for j in (i + 1)..outputs.len() {
            assert_ne!(
                outputs[i], outputs[j],
                "Topics {:?} and {:?} produced identical output",
                topics[i], topics[j]
            );
        }
    }
}
