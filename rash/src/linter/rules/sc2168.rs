// SC2168: 'local' is only valid in functions
//
// The `local` keyword can only be used inside shell functions.
// Using it at the top level is a syntax error.
//
// Examples:
// Bad:
//   local var="value"  # At top level - ERROR
//
// Good:
//   function test() {
//       local var="value"  # Inside function - OK
//   }

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static FUNCTION_START: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\b(function\s+[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_][A-Za-z0-9_]*\s*\(\s*\))").unwrap()
});

static FUNCTION_END: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\s*\}").unwrap());

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if a position in a line is inside single or double quotes
///
/// This function tracks quote state to determine if a position is inside a quoted string.
/// It handles both single quotes ('...') and double quotes ("...").
#[cfg(test)]
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_char = '\0';

    for (i, ch) in line.chars().enumerate() {
        if i >= pos {
            break;
        }

        // Handle escape sequences
        if prev_char == '\\' {
            prev_char = ch;
            continue;
        }

        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            _ => {}
        }

        prev_char = ch;
    }

    in_single_quote || in_double_quote
}

/// Check if line starts a function
fn is_function_start(line: &str) -> bool {
    FUNCTION_START.is_match(line)
}

/// Check if line has opening brace
fn has_opening_brace(line: &str) -> bool {
    line.contains('{')
}

/// Check if next line has opening brace
fn has_opening_brace_next_line(lines: &[&str], i: usize) -> bool {
    i + 1 < lines.len() && lines[i + 1].contains('{')
}

/// Count opening braces in line
fn count_opening_braces(line: &str) -> usize {
    line.matches('{').count()
}

/// Count closing braces in line
fn count_closing_braces(line: &str) -> usize {
    line.matches('}').count()
}

/// Check if line is function end
fn is_function_end(line: &str) -> bool {
    FUNCTION_END.is_match(line)
}

/// Update function depth for function start
fn update_depth_for_function_start(
    function_depth: &mut usize,
    line: &str,
    lines: &[&str],
    i: usize,
) {
    if is_function_start(line) {
        // Look ahead to see if there's an opening brace
        if has_opening_brace(line) {
            *function_depth += 1;
        } else if has_opening_brace_next_line(lines, i) {
            // Brace on next line
            *function_depth += 1;
        }
    }
}

/// Update function depth for braces
fn update_depth_for_braces(function_depth: &mut usize, line: &str) {
    // Track closing braces
    if is_function_end(line) && *function_depth > 0 {
        *function_depth = function_depth.saturating_sub(1);
    }

    // Count opening braces on current line
    *function_depth += count_opening_braces(line);
    // Subtract closing braces
    if *function_depth > 0 {
        let closing = count_closing_braces(line);
        *function_depth = function_depth.saturating_sub(closing);
    }
}

/// Create diagnostic for local outside function
fn create_local_outside_function_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2168",
        Severity::Error,
        "'local' is only valid in functions",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

/// Strip Make recipe prefixes (`@`, `-`, `+`) from a tab-led recipe line.
/// Returns the byte offset removed, so columns can be mapped back.
fn recipe_prefix_len(line: &str) -> usize {
    if !line.starts_with('\t') {
        return 0;
    }
    let rest = line.trim_start_matches('\t');
    let body = rest.trim_start_matches(['@', '-', '+', ' ', '\t']);
    line.len() - body.len()
}

/// True when the line ends in an odd number of backslashes (continues).
fn continues(line: &str) -> bool {
    (line.len() - line.trim_end_matches('\\').len()) % 2 == 1
}

/// Columns (1-indexed, within `logical`) of every `local` in command position.
fn local_command_cols(logical: &str) -> Vec<usize> {
    crate::linter::shell_words::simple_commands(logical)
        .iter()
        .filter(|c| c.name.as_deref() == Some("local"))
        .filter_map(|c| c.words.iter().find(|w| w.literal == "local").map(|w| w.col))
        .collect()
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut function_depth: usize = 0;
    // GH-438: a backslash-continued command is one logical line. `pending`
    // holds (physical line number, text) pieces and the depth at its start.
    let mut pending: Vec<(usize, &str)> = Vec::new();
    let mut pending_depth = 0usize;

    for (i, line) in lines.iter().enumerate() {
        if pending.is_empty() {
            if is_comment_line(line) {
                continue;
            }
            // Track function depth
            update_depth_for_function_start(&mut function_depth, line, &lines, i);
            update_depth_for_braces(&mut function_depth, line);
            pending_depth = function_depth;
        } else {
            update_depth_for_braces(&mut function_depth, line);
        }
        pending.push((i + 1, line));
        if continues(line) && i + 1 < lines.len() {
            continue;
        }
        if pending_depth == 0 {
            flag_locals(&pending, &mut result);
        }
        pending.clear();
    }

    result
}

/// Join the pieces of one logical line and report each command-position `local`.
fn flag_locals(pieces: &[(usize, &str)], result: &mut LintResult) {
    let mut logical = String::new();
    // (start offset in `logical`, line number, prefix removed from that piece)
    let mut starts: Vec<(usize, usize, usize)> = Vec::new();
    for (n, text) in pieces {
        let skip = recipe_prefix_len(text);
        let body = text.get(skip..).unwrap_or("");
        let body = body.strip_suffix('\\').unwrap_or(body);
        starts.push((logical.len(), *n, skip));
        logical.push_str(body);
        logical.push(' ');
    }
    if logical.trim_start().starts_with('#') {
        return;
    }
    for col in local_command_cols(&logical) {
        let off = col - 1;
        let (start, line_num, skip) = starts
            .iter()
            .rev()
            .find(|(st, _, _)| *st <= off)
            .copied()
            .unwrap_or((0, pieces[0].0, 0));
        let start_col = off - start + skip + 1;
        result.add(create_local_outside_function_diagnostic(
            line_num,
            start_col,
            start_col + "local ".len(),
        ));
    }
}

#[cfg(test)]
#[path = "sc2168_tests_prop_sc2168.rs"]
mod tests_extracted;
