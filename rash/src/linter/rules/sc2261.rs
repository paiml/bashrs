// SC2261: Multiple redirections compete for stdout (placeholder - complex detection)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Requires parsing redirect chains
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2261_placeholder() {
        assert_eq!(check("cmd > file1 > file2").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_single_ok() {
        assert_eq!(check("cmd > file").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_stderr() {
        assert_eq!(check("cmd > out 2> err").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_append() {
        assert_eq!(check("cmd > file1 >> file2").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_pipe() {
        assert_eq!(check("cmd | tee file").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_both() {
        assert_eq!(check("cmd &> all").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2261_fd() {
        assert_eq!(check("cmd 3> file").diagnostics.len(), 0);
    }
}
