// SC2267: Use parameter expansion instead of sed for simple substitutions
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SED_SIMPLE_SUBST: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"sed\s+['"]s/[^/]+/[^/]+/['"]\s*<<<?\s*\$"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SED_SIMPLE_SUBST.is_match(line) {
            let diagnostic = Diagnostic::new(
                "SC2267",
                Severity::Info,
                "Use ${var//old/new} instead of sed for simple substitutions".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
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
    fn test_sc2267_sed_substitution() {
        let code = r#"result=$(sed 's/old/new/' <<< $var)"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2267_parameter_expansion_ok() {
        let code = r#"result=${var//old/new}"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_sed_with_file_ok() {
        let code = r#"sed 's/old/new/' file.txt"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_comment() {
        let code = r#"# sed 's/old/new/' <<< $var"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_sed_double_quotes() {
        let code = r#"result=$(sed "s/foo/bar/" <<< $var)"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2267_complex_sed_ok() {
        let code = r#"sed 's/old/new/; s/foo/bar/' file.txt"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_sed_redirect_ok() {
        let code = r#"sed 's/old/new/' < input.txt"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2267_parameter_expansion() {
        let code = r#"new_var="${old_var/pattern/replacement}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
