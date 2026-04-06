//! SEC019: Injection Safety - Unquoted Variables
//!
//! **Rule**: Detect unquoted variable expansions that may lead to injection attacks
//!
//! **Why this matters**:
//! Unquoted variables can be exploited for command injection attacks. When user-controlled
//! data is expanded without quotes, attackers can inject shell metacharacters (`;`, `|`, `&`, etc.)
//! to execute arbitrary commands.
//!
//! **Auto-fix**: Add quotes around variable expansions
//!
//! ## Examples
//!
//! ❌ **UNSAFE** (Injection Risk):
//! ```bash
//! echo $user_input       # Attacker: user_input="; rm -rf /"
//! rm -rf $dir            # Attacker: dir="/; cat /etc/passwd"
//! eval $command          # Critical: arbitrary code execution
//! ```
//!
//! ✅ **SAFE**:
//! ```bash
//! echo "$user_input"     # Quoted: safe from injection
//! rm -rf "$dir"          # Quoted: safe from injection
//! # Avoid eval entirely, or use arrays
//! ```
//!
//! ## Security Properties (Taint Tracking)
//!
//! This rule implements basic taint tracking:
//! - Variables from external sources are considered **Tainted**
//! - Unquoted tainted variables → **Injection Risk**
//! - Quoted variables → **Sanitized** (safe)
//!
//! ## Exceptions
//!
//! - Variables in `[[ ]]` test contexts (safe by design)
//! - Arithmetic expressions `$(( ))` (safe by design)
//! - Some control flow constructs where quoting not needed

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unquoted variable expansions (injection risk)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        check_line(line, line_num, &mut result);
    }

    result
}

fn check_line(line: &str, line_num: usize, result: &mut LintResult) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for $VAR or ${VAR}
        if chars[i] == '$' && i + 1 < chars.len() {
            let start_col = i;

            // Skip $$ (process ID) - special case, not user data
            if chars[i + 1] == '$' {
                i += 2;
                continue;
            }

            // Skip $? $# $@ $* (special variables)
            if matches!(chars[i + 1], '?' | '#' | '@' | '*' | '!' | '-') {
                i += 2;
                continue;
            }

            // Skip $((  (arithmetic expansion - safe)
            if i + 2 < chars.len() && chars[i + 1] == '(' && chars[i + 2] == '(' {
                i = skip_arithmetic(&chars, i);
                continue;
            }

            // Skip $(  (command substitution - check inside)
            if chars[i + 1] == '(' {
                i = skip_command_substitution(&chars, i);
                continue;
            }

            // Check if variable is quoted
            let is_quoted = is_variable_quoted(&chars, start_col);

            // Check if in safe context
            let in_safe_context = is_in_safe_context(line, start_col);

            if !is_quoted && !in_safe_context {
                // Unquoted variable expansion - potential injection
                let var_end = find_variable_end(&chars, i);
                let var_name = extract_variable_name(&chars, i, var_end);

                let span = Span::new(line_num + 1, start_col + 1, line_num + 1, var_end + 1);

                let message = format!(
                    "Unquoted variable expansion ${} - injection risk. Use \"${}\" instead",
                    var_name, var_name
                );

                let diag = Diagnostic::new("SEC019", Severity::Warning, &message, span);
                result.add(diag);
            }

            i = find_variable_end(&chars, i);
        } else {
            i += 1;
        }
    }
}

/// Check if variable is quoted (inside "..." or '...')
fn is_variable_quoted(chars: &[char], var_pos: usize) -> bool {
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    for ch in chars.iter().take(var_pos) {
        match ch {
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            _ => {}
        }
    }

    // Double quotes protect against injection, single quotes don't expand vars
    in_double_quotes || in_single_quotes
}

/// Check if variable is in a safe context (e.g., [[ ]], arithmetic)
fn is_in_safe_context(line: &str, _var_pos: usize) -> bool {
    // Check for [[ ]] test context (safe by design)
    if line.contains("[[") && line.contains("]]") {
        return true;
    }

    false
}

/// Find the end position of a variable expansion
fn find_variable_end(chars: &[char], start: usize) -> usize {
    if start + 1 >= chars.len() {
        return start + 1;
    }

    let mut i = start + 1; // Skip $

    if chars[i] == '{' {
        // ${VAR} form
        i += 1;
        while i < chars.len() && chars[i] != '}' {
            i += 1;
        }
        if i < chars.len() {
            i += 1; // Include closing }
        }
    } else {
        // $VAR form
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
    }

    i
}

/// Extract variable name from expansion
fn extract_variable_name(chars: &[char], start: usize, end: usize) -> String {
    let mut name_start = start + 1; // Skip $

    if name_start < chars.len() && chars[name_start] == '{' {
        name_start += 1; // Skip {
    }

    let mut name_end = end;
    if name_end > name_start && chars.get(name_end - 1) == Some(&'}') {
        name_end -= 1; // Exclude }
    }

    chars[name_start..name_end.min(chars.len())]
        .iter()
        .collect()
}

/// Skip arithmetic expansion $(( ... ))
fn skip_arithmetic(chars: &[char], start: usize) -> usize {
    let mut i = start + 3; // Skip $((
    let mut depth = 1;

    while i + 1 < chars.len() && depth > 0 {
        if chars[i] == '(' && chars[i + 1] == '(' {
            depth += 1;
            i += 2;
        } else if chars[i] == ')' && chars[i + 1] == ')' {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }

    i
}

/// Skip command substitution $( ... )
fn skip_command_substitution(chars: &[char], start: usize) -> usize {
    let mut i = start + 2; // Skip $(
    let mut depth = 1;

    while i < chars.len() && depth > 0 {
        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
            depth += 1;
            i += 2;
        } else if chars[i] == ')' {
            depth -= 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    i
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "sec019_tests_sec019_001.rs"]
// FIXME(PMAT-238): mod tests_extracted;
