// SC2255: Quote to prevent glob expansion (placeholder - overlaps SC2086)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2086
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2255_placeholder() {
        assert_eq!(check("echo $var").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_quoted() {
        assert_eq!(check(r#"echo "$var""#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_simple() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_glob() {
        assert_eq!(check("echo *.txt").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_var_glob() {
        assert_eq!(check("echo $pattern").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_for_loop() {
        assert_eq!(check("for f in $files; do").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_array_glob() {
        assert_eq!(check("arr=($var)").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2255_test() {
        assert_eq!(check("[ -f $file ]").diagnostics.len(), 0);
    }
}
