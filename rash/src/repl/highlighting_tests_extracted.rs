#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-015-002-001 - Highlight keywords
    #[test]
    fn test_REPL_015_002_001_highlight_keywords() {
        let input = "if then else fi";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34melse\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
    }

    /// Test: REPL-015-002-002 - Highlight strings
    #[test]
    fn test_REPL_015_002_002_highlight_strings() {
        let input = r#"echo "hello world" 'single'"#;
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[32m\"hello world\"\x1b[0m"));
        assert!(highlighted.contains("\x1b[32m'single'\x1b[0m"));
    }

    /// Test: REPL-015-002-003 - Highlight variables
    #[test]
    fn test_REPL_015_002_003_highlight_variables() {
        let input = "echo $HOME ${USER} $?";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
        assert!(highlighted.contains("\x1b[33m${USER}\x1b[0m"));
        assert!(highlighted.contains("\x1b[33m$?\x1b[0m"));
    }

    /// Test: REPL-015-002-004 - Highlight commands
    #[test]
    fn test_REPL_015_002_004_highlight_commands() {
        let input = "mkdir -p /tmp";
        let highlighted = highlight_bash(input);

        // First word should be highlighted as command
        assert!(highlighted.contains("\x1b[36mmkdir\x1b[0m"));
    }

    /// Test: REPL-015-002-005 - Highlight comments
    #[test]
    fn test_REPL_015_002_005_highlight_comments() {
        let input = "echo hello # this is a comment";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[90m# this is a comment\x1b[0m"));
    }

    /// Test: REPL-015-002-006 - Highlight operators
    #[test]
    fn test_REPL_015_002_006_highlight_operators() {
        let input = "cat file | grep pattern && echo done";
        let highlighted = highlight_bash(input);

        assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
        assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
    }

    /// Test: REPL-015-002-007 - is_keyword function
    #[test]
    fn test_REPL_015_002_007_is_keyword() {
        assert!(is_keyword("if"));
        assert!(is_keyword("then"));
        assert!(is_keyword("while"));
        assert!(is_keyword("for"));
        assert!(is_keyword("do"));
        assert!(is_keyword("done"));

        assert!(!is_keyword("echo"));
        assert!(!is_keyword("test"));
        assert!(!is_keyword("hello"));
    }

    /// Test: REPL-015-002-INT-001 - Full bash statement
    #[test]
    fn test_REPL_015_002_INT_001_full_statement() {
        let input = r#"if [ -f "$file" ]; then echo "found"; fi"#;
        let highlighted = highlight_bash(input);

        // Should have keyword highlighting
        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));

        // Should have string highlighting
        assert!(highlighted.contains("\"$file\""));
        assert!(highlighted.contains("\"found\""));

        // Should have operator highlighting
        assert!(highlighted.contains("\x1b[35m;\x1b[0m"));
    }
}
