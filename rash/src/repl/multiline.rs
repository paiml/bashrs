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
    brace_depth: i32,   // {}
    paren_depth: i32,   // ()
    bracket_depth: i32, // []
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
            '\'' if !quote_state.in_double_quote => {
                quote_state.in_single_quote = !quote_state.in_single_quote;
            }
            '"' if !quote_state.in_single_quote => {
                quote_state.in_double_quote = !quote_state.in_double_quote;
            }
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

/// Check if a closing keyword appears as a standalone word
fn has_closing_keyword(input: &str, keyword: &str) -> bool {
    // Split on whitespace and check for exact word match
    input.split_whitespace().any(|word| {
        // Remove trailing semicolons and check
        word.trim_end_matches(';') == keyword
    })
}

/// Check if bash keywords expect continuation
/// Check if 'if' statement needs continuation
fn needs_continuation_if(trimmed: &str) -> bool {
    trimmed.starts_with("if ") && !has_closing_keyword(trimmed, "fi")
}

/// Check if 'for' loop needs continuation
fn needs_continuation_for(trimmed: &str) -> bool {
    trimmed.starts_with("for ") && !has_closing_keyword(trimmed, "done")
}

/// Check if 'while' loop needs continuation
fn needs_continuation_while(trimmed: &str) -> bool {
    trimmed.starts_with("while ") && !has_closing_keyword(trimmed, "done")
}

/// Check if 'until' loop needs continuation
fn needs_continuation_until(trimmed: &str) -> bool {
    trimmed.starts_with("until ") && !has_closing_keyword(trimmed, "done")
}

/// Check if 'case' statement needs continuation
fn needs_continuation_case(trimmed: &str) -> bool {
    trimmed.starts_with("case ") && !has_closing_keyword(trimmed, "esac")
}

/// Check if function definition needs continuation
fn needs_continuation_function(trimmed: &str) -> bool {
    (trimmed.starts_with("function ") || trimmed.contains("() {")) && !trimmed.ends_with('}')
}

/// Check if line ends with block keyword that expects continuation
fn needs_continuation_block(trimmed: &str) -> bool {
    trimmed.ends_with(" then") || trimmed.ends_with(" do")
}

fn bash_keywords_need_continuation(input: &str) -> bool {
    let trimmed = input.trim();

    // Keywords that expect a closing keyword
    if needs_continuation_if(trimmed) {
        return true;
    }

    if needs_continuation_for(trimmed) {
        return true;
    }

    if needs_continuation_while(trimmed) {
        return true;
    }

    if needs_continuation_until(trimmed) {
        return true;
    }

    if needs_continuation_case(trimmed) {
        return true;
    }

    if needs_continuation_function(trimmed) {
        return true;
    }

    // Keywords that expect a following block
    if needs_continuation_block(trimmed) {
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
#[path = "multiline_tests_ext.rs"]
mod tests_ext;
