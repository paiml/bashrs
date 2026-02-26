// SC2306: Prefer ${var//old/new} over sed for simple substitutions
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SED_SUBST: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r#"sed\s+['"]s/[^/]+/[^/]+/[g]?['"]"#).unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if SED_SUBST.is_match(line) && line.contains("<<<") {
            let diagnostic = Diagnostic::new(
                "SC2306",
                Severity::Info,
                "Consider using ${var//old/new} instead of sed for simple substitutions"
                    .to_string(),
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
    fn test_sc2306_sed_simple_subst() {
        let code = r#"result=$(sed 's/foo/bar/' <<< "$text")"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2306_param_expansion_ok() {
        let code = r#"result="${text//foo/bar}""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_sed_with_file_ok() {
        let code = r#"sed 's/foo/bar/' file.txt"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_comment() {
        let code = r#"# sed 's/foo/bar/' <<< "$text""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_sed_global() {
        let code = r#"result=$(sed 's/foo/bar/g' <<< "$text")"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2306_complex_sed_ok() {
        let code = r#"sed -e 's/foo/bar/' -e 's/baz/qux/' file.txt"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2306_sed_double_quotes() {
        let code = r#"result=$(sed "s/foo/bar/" <<< "$text")"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2306_multiple_lines() {
        let code = r#"
result1=$(sed 's/a/b/' <<< "$x")
result2=$(sed 's/c/d/' <<< "$y")
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }
}
