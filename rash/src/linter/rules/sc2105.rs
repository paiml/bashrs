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

// `break`/`continue` as a whole word, exactly as before (#535 keeps main's
// reach); `is_command_word` then drops only the occurrences that are not a
// command: an argument, part of a hyphenated word or an assignment value.
static BREAK_CONTINUE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\b(break|continue)\b").unwrap());

/// Words after which the next word is still in command position.
const COMMAND_PREFIXES: &[&str] = &[
    "do", "then", "else", "elif", "if", "while", "until", "time", "command", "builtin",
];

/// Is the keyword at `line[start..end]` in command position?
///
/// Before it: the line start, a separator or opener (`;` `&` `|` `(` `)` `{`,
/// so a case arm `q) break` counts), a `!`, or a compound-command keyword /
/// `time` / `command`. After it: whitespace, a separator, a closer, `#` or the
/// line end. A `-`, `=` or `.` on either side makes it part of a word.
fn is_command_word(line: &str, start: usize, end: usize) -> bool {
    let after_ok = match line[end..].chars().next() {
        None => true,
        Some(c) => c.is_whitespace() || matches!(c, ';' | '&' | '|' | ')' | '}' | '#'),
    };
    if !after_ok {
        return false;
    }
    let before = line[..start].trim_end();
    let Some(last) = before.chars().next_back() else {
        return true;
    };
    if before.ends_with("${") {
        return false; // `${break}` is a parameter expansion
    }
    if matches!(last, ';' | '&' | '|' | '(' | ')' | '{') {
        return true;
    }
    if last == '!' {
        let rest = &before[..before.len() - 1];
        return rest.is_empty() || rest.ends_with(char::is_whitespace);
    }
    let word_start = before
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let word = &before[word_start..];
    let attached = before[..word_start].ends_with(['-', '.', '=', '$']);
    // `break` must be separated from the prefix word by whitespace.
    let spaced = line[before.len()..start]
        .chars()
        .next()
        .is_some_and(char::is_whitespace);
    !attached && spaced && COMMAND_PREFIXES.contains(&word)
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
        if !is_command_word(line, m.start(), m.end()) {
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

    // #535 differential table: (input, findings). Every row is what main
    // reports EXCEPT the rows marked `main: N`, the only inputs that change:
    // `break`/`continue` is an argument, part of a hyphenated word or an
    // assignment value there, so it is not a command word.
    const TABLE: &[(&str, usize)] = &[
        // still fire, as on main
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
        ("command break\n", 1),
        // not a command word (main: 1 each)
        ("expect escaped-line-break-fails    1 \"$tmp/esc-nl\"\n", 0),
        ("echo -break- x\n", 0),
        ("echo no-break\n", 0),
        ("echo continue-on-error\n", 0),
        ("echo x break\n", 0),
        ("echo continue\n", 0),
        ("x=break\n", 0),
        ("break=1\n", 0),
        ("echo ${break}\n", 0),
        // never fired
        ("echo breakfast\n", 0),
        ("# break\n", 0),
        ("for i in 1; do break; done\n", 0),
    ];

    #[test]
    fn test_PMAT535_differential_table() {
        assert!(TABLE.len() >= 25);
        for (code, want) in TABLE {
            let got = check(code).diagnostics.len();
            assert_eq!(got, *want, "wrong count on {code:?}");
        }
    }

    #[test]
    fn test_PMAT535_span_points_at_keyword() {
        let result = check("echo hi; break\n");
        let d = &result.diagnostics[0];
        assert_eq!((d.span.start_col, d.span.end_col), (10, 15));
    }
}
