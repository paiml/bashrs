// SC2242: Can only break/continue from loops, not from a `case` outside one.
//
// Nesting is counted, NOT flagged. Three booleans cannot describe nesting:
// an inner `done` cleared `in_loop` while the outer `while` was still open,
// and a ONE-LINE `case … esac` never cleared `in_case` because the closer
// test demanded the line BE `esac`. Together they made this shape a false
// positive — valid bash, accepted by `bash -n`, reported as an error:
//
//     while IFS= read -r id; do
//       case "$id" in '') continue ;; esac
//       for a in 1 2; do
//         for b in 3 4; do printf '%s%s\n' "$a" "$b"; done
//       done
//       if [ -z "$id" ]; then
//         continue            # <- reported as "outside a loop"
//       fi
//     done < <(printf 'x\n')
//
// This is still a line/keyword heuristic, not a parser: it counts shell
// KEYWORD TOKENS per line, so `case`/`esac` and `for`/`while`/`until`/`done`
// opening and closing on one line balance correctly. A keyword inside a
// quoted string or a heredoc is still miscounted; that needs the parser, and
// the point of this change is that the counting is no longer wrong for code
// anyone writes.
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check if line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Splits a line into shell-ish word tokens. `;`, `(`, `)`, `&`, `|` and
/// backticks are separators, so `a) break ;; esac` yields `break` and
/// `esac` as tokens of their own.
fn tokens(line: &str) -> Vec<&str> {
    line.split(|c: char| c.is_whitespace() || matches!(c, ';' | '(' | ')' | '&' | '|' | '`'))
        .filter(|t| !t.is_empty())
        .collect()
}

/// How many times `keyword` appears as a WORD on this line.
fn count_keyword(line: &str, keyword: &str) -> usize {
    tokens(line).iter().filter(|t| **t == keyword).count()
}

/// Loop openers: `for`, `while`, `until`. `do` is not counted — it belongs
/// to the opener that precedes it, and `done` is the single closer.
fn count_loop_starts(line: &str) -> usize {
    count_keyword(line, "for") + count_keyword(line, "while") + count_keyword(line, "until")
}

/// Check if line starts a function
fn is_function_start(line: &str) -> bool {
    line.trim_start().contains("() {") || line.trim_start().starts_with("function ")
}

/// Check if line ends a function
fn is_function_end(line: &str) -> bool {
    line.trim_start() == "}"
}

/// Check if line contains break or continue
fn has_break_or_continue(line: &str) -> bool {
    tokens(line)
        .iter()
        .any(|t| *t == "break" || *t == "continue")
}

/// Build diagnostic for invalid break/continue in case
fn build_diagnostic(line_num: usize, line_len: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2242",
        Severity::Error,
        "Can only break/continue from loops. Use 'exit' to exit case or function".to_string(),
        Span::new(line_num, 1, line_num, line_len + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut case_depth: usize = 0;
    let mut loop_depth: usize = 0;
    let mut fn_depth: usize = 0;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        let opened_case = count_keyword(line, "case");
        let closed_case = count_keyword(line, "esac");
        let opened_loop = count_loop_starts(line);
        let closed_loop = count_keyword(line, "done");

        // Openers count BEFORE the check so a `case … ) break ;; esac`
        // written on one line is still judged inside its own case; closers
        // count AFTER, so that same line's `esac` does not exempt it.
        case_depth += opened_case;
        loop_depth += opened_loop;
        if is_function_start(line) {
            fn_depth += 1;
        }

        if case_depth > 0 && loop_depth == 0 && fn_depth == 0 && has_break_or_continue(line) {
            result.add(build_diagnostic(line_num, line.len()));
        }

        case_depth = case_depth.saturating_sub(closed_case);
        loop_depth = loop_depth.saturating_sub(closed_loop);
        if is_function_end(line) {
            fn_depth = fn_depth.saturating_sub(1);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2242_break_in_case() {
        let code = "case $x in\n  a) break;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2242_break_in_loop_ok() {
        let code = "while true; do\n  break\ndone";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_continue_in_case() {
        let code = "case $x in\n  a) continue;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2242_exit_in_case_ok() {
        let code = "case $x in\n  a) exit 1;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_comment_skipped() {
        let code = "# case x in a) break;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_case_in_loop() {
        let code = "for x in *; do\n  case $x in\n    *.txt) break;;\n  esac\ndone";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // In loop, so break is valid
    }
    #[test]
    fn test_sc2242_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_normal_command() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_return_in_function_ok() {
        let code = "foo() {\n  case $1 in\n    a) return 1;;\n  esac\n}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2242_simple_case() {
        let code = "case $var in\n  x) echo ok;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // No break/continue
    }
}

#[cfg(test)]
mod nesting_tests {
    use super::*;

    /// The false positive this rule's counters were rewritten for: valid
    /// bash that `bash -n` accepts and that runs, reported as an error.
    /// Reverting `check` to the three booleans turns this red.
    #[test]
    fn a_continue_after_nested_loops_close_is_not_in_the_case() {
        let code = "while IFS= read -r id; do\n\
                    \x20 case \"$id\" in '') continue ;; esac\n\
                    \x20 for a in 1 2; do\n\
                    \x20   for b in 3 4; do\n\
                    \x20     printf '%s%s\\n' \"$a\" \"$b\"\n\
                    \x20   done\n\
                    \x20 done\n\
                    \x20 if [ -z \"$id\" ]; then\n\
                    \x20   continue\n\
                    \x20 fi\n\
                    done < <(printf 'x\\n')";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "{:?}",
            result
                .diagnostics
                .iter()
                .map(|d| d.span.start_line)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_one_line_case_closes_on_its_own_line() {
        // Before: `is_case_end` demanded the line BE `esac`, so a one-line
        // case stayed open for the rest of the file and every later
        // break/continue outside a loop was flagged.
        let code = "case $x in a) echo hi ;; esac\nbreak";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn an_inner_done_does_not_close_the_outer_loop() {
        let code = "for a in 1 2; do\n  for b in 3; do\n    echo x\n  done\n  break\ndone";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn a_break_in_a_case_outside_any_loop_is_still_an_error() {
        // The rule must still be capable of firing — the whole point.
        let code = "case $x in\n  a) break ;;\nesac";
        assert_eq!(check(code).diagnostics.len(), 1);
        let one_line = "case $x in a) break ;; esac";
        assert_eq!(check(one_line).diagnostics.len(), 1);
    }

    #[test]
    fn a_word_containing_a_keyword_is_not_the_keyword() {
        // `donefile` / `casework` are not `done` / `case`.
        let code = "case $x in\n  a) donefile=1; break ;;\nesac";
        assert_eq!(check(code).diagnostics.len(), 1);
        let code2 = "for f in casework; do\n  break\ndone";
        assert_eq!(check(code2).diagnostics.len(), 0);
    }

    #[test]
    fn a_case_inside_a_loop_inside_a_case_still_resolves() {
        let code = "case $x in\n  a)\n    for f in 1; do\n      case $f in 1) continue ;; esac\n    done\n    break ;;\nesac";
        // The outer `break` is inside a case with NO enclosing loop.
        assert_eq!(check(code).diagnostics.len(), 1);
    }
}
