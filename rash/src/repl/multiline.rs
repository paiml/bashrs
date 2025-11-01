// REPL Multi-line Input Module
//
// Task: REPL-011 - Multi-line input support
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 15+ scenarios
// - Integration tests: Multi-line input in REPL
// - Mutation score: ≥90%
// - Complexity: <10 per function

/// Quote state for a bash input
#[derive(Debug, Clone, Copy)]
struct QuoteState {
    in_single_quote: bool,
    in_double_quote: bool,
}

impl QuoteState {
    fn new() -> Self {
        QuoteState {
            in_single_quote: false,
            in_double_quote: false,
        }
    }

    fn is_quoted(&self) -> bool {
        self.in_single_quote || self.in_double_quote
    }
}

/// Bracket/brace/paren depth tracking
#[derive(Debug, Clone, Copy)]
struct BracketState {
    brace_depth: i32,    // {}
    paren_depth: i32,    // ()
    bracket_depth: i32,  // []
}

impl BracketState {
    fn new() -> Self {
        BracketState {
            brace_depth: 0,
            paren_depth: 0,
            bracket_depth: 0,
        }
    }

    fn has_unclosed(&self) -> bool {
        self.brace_depth > 0 || self.paren_depth > 0 || self.bracket_depth > 0
    }
}

/// Check if input ends with backslash continuation
fn has_backslash_continuation(input: &str) -> bool {
    input.trim_end().ends_with('\\')
}

/// Analyze quote state throughout input
fn analyze_quote_state(input: &str) -> QuoteState {
    let mut state = QuoteState::new();
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '\'' if !state.in_double_quote => state.in_single_quote = !state.in_single_quote,
            '"' if !state.in_single_quote => state.in_double_quote = !state.in_double_quote,
            _ => {}
        }
    }

    state
}

/// Analyze bracket/brace/paren depth, respecting quote context
fn analyze_bracket_state(input: &str) -> BracketState {
    let mut state = BracketState::new();
    let mut quote_state = QuoteState::new();
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '\'' if !quote_state.in_double_quote => quote_state.in_single_quote = !quote_state.in_single_quote,
            '"' if !quote_state.in_single_quote => quote_state.in_double_quote = !quote_state.in_double_quote,
            '{' if !quote_state.is_quoted() => state.brace_depth += 1,
            '}' if !quote_state.is_quoted() => state.brace_depth -= 1,
            '(' if !quote_state.is_quoted() => state.paren_depth += 1,
            ')' if !quote_state.is_quoted() => state.paren_depth -= 1,
            '[' if !quote_state.is_quoted() => state.bracket_depth += 1,
            ']' if !quote_state.is_quoted() => state.bracket_depth -= 1,
            _ => {}
        }
    }

    state
}

/// Check if bash keywords expect continuation
fn bash_keywords_need_continuation(input: &str) -> bool {
    let trimmed = input.trim();

    // Keywords that expect a closing keyword
    if trimmed.starts_with("if ") && !trimmed.contains(" fi") {
        return true;
    }

    if trimmed.starts_with("for ") && !trimmed.contains(" done") {
        return true;
    }

    if trimmed.starts_with("while ") && !trimmed.contains(" done") {
        return true;
    }

    if trimmed.starts_with("until ") && !trimmed.contains(" done") {
        return true;
    }

    if trimmed.starts_with("case ") && !trimmed.contains(" esac") {
        return true;
    }

    if (trimmed.starts_with("function ") || trimmed.contains("() {")) && !trimmed.ends_with('}') {
        return true;
    }

    // Keywords that expect a following block
    if trimmed.ends_with(" then") || trimmed.ends_with(" do") {
        return true;
    }

    false
}

