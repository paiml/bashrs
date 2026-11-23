// SC2274: intentionally silent.
//
// ShellCheck does not flag `[ a ] && [ b ]` (POSIX-correct; `[[` is a bashism),
// so reporting it is a false positive. Kept as a no-op so the rule id stays registered.
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2274_separate_tests() {
        let code = "[ -f file ] && [ -r file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_combined_ok() {
        let code = "[[ -f file && -r file ]]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_comment() {
        let code = "# [ test ] && [ test ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_single_test_ok() {
        let code = "[ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_command_and_test_ok() {
        let code = "command && [ -f file ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_test_and_command_ok() {
        let code = "[ -f file ] && command";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_or_tests() {
        let code = "[ -f file ] || [ -d dir ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2274_three_tests() {
        let code = "[ -f f1 ] && [ -f f2 ] && [ -f f3 ]";
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
