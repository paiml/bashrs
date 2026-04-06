// SC2096: Redirections override previously specified redirections
//
// When multiple redirections of the same type are specified for the same file
// descriptor, only the last one takes effect. Earlier redirections are silently
// ignored, which is often unintentional.
//
// Examples:
// Bad:
//   command > file1.txt > file2.txt
//   # Only file2.txt is written, file1.txt is untouched
//
//   command 2> err1.log 2> err2.log
//   # Only err2.log gets stderr
//
// Good:
//   command > file1.txt
//   # Single output
//
//   command > stdout.txt 2> stderr.txt
//   # Different streams to different files

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static MULTIPLE_STDOUT_REDIRECTS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: > file1 ... > file2  (without 2> in between)
    Regex::new(r">\s*[^\s;&|]+[^2>]*>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_STDERR_REDIRECTS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: 2> file1 ... 2> file2
    Regex::new(r"2>\s*[^\s;&|]+.*2>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_APPEND_REDIRECTS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: >> file1 ... >> file2
    Regex::new(r">>\s*[^\s;&|]+.*>>\s*[^\s;&|]+").unwrap()
});

/// Check if a line should be skipped (comments, heredocs)
fn should_skip_line(line: &str) -> bool {
    if line.trim_start().starts_with('#') {
        return true;
    }

    // Skip heredocs and here strings
    line.contains("<<") || line.contains("<<<")
}

/// Check if characters form a multi-character operator (&& or ||)
fn is_multi_char_operator(ch: char, next_ch: char) -> bool {
    (ch == '&' && next_ch == '&') || (ch == '|' && next_ch == '|')
}

/// Check if character is a single pipe (not part of ||)
fn is_single_pipe(ch: char, prev_ch: Option<char>) -> bool {
    ch == '|' && prev_ch != Some('|')
}

/// Add command to list if non-empty
fn add_command_if_nonempty<'a>(commands: &mut Vec<&'a str>, cmd: &'a str) {
    if !cmd.trim().is_empty() {
        commands.push(cmd);
    }
}

/// Get next position after operator from iterator
fn get_next_position<I>(chars: &mut std::iter::Peekable<I>, line_len: usize) -> usize
where
    I: Iterator<Item = (usize, char)>,
{
    if let Some(&(next_pos, _)) = chars.peek() {
        next_pos
    } else {
        line_len
    }
}

/// Split line into separate commands (by &&, ||, ;, |)
/// This ensures redirects in different commands are checked independently
fn split_commands(line: &str) -> Vec<&str> {
    // Simple split on command separators
    // Note: This is a simplification - proper bash parsing would handle quotes, escapes, etc.
    let mut commands = Vec::new();
    let mut current_start = 0;
    let mut chars = line.char_indices().peekable();
    let mut prev_ch: Option<char> = None;

    while let Some((byte_pos, ch)) = chars.next() {
        // Check for multi-character operators (&&, ||)
        if let Some(&(_, next_ch)) = chars.peek() {
            if is_multi_char_operator(ch, next_ch) {
                if byte_pos > current_start {
                    add_command_if_nonempty(&mut commands, &line[current_start..byte_pos]);
                }
                // Consume the next character
                chars.next();
                // Set start position after the operator
                current_start = get_next_position(&mut chars, line.len());
                prev_ch = Some(next_ch);
                continue;
            }
        }

        // Check for single-character operators (;, |)
        if ch == ';' || is_single_pipe(ch, prev_ch) {
            if byte_pos > current_start {
                add_command_if_nonempty(&mut commands, &line[current_start..byte_pos]);
            }
            // Set start position after the operator
            current_start = get_next_position(&mut chars, line.len());
        }

        prev_ch = Some(ch);
    }

    // Add the last command
    if current_start < line.len() {
        add_command_if_nonempty(&mut commands, &line[current_start..]);
    }

    commands
}

/// Count the number of stdout redirects in a line
fn count_stdout_redirects(line: &str) -> usize {
    let parts: Vec<&str> = line.split('>').collect();
    let mut stdout_count = 0;

    for (i, part) in parts.iter().enumerate() {
        if i > 0 && !parts[i - 1].ends_with('2') && !parts[i - 1].ends_with('&') {
            stdout_count += 1;
        }
    }

    stdout_count
}

/// Create a diagnostic for SC2096
fn create_diagnostic(message: String, line_num: usize, line_len: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2096",
        Severity::Warning,
        message,
        Span::new(line_num, 1, line_num, line_len),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if should_skip_line(line) {
            continue;
        }

        // Split line into separate commands and check each independently
        let commands = split_commands(line);

        for cmd in commands {
            // Check for multiple stdout redirects (skip if it has >>)
            if MULTIPLE_STDOUT_REDIRECTS.is_match(cmd) && !cmd.contains(">>") {
                let stdout_count = count_stdout_redirects(cmd);

                if stdout_count > 1 {
                    let diagnostic = create_diagnostic(
                        "Multiple stdout redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                        line_num,
                        line.len(),
                    );
                    result.add(diagnostic);
                }
            }

            // Check for multiple stderr redirects
            if MULTIPLE_STDERR_REDIRECTS.is_match(cmd) {
                let diagnostic = create_diagnostic(
                    "Multiple stderr redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                    line_num,
                    line.len(),
                );
                result.add(diagnostic);
            }

            // Check for multiple append redirects
            if MULTIPLE_APPEND_REDIRECTS.is_match(cmd) {
                let diagnostic = create_diagnostic(
                    "Multiple append redirections specified. Only the last one will be used, earlier ones are ignored".to_string(),
                    line_num,
                    line.len(),
                );
                result.add(diagnostic);
            }
        }
    }

    result
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "sc2096_tests_prop_sc2096.rs"]
// FIXME(PMAT-238): mod tests_extracted;
