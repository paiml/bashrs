// SC2203: DoS via default assignment
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static RECURSIVE_DEFAULT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match ${var:=...$var} or ${var:=${var}}
    // Can't use backreferences, so capture both and compare manually
    Regex::new(r"\$\{(\w+):=.*\$\{?(\w+)\}?").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if the two variables in the pattern match
        for cap in RECURSIVE_DEFAULT.captures_iter(line) {
            let var1 = cap.get(1).map_or("", |m| m.as_str());
            let var2 = cap.get(2).map_or("", |m| m.as_str());

            if var1 == var2 {
                let diagnostic = Diagnostic::new(
                    "SC2203",
                    Severity::Warning,
                    "Recursive default assignment can cause infinite loop".to_string(),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2203_recursive() {
        let code = r#"echo "${var:=$var}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2203_normal_ok() {
        let code = r#"echo "${var:=default}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
