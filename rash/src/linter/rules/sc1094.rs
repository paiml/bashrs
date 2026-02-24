// SC1094: Parsing of sourced file had errors
//
// When source/dot is used, the sourced file might contain syntax errors.
// Since the linter cannot actually parse sourced files, this rule provides
// an informational note when sourcing is detected.
//
// Examples:
// Flagged:
//   source config.sh
//   . /opt/scripts/setup.sh
//
// Impact: Informational - linter cannot verify sourced file syntax

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Check `source <arg>`
        if let Some(rest) = trimmed.strip_prefix("source") {
            if rest.starts_with(char::is_whitespace) {
                let arg = rest.trim_start();
                if !arg.is_empty() {
                    let path = extract_path(arg);
                    result.add(Diagnostic::new(
                        "SC1094",
                        Severity::Info,
                        format!(
                            "SC1094: Parsing of sourced file {} was not performed",
                            path
                        ),
                        Span::new(line_num, 1, line_num, line.len() + 1),
                    ));
                }
            }
        }

        // Check `. <arg>` (dot-source, not `./path`)
        if is_dot_source(trimmed) {
            let rest = &trimmed[1..];
            let arg = rest.trim_start();
            if !arg.is_empty() {
                let path = extract_path(arg);
                result.add(Diagnostic::new(
                    "SC1094",
                    Severity::Info,
                    format!(
                        "SC1094: Parsing of sourced file {} was not performed",
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

/// Extract the path argument (first word), stripping surrounding quotes
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
    fn test_sc1094_source_literal() {
        let code = "source config.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1094");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("config.sh"));
    }

    #[test]
    fn test_sc1094_dot_source() {
        let code = ". /opt/scripts/setup.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("/opt/scripts/setup.sh"));
    }

    #[test]
    fn test_sc1094_source_with_variable() {
        let code = r#"source "$config""#;
        let result = check(code);
        // SC1094 still fires - it flags all source usage
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1094_comment_no_match() {
        let code = "# source config.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1094_dot_slash_no_match() {
        // ./script is execution, not sourcing
        let code = "./script.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1094_empty_no_match() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1094_source_alone_no_match() {
        // `source` with no argument
        let code = "source";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1094_multiple_sources() {
        let code = "source a.sh\nsource b.sh\n. c.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc1094_double_dot_no_match() {
        let code = "..";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
