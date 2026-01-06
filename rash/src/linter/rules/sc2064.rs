// SC2064: Trap quote checking - THIN SHIM
// All logic extracted to sc2064_logic.rs

use super::sc2064_logic;
use crate::linter::LintResult;

pub fn check(source: &str) -> LintResult {
    let result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        if sc2064_logic::is_comment_line(line) {
            continue;
        }
        // F082: SC2064 disabled - double quotes in trap is intentional
        let _ = (line_num, line);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_returns_empty() {
        let result = check(r#"trap "rm $tmpfile" EXIT"#);
        assert!(result.diagnostics.is_empty());
    }
}
