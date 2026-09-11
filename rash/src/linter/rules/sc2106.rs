// SC2106: This only exits the subshell caused by the pipeline (or parentheses)
//
// `break` and `continue` unwind the loop stack of the shell process that
// executes them. A loop body that itself runs in a subshell — because it is
// wrapped in `( ... )`, or because it sits on the right side of a pipe
// (`producer | while ...; do ...; done` runs the whole `while` in a subshell
// of the pipeline) — only has that subshell's loop stack to unwind.
// `break`/`continue` there leave the subshell; they do not, and cannot,
// affect any loop the author may have been expecting to influence.
//
// Examples:
// Bad:
//   for f in *; do
//       ( break )          # only leaves the subshell; the `for` keeps going
//   done
//
//   cat x | while read -r l; do
//       break              # only leaves the pipeline's subshell
//   done
//
// Good:
//   for f in *; do
//       break              # breaks the for loop directly, no subshell involved
//   done
//
//   while read -r l; do
//       break
//   done < x               # redirect input instead of piping into the loop
//
// Impact: `break`/`continue` silently fail to affect the loop the author
// intended, so the loop the author meant to stop keeps iterating.
//
// Note: this is a rename target. The code SC2106 used to be misassigned in
// this codebase to ShellCheck's SC2009 check ("Consider using pgrep instead
// of grepping ps output"). That check now lives in `sc2009.rs` under its
// correct code; see PMAT-258 / #303.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LOOP_START: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(while|for|until|select)\s+").unwrap());

static LOOP_END: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bdone\b").unwrap());

static PAREN_GROUP: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\(([^()]*)\)").unwrap());

static BREAK_CONTINUE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(break|continue)\b").unwrap());

/// One occurrence of a loop-structure or break/continue event on a physical
/// line, tagged with its byte position so events on the same line replay in
/// the order the shell would actually encounter them (mirrors SC2105's
/// PMAT-244 line-event design).
enum LineEvent {
    /// A `while`/`for`/`until`/`select` keyword opens a new loop. `pipe_fed`
    /// is true when the keyword is immediately preceded (ignoring
    /// whitespace) by `|` — the loop then runs in a subshell of the
    /// pipeline.
    LoopStart {
        pipe_fed: bool,
    },
    LoopEnd,
    /// `break`/`continue` found wrapped in `( ... )` on this line.
    SubshellBreakContinue {
        keyword: &'static str,
        start: usize,
        end: usize,
    },
    /// `break`/`continue` found NOT wrapped in parentheses on this line.
    BareBreakContinue {
        keyword: &'static str,
        start: usize,
        end: usize,
    },
}

fn keyword_name(text: &str) -> &'static str {
    if text == "break" {
        "break"
    } else {
        "continue"
    }
}

/// Collect every loop-start, loop-end and break/continue occurrence on a
/// single physical line, in left-to-right (byte) order. A break/continue
/// captured inside a `( ... )` group is reported once, as
/// [`LineEvent::SubshellBreakContinue`]; it is excluded from the bare scan
/// so it is never double-counted.
fn line_events(line: &str) -> Vec<(usize, LineEvent)> {
    let mut events: Vec<(usize, LineEvent)> = Vec::new();
    let mut consumed: Vec<(usize, usize)> = Vec::new();

    for cap in PAREN_GROUP.captures_iter(line) {
        let inner = cap.get(1).unwrap();
        if let Some(kw) = BREAK_CONTINUE.find(inner.as_str()) {
            let start = inner.start() + kw.start();
            let end = inner.start() + kw.end();
            events.push((
                start,
                LineEvent::SubshellBreakContinue {
                    keyword: keyword_name(&line[start..end]),
                    start,
                    end,
                },
            ));
            consumed.push((start, end));
        }
    }

    for m in LOOP_START.find_iter(line) {
        let prefix = &line[..m.start()];
        let pipe_fed = prefix.trim_end().ends_with('|');
        events.push((m.start(), LineEvent::LoopStart { pipe_fed }));
    }

    for m in LOOP_END.find_iter(line) {
        events.push((m.start(), LineEvent::LoopEnd));
    }

    for m in BREAK_CONTINUE.find_iter(line) {
        if consumed
            .iter()
            .any(|&(s, e)| m.start() >= s && m.start() < e)
        {
            continue;
        }
        events.push((
            m.start(),
            LineEvent::BareBreakContinue {
                keyword: keyword_name(m.as_str()),
                start: m.start(),
                end: m.end(),
            },
        ));
    }

    events.sort_by_key(|(pos, _)| *pos);
    events
}

