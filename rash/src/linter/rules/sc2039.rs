// SC2039: In POSIX sh, <feature> is undefined.
//
// Detects bash-specific features in scripts with #!/bin/sh shebang.
// POSIX sh doesn't support many bash features, so using them breaks portability.
//
// Examples:
// Bad (with #!/bin/sh):
//   #!/bin/sh
//   array=(1 2 3)        # Arrays not in POSIX
//   echo $((2**3))       # ** exponentiation not in POSIX
//   [[ -n $var ]]        # [[ ]] not in POSIX
//   source file.sh       # 'source' not in POSIX (use '.')
//   function foo() { }   # 'function' keyword not in POSIX
//
// Good:
//   #!/bin/sh
//   # Use POSIX-compatible constructs
//   [ -n "$var" ]        # Use [ ] not [[ ]]
//   . file.sh            # Use . not source
//   foo() { }            # Omit 'function' keyword
//
// Note: This is a simplified check. Full POSIX compliance requires deeper analysis.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_SYNTAX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\w+\s*=\s*\(").unwrap());

static DOUBLE_BRACKET: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\[\[").unwrap());

static SOURCE_COMMAND: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bsource\s+").unwrap());

static FUNCTION_KEYWORD: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bfunction\s+\w+\s*\(\s*\)").unwrap());

static EXPONENTIATION: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\(\([^)]*\*\*[^)]*\)\)").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Check if script uses #!/bin/sh shebang (POSIX intent)
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return result;
    }

    let has_posix_shebang = lines[0] == "#!/bin/sh" || lines[0] == "#!/usr/bin/env sh";

    if !has_posix_shebang {
        // Not claiming to be POSIX sh, OK to use bash features
        return result;
    }

    // Check for bash-specific features
    for (line_num, line) in lines.iter().enumerate().skip(1) {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for array syntax
        if let Some(mat) = ARRAY_SYNTAX.find(line) {
            let pos = mat.start();
            let diagnostic = Diagnostic::new(
                "SC2039",
                Severity::Warning,
                "In POSIX sh, arrays are undefined. Use space-separated strings or multiple variables.".to_string(),
                Span::new(line_num, pos + 1, line_num, mat.end() + 1),
            );
            result.add(diagnostic);
        }

        // Check for [[ ]]
        if let Some(mat) = DOUBLE_BRACKET.find(line) {
            let pos = mat.start();
            let diagnostic = Diagnostic::new(
                "SC2039",
                Severity::Warning,
                "In POSIX sh, [[ ]] is undefined. Use [ ] instead.".to_string(),
                Span::new(line_num, pos + 1, line_num, mat.end() + 1),
            );
            result.add(diagnostic);
        }

        // Check for source command
        if let Some(mat) = SOURCE_COMMAND.find(line) {
            let pos = mat.start();
            let diagnostic = Diagnostic::new(
                "SC2039",
                Severity::Warning,
                "In POSIX sh, 'source' is undefined. Use '.' instead.".to_string(),
                Span::new(line_num, pos + 1, line_num, mat.end() + 1),
            );
            result.add(diagnostic);
        }

        // Check for function keyword
        if let Some(mat) = FUNCTION_KEYWORD.find(line) {
            let pos = mat.start();
            let diagnostic = Diagnostic::new(
                "SC2039",
                Severity::Warning,
                "In POSIX sh, 'function' keyword is undefined. Use name() syntax instead."
                    .to_string(),
                Span::new(line_num, pos + 1, line_num, mat.end() + 1),
            );
            result.add(diagnostic);
        }

        // Check for exponentiation
        if let Some(mat) = EXPONENTIATION.find(line) {
            let pos = mat.start();
            let diagnostic = Diagnostic::new(
                "SC2039",
                Severity::Warning,
                "In POSIX sh, ** exponentiation is undefined. Use * for multiplication or bc for powers.".to_string(),
                Span::new(line_num, pos + 1, line_num, mat.end() + 1),
            );
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2039_array_syntax() {
        let code = r#"#!/bin/sh
arr=(1 2 3)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2039");
        assert!(result.diagnostics[0].message.contains("array"));
    }

    #[test]
    fn test_sc2039_double_bracket() {
        let code = r#"#!/bin/sh
[[ -n $var ]]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("[["));
    }

    #[test]
    fn test_sc2039_source_command() {
        let code = r#"#!/bin/sh
source config.sh
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("source"));
    }

    #[test]
    fn test_sc2039_function_keyword() {
        let code = r#"#!/bin/sh
function foo() {
  echo "bar"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("function"));
    }

    #[test]
    fn test_sc2039_exponentiation() {
        let code = r#"#!/bin/sh
result=$((2**3))
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("**"));
    }

    #[test]
    fn test_sc2039_bash_shebang_ok() {
        let code = r#"#!/bin/bash
arr=(1 2 3)
[[ -n $var ]]
"#;
        let result = check(code);
        // Bash shebang, features are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2039_posix_compatible() {
        let code = r#"#!/bin/sh
[ -n "$var" ]
. config.sh
foo() {
  echo "bar"
}
"#;
        let result = check(code);
        // All POSIX-compatible
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2039_multiple_issues() {
        let code = r#"#!/bin/sh
arr=(1 2 3)
[[ -n $x ]]
source file.sh
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2039_comment_ok() {
        let code = r#"#!/bin/sh
# arr=(1 2 3)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2039_env_sh_shebang() {
        let code = r#"#!/usr/bin/env sh
[[ -n $var ]]
"#;
        let result = check(code);
        // Also POSIX sh intent
        assert_eq!(result.diagnostics.len(), 1);
    }
}
