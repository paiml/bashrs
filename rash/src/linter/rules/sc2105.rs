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
#[path = "sc2105_tests.rs"]
mod sc2105_tests;