/// Replay one physical line's events in order, updating the running
/// loop-context stack (innermost last, `true` = the loop runs in a subshell
/// because it is pipe-fed) and reporting SC2106 where a break/continue can
/// only reach a subshell's loop stack.
fn process_line(
    line: &str,
    line_num: usize,
    mut loop_stack: Vec<bool>,
    result: &mut LintResult,
) -> Vec<bool> {
    for (_, event) in line_events(line) {
        match event {
            LineEvent::LoopStart { pipe_fed } => loop_stack.push(pipe_fed),
            LineEvent::LoopEnd => {
                loop_stack.pop();
            }
            LineEvent::SubshellBreakContinue {
                keyword,
                start,
                end,
            } => {
                // The `( ... )` itself is a subshell; a break/continue in it
                // only matters (as a defect) when there is an enclosing loop
                // it fails to reach.
                if !loop_stack.is_empty() {
                    result.add(Diagnostic::new(
                        "SC2106",
                        Severity::Warning,
                        format!(
                            "This '{keyword}' only exits the subshell caused by the \
                             parentheses, not the enclosing loop"
                        ),
                        Span::new(line_num, start + 1, line_num, end + 1),
                    ));
                }
            }
            LineEvent::BareBreakContinue {
                keyword,
                start,
                end,
            } => {
                // Only the innermost loop matters: a plain break/continue
                // exits it directly, and whether that loop runs in a
                // subshell is what determines whether the exit is visible
                // outside that subshell.
                if matches!(loop_stack.last(), Some(true)) {
                    result.add(Diagnostic::new(
                        "SC2106",
                        Severity::Warning,
                        format!(
                            "This '{keyword}' only exits the subshell caused by the \
                             pipeline, not the enclosing loop"
                        ),
                        Span::new(line_num, start + 1, line_num, end + 1),
                    ));
                }
            }
        }
    }
    loop_stack
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut loop_stack: Vec<bool> = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        loop_stack = process_line(line, line_num, loop_stack, &mut result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_PMAT258_sc2106_break_in_subshell_inside_for_loop_fires() {
        let code = "for f in *; do\n    ( break )\ndone\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC2106");
        assert!(result.diagnostics[0].message.contains("subshell"));
        assert!(result.diagnostics[0].message.contains("break"));
    }

    #[test]
    fn test_PMAT258_sc2106_continue_in_subshell_inside_for_loop_fires() {
        let code = "for f in *; do\n    ( continue )\ndone\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC2106");
        assert!(result.diagnostics[0].message.contains("continue"));
    }

    #[test]
    fn test_PMAT258_sc2106_break_in_while_read_on_right_of_pipe_fires() {
        let code = "cat x | while read l; do break; done\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC2106");
        assert!(result.diagnostics[0].message.contains("subshell"));
        assert!(result.diagnostics[0].message.contains("pipeline"));
    }

    #[test]
    fn test_PMAT258_sc2106_plain_break_in_loop_body_does_not_fire() {
        let code = "while true; do\n    break\ndone\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "a plain break directly in a loop body is SC2105's business, not SC2106: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_PMAT258_sc2106_plain_continue_in_for_loop_body_does_not_fire() {
        let code = "for i in 1 2 3; do\n    continue\ndone\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_PMAT258_sc2106_break_in_subshell_outside_any_loop_is_not_our_business() {
        // No enclosing loop: a top-level `( break )` is SC2105's "break
        // outside a loop" territory, not SC2106's "only exits the
        // subshell" territory.
        let code = "( break )\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_PMAT258_sc2106_severity_is_not_error() {
        let code = "for f in *; do\n    ( break )\ndone\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_PMAT258_sc2106_comment_ok() {
        let code = "# cat x | while read l; do break; done\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_PMAT258_sc2106_break_in_plain_inner_loop_nested_in_piped_outer_loop_does_not_fire() {
        // The innermost loop the break targets is the plain `for`, which is
        // not itself pipe-fed — only the outer `while` is. The break exits
        // the `for` directly and is not swallowed by a subshell.
        let code =
            "cat file | while read -r l; do\n    for x in a b; do\n        break\n    done\ndone\n";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    /// Regression guard (does not modify sc2105.rs, which is out of scope
    /// for PMAT-258): confirm SC2105's "break outside a loop" behaviour is
    /// unaffected by this rewrite of SC2106.
    #[test]
    fn test_PMAT258_sc2105_break_outside_loop_still_fires_as_sc2105() {
        let code = "break\n";
        let result = crate::linter::rules::sc2105::check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2105");
    }

    /// Regression guard: SC2105 still treats a plain break/continue inside a
    /// normal loop as OK (no diagnostic) — the two rules must agree that
    /// this case is clean, not merely that neither happens to fire on it.
    #[test]
    fn test_PMAT258_sc2105_break_in_loop_body_still_ok() {
        let code = "while true; do\n    break\ndone\n";
        let sc2105_result = crate::linter::rules::sc2105::check(code);
        let sc2106_result = check(code);
        assert_eq!(sc2105_result.diagnostics.len(), 0);
        assert_eq!(sc2106_result.diagnostics.len(), 0);
    }
}
