//! SC2178: Variable was used as an array but is now assigned a string
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! array=(a b c)
//! array="single value"  # Overwrites array with string
//! echo "${array[@]}"    # Only prints "single value"
//! ```
//!
//! Good:
//! ```bash
//! array=(a b c)
//! array[0]="first"      # Update specific element
//! # OR
//! array=("single value") # Keep as array
//! ```
//!
//! # Rationale
//!
//! Assigning a string to an array variable:
//! - Converts array to string
//! - Loses all elements except the first
//! - Usually a bug
//!
//! Use array syntax for assignment or update individual elements.
//!
//! # Auto-fix
//!
//! Warning only - context needed to determine correct fix

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Create diagnostic for array-to-string assignment
fn create_array_to_string_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2178",
        Severity::Warning,
        "Variable was used as an array but is now assigned a string",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

/// Check for string assignment to array variable
static ARRAY_DECL_PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=\(").unwrap());
/// `declare -a x` / `declare -A x` / `typeset -A x` / `local -a x` / `readonly -A x`
static ARRAY_FLAG_DECL_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\b(?:declare|typeset|local|readonly)\s+-[A-Za-z]*[aA][A-Za-z]*\s+([A-Za-z_][A-Za-z0-9_]*)")
        .unwrap()
});
static SC2178_RE_1: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"([A-Za-z_][A-Za-z0-9_]*)=(?:"[^"]*"|'[^']*'|[^\s;]*)"#).unwrap()
});

/// Array declarations on one line as (name, start offset of the name).
fn line_array_decls(line: &str) -> Vec<(String, usize)> {
    [&*ARRAY_DECL_PATTERN, &*ARRAY_FLAG_DECL_PATTERN]
        .iter()
        .flat_map(|re| re.captures_iter(line))
        .filter_map(|cap| cap.get(1))
        .map(|m| (m.as_str().to_string(), m.start()))
        .collect()
}

/// True if `name` was declared an array on an earlier line, or earlier on this line.
fn declared_before(
    name: &str,
    at: usize,
    known: &std::collections::HashSet<String>,
    line_decls: &[(String, usize)],
) -> bool {
    known.contains(name) || line_decls.iter().any(|(n, pos)| n == name && *pos < at)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut known = std::collections::HashSet::new();

    for (idx, line) in source.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        let line_decls = line_array_decls(line);

        for cap in SC2178_RE_1.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();
            // `name=(` is an array (re)assignment, not a string
            if full_match.as_str().as_bytes().get(var_name.len() + 1) == Some(&b'(') {
                continue;
            }
            if declared_before(var_name, full_match.start(), &known, &line_decls) {
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;
                result.add(create_array_to_string_diagnostic(
                    idx + 1,
                    start_col,
                    end_col,
                ));
            }
        }

        known.extend(line_decls.into_iter().map(|(n, _)| n));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2178_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "array=(a b c)\n# array=\"string\"",
            "files=(*.txt)\n  # files='file'",
            "items=(x y)\n\t# items=value",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_array_reassignment_never_diagnosed() {
        // Property: Array-to-array reassignment should never be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray=(x y z)",
            "files=(*.txt)\nfiles=(*.md)",
            "items=(1 2)\nitems=(3 4 5)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_indexed_assignment_never_diagnosed() {
        // Property: Indexed array assignment should never be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray[0]=\"updated\"",
            "files=(*.txt)\nfiles[1]='file.md'",
            "items=(x y)\nitems[2]=z",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_non_array_vars_never_diagnosed() {
        // Property: Non-array variables should never be diagnosed
        let test_cases = vec![
            "var=\"string\"\nvar=\"other string\"",
            "name=value\nname=newvalue",
            "file='test.txt'\nfile='other.txt'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2178_string_to_array_always_diagnosed() {
        // Property: String assignment to array variable should always be diagnosed
        let test_cases = vec![
            "array=(a b c)\narray=\"string\"",
            "files=(*.txt)\nfiles='single file'",
            "items=(x y z)\nitems=single",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("array"));
        }
    }

    #[test]
    fn prop_sc2178_multiple_violations_all_diagnosed() {
        // Property: Multiple string assignments to arrays should all be diagnosed
        let code = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2178_diagnostic_code_always_sc2178() {
        // Property: All diagnostics must have code "SC2178"
        let code = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2178");
        }
    }

    #[test]
    fn prop_sc2178_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "array=(a b c)\narray=\"string\"";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2178_works_across_lines() {
        // Property: Detection works regardless of line distance
        let test_cases = vec![
            "array=(a b)\narray=\"str\"",            // Immediate
            "array=(a b)\necho test\narray=\"str\"", // One line between
            "array=(a b)\n\n\narray=\"str\"",        // Multiple empty lines
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        }
    }

    #[test]
    fn prop_sc2178_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2178_basic_detection() {
        let script = "array=(a b c)\narray=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2178");
    }

    #[test]
    fn test_sc2178_with_quotes() {
        let script = "files=(*.txt)\nfiles='single file'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_no_quotes() {
        let script = "items=(x y z)\nitems=single";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_multiple_vars() {
        let script = "a=(1 2)\nb=(3 4)\na=\"str1\"\nb=\"str2\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2178_false_positive_indexed_assign() {
        let script = "array=(a b c)\narray[0]=\"updated\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_false_positive_array_reassign() {
        let script = "array=(a b c)\narray=(x y z)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_false_positive_in_comment() {
        let script = "array=(a b c)\n# array=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_multiline() {
        let script = "array=(a b c)\necho test\narray=\"string\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_in_function() {
        let script = "func() {\n  local arr=(1 2 3)\n  arr=\"value\"\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_underscore_var() {
        let script = "my_array=(a b)\nmy_array=\"text\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // ===== #526 parity (shellcheck 0.8.0) =====

    #[test]
    fn test_sc2178_same_line_array_then_string() {
        let result = check("a=(1 2); a=foo");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_declare_assoc_then_string() {
        let result = check("declare -A m; m=x");
        assert_eq!(result.diagnostics.len(), 1);
        let result = check("declare -a b\nb=x");
        assert_eq!(result.diagnostics.len(), 1);
        let result = check("typeset -A c\nc=y");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_empty_string_assignment() {
        let result = check("a=(1 2); a=; echo $a");
        assert_eq!(result.diagnostics.len(), 1);
        let result = check("a=(1 2)\na=");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2178_string_before_array_silent() {
        assert_eq!(check("a=foo\na=(1 2)").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2178_same_line_array_reassign_and_index_silent() {
        assert_eq!(check("a=(1 2); a=(3 4)").diagnostics.len(), 0);
        assert_eq!(check("a=(1 2); a[0]=x").diagnostics.len(), 0);
        assert_eq!(check("declare -A m; m[k]=v").diagnostics.len(), 0);
        assert_eq!(check("declare -A m=([k]=v)").diagnostics.len(), 0);
    }
}
