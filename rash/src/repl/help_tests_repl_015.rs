#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Write failing tests first =====

    #[test]
    fn test_repl_015_004_general_help_contains_overview() {
        let help = show_help(None);
        assert!(help.contains("bashrs REPL"));
        assert!(help.contains("OVERVIEW"));
        assert!(help.contains("Purifying") || help.contains("purifying"));
    }

    #[test]
    fn test_repl_015_004_commands_help_lists_all_commands() {
        let help = show_help(Some("commands"));
        assert!(help.contains(":mode"));
        assert!(help.contains(":purify"));
        assert!(help.contains(":lint"));
        assert!(help.contains(":load"));
        assert!(help.contains(":help"));
    }

    #[test]
    fn test_repl_015_004_modes_help_explains_modes() {
        let help = show_help(Some("modes"));
        assert!(help.contains("NORMAL MODE"));
        assert!(help.contains("PURIFY MODE"));
        assert!(help.contains("LINT MODE"));
        assert!(help.contains("EXPLAIN MODE"));
        assert!(help.contains("DEBUG MODE"));
    }

    #[test]
    fn test_repl_015_004_purify_help_explains_purification() {
        let help = show_help(Some("purify"));
        assert!(help.contains("DETERMINISM"));
        assert!(help.contains("IDEMPOTENCY"));
        assert!(help.contains("mkdir -p"));
        assert!(help.contains("rm -f"));
    }

    #[test]
    fn test_repl_015_004_unknown_topic_shows_error() {
        let help = show_help(Some("nonexistent"));
        assert!(help.contains("Unknown help topic"));
        assert!(help.contains("nonexistent"));
        assert!(help.contains("Available topics"));
    }

    #[test]
    fn test_repl_015_004_shortcuts_help_lists_keybindings() {
        let help = show_help(Some("shortcuts"));
        assert!(help.contains("Ctrl-R"));
        assert!(help.contains("Up arrow"));
        assert!(help.contains("Ctrl-A"));
        assert!(help.contains("HISTORY NAVIGATION"));
    }

    #[test]
    fn test_repl_015_004_history_help_mentions_ctrl_r() {
        let help = show_help(Some("history"));
        assert!(help.contains("Ctrl-R"));
        assert!(help.contains("Reverse search"));
        assert!(help.contains(".bashrs_history"));
    }

    #[test]
    fn test_repl_015_004_variables_help_explains_expansions() {
        let help = show_help(Some("variables"));
        assert!(help.contains("x=5"));
        assert!(help.contains("${var:-default}"));
        assert!(help.contains(":vars"));
        assert!(help.contains("PARAMETER EXPANSIONS"));
    }

    #[test]
    fn test_repl_015_004_lint_help_categorizes_issues() {
        let help = show_help(Some("lint"));
        assert!(help.contains("SECURITY ISSUES"));
        assert!(help.contains("DETERMINISM ISSUES"));
        assert!(help.contains("IDEMPOTENCY ISSUES"));
        assert!(help.contains("SEC-"));
        assert!(help.contains("DET-"));
    }

    #[test]
    fn test_repl_015_004_parse_help_explains_ast() {
        let help = show_help(Some("parse"));
        assert!(help.contains("Abstract Syntax Tree"));
        assert!(help.contains("AST"));
        assert!(help.contains(":parse"));
    }

    #[test]
    fn test_repl_015_004_explain_help_covers_constructs() {
        let help = show_help(Some("explain"));
        assert!(help.contains("BASH EXPLANATIONS"));
        assert!(help.contains(":explain"));
        assert!(help.contains("Parameter Expansion"));
        assert!(help.contains("${var:-default}"));
        assert!(help.contains("for i in"));
        assert!(help.contains("while"));
        assert!(help.contains("case $x in"));
    }

    #[test]
    fn test_repl_015_004_debug_help() {
        let help = show_help(Some("debug"));
        assert!(help.contains("DEBUGGING"));
    }
}
