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

// Main's whole-word match; `is_not_a_command` drops only certain non-commands.
static BREAK_CONTINUE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(break|continue)\b").unwrap());

/// Is the keyword at `line[start..end]` certainly NOT a command word?
///
/// Deny-list, deliberately narrow; when unsure this returns false and the
/// rule fires. Two shapes only:
/// - glued to a neighbour that makes it part of a larger word: `-`, `.`, `/`
///   or `=` right after it; `-`, `.`, `/` or `$` right before it, or `${`
///   (`no-break`, `--break`, `break.txt`, `$break`, `${break}`), or `=` right
///   before it when the text up to there is a plain assignment that is the
///   whole command (`x=break`, `FOO=1 x=break`, never `FOO=1 break`);
/// - a plain argument of `echo` or `printf` (see `is_output_argument`).
fn is_not_a_command(line: &str, start: usize, end: usize) -> bool {
    let next = line[end..].chars().next();
    if matches!(next, Some('-' | '.' | '/' | '=')) {
        return true;
    }
    match line[..start].chars().next_back() {
        Some('-' | '.' | '/' | '$') => return true,
        Some('{') if line[..start - 1].ends_with('$') => return true,
        Some('=') => {
            if ends_the_command(&line[end..]) && is_assignment_value(&line[..start - 1]) {
                return true;
            }
        }
        _ => {}
    }
    is_output_argument(line, start)
}

/// Nothing but whitespace, or a `;`, follows: the assignment is the whole command.
fn ends_the_command(rest: &str) -> bool {
    let rest = rest.trim_start();
    rest.is_empty() || rest.starts_with(';')
}

/// `head` ends right before an `=`: is it `NAME` of a simple assignment, i.e.
/// preceded only by other assignment words since the last separator?
fn is_assignment_value(head: &str) -> bool {
    let name_start = head
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let name = &head[name_start..];
    if !name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        return false;
    }
    let before = &head[..name_start];
    let seg_start = before.rfind([';', '&', '|', '(']).map_or(0, |i| i + 1);
    let seg = &before[seg_start..];
    // `FOO=1 x=break`: whitespace-separated words before the name must all be
    // assignments; `echo x=break` and `FOO=1 break` are not.
    (seg.is_empty() || seg.ends_with(char::is_whitespace))
        && seg.split_whitespace().all(|w| w.contains('='))
}

/// Is the keyword a plain argument of `echo` or `printf`? The simple command
/// The simple command is the text since the last `;` `&` `|` `(` `)`; its first
/// word must be `echo`/`printf`, with no quote, backtick, redirect or `$` before.
fn is_output_argument(line: &str, start: usize) -> bool {
    let head = &line[..start];
    let seg_start = head.rfind([';', '&', '|', '(', ')']).map_or(0, |i| i + 1);
    let seg = &head[seg_start..];
    if seg.contains(['`', '"', '\'', '\\', '<', '>', '$']) {
        return false;
    }
    let first = seg.split_whitespace().next();
    matches!(first, Some("echo" | "printf")) && seg.ends_with(char::is_whitespace)
}

/// One occurrence of a loop-structure keyword on a physical line, tagged
/// with its byte position so events on the same line can be replayed in the
/// order the shell would actually encounter them (PMAT-244).
enum LineEvent {
    LoopStart,
    LoopEnd,
    BreakContinue {
        keyword: &'static str,
        start: usize,
        end: usize,
    },
}

/// Collect every loop-start, loop-end and break/continue occurrence on a
/// single physical line, in left-to-right (byte) order.
fn line_events(line: &str) -> Vec<(usize, LineEvent)> {
    let mut events: Vec<(usize, LineEvent)> = Vec::new();

    for m in LOOP_START.find_iter(line) {
        events.push((m.start(), LineEvent::LoopStart));
    }
    for m in LOOP_END.find_iter(line) {
        events.push((m.start(), LineEvent::LoopEnd));
    }
    for caps in BREAK_CONTINUE.captures_iter(line) {
        let m = caps.get(1).expect("group 1 always participates");
        if is_not_a_command(line, m.start(), m.end()) {
            continue;
        }
        let keyword = if m.as_str() == "break" {
            "break"
        } else {
            "continue"
        };
        events.push((
            m.start(),
            LineEvent::BreakContinue {
                keyword,
                start: m.start(),
                end: m.end(),
            },
        ));
    }

    events.sort_by_key(|(pos, _)| *pos);
    events
}

