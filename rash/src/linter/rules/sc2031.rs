// SC2031: Variable was modified in subshell. Double check or use var=$(cmd).
//
// This is the complementary warning to SC2030. It detects when you use a variable
// that was previously assigned in a subshell, which means the value will be empty/wrong.
//
// Examples:
// Bad:
//   (foo=bar)
//   echo "$foo"  # SC2031: foo was assigned in subshell, will be empty
//
//   (x=5; y=10)
//   echo "$x"    # Empty
//
// Good:
//   foo=bar      # Assign in current shell
//   echo "$foo"
//
//   result=$(foo=bar; echo "$foo")  # Capture output
//   echo "$result"
//
// Note: This requires tracking variable assignments across lines (stateful analysis).
// For MVP, we detect the pattern heuristically: subshell assignment followed by usage.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static SUBSHELL_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(").unwrap());

static ASSIGNMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)=").unwrap());

static VAR_USAGE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{?([a-zA-Z_][a-zA-Z0-9_]*)\}?").unwrap());

/// Check if opening paren at position `i` should be skipped (not a subshell)
fn is_non_subshell_paren(chars: &[char], i: usize) -> bool {
    // Command substitution $()
    if i > 0 && chars[i - 1] == '$' {
        return true;
    }
    // Array declaration =()
    if i > 0 && chars[i - 1] == '=' {
        return true;
    }
    // Second paren in arithmetic $((
    if i > 1 && chars[i - 1] == '(' && chars[i - 2] == '$' {
        return true;
    }
    // Empty subshell () - skip
    if i + 1 < chars.len() && chars[i + 1] == ')' {
        return true;
    }
    false
}

/// Check if line contains a subshell (standalone parentheses, not command substitution or arithmetic)
fn has_subshell(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let mut in_arithmetic = false;
    let mut paren_depth = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for i in 0..chars.len() {
        // Track quote context (Issue #132: skip parens inside quotes like regex (?=...))
        if chars[i] == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }
        if chars[i] == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }
        if in_single_quote || in_double_quote {
            continue;
        }

        // Track arithmetic expansion context $((
        if i > 0 && chars[i] == '(' && chars[i - 1] == '(' && i > 1 && chars[i - 2] == '$' {
            in_arithmetic = true;
            paren_depth = 2;
            continue;
        }

        // Track nested parens in arithmetic context
        if in_arithmetic {
            match chars[i] {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 { in_arithmetic = false; }
                }
                _ => {}
            }
            continue;
        }

        if chars[i] == '(' {
            if is_non_subshell_paren(&chars, i) {
                continue;
            }
            return true;
        }
    }
    false
}

/// Check if position in line is inside quotes (double or single)
fn is_in_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
}

/// Check if position in line is inside single quotes (where variables don't expand)
fn is_in_single_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let single_quote_count = before.matches('\'').count();
    single_quote_count % 2 == 1
}

/// Check if variable usage is on same line as assignment (which is OK)
fn is_same_line_assignment(line: &str, var_name: &str) -> bool {
    line.contains('(') && line.contains(')') && line.contains(&format!("{}=", var_name))
}

/// Find subshell variable assignments on a line
fn find_subshell_assignments(line: &str) -> HashSet<String> {
    let mut vars = HashSet::new();

    if !line.contains('(') || !line.contains(')') {
        return vars;
    }

    if !has_subshell(line) {
        return vars;
    }

    // Find all variable assignments on this line
    for cap in ASSIGNMENT.captures_iter(line) {
        let var_name = cap.get(1).unwrap().as_str();
        let full_match = cap.get(0).unwrap().as_str();
        let pos = line.find(full_match).unwrap_or(0);

        // Skip if inside quotes
        if !is_in_quotes(line, pos) {
            vars.insert(var_name.to_string());
        }
    }

    vars
}

