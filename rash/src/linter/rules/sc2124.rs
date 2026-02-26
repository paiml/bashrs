// SC2124: Assigning an array to a string! Assign as array, or use * instead of @ to concatenate
//
// When assigning $@ or ${array[@]} to a regular variable (not an array),
// only the first element is preserved. This is usually not intended.
//
// Examples:
// Bad:
//   var="$@"                 # Only gets first argument
//   str="${array[@]}"        # Only gets first element
//
// Good:
//   var=("$@")              # Preserve all arguments as array
//   str="$*"                # Concatenate all with IFS
//   var="${array[*]}"       # Concatenate array elements

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_TO_STRING: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: var="$@" or var="${array[@]}"
    Regex::new(r#"([A-Za-z_][A-Za-z0-9_]*)="(\$@|\$\{[A-Za-z_][A-Za-z0-9_]*\[@\]\})""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip array assignments (with parentheses)
        if line.contains("=(") {
            continue;
        }

        for cap in ARRAY_TO_STRING.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let array_ref = cap.get(2).unwrap().as_str();

            let start_col = cap.get(0).unwrap().start() + 1;
            let end_col = cap.get(0).unwrap().end() + 1;

            // Suggest using * for concatenation or () for array
            let suggestion = if array_ref == "$@" {
                "Use var=(\"$@\") to assign as array, or var=\"$*\" to concatenate"
            } else {
                "Use var=(\"${array[@]}\") to assign as array, or var=\"${array[*]}\" to concatenate"
            };

            let diagnostic = Diagnostic::new(
                "SC2124",
                Severity::Warning,
                format!(
                    "Assigning an array to a string! {}. Assignment to {} loses all but first element",
                    suggestion, var_name
                ),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2124_at_to_string() {
        let code = r#"var="$@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2124");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("array"));
    }

    #[test]
    fn test_sc2124_array_at_to_string() {
        let code = r#"str="${array[@]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2124_array_assignment_ok() {
        let code = r#"var=("$@")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_star_concatenation_ok() {
        let code = r#"var="$*""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_array_star_ok() {
        let code = r#"str="${array[*]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_array_index_ok() {
        let code = r#"val="${array[0]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_regular_var_ok() {
        let code = r#"var="$other""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_multiple_assignments() {
        let code = r#"
a="$@"
b="${arr[@]}"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2124_in_command_ok() {
        let code = r#"echo "$@""#;
        let result = check(code);
        // Not an assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2124_array_to_array_ok() {
        let code = r#"new_arr=("${old_arr[@]}")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
