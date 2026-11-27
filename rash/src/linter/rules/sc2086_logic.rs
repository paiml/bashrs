//! SC2086 Pure Logic - Double quote detection
//!
//! Extracted for EXTREME TDD testability.

use regex::Regex;
use std::collections::HashSet;

/// Check if line should be skipped (comments or assignments)
pub fn should_skip_line(line: &str) -> bool {
    // Skip comments
    if line.trim_start().starts_with('#') {
        return true;
    }

    // Skip assignments: keyword form, bare single word, or NAME=value before a space
    if is_keyword_assignment(line) || is_bare_assignment_word(line) || is_prefix_assignment(line) {
        return true;
    }

    false
}

/// `VAR=value rest` where `=` precedes the first space (not a test expression).
fn is_prefix_assignment(line: &str) -> bool {
    if line.contains("if [") || line.contains("[ ") {
        return false;
    }
    match (line.find('='), line.find(' ')) {
        (Some(eq), Some(sp)) => eq < sp,
        _ => false,
    }
}

/// True if the line is a single `NAME=value` word (e.g. `X=$RANDOM`).
fn is_bare_assignment_word(line: &str) -> bool {
    let t = line.trim();
    is_assign_word(t) && !t.contains(char::is_whitespace)
}

/// True if `w` starts with `NAME=`.
fn is_assign_word(w: &str) -> bool {
    let Some(p) = w.find('=').filter(|&p| p > 0) else {
        return false;
    };
    let mut cs = w[..p].chars();
    cs.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// True for `local|export|readonly|declare [-flags] NAME=value` lines.
fn is_keyword_assignment(line: &str) -> bool {
    let mut words = line.split_whitespace();
    let kw = matches!(
        words.next(),
        Some("local" | "export" | "readonly" | "declare" | "typeset")
    );
    kw && words
        .find(|w| !w.starts_with('-'))
        .is_some_and(is_assign_word)
}

/// Find the position of $ character before a variable
pub fn find_dollar_position(line: &str, var_start: usize) -> usize {
    line[..var_start].rfind('$').unwrap_or(var_start)
}

/// Calculate end column for variable span, including closing brace if present
pub fn calculate_end_column(line: &str, var_end: usize, is_braced: bool) -> usize {
    if is_braced {
        let after_var = &line[var_end..];
        if let Some(brace_pos) = after_var.find('}') {
            var_end + brace_pos + 2 // +1 for }, +1 for 1-indexing
        } else {
            var_end + 1 // Fallback
        }
    } else {
        var_end + 1 // Simple $VAR case
    }
}

/// Check if variable is in arithmetic context (inside $(( )) or (( )))
/// Issue #107: Also handles C-style for loops, standalone (( )), while/if (( ))
pub fn is_in_arithmetic_context(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before = &line[..dollar_pos];
    let after = &line[var_end..];

    // Case 1: Command substitution arithmetic $(( ))
    if before.contains("$((") && after.contains("))") {
        return true;
    }

    // Case 2: Standalone arithmetic (( )) - for loops, while, if, statements
    // Look for (( that is NOT preceded by $ (to distinguish from $(( ))
    if let Some(paren_pos) = before.rfind("((") {
        // Verify it's standalone (( not $((
        let is_standalone = if paren_pos > 0 {
            !before[..paren_pos].ends_with('$')
        } else {
            true
        };

        if is_standalone && after.contains("))") {
            return true;
        }
    }

    false
}

/// F048: Extract C-style for loop variable names from source
/// C-style for loops: for ((i=0; i<n; i++)) define numeric loop variables
/// These variables are always numeric, so SC2086 should not flag them
pub fn get_cstyle_for_loop_vars(source: &str) -> HashSet<String> {
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static CSTYLE_FOR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\bfor\s*\(\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap()
    });

    let mut vars = HashSet::new();
    for cap in CSTYLE_FOR.captures_iter(source) {
        if let Some(m) = cap.get(1) {
            vars.insert(m.as_str().to_string());
        }
    }
    vars
}

