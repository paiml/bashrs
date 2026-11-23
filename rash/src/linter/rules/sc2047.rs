// SC2047: shellcheck's own SC2047 is a different, long-retired check; for an
// unquoted `$var` operand of `[ ]` / `test` shellcheck reports SC2086 (and
// bashrs's SC2086 already does the same, on the same span). Reporting it a
// second time here as SC2047 was pure excess against shellcheck (#236), so
// this rule emits nothing. The quoting diagnostic and its autofix live in
// SC2086.
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2047_never_reports_unquoted_test_operands() {
        for code in [
            "[ -z $var ]",
            "[ $count -gt 5 ]",
            "test $status = ok",
            "[ $a = $b ]",
            "[ -n ${x} ]",
            "[ \"$a\" = ok ]",
            "[[ -z $var ]]",
            "# [ -z $var ]",
        ] {
            assert_eq!(check(code).diagnostics.len(), 0, "{code}");
        }
    }
}
