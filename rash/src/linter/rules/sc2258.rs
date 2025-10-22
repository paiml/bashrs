// SC2258: Prefer [[ ]] or quote to prevent glob matching (placeholder - overlaps SC2086)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2086
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2258_placeholder() {
        assert_eq!(check("[ $var = pattern ]").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_quoted() {
        assert_eq!(check(r#"[ "$var" = pattern ]"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_double_bracket() {
        assert_eq!(check("[[ $var = pattern ]]").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_glob() {
        assert_eq!(check("[ $file = *.txt ]").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_case() {
        assert_eq!(check("case $x in *.txt) ;;").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_test() {
        assert_eq!(check("test $var = value").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2258_if() {
        assert_eq!(check("if [ $status = ok ]; then").diagnostics.len(), 0);
    }
}