/// PMAT-654: names assigned only integer literals (`n=1`) anywhere in the file.
/// A name that is also a for/read target, or has any non-literal assignment, is excluded.
pub fn get_integer_literal_vars(source: &str) -> HashSet<String> {
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static ASSIGN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(
            r"^(?:export\s+|local\s+|declare\s+(?:-[a-zA-Z]+\s+)?)?([A-Za-z_][A-Za-z0-9_]*)=(\S*)",
        )
        .unwrap()
    });
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static LITERAL: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^-?[0-9]+$").unwrap());
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static REBIND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\b(?:for|read)\s+(?:-[a-zA-Z]+\s+)*([A-Za-z_][A-Za-z0-9_]*)").unwrap()
    });

    let mut literal = HashSet::new();
    let mut other = HashSet::new();
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if let Some(c) = ASSIGN.captures(t) {
            let name = c[1].to_string();
            if LITERAL.is_match(&c[2]) {
                literal.insert(name);
            } else {
                other.insert(name);
            }
        }
        for c in REBIND.captures_iter(t) {
            other.insert(c[1].to_string());
        }
    }
    literal.difference(&other).cloned().collect()
}

/// Issue #105: Check if variable is inside [[ ]] (bash extended test)
/// In [[ ]], word splitting and glob expansion do NOT occur on unquoted variables
/// This is safe: [[ -n $var ]] (no word splitting inside [[ ]])
/// This is NOT safe: [ -n $var ] (word splitting occurs in [ ])
pub fn is_in_double_bracket_context(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before = &line[..dollar_pos];
    let after = &line[var_end..];

    // Check for [[ before and ]] after
    // Must be [[ not [ (single bracket still has word splitting)
    if let Some(open_pos) = before.rfind("[[") {
        // Make sure it's not a single bracket by checking the character before
        let is_double = if open_pos > 0 {
            // Check there's no [ immediately before (would be [[[)
            !before[..open_pos].ends_with('[')
        } else {
            true
        };

        if is_double && after.contains("]]") {
            return true;
        }
    }

    false
}

/// Check if variable is immediately surrounded by double quotes (simple or braced)
fn is_immediately_quoted(before_context: &str, after_context: &str) -> bool {
    // Simple case: "$VAR"
    if before_context.ends_with('"') && after_context.starts_with('"') {
        return true;
    }
    // Braced case: "${VAR}"
    if after_context.starts_with('}') {
        if let Some(brace_pos) = after_context.find('}') {
            let after_brace = &after_context[brace_pos + 1..];
            if before_context.ends_with('"') && after_brace.starts_with('"') {
                return true;
            }
        }
    }
    false
}

/// Count unescaped double quotes in a string
fn count_unescaped_quotes(s: &str) -> usize {
    let mut count = 0;
    let mut escaped = false;
    for ch in s.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            count += 1;
        }
    }
    count
}

/// Check if variable is inside a quoted string based on quote parity
fn is_inside_quoted_string(before_context: &str, after_context: &str) -> bool {
    let quote_count = count_unescaped_quotes(before_context);
    if quote_count.is_multiple_of(2) {
        return false;
    }
    // For braced variables, check after the closing brace
    if after_context.starts_with('}') {
        if let Some(brace_pos) = after_context.find('}') {
            return after_context[brace_pos + 1..].contains('"');
        }
    }
    after_context.contains('"')
}

/// Check if variable is already quoted
pub fn is_already_quoted(line: &str, dollar_pos: usize, var_end: usize) -> bool {
    let before_context = &line[..dollar_pos];
    let after_context = &line[var_end..];

    is_immediately_quoted(before_context, after_context)
        || is_inside_quoted_string(before_context, after_context)
}

/// Format variable text for display
pub fn format_var_text(var_name: &str, is_braced: bool) -> String {
    if is_braced {
        format!("${{{}}}", var_name)
    } else {
        format!("${}", var_name)
    }
}

/// Format quoted variable for fix suggestion
pub fn format_quoted_var(var_name: &str, is_braced: bool) -> String {
    format!("\"{}\"", format_var_text(var_name, is_braced))
}

/// Check if line has any arithmetic context markers
pub fn line_has_arithmetic_markers(line: &str) -> bool {
    line.contains("$((") || line.contains("((")
}

/// Unquoted variable info
pub struct UnquotedVar {
    pub var_name: String,
    pub col: usize,
    pub end_col: usize,
    pub is_braced: bool,
}

