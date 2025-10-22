// SC2257: Prefer -n to test non-empty string (placeholder - overlaps SC2244/SC2256)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2244 and SC2256
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2257_placeholder() {
        assert_eq!(check(r#"[ "$var" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_explicit() {
        assert_eq!(check(r#"[ -n "$var" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_if() {
        assert_eq!(check(r#"if [ "$x" ]; then"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_while() {
        assert_eq!(check(r#"while [ "$running" ]; do"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_z_test() {
        assert_eq!(check(r#"[ -z "$var" ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_double_bracket() {
        assert_eq!(check(r#"[[ $var ]]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2257_negated() {
        assert_eq!(check(r#"[ ! "$var" ]"#).diagnostics.len(), 0);
    }
}
