//! SC2086 Pure Logic - Double quote detection
//!
//! Extracted for EXTREME TDD testability.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Check if line should be skipped (comments or assignments)
pub fn should_skip_line(line: &str) -> bool {
    // Skip comments
    if line.trim_start().starts_with('#') {
        return true;
    }

    // Skip variable assignments (VAR=value)
    if line.contains('=') && !line.contains("if [") && !line.contains("[ ") {
        if let Some(eq_pos) = line.find('=') {
            if let Some(first_space) = line.find(' ') {
                if eq_pos < first_space {
                    return true; // Assignment, not command
                }
            }
        }
    }

    false
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
    static CSTYLE_FOR: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\bfor\s*\(\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap());

    let mut vars = HashSet::new();
    for cap in CSTYLE_FOR.captures_iter(source) {
        if let Some(m) = cap.get(1) {
            vars.insert(m.as_str().to_string());
        }
    }
    vars
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
    Regex::new(r#"(?m)(?P<pre>[^"']|^)\$(?:\{(?P<brace>[A-Za-z_][A-Za-z0-9_]*)\}|(?P<simple>[A-Za-z_][A-Za-z0-9_]*))"#).unwrap()
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

    for cap in pattern.captures_iter(line) {
        let var_capture = match cap.name("brace").or_else(|| cap.name("simple")) {
            Some(v) => v,
            None => continue,
        };
        let var_name = var_capture.as_str();
        let dollar_pos = find_dollar_position(line, var_capture.start());
        let col = dollar_pos + 1;
        let is_braced = cap.name("brace").is_some();
        let end_col = calculate_end_column(line, var_capture.end(), is_braced);

        if is_arithmetic && is_in_arithmetic_context(line, dollar_pos, var_capture.end()) {
            continue;
        }
        if is_already_quoted(line, dollar_pos, var_capture.end()) {
            continue;
        }
        if is_in_double_bracket_context(line, dollar_pos, var_capture.end()) {
            continue;
        }
        if cstyle_vars.contains(var_name) {
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
mod tests {
    use super::*;

    // ===== SHOULD SKIP LINE =====

    #[test]
    fn test_should_skip_line_comment() {
        assert!(should_skip_line("# this is a comment"));
        assert!(should_skip_line("  # indented comment"));
    }

    #[test]
    fn test_should_skip_line_assignment() {
        // Only skips if there's a space after the assignment
        assert!(should_skip_line("FOO=bar baz")); // assignment before space
                                                  // No space = no skip (simple assignment handled elsewhere)
        assert!(!should_skip_line("VAR=value"));
    }

    #[test]
    fn test_should_skip_line_not_assignment() {
        assert!(!should_skip_line("echo $VAR"));
        assert!(!should_skip_line("ls -la"));
        assert!(!should_skip_line("if [ $x = 1 ]; then")); // condition, not assignment
    }

    #[test]
    fn test_should_skip_line_if_bracket() {
        assert!(!should_skip_line("if [ $x = 1 ]; then"));
        assert!(!should_skip_line("[ $x = 1 ]"));
    }

    #[test]
    fn test_should_skip_line_equals_after_space() {
        // Equals sign is after space - NOT an assignment
        assert!(!should_skip_line("echo x = y")); // command, not assignment
    }

    #[test]
    fn test_should_skip_line_no_space() {
        // No space at all - simple assignment, but no space makes it not skip
        assert!(!should_skip_line("VAR=value"));
    }

    // ===== FIND DOLLAR POSITION =====

    #[test]
    fn test_find_dollar_position_simple() {
        let line = "echo $VAR";
        assert_eq!(find_dollar_position(line, 6), 5); // $ is at position 5
    }

    #[test]
    fn test_find_dollar_position_braced() {
        let line = "echo ${VAR}";
        assert_eq!(find_dollar_position(line, 7), 5); // $ is at position 5
    }

    #[test]
    fn test_find_dollar_position_multiple() {
        let line = "echo $A $B";
        // Looking for B which starts at 9, $ at 8
        assert_eq!(find_dollar_position(line, 9), 8);
    }

    // ===== CALCULATE END COLUMN =====

    #[test]
    fn test_calculate_end_column_simple() {
        let line = "echo $VAR more";
        // var_end is after "VAR" (position 9)
        assert_eq!(calculate_end_column(line, 9, false), 10);
    }

    #[test]
    fn test_calculate_end_column_braced() {
        let line = "echo ${VAR} more";
        // var_end is after "VAR" (position 10), brace at 10
        assert_eq!(calculate_end_column(line, 10, true), 12);
    }

    #[test]
    fn test_calculate_end_column_braced_no_brace() {
        // Edge case: braced=true but no closing brace in remainder
        let line = "echo ${VAR";
        // var_end is at 10, no } found - fallback to var_end + 1
        assert_eq!(calculate_end_column(line, 10, true), 11);
    }

    // ===== IS IN ARITHMETIC CONTEXT =====

    #[test]
    fn test_is_in_arithmetic_context_command_sub() {
        let line = "x=$(( a + b ))";
        // dollar_pos=0 for a, var_end=7
        // Actually we need to find the variable positions correctly
        // "a" is at position 6
        assert!(is_in_arithmetic_context(line, 6, 7));
    }

    #[test]
    fn test_is_in_arithmetic_context_standalone() {
        let line = "(( i++ ))";
        // "i" is at position 3
        assert!(is_in_arithmetic_context(line, 3, 4));
    }

    #[test]
    fn test_is_in_arithmetic_context_for_loop() {
        // In real usage, this checks $var positions, not raw identifiers
        let line = "for (( i=0; i<$n; i++ )); do";
        // $n starts at position 14
        assert!(is_in_arithmetic_context(line, 14, 16));
    }

    #[test]
    fn test_is_in_arithmetic_context_not_arithmetic() {
        let line = "echo $VAR";
        assert!(!is_in_arithmetic_context(line, 5, 8));
    }

    // ===== GET CSTYLE FOR LOOP VARS =====

    #[test]
    fn test_get_cstyle_for_loop_vars_single() {
        let source = "for (( i=0; i<10; i++ )); do echo $i; done";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.contains("i"));
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn test_get_cstyle_for_loop_vars_multiple() {
        let source =
            "for ((i=0; i<10; i++)); do\n  for ((j=0; j<5; j++)); do\n    echo\n  done\ndone";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.contains("i"));
        assert!(vars.contains("j"));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_get_cstyle_for_loop_vars_none() {
        let source = "for item in *.txt; do echo $item; done";
        let vars = get_cstyle_for_loop_vars(source);
        assert!(vars.is_empty());
    }

    // ===== IS IN DOUBLE BRACKET CONTEXT =====

    #[test]
    fn test_is_in_double_bracket_context_true() {
        let line = "[[ -n $var ]]";
        // $var: dollar at 6, var ends at 10
        assert!(is_in_double_bracket_context(line, 6, 10));
    }

    #[test]
    fn test_is_in_double_bracket_context_false_single() {
        let line = "[ -n $var ]";
        // Single bracket - word splitting occurs
        assert!(!is_in_double_bracket_context(line, 5, 9));
    }

    #[test]
    fn test_is_in_double_bracket_context_comparison() {
        let line = "[[ $a == $b ]]";
        // $a: dollar at 3, a ends at 5
        assert!(is_in_double_bracket_context(line, 3, 5));
    }

    // ===== IS ALREADY QUOTED =====

    #[test]
    fn test_is_already_quoted_simple() {
        let line = r#"echo "$VAR""#;
        // $VAR: dollar at 6, VAR ends at 10
        assert!(is_already_quoted(line, 6, 10));
    }

    #[test]
    fn test_is_already_quoted_braced() {
        let line = r#"echo "${VAR}""#;
        // ${VAR}: dollar at 6, VAR ends at 11
        assert!(is_already_quoted(line, 6, 11));
    }

    #[test]
    fn test_is_already_quoted_inside_string() {
        let line = r#"echo "prefix${VAR}suffix""#;
        // ${VAR}: dollar at 13, VAR ends at 18
        assert!(is_already_quoted(line, 13, 18));
    }

    #[test]
    fn test_is_already_quoted_not_quoted() {
        let line = "echo $VAR";
        assert!(!is_already_quoted(line, 5, 9));
    }

    #[test]
    fn test_is_already_quoted_edge_case() {
        // Pattern looks quoted due to adjacent quotes
        let line = r#"echo \"$VAR""#;
        // Before ends with " and after starts with " - treated as quoted
        assert!(is_already_quoted(line, 7, 11));
    }

    #[test]
    fn test_is_already_quoted_braced_not_immediately() {
        // Brace starts but no quote immediately around
        let line = r#"cmd ${VAR}end"#;
        // dollar at 4, var ends at 8, after starts with }
        // No quote immediately before or after
        assert!(!is_already_quoted(line, 4, 8));
    }

    #[test]
    fn test_is_already_quoted_odd_quote_braced() {
        // Inside quoted string with braced var - odd quote count
        let line = r#"x="prefix${VAR}suffix""#;
        // One quote before the $, then }suffix" has quote
        // dollar at 10, var ends at 14 (VAR)
        assert!(is_already_quoted(line, 10, 14));
    }

    #[test]
    fn test_is_already_quoted_escaped_quote_before() {
        // Escaped quote doesn't count
        let line = r#"echo \"$VAR"#;
        // The \" is escaped, so quote count is 0 (even)
        // dollar at 7, ends at 11
        assert!(!is_already_quoted(line, 7, 11));
    }

    // ===== FORMAT VAR TEXT =====

    #[test]
    fn test_format_var_text_simple() {
        assert_eq!(format_var_text("VAR", false), "$VAR");
        assert_eq!(format_var_text("foo", false), "$foo");
    }

    #[test]
    fn test_format_var_text_braced() {
        assert_eq!(format_var_text("VAR", true), "${VAR}");
        assert_eq!(format_var_text("foo", true), "${foo}");
    }

    // ===== FORMAT QUOTED VAR =====

    #[test]
    fn test_format_quoted_var_simple() {
        assert_eq!(format_quoted_var("VAR", false), "\"$VAR\"");
    }

    #[test]
    fn test_format_quoted_var_braced() {
        assert_eq!(format_quoted_var("VAR", true), "\"${VAR}\"");
    }

    // ===== LINE HAS ARITHMETIC MARKERS =====

    #[test]
    fn test_line_has_arithmetic_markers_command_sub() {
        assert!(line_has_arithmetic_markers("x=$(( a + b ))"));
    }

    #[test]
    fn test_line_has_arithmetic_markers_standalone() {
        assert!(line_has_arithmetic_markers("(( i++ ))"));
    }

    #[test]
    fn test_line_has_arithmetic_markers_none() {
        assert!(!line_has_arithmetic_markers("echo $VAR"));
        assert!(!line_has_arithmetic_markers("ls -la"));
    }
}