/// Get the variable pattern regex
#[allow(clippy::unwrap_used)] // Compile-time regex
pub fn get_var_pattern() -> Regex {
    // Positional parameters $1..$9 word-split like any other unquoted expansion (shellcheck 0.8.0 SC2086).
    Regex::new(r#"(?m)(?P<pre>[^"']|^)\$(?:\{(?P<brace>[A-Za-z_][A-Za-z0-9_]*)\}|(?P<simple>[A-Za-z_][A-Za-z0-9_]*|[1-9]))"#).unwrap()
}

/// True if the variable sits in a context where SC2086 does not apply.
fn is_exempt_context(line: &str, dollar_pos: usize, var_end: usize, is_arithmetic: bool) -> bool {
    (is_arithmetic && is_in_arithmetic_context(line, dollar_pos, var_end))
        || is_already_quoted(line, dollar_pos, var_end)
        || is_in_double_bracket_context(line, dollar_pos, var_end)
        || case_subject_range(line).is_some_and(|(s, e)| dollar_pos >= s && dollar_pos < e)
}

/// True if the `$` at `dollar_pos` is escaped by an odd run of backslashes (`\$x` is literal).
fn is_backslash_escaped(line: &str, dollar_pos: usize) -> bool {
    let run = line[..dollar_pos]
        .bytes()
        .rev()
        .take_while(|&b| b == b'\\')
        .count();
    run % 2 == 1
}

/// True if `dollar_pos` lies in the word list of `for NAME in WORDS` (shellcheck does not
/// flag it: the list is split on purpose), outside any command substitution.
fn in_for_in_word_list(line: &str, dollar_pos: usize) -> bool {
    static FOR_IN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"^\s*for\s+\w+\s+in\s+(?P<list>[^;]*)").expect("valid regex")
    });
    let Some(list) = FOR_IN.captures(line).and_then(|c| c.name("list")) else {
        return false;
    };
    if dollar_pos < list.start() || dollar_pos >= list.end() {
        return false;
    }
    let before = &line[list.start()..dollar_pos];
    !(before.contains("$(") || before.contains('`'))
}

/// Byte range of the subject word(s) of a `case SUBJECT in` on this line, if any.
/// shellcheck does not flag SC2086 there: a case subject is neither split nor globbed.
fn case_subject_range(line: &str) -> Option<(usize, usize)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("case")?;
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let start = line.len() - rest.len();
    let end = start + rest.find(" in")?;
    Some((start, end))
}

/// Find unquoted variables in a line
pub fn find_unquoted_vars(
    line: &str,
    pattern: &Regex,
    cstyle_vars: &HashSet<String>,
) -> Vec<UnquotedVar> {
    let mut result = Vec::new();
    if should_skip_line(line) {
        return result;
    }
    let is_arithmetic = line_has_arithmetic_markers(line);

    // Resume the scan just past each `$` (not past the whole match): the pattern's `pre` group
    // consumes the byte before `$`, so a match that ends in a name char (`$1` in `$1$foo`) would
    // otherwise swallow the `pre` byte that the next `$` needs.
    let mut from = 0;
    while let Some(cap) = pattern.captures_at(line, from) {
        let var_capture = match cap.name("brace").or_else(|| cap.name("simple")) {
            Some(v) => v,
            None => break,
        };
        let var_name = var_capture.as_str();
        let dollar_pos = find_dollar_position(line, var_capture.start());
        from = dollar_pos + 1;
        let col = dollar_pos + 1;
        let is_braced = cap.name("brace").is_some();
        let end_col = calculate_end_column(line, var_capture.end(), is_braced);

        let var_end = var_capture.end();
        if is_backslash_escaped(line, dollar_pos) || in_for_in_word_list(line, dollar_pos) {
            continue;
        }
        if is_exempt_context(line, dollar_pos, var_end, is_arithmetic)
            || cstyle_vars.contains(var_name)
        {
            continue;
        }

        result.push(UnquotedVar {
            var_name: var_name.to_string(),
            col,
            end_col,
            is_braced,
        });
    }
    result
}

#[cfg(test)]
#[path = "sc2086_logic_tests.rs"]
mod tests;