/// Create diagnostic for subshell variable usage
fn create_diagnostic(
    line_num: usize,
    var_name: &str,
    pos: usize,
    full_match_len: usize,
) -> Diagnostic {
    let start_col = pos + 1;
    let end_col = start_col + full_match_len;

    Diagnostic::new(
        "SC2031",
        Severity::Warning,
        format!(
            "Variable '{}' was assigned in a subshell. It will be empty here. Use var=$(cmd) or assign in current shell",
            var_name
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut subshell_vars: HashSet<String> = HashSet::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Track variables assigned in subshells
        subshell_vars.extend(find_subshell_assignments(line));

        // Check for usage of subshell-assigned variables
        for cap in VAR_USAGE.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();

            if !subshell_vars.contains(var_name) {
                continue;
            }

            // Skip if same line assignment or inside single quotes
            if is_same_line_assignment(line, var_name) {
                continue;
            }

            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            if is_in_single_quotes(line, pos) {
                continue;
            }

            let diagnostic = create_diagnostic(line_num, var_name, pos, full_match.len());
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Issue #76: FALSE POSITIVE - command substitution assignment is NOT a subshell assignment
    #[test]
    fn test_sc2031_command_subst_assignment_is_not_subshell() {
        // This is VAR=$(cmd) - the variable is assigned in the CURRENT shell
        // The command runs in a subshell but output is captured to current shell variable
        let code = r#"
py_mean="$(jq -r '.benchmarks[0].statistics.mean_ms' "$json")"
echo "$py_mean"
"#;
        let result = check(code);
        // NO warning - this is a valid assignment in current shell
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Command substitution assignment VAR=$(cmd) should NOT trigger SC2031. Got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_sc2031_command_subst_multiple_vars() {
        let code = r#"
output="$(command arg1 arg2)"
count="$(wc -l < file.txt)"
echo "$output $count"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple command substitution assignments should not trigger SC2031"
        );
    }

    #[test]
    fn test_sc2031_usage_after_subshell_assignment() {
        let code = r#"
(foo=bar)
echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2031");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("foo"));
    }

    #[test]
    fn test_sc2031_multiple_vars() {
        let code = r#"
(x=5; y=10)
echo "$x"
echo "$y"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2031_direct_assignment_ok() {
        let code = r#"
foo=bar
echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_command_subst_ok() {
        let code = r#"
result=$(foo=bar; echo "$foo")
echo "$result"
"#;
        let result = check(code);
        // Command substitution captures output, result is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_same_line_ok() {
        let code = r#"(foo=bar; echo "$foo")"#;
        let result = check(code);
        // Same line usage inside subshell is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_in_quotes_ok() {
        let code = r#"
(foo=bar)
echo '(foo=bar)'
"#;
        let result = check(code);
        // Variable in quotes (literal string)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_unrelated_var_ok() {
        let code = r#"
(foo=bar)
echo "$baz"
"#;
        let result = check(code);
        // Different variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_braced_var() {
        let code = r#"
(VAR=test)
echo "${VAR}"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2031_comment_ok() {
        let code = r#"
(foo=bar)
# echo "$foo"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2031_nested_subshell() {
        let code = r#"
((foo=bar))
echo "$foo"
"#;
        let result = check(code);
        // Still a subshell assignment
        assert_eq!(result.diagnostics.len(), 1);
    }

    // Issue #86: SC2031 should NOT flag arithmetic expansion $((
    #[test]
    fn test_sc2031_issue_86_arithmetic_expansion_not_subshell() {
        // From issue #86 reproduction case
        let code = r#"
S1_SCORE=0
add_points() {
    local section="$1"
    case "$section" in
        1) S1_SCORE=$((S1_SCORE + 1)) ;;
    esac
}
get_score() {
    local section="$1"
    case "$section" in
        1) echo "$S1_SCORE" ;;
    esac
}
"#;
        let result = check(code);
        // NO SC2031 - arithmetic expansion $((...)) is NOT a subshell
        let has_s1_score_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2031" && d.message.contains("S1_SCORE"));
        assert!(
            !has_s1_score_warning,
            "SC2031 must NOT flag arithmetic expansion $((...)) as subshell assignment"
        );
    }

    #[test]
    fn test_sc2031_issue_86_simple_arithmetic_expansion() {
        let code = r#"
count=$((count + 1))
echo "$count"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic expansion should not trigger SC2031"
        );
    }

    // Issue #132: Array declarations should NOT trigger SC2031
    #[test]
    fn test_sc2031_issue_132_array_declaration_not_subshell() {
        let code = r#"
formats=("gguf" "apr")
echo "${formats[@]}"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Array declaration formats=(...) should NOT trigger SC2031"
        );
    }

    #[test]
    fn test_sc2031_issue_132_arithmetic_with_grouping() {
        // Nested parentheses in arithmetic expansion
        let code = r#"
duration=$(( (end_ts - start_ts) / 1000000 ))
echo "$duration"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic grouping $(( (a - b) / c )) should NOT trigger SC2031"
        );
    }

    #[test]
    fn test_sc2031_issue_86_local_variable_ok() {
        let code = r#"
get_grade() {
    local pct="$1"
    if [[ $pct -ge 93 ]]; then echo "A+"
    elif [[ $pct -ge 90 ]]; then echo "A"
    fi
}
"#;
        let result = check(code);
        let has_pct_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2031" && d.message.contains("pct"));
        assert!(
            !has_pct_warning,
            "SC2031 must NOT flag local variable assignments"
        );
    }

    // Issue #132: Regex patterns with (?= should not be detected as subshells
    #[test]
    fn test_sc2031_issue_132_regex_lookahead_not_subshell() {
        let code = r#"
measure_throughput() {
    local tps
    tps=$(echo "$output" | grep -oP '[0-9.]+(?= tok/s)' | tail -n1 || echo "0")
    echo "$tps"
}
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Regex lookahead (?= inside quotes should NOT trigger SC2031"
        );
    }

    #[test]
    fn test_sc2031_issue_132_parens_in_double_quotes() {
        let code = r#"
result=$(echo "test (value)" | grep -o "(.*)")
echo "$result"
"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Parentheses inside double quotes should NOT trigger SC2031"
        );
    }
}
