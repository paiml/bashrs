// SC1090: Can't follow non-constant source
//
// When source/dot is used with a variable argument, the linter cannot
// statically determine which file is being sourced.
//
// Examples:
// Bad:
//   source "$config_file"
//   . "$HOME/.bashrc"
//   source "${DIR}/lib.sh"
//
// Good:
//   source /etc/profile
//   . ./lib.sh
//
// Impact: Informational - static analysis cannot follow variable sources

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Check for `source $var` patterns
        if let Some(rest) = trimmed.strip_prefix("source") {
            // Must be followed by whitespace (not part of a longer word)
            if rest.starts_with(char::is_whitespace) {
                let arg = rest.trim_start();
                if arg_is_variable(arg) {
                    result.add(Diagnostic::new(
                        "SC1090",
                        Severity::Info,
                        "SC1090: Can't follow non-constant source. Use a directive to specify location".to_string(),
                        Span::new(line_num, 1, line_num, line.len() + 1),
                    ));
                }
            }
        }

        // Check for `. $var` patterns (dot-sourcing)
        // The dot must be standalone: `. ` not `./` or `..`
        if is_dot_source(trimmed) {
            let rest = &trimmed[1..]; // skip the dot
            let arg = rest.trim_start();
            if arg_is_variable(arg) {
                result.add(Diagnostic::new(
                    "SC1090",
                    Severity::Info,
                    "SC1090: Can't follow non-constant source. Use a directive to specify location"
                        .to_string(),
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
    // Must have at least 2 chars, and second char must be whitespace
    if trimmed.len() < 2 {
        return false;
    }
    let second = trimmed.as_bytes()[1];
    second == b' ' || second == b'\t'
}

/// Check if the argument starts with `$` (variable reference)
fn arg_is_variable(arg: &str) -> bool {
    // Strip optional leading quote (either " or ')
    let unquoted = arg
        .strip_prefix('"')
        .or_else(|| arg.strip_prefix('\''))
        .unwrap_or(arg);
    unquoted.starts_with('$')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1090_source_variable() {
        let code = r#"source "$config_file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1090");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_sc1090_dot_variable() {
        let code = r#". "$HOME/.bashrc""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1090");
    }

    #[test]
    fn test_sc1090_source_env_var() {
        let code = "source ${DIR}/lib.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1090_literal_path_no_match() {
        let code = "source /etc/profile";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_dot_literal_no_match() {
        let code = ". ./lib.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_dot_slash_path_no_match() {
        // `./script` is running a script, not dot-sourcing
        let code = "./script.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_double_dot_no_match() {
        let code = "../script.sh";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_comment_no_match() {
        let code = r#"# source "$config_file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_empty_source() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1090_source_unquoted_var() {
        let code = "source $MY_CONFIG";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1090_multiple_lines() {
        let code = "source $a\nsource /etc/profile\n. $b";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
