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
    if input.trim_end().ends_with('\\') {
        return true;
    }

    // Track quote state
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    // Track bracket/brace/paren depth
    let mut brace_depth = 0; // {}
    let mut paren_depth = 0; // ()
    let mut bracket_depth = 0; // []

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '{' if !in_single_quote && !in_double_quote => {
                brace_depth += 1;
            }
            '}' if !in_single_quote && !in_double_quote => {
                brace_depth -= 1;
            }
            '(' if !in_single_quote && !in_double_quote => {
                paren_depth += 1;
            }
            ')' if !in_single_quote && !in_double_quote => {
                paren_depth -= 1;
            }
            '[' if !in_single_quote && !in_double_quote => {
                bracket_depth += 1;
            }
            ']' if !in_single_quote && !in_double_quote => {
                bracket_depth -= 1;
            }
            _ => {}
        }
    }

    // Input is incomplete if:
    // - Inside quotes
    if in_single_quote || in_double_quote {
        return true;
    }

    // - Unclosed braces, parens, or brackets
    if brace_depth > 0 || paren_depth > 0 || bracket_depth > 0 {
        return true;
    }

    // Check for bash keywords that expect continuation
    let trimmed = input.trim();

    // Keywords that expect a block
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

    // Check for keywords that expect continuation
    if trimmed.ends_with(" then") || trimmed.ends_with(" do") {
        return true;
    }

    if trimmed.ends_with("\\") {
        return true;
    }

    false
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
}
