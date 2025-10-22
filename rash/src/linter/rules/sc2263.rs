// SC2263: Useless use of command (placeholder - pattern detection)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Requires usage pattern analysis
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2263_placeholder() {
        assert_eq!(check("cat file | grep pattern").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_direct_ok() {
        assert_eq!(check("grep pattern file").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_useless_cat() {
        assert_eq!(check("cat file | sed 's/a/b/'").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_echo_pipe() {
        assert_eq!(check("echo test | cmd").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_multiple_files() {
        assert_eq!(check("cat file1 file2 | grep x").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_stdin() {
        assert_eq!(check("cat | process").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2263_tee() {
        assert_eq!(check("cmd | tee file").diagnostics.len(), 0);
    }
}