/// Detects if a bash input line is incomplete and needs continuation
///
/// An input is considered incomplete if it has:
/// - Unclosed quotes (single or double)
/// - Unclosed braces, parentheses, or brackets
/// - Bash keywords that expect a block (if, for, while, function, case)
/// - Line ending with backslash continuation
///
/// # Examples
///
/// ```
/// use bashrs::repl::multiline::is_incomplete;
///
/// assert!(is_incomplete("for i in 1 2 3; do"));
/// assert!(is_incomplete("if [ -f file ]; then"));
/// assert!(is_incomplete("function greet() {"));
/// assert!(!is_incomplete("echo hello"));
/// ```
pub fn is_incomplete(input: &str) -> bool {
    // Check for backslash line continuation
    if has_backslash_continuation(input) {
        return true;
    }

    // Check quote state
    let quote_state = analyze_quote_state(input);
    if quote_state.is_quoted() {
        return true;
    }

    // Check bracket/brace/paren depth
    let bracket_state = analyze_bracket_state(input);
    if bracket_state.has_unclosed() {
        return true;
    }

    // Check bash keywords
    bash_keywords_need_continuation(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Write failing tests first =====

    #[test]
    fn test_REPL_011_complete_simple_command() {
        assert!(!is_incomplete("echo hello"));
        assert!(!is_incomplete("ls -la"));
        assert!(!is_incomplete("pwd"));
    }

    #[test]
    fn test_REPL_011_incomplete_single_quote() {
        assert!(is_incomplete("echo 'hello"));
        assert!(is_incomplete("echo 'hello world"));
    }

    #[test]
    fn test_REPL_011_complete_single_quote() {
        assert!(!is_incomplete("echo 'hello'"));
        assert!(!is_incomplete("echo 'hello world'"));
    }

    #[test]
    fn test_REPL_011_incomplete_double_quote() {
        assert!(is_incomplete("echo \"hello"));
        assert!(is_incomplete("echo \"hello world"));
    }

    #[test]
    fn test_REPL_011_complete_double_quote() {
        assert!(!is_incomplete("echo \"hello\""));
        assert!(!is_incomplete("echo \"hello world\""));
    }

    #[test]
    fn test_REPL_011_incomplete_braces() {
        assert!(is_incomplete("function greet() {"));
        assert!(is_incomplete("if true; then {"));
        assert!(is_incomplete("{ echo hello"));
    }

    #[test]
    fn test_REPL_011_complete_braces() {
        assert!(!is_incomplete("function greet() { echo hi; }"));
        assert!(!is_incomplete("{ echo hello; }"));
    }

    #[test]
    fn test_REPL_011_incomplete_parentheses() {
        assert!(is_incomplete("echo (hello"));
        assert!(is_incomplete("if ( test"));
    }

    #[test]
    fn test_REPL_011_complete_parentheses() {
        assert!(!is_incomplete("echo (hello)"));
        assert!(!is_incomplete("(echo hello)"));
    }

    #[test]
    fn test_REPL_011_incomplete_brackets() {
        assert!(is_incomplete("if [ -f file"));
        assert!(is_incomplete("test [ condition"));
    }

    #[test]
    fn test_REPL_011_complete_brackets() {
        assert!(!is_incomplete("if [ -f file ]; then echo yes; fi"));
        assert!(!is_incomplete("[ -f file ]"));
    }

    #[test]
    fn test_REPL_011_incomplete_if_statement() {
        assert!(is_incomplete("if [ -f file ]; then"));
        assert!(is_incomplete("if true"));
    }

    #[test]
    fn test_REPL_011_complete_if_statement() {
        assert!(!is_incomplete("if [ -f file ]; then echo yes; fi"));
    }

    #[test]
    fn test_REPL_011_incomplete_for_loop() {
        assert!(is_incomplete("for i in 1 2 3; do"));
        assert!(is_incomplete("for i in 1 2 3"));
    }

    #[test]
    fn test_REPL_011_complete_for_loop() {
        assert!(!is_incomplete("for i in 1 2 3; do echo $i; done"));
    }

    #[test]
    fn test_REPL_011_incomplete_while_loop() {
        assert!(is_incomplete("while true; do"));
        assert!(is_incomplete("while [ -f file ]"));
    }

    #[test]
    fn test_REPL_011_complete_while_loop() {
        assert!(!is_incomplete("while true; do echo hi; done"));
    }

    #[test]
    fn test_REPL_011_incomplete_function() {
        assert!(is_incomplete("function greet() {"));
        assert!(is_incomplete("greet() {"));
    }

    #[test]
    fn test_REPL_011_complete_function() {
        assert!(!is_incomplete("function greet() { echo hello; }"));
        assert!(!is_incomplete("greet() { echo hello; }"));
    }

    #[test]
    fn test_REPL_011_incomplete_case_statement() {
        assert!(is_incomplete("case $var in"));
        assert!(is_incomplete("case $var"));
    }

    #[test]
    fn test_REPL_011_complete_case_statement() {
        assert!(!is_incomplete("case $var in foo) echo bar;; esac"));
    }

    #[test]
    fn test_REPL_011_incomplete_backslash_continuation() {
        assert!(is_incomplete("echo hello \\"));
        assert!(is_incomplete("ls -la \\"));
    }

    #[test]
    fn test_REPL_011_escaped_quote_not_incomplete() {
        assert!(!is_incomplete("echo \\'hello\\'"));
        assert!(!is_incomplete("echo \\\"hello\\\""));
    }

    #[test]
    fn test_REPL_011_nested_quotes() {
        assert!(is_incomplete("echo \"hello 'world"));
        assert!(!is_incomplete("echo \"hello 'world'\""));
    }

    #[test]
    fn test_REPL_011_multiple_statements() {
        assert!(!is_incomplete("echo hello; echo world"));
        assert!(!is_incomplete("ls -la && pwd"));
    }

    // ===== PROPERTY TESTS: Establish baseline behavior =====

    /// Property: Complete commands never need continuation
    #[test]
    fn prop_complete_commands_not_incomplete() {
        let complete_commands = vec![
            "echo hello",
            "ls -la",
            "pwd",
            "cd /tmp",
            "mkdir test",
            "rm file",
            "cat file.txt",
            "grep pattern file",
            "sed 's/old/new/' file",
            "awk '{print $1}' file",
        ];

        for cmd in complete_commands {
            assert!(!is_incomplete(cmd), "Command should be complete: {}", cmd);
        }
    }

    /// Property: Unclosed quotes always incomplete
    #[test]
    fn prop_unclosed_quotes_always_incomplete() {
        let unclosed_single = vec![
            "echo 'hello",
            "msg='test",
            "echo 'world",
        ];

        let unclosed_double = vec![
            "echo \"hello",
            "msg=\"test",
            "echo \"world",
        ];

        for cmd in unclosed_single {
            assert!(is_incomplete(cmd), "Unclosed single quote should be incomplete: {}", cmd);
        }

        for cmd in unclosed_double {
            assert!(is_incomplete(cmd), "Unclosed double quote should be incomplete: {}", cmd);
        }
    }

    /// Property: Balanced quotes always complete (if no other issues)
    #[test]
    fn prop_balanced_quotes_complete() {
        let balanced = vec![
            "echo 'hello'",
            "echo \"world\"",
            "echo 'test' \"foo\"",
            "msg='value'",
            "path=\"/tmp/file\"",
        ];

        for cmd in balanced {
            assert!(!is_incomplete(cmd), "Balanced quotes should be complete: {}", cmd);
        }
    }

    /// Property: Unclosed braces always incomplete
    #[test]
    fn prop_unclosed_braces_incomplete() {
        let unclosed = vec![
            "function foo {",
            "if true; then {",
            "{ echo hello",
            "while true; do {",
        ];

        for cmd in unclosed {
            assert!(is_incomplete(cmd), "Unclosed brace should be incomplete: {}", cmd);
        }
    }

    /// Property: Balanced braces complete (if no other issues)
    #[test]
    fn prop_balanced_braces_complete() {
        let balanced = vec![
            "{ echo hello; }",
            "function foo { echo hi; }",
        ];

        for cmd in balanced {
            assert!(!is_incomplete(cmd), "Balanced braces should be complete: {}", cmd);
        }
    }

    /// Property: Unclosed parentheses always incomplete
    #[test]
    fn prop_unclosed_parens_incomplete() {
        let unclosed = vec![
            "echo (hello",
            "if ( test",
            "result=$(echo test",
        ];

        for cmd in unclosed {
            assert!(is_incomplete(cmd), "Unclosed paren should be incomplete: {}", cmd);
        }
    }

    /// Property: Balanced parentheses complete (if no other issues)
    #[test]
    fn prop_balanced_parens_complete() {
        let balanced = vec![
            "echo (hello)",
            "(echo test)",
            "result=$(echo done)",
        ];

        for cmd in balanced {
            assert!(!is_incomplete(cmd), "Balanced parens should be complete: {}", cmd);
        }
    }

    /// Property: Unclosed brackets always incomplete
    #[test]
    fn prop_unclosed_brackets_incomplete() {
        let unclosed = vec![
            "if [ -f file",
            "test [ condition",
            "[ -z $var",
        ];

        for cmd in unclosed {
            assert!(is_incomplete(cmd), "Unclosed bracket should be incomplete: {}", cmd);
        }
    }

    /// Property: Keywords expecting blocks are incomplete without closing
    #[test]
    fn prop_keywords_incomplete_without_close() {
        let incomplete_keywords = vec![
            ("if true; then", " fi"),
            ("for i in 1 2 3; do", " done"),
            ("while true; do", " done"),
            ("until false; do", " done"),
            ("case $var in", " esac"),
        ];

        for (start, _close) in incomplete_keywords {
            assert!(is_incomplete(start), "Keyword should need continuation: {}", start);
        }
    }

    /// Property: Backslash continuation always incomplete
    #[test]
    fn prop_backslash_continuation_incomplete() {
        let continuations = vec![
            "echo hello \\",
            "ls -la \\",
            "cat file \\",
            "grep pattern \\",
        ];

        for cmd in continuations {
            assert!(is_incomplete(cmd), "Backslash continuation should be incomplete: {}", cmd);
        }
    }

    /// Property: Complete control structures not incomplete
    #[test]
    fn prop_complete_control_structures() {
        let complete = vec![
            "if true; then echo yes; fi",
            "for i in 1 2 3; do echo $i; done",
            "while true; do echo hi; done",
            "until false; do echo wait; done",
            "case $x in foo) echo bar;; esac",
        ];

        for cmd in complete {
            assert!(!is_incomplete(cmd), "Complete control structure should not be incomplete: {}", cmd);
        }
    }

    /// Property: Quotes inside quotes handled correctly
    #[test]
    fn prop_nested_quotes() {
        // Single quotes inside double quotes don't need escaping
        assert!(!is_incomplete("echo \"hello 'world'\""));

        // Double quotes inside single quotes don't need escaping
        assert!(!is_incomplete("echo 'hello \"world\"'"));

        // Unclosed outer quote makes it incomplete
        assert!(is_incomplete("echo \"hello 'world"));
        assert!(is_incomplete("echo 'hello \"world"));
    }

    /// Property: Empty input is complete
    #[test]
    fn prop_empty_complete() {
        assert!(!is_incomplete(""));
        assert!(!is_incomplete("   "));
        assert!(!is_incomplete("\t"));
    }

    /// Property: Comments are complete
    #[test]
    fn prop_comments_complete() {
        let comments = vec![
            "# this is a comment",
            "  # indented comment",
            "echo test # inline comment",
        ];

        for cmd in comments {
            assert!(!is_incomplete(cmd), "Comment should be complete: {}", cmd);
        }
    }
}