/// Replay one physical line's loop-structure events in order, updating the
/// running loop-depth and reporting any break/continue seen while the depth
/// is zero. Returns the loop-depth carried forward to the next line.
fn process_line(
    line: &str,
    line_num: usize,
    mut loop_depth: usize,
    result: &mut LintResult,
) -> usize {
    for (_, event) in line_events(line) {
        match event {
            LineEvent::LoopStart => loop_depth += 1,
            LineEvent::LoopEnd => loop_depth = loop_depth.saturating_sub(1),
            LineEvent::BreakContinue {
                keyword,
                start,
                end,
            } => {
                if loop_depth == 0 {
                    let diagnostic = Diagnostic::new(
                        "SC2105",
                        Severity::Error,
                        format!("'{}' is only valid in loops", keyword),
                        Span::new(line_num, start + 1, line_num, end + 1),
                    );
                    result.add(diagnostic);
                }
            }
        }
    }
    loop_depth
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut loop_depth: usize = 0;

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        loop_depth = process_line(line, line_num, loop_depth, &mut result);
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

    // #535 differential table: (input, findings). Every count was measured with
    // BOTH the origin/main binary and this tree's binary; a row differs from
    // main only where marked `// main: N`: `break`/`continue` is then certainly
    // not a command (part of a larger word, a whole-command assignment value, or
    // a plain argument of `echo`/`printf`). Every other shape fires, as on main.
    const TABLE: &[(&str, usize)] = &[
        ("break\n", 1),
        ("  continue\n", 1),
        ("\tbreak\n", 1),
        ("break 2\n", 1),
        ("echo hi; break\n", 1),
        ("true && break\n", 1),
        ("false || continue\n", 1),
        ("echo x | break\n", 1),
        ("if true; then break; fi\n", 1),
        ("(break)\n", 1),
        ("( continue )\n", 1),
        ("{ break; }\n", 1),
        ("f() {\n  break\n}\n", 1),
        ("break # note\n", 1),
        ("case \"$1\" in q) break ;; esac\n", 1),
        ("case $1 in *) continue;; esac\n", 1),
        ("true; break; continue\n", 2),
        ("break;break\n", 2),
        ("! break\n", 1),
        ("time break\n", 1),
        ("time -p break\n", 1),
        ("command break\n", 1),
        ("builtin break\n", 1),
        ("eval break\n", 1),
        ("eval \"break\"\n", 0),
        ("x=`break`\n", 1),
        ("`break`\n", 1),
        ("$(break)\n", 1),
        ("echo $(break)\n", 1),
        ("echo `break`\n", 1),
        ("FOO=1 break\n", 1),
        ("FOO=1 x=break foo\n", 1),
        ("x=break foo\n", 1),
        ("x=break # c\n", 1),
        ("break>/dev/null\n", 1),
        ("break>&2\n", 1),
        ("break 2>&1\n", 1),
        (">/dev/null break\n", 1),
        ("\"break\"\n", 0),
        ("'break'\n", 0),
        ("\\break\n", 1),
        ("break)\n", 1),
        ("break;\n", 1),
        ("if true; then\n\tbreak\nfi\n", 1),
        ("while true; do :; done; break\n", 1),
        ("echo \"a; break\"\n", 0),
        ("echo 'a; break'\n", 0),
        ("echo \"$x\" break\n", 1),
        ("printf '%s\\n' break\n", 1),
        ("echo a || break\n", 1),
        ("echo then break\n", 0),      // main: 1
        ("echo break\n", 0),           // main: 1
        ("echo x break\n", 0),         // main: 1
        ("printf %s break\n", 0),      // main: 1
        ("echo x && echo break\n", 0), // main: 1
        ("expect escaped-line-break-fails    1 \"$tmp/esc-nl\"\n", 0), // main: 1
        ("echo -break- x\n", 0),       // main: 1
        ("echo no-break\n", 0),        // main: 1
        ("echo --break\n", 0),         // main: 1
        ("echo continue-on-error\n", 0), // main: 1
        ("cat break.txt\n", 0),        // main: 1
        ("ls ./break\n", 0),           // main: 1
        ("x=break\n", 0),              // main: 1
        ("FOO=1 x=break\n", 0),        // main: 1
        ("break=1\n", 0),              // main: 1
        ("echo ${break}\n", 0),        // main: 1
        ("echo $break\n", 0),          // main: 1
        ("echo breakfast\n", 0),
        ("# break\n", 0),
        ("for i in 1; do break; done\n", 0),
        ("while true; do\n  if x; then continue; fi\ndone\n", 0),
    ];

    #[test]
    fn test_PMAT535_differential_table() {
        assert!(TABLE.len() >= 70);
        // Through the full lint pipeline, as the binary runs it.
        let bad: Vec<_> = TABLE
            .iter()
            .filter_map(|(code, want)| {
                let got = crate::linter::rules::lint_shell(code)
                    .diagnostics
                    .iter()
                    .filter(|d| d.code == "SC2105")
                    .count();
                (got != *want).then(|| format!("{code:?}: got {got}, want {want}"))
            })
            .collect();
        assert!(bad.is_empty(), "wrong counts: {bad:#?}");
    }

    #[test]
    fn test_PMAT535_span_points_at_keyword() {
        let result = check("echo hi; break\n");
        let d = &result.diagnostics[0];
        assert_eq!((d.span.start_col, d.span.end_col), (10, 15));
    }
}
