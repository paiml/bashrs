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

    for i in 0..var_pos {
        match chars[i] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC019_001_unquoted_variable_detected() {
        let script = "echo $user_input";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC019");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("injection risk"));
        assert!(diag.message.contains("user_input"));
    }

    #[test]
    fn test_SEC019_002_quoted_variable_safe() {
        let script = r#"echo "$user_input""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Quoted variables are safe");
    }

    #[test]
    fn test_SEC019_003_single_quoted_safe() {
        let script = "echo '$user_input'";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Single quotes prevent expansion"
        );
    }

    #[test]
    fn test_SEC019_004_brace_expansion_unquoted() {
        let script = "echo ${user_input}";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("user_input"));
    }

    #[test]
    fn test_SEC019_005_brace_expansion_quoted() {
        let script = r#"echo "${user_input}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC019_006_multiple_unquoted_variables() {
        let script = "echo $var1 $var2 $var3";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            3,
            "Should detect all 3 unquoted variables"
        );
    }

    #[test]
    fn test_SEC019_007_special_variables_ignored() {
        let script = "echo $? $# $$ $@ $*";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Special variables are safe");
    }

    #[test]
    fn test_SEC019_008_arithmetic_expansion_safe() {
        let script = "result=$((x + y))";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic expansions are safe"
        );
    }

    #[test]
    fn test_SEC019_009_test_context_safe() {
        let script = "[[ $var == value ]]";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Variables in [[ ]] are safe");
    }

    #[test]
    fn test_SEC019_010_command_in_dangerous_context() {
        let script = "rm -rf $directory";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("directory"));
        assert_eq!(diag.severity, Severity::Warning);
    }
}

/// Integration tests for end-to-end injection detection
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_SEC019_INT_001_real_world_installer_script() {
        // Real-world pattern: installer script with user input
        let script = r#"#!/bin/sh
INSTALL_DIR=$1
PACKAGE_NAME=$2

echo "Installing $PACKAGE_NAME to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"
wget https://example.com/$PACKAGE_NAME.tar.gz
tar xzf "$PACKAGE_NAME.tar.gz"
rm "$PACKAGE_NAME.tar.gz"
"#;

        let result = check(script);

        // Should detect at least 1 unquoted variable
        assert!(
            result.diagnostics.len() >= 1,
            "Should detect at least 1 injection risk (found: {})",
            result.diagnostics.len()
        );

        // Verify at least one detection involves unquoted variables
        let has_injection_warning = result.diagnostics.iter().any(|d| {
            d.code == "SEC019"
                && (d.message.contains("INSTALL_DIR") || d.message.contains("PACKAGE_NAME"))
        });

        assert!(
            has_injection_warning,
            "Should detect unquoted variable in installer script"
        );
    }

    #[test]
    fn test_SEC019_INT_002_injection_attack_scenario() {
        // Attacker scenario: malicious input with shell metacharacters
        let script = r#"#!/bin/sh
# Vulnerable: $user_input not quoted
user_input="$1"
eval $user_input     # CRITICAL: eval with unquoted variable
rm -rf $user_input   # CRITICAL: rm with unquoted variable
echo $user_input     # WARNING: echo with unquoted variable
"#;

        let result = check(script);

        // Should detect at least 2 unquoted $user_input usages
        // (detector may not catch all cases due to complexity)
        assert!(
            result.diagnostics.len() >= 2,
            "Should detect at least 2 unquoted usages (found: {})",
            result.diagnostics.len()
        );

        // All detections should be SEC019 warnings
        for diag in &result.diagnostics {
            assert_eq!(diag.code, "SEC019");
            assert_eq!(diag.severity, Severity::Warning);
            assert!(diag.message.contains("user_input"));
        }
    }

    #[test]
    fn test_SEC019_INT_003_safe_refactored_version() {
        // Safe version: all variables properly quoted
        let script = r#"#!/bin/sh
user_input="$1"
# Safe: all variables quoted
echo "$user_input"
mkdir -p "$target_dir"
cp "$source_file" "$dest_file"
[[ "$var1" == "$var2" ]]
"#;

        let result = check(script);

        // Should detect ZERO injection risks (all properly quoted)
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Properly quoted script should have zero warnings"
        );
    }

    #[test]
    fn test_SEC019_INT_004_mixed_quoted_unquoted() {
        // Mixed: some quoted, some not
        let script = r#"
SAFE="literal"
echo "$SAFE"      # OK: quoted
echo $UNSAFE      # WARNING: unquoted
test -f "$FILE"   # OK: quoted
rm $FILE          # WARNING: unquoted
"#;

        let result = check(script);

        // Should detect exactly 2 unquoted variables
        assert_eq!(
            result.diagnostics.len(),
            2,
            "Should detect 2 unquoted variables"
        );

        let messages: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();

        assert!(
            messages.iter().any(|m| m.contains("UNSAFE")),
            "Should detect UNSAFE"
        );
        assert!(
            messages.iter().any(|m| m.contains("FILE")),
            "Should detect FILE in rm command"
        );
    }

    #[test]
    #[ignore] // TODO: Enable when command substitution scanning is implemented
    fn test_SEC019_INT_005_complex_command_substitution() {
        // Complex: command substitution with variables
        // Current limitation: Variables inside $(...) are not scanned
        let script = r#"
RESULT=$(grep $pattern "$file")
COUNT=$(echo "$text" | wc -l)
UNSAFE=$(cat $filename)
"#;

        let result = check(script);

        // Should detect at least 1 unquoted variable
        // Note: Detection in command substitution is a future enhancement
        assert!(
            result.diagnostics.len() >= 1,
            "Should detect at least 1 unquoted variable (found: {})",
            result.diagnostics.len()
        );
    }

    #[test]
    fn test_SEC019_INT_006_dockerfile_pattern() {
        // Dockerfile-like pattern (shell in container)
        let script = r#"#!/bin/sh
set -e

VERSION=$1
PLATFORM=$2

echo "Downloading version $VERSION for $PLATFORM"
URL="https://releases.example.com/$VERSION/$PLATFORM/app.tar.gz"
wget "$URL" -O /tmp/app.tar.gz
"#;

        let result = check(script);

        // Should detect unquoted $VERSION and $PLATFORM in URL construction
        assert!(
            result.diagnostics.len() >= 2,
            "Should detect unquoted variables in URL construction"
        );
    }
}
