// SC2211: Constant without $ not dereferenced
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static BARE_CONSTANT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match literal words in arithmetic/comparison contexts without $
    // Examples: [ MAX -gt 10 ] should be [ $MAX -gt 10 ]
    Regex::new(r"\[\s+([A-Z_][A-Z0-9_]*)\s+(-eq|-ne|-gt|-lt|-ge|-le)\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] (different semantics)
        if line.contains("[[") {
            continue;
        }

        if let Some(cap) = BARE_CONSTANT.captures(line) {
            let var = cap.get(1).map_or("", |m| m.as_str());
            // Only flag if it looks like a constant (all uppercase)
            if var
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            {
                let diagnostic = Diagnostic::new(
                    "SC2211",
                    Severity::Warning,
                    format!(
                        "Constant '{}' needs $ to be dereferenced. Use [ ${} -gt ... ]",
                        var, var
                    ),
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
    fn test_sc2211_bare_constant() {
        let code = r#"[ MAX -gt 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2211_with_dollar_ok() {
        let code = r#"[ $MAX -gt 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2211_lowercase_ok() {
        let code = r#"[ max -gt 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Lowercase not flagged
    }
    #[test]
    fn test_sc2211_double_bracket_ok() {
        let code = r#"[[ MAX -gt 10 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2211_ne_operator() {
        let code = r#"[ COUNT -ne 0 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2211_lt_operator() {
        let code = r#"[ LIMIT -lt 100 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2211_with_underscore() {
        let code = r#"[ MAX_VALUE -ge 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2211_comment_skipped() {
        let code = r#"# [ MAX -gt 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2211_numbers_in_name() {
        let code = r#"[ MAX_2 -eq 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2211_normal_string_ok() {
        let code = r#"[ "$status" = "OK" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
