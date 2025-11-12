// SC2171: Found trailing ] without opening [.
//
// Unmatched ] indicates syntax error or typo.
//
// Examples:
// Bad:
//   if  "$a" = x ]; then         // Missing [
//   ] && echo "ok"                // Standalone ]
//
// Good:
//   if [ "$a" = x ]; then         // Matched brackets
//   [[ "$a" = x ]] && echo "ok"   // Proper syntax
//
// Impact: Syntax error

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TRAILING_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\]").unwrap());
static HEREDOC_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"<<-?\s*'?(\w+)'?").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut in_heredoc = false;
    let mut heredoc_marker: Option<String> = None;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;

        // Check if entering heredoc
        if !in_heredoc {
            if let Some(caps) = HEREDOC_START.captures(line) {
                if let Some(marker) = caps.get(1) {
                    heredoc_marker = Some(marker.as_str().to_string());
                    in_heredoc = true;
                    continue;
                }
            }
        }

        // Check if exiting heredoc
        if in_heredoc {
            if let Some(ref marker) = heredoc_marker {
                if line.trim() == marker {
                    in_heredoc = false;
                    heredoc_marker = None;
                }
            }
            continue; // Skip all lines inside heredoc
        }

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for line starting with ]
        if TRAILING_BRACKET.is_match(line) {
            let start_col = line.find(']').map(|i| i + 1).unwrap_or(1);
            let end_col = start_col + 1;

            let diagnostic = Diagnostic::new(
                "SC2171",
                Severity::Error,
                "Found trailing ] without opening [".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2171_trailing_bracket() {
        let code = "] && echo ok";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2171_if_missing_open() {
        let code = r#"if  "$a" = x ]; then"#;
        let result = check(code);
        // Would need more context to detect this case
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_matched_ok() {
        let code = r#"[ "$a" = x ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_comment_ok() {
        let code = "# ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_array_subscript_ok() {
        let code = r#"echo "${arr[0]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_double_bracket_ok() {
        let code = "[[ $a = x ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_standalone_close() {
        let code = "  ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2171_multiple() {
        let code = "]\n]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2171_end_of_test_ok() {
        let code = "if [ -f file ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_case_pattern_ok() {
        let code = "  pattern)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_21_json_bracket_in_heredoc() {
        // Reproduce issue #21: JSON bracket in heredoc should not trigger SC2171
        let code = r#"#!/bin/bash
cat > config.json <<'EOF'
{
  "transitions": [
    {"from": "a", "to": "b"}
  ]
}
EOF"#;
        let result = check(code);
        // EXPECTED: No diagnostics (heredoc content is data, not bash syntax)
        assert_eq!(
            result.diagnostics.len(),
            0,
            "JSON brackets inside heredocs should not trigger SC2171"
        );
    }

    #[test]
    fn test_issue_21_yaml_bracket_in_heredoc() {
        let code = r#"cat <<EOF
items:
  - name: test
    values: [1, 2, 3]
EOF"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "YAML brackets inside heredocs should not trigger SC2171");
    }

    #[test]
    fn test_issue_21_multiline_heredoc() {
        let code = r#"cat <<'END'
line 1
  ]
line 3
END"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "Brackets inside heredocs should be ignored");
    }

    #[test]
    fn test_sc2171_heredoc_dash_variant() {
        // Test <<- variant (strip leading tabs)
        let code = r#"cat <<-EOF
	]
EOF"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_outside_heredoc_still_detects() {
        // Ensure we still detect actual errors outside heredocs
        let code = r#"cat <<EOF
valid heredoc
EOF
]
echo "after heredoc""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "Should detect ] outside heredoc");
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_heredoc_content_never_triggers_sc2171(
                content in r"[ \]\[\{\}a-zA-Z0-9\n]{1,100}"
            ) {
                // Any content inside heredoc should not trigger SC2171
                let code = format!("cat <<EOF\n{}\nEOF", content);
                let result = check(&code);
                prop_assert_eq!(result.diagnostics.len(), 0);
            }

            #[test]
            fn prop_standalone_bracket_always_detected(
                prefix in r"[ \t]{0,10}"
            ) {
                // Standalone ] should always be detected (outside heredoc)
                let code = format!("{}]", prefix);
                let result = check(&code);
                prop_assert_eq!(result.diagnostics.len(), 1);
            }

            #[test]
            fn prop_heredoc_markers_are_case_sensitive(
                marker in r"[A-Z]{3,10}"
            ) {
                // Heredoc markers are case-sensitive
                let code = format!("cat <<{}\n  ]\n{}", marker, marker);
                let result = check(&code);
                prop_assert_eq!(result.diagnostics.len(), 0, "Bracket inside heredoc with marker {} should not trigger", marker);
            }
        }
    }
}
