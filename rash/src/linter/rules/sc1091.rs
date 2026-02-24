// SC1091: Not following: <path>. Use shellcheck -x to follow sourced files
//
// When source/dot is used with a literal constant path, the linter notes
// that it is not verifying the sourced file exists or is valid.
//
// Examples:
// Flagged:
//   source ./lib.sh
//   . /etc/profile
//   source ../helpers/utils.sh
//
// Not flagged:
//   source "$variable"          # SC1090 handles this
//   source "${DIR}/lib.sh"      # SC1090 handles this
//
// Impact: Informational - linter does not follow sourced files

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Check `source <literal_path>`
        if let Some(rest) = trimmed.strip_prefix("source") {
            if rest.starts_with(char::is_whitespace) {
                let arg = rest.trim_start();
                if is_literal_path(arg) {
                    let path = extract_path(arg);
                    result.add(Diagnostic::new(
                        "SC1091",
                        Severity::Info,
                        format!(
                            "SC1091: Not following: {}. Use shellcheck -x to follow sourced files",
                            path
                        ),
                        Span::new(line_num, 1, line_num, line.len() + 1),
                    ));
                }
            }
        }

        // Check `. <literal_path>` (dot-source, not `./path`)
        if is_dot_source(trimmed) {
            let rest = &trimmed[1..];
            let arg = rest.trim_start();
            if is_literal_path(arg) {
                let path = extract_path(arg);
                result.add(Diagnostic::new(
                    "SC1091",
                    Severity::Info,
                    format!(
                        "SC1091: Not following: {}. Use shellcheck -x to follow sourced files",
                        path
                    ),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                ));
            }
        }
    }

    result
}

/// Check if a trimmed line starts with dot-source (`. ` but not `./` or `..`)
fn is_dot_source(trimmed: &str) -> bool {
    if !trimmed.starts_with('.') {
        return false;
    }
    if trimmed.len() < 2 {
        return false;
    }
    let second = trimmed.as_bytes()[1];
    second == b' ' || second == b'\t'
}

/// Check if argument is a literal path (not a variable)
fn is_literal_path(arg: &str) -> bool {
    if arg.is_empty() {
        return false;
    }
    // Strip optional leading quote (either " or ')
    let unquoted = arg
        .strip_prefix('"')
        .or_else(|| arg.strip_prefix('\''))
        .unwrap_or(arg);
    // A literal path does NOT start with $
    !unquoted.starts_with('$') && !unquoted.is_empty()
}

/// Extract the path argument, stripping surrounding quotes
fn extract_path(arg: &str) -> &str {
    let first_word = arg.split_whitespace().next().unwrap_or(arg);
    first_word
        .trim_matches('"')
        .trim_matches('\'')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1091_source_literal() {
        let code = "source ./lib.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1091");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("./lib.sh"));
    }

    #[test]
    fn test_sc1091_dot_literal() {
        let code = ". /etc/profile";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("/etc/profile"));
    }

    #[test]
    fn test_sc1091_source_relative() {
        let code = "source ../helpers/utils.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("../helpers/utils.sh"));
    }

    #[test]
    fn test_sc1091_variable_no_match() {
        // Variables are SC1090's domain
        let code = r#"source "$config""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1091_dot_variable_no_match() {
        let code = ". $HOME/.bashrc";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1091_comment_no_match() {
        let code = "# source ./lib.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1091_dot_slash_executable_no_match() {
        // ./script is running, not sourcing
        let code = "./script.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1091_empty_source() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1091_quoted_literal_path() {
        let code = r#"source "/usr/local/lib/functions.sh""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("/usr/local/lib/functions.sh"));
    }

    #[test]
    fn test_sc1091_multiple() {
        let code = "source ./a.sh\nsource ./b.sh\n. /etc/profile";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }
}
