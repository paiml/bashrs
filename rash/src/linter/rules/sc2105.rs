// SC2105: break/continue outside loop
//
// Detects break or continue statements used outside of loop contexts.
// These statements are only valid within while, for, until, or select loops.
//
// Examples:
// Bad:
//   if [ "$var" = "skip" ]; then
//       continue  # ERROR: not in a loop
//   fi
//
// Good:
//   for file in *.txt; do
//       if [ ! -r "$file" ]; then
//           continue  # OK: inside loop
//       fi
//   done

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LOOP_START: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(while|for|until|select)\s+").unwrap());

static LOOP_END: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bdone\b").unwrap());

static BREAK_CONTINUE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(break|continue)\b").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut loop_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Track loop depth
        if LOOP_START.is_match(line) {
            loop_depth += 1;
        }

        if LOOP_END.is_match(line) {
            loop_depth = loop_depth.saturating_sub(1);
        }

        // Check for break/continue
        if let Some(cap) = BREAK_CONTINUE.find(line) {
            if loop_depth == 0 {
                let keyword = cap.as_str();
                let start_col = cap.start() + 1;
                let end_col = cap.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2105",
                    Severity::Error,
                    format!("'{}' is only valid in loops", keyword),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2105_break_outside_loop() {
        let code = r#"
if [ "$skip" = "true" ]; then
    break
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2105");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("break"));
    }

    #[test]
    fn test_sc2105_continue_outside_loop() {
        let code = r#"
if [ ! -r "$file" ]; then
    continue
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2105");
        assert!(result.diagnostics[0].message.contains("continue"));
    }

    #[test]
    fn test_sc2105_break_in_while_loop_ok() {
        let code = r#"
while read -r line; do
    if [ "$line" = "STOP" ]; then
        break
    fi
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_continue_in_for_loop_ok() {
        let code = r#"
for file in *.txt; do
    if [ ! -r "$file" ]; then
        continue
    fi
    echo "$file"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_nested_loops() {
        let code = r#"
for i in 1 2 3; do
    for j in a b c; do
        if [ "$j" = "b" ]; then
            continue
        fi
    done
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_until_loop_ok() {
        let code = r#"
until [ "$count" -eq 10 ]; do
    if [ "$count" -eq 5 ]; then
        break
    fi
    count=$((count + 1))
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_select_loop_ok() {
        let code = r#"
select option in "Continue" "Exit"; do
    if [ "$option" = "Exit" ]; then
        break
    fi
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_break_after_loop() {
        let code = r#"
for i in 1 2 3; do
    echo "$i"
done
break
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2105_no_loops_or_breaks() {
        let code = r#"
echo "Hello"
ls -la
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2105_function_without_loop() {
        let code = r#"
function process() {
    if [ "$1" = "skip" ]; then
        continue
    fi
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

/// PMAT-244 (contracts/linter-lexer-context-v1.yaml F-SC2105-PMAT244-*):
/// a one-line loop must be read as a loop, not as three independent
/// keyword-counts computed for the whole physical line.
#[cfg(test)]
mod tests_pmat244 {
    use super::*;

    #[test]
    fn test_PMAT255_pmat244_v1_inline_andand_break_is_clean() {
        let code = "for m in a b; do [[ $m == b ]] && { x=1; break; }; done\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "break is inside the one-line for-loop, got {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_PMAT255_pmat244_v2_inline_if_break_is_clean() {
        let code = "for m in a b; do if [[ $m == b ]]; then x=1; break; fi; done\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "break is inside the one-line for-loop, got {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_PMAT255_pmat244_v3_multiline_still_clean() {
        let code = r#"
for m in a b; do
    if [[ $m == b ]]; then
        x=1
        break
    fi
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);

        let while_code = "while true; do\n    break\ndone\n";
        let while_result = check(while_code);
        assert_eq!(while_result.diagnostics.len(), 0);

        let while_inline = "while true; do break; done\n";
        let while_inline_result = check(while_inline);
        assert_eq!(while_inline_result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT255_pmat244_v5_toplevel_break_still_fires() {
        let code = "break\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2105");
    }
}
