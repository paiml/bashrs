// SC2264: Prefer $(..) for command substitution (placeholder - overlaps SC2225/SC2240)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2225
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2264_placeholder() {
        assert_eq!(check("var=`cmd`").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_dollar_paren() {
        assert_eq!(check("var=$(cmd)").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_nested() {
        assert_eq!(check("var=$(echo `date`)").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_echo() {
        assert_eq!(check("echo `pwd`").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_assignment() {
        assert_eq!(check("x=`ls`").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_for_loop() {
        assert_eq!(check("for f in `ls`; do").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2264_if_test() {
        assert_eq!(check("if [ \"`cmd`\" = x ]; then").diagnostics.len(), 0);
    }
}
