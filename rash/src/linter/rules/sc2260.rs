// SC2260: Arithmetic context already handles expansion (placeholder - overlaps SC2245)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2245
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2260_placeholder() {
        assert_eq!(check("(( $x + 1 ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_no_dollar() {
        assert_eq!(check("(( x + 1 ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_expr() {
        assert_eq!(check("$(( $a + $b ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_let() {
        assert_eq!(check("let x=$y+1").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_assign() {
        assert_eq!(check("(( x = $y ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_compare() {
        assert_eq!(check("(( $a > $b ))").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2260_increment() {
        assert_eq!(check("(( $i++ ))").diagnostics.len(), 0);
    }
}
