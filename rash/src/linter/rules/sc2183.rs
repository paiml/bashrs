// SC2183: shellcheck's SC2183 is a printf-format arity check ("This format
// string has N variables, but is passed M arguments"), not a "variable used as
// command name" warning. shellcheck emits nothing for `$cmd arg` (#236), so
// this rule must not either. Variable-as-command-name injection is SEC002's
// subject.
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2183_var_command_not_reported_like_shellcheck() {
        for code in ["$cmd arg", "  ${cmd} arg", "n=1; $n x"] {
            assert_eq!(check(code).diagnostics.len(), 0, "{code}");
        }
    }
    #[test]
    fn test_sc2183_normal_ok() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
}
