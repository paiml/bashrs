// SC1139: Use || instead of -o in [[ ]]
//
// The -o operator does not work in [[ ]] conditional expressions.
// Use || instead for logical OR.
// Note: This is similar to SC2109 but focuses specifically on the parse error
// aspect — -o is a test(1) operator, not a [[ ]] operator.
//
// Examples:
// Bad:
//   [[ $x -eq 1 -o $y -eq 2 ]]   # -o not valid in [[ ]]
//   [[ -f file -o -d dir ]]       # Will not work as expected
//
// Good:
//   [[ $x -eq 1 || $y -eq 2 ]]   # Correct: use ||
//   [[ -f file || -d dir ]]       # Correct: use ||
//   [ $x -eq 1 -o $y -eq 2 ]     # -o is valid in [ ]

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches [[ ... -o ... ]] — using -o inside double brackets
static DOUBLE_BRACKET_O: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\[\[.*\s-o\s.*\]\]").expect("SC1139 regex must compile")
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        for mat in DOUBLE_BRACKET_O.find_iter(line) {
            let matched = mat.as_str();

            // Find all positions of -o within the match
            let mut search_pos = 0;
            while let Some(o_pos) = matched[search_pos..].find(" -o ") {
                let actual_pos = search_pos + o_pos;
                let col = mat.start() + actual_pos + 1; // +1 for 1-based

                result.add(Diagnostic::new(
                    "SC1139",
                    Severity::Warning,
                    "Use || instead of -o in [[ ]]. The -o operator is not supported in [[ ]]."
                        .to_string(),
                    Span::new(line_num, col + 1, line_num, col + 4),
                ));

                search_pos = actual_pos + 3;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1139_o_in_double_brackets() {
        let code = "[[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1139");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("||"));
    }

    #[test]
    fn test_sc1139_file_tests() {
        let code = "[[ -f file -o -d dir ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1139_or_pipes_ok() {
        let code = "[[ $x -eq 1 || $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1139_single_bracket_ok() {
        let code = "[ $x -eq 1 -o $y -eq 2 ]";
        let result = check(code);
        // -o is valid in single brackets
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1139_comment_ok() {
        let code = "# [[ $x -eq 1 -o $y -eq 2 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1139_multiple_o() {
        let code = "[[ $a -eq 1 -o $b -eq 2 -o $c -eq 3 ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1139_in_if_statement() {
        let code = "if [[ -f foo -o -d bar ]]; then echo yes; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
