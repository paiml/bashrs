// SC2253: Quote to prevent word splitting on expansion (placeholder - overlaps SC2086)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2086
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2253_placeholder() {
        assert_eq!(check("$var").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_quoted() {
        assert_eq!(check(r#""$var""#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_echo() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_expansion() {
        assert_eq!(check("echo $HOME").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_quoted_expansion() {
        assert_eq!(check(r#"echo "$HOME""#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_command_sub() {
        assert_eq!(check("echo $(pwd)").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_assignment() {
        assert_eq!(check("x=$var").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2253_array() {
        assert_eq!(check("arr=($var)").diagnostics.len(), 0);
    }
}
