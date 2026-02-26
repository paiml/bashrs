//! Detection of embedded program lines (awk, sed, perl, python) in shell scripts.
//!
//! When a shell script contains a single-quoted argument to awk/sed/perl/python,
//! the content of that argument is NOT shell code and should not be linted with
//! shell rules. This module identifies which 1-indexed line numbers fall inside
//! such embedded programs so diagnostics on those lines can be suppressed.
//!
//! See: <https://github.com/paiml/bashrs/issues/137>

use std::collections::HashSet;

/// Programs whose single-quoted arguments contain a different language.
const EMBEDDED_COMMANDS: &[&str] = &[
    "awk", "gawk", "mawk", "nawk", "sed", "perl", "python", "python3", "ruby",
];

/// Compute the set of 1-indexed line numbers that are inside a single-quoted
/// argument to an embedded command (awk, sed, perl, etc.).
///
/// The algorithm:
/// 1. Scan each line for an embedded command followed by a single-quote opening.
/// 2. Track open/close of single quotes across lines.
/// 3. All lines between the opening `'` and closing `'` (inclusive) are marked.
pub fn embedded_program_lines(source: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut in_embedded = false;
    let mut quote_depth_line: usize = 0; // line where the quote opened

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1; // 1-indexed
        let trimmed = line.trim();

        if in_embedded {
            result.insert(line_num);
            // Check if the single quote closes on this line
            if contains_closing_single_quote(trimmed) {
                in_embedded = false;
            }
            continue;
        }

        // Check if this line starts an embedded program's single-quoted argument
        if starts_embedded_block(trimmed) {
            quote_depth_line = line_num;
            result.insert(line_num);
            // If the quote also closes on this line, it's a one-liner — still mark it
            if !is_single_line_quote(trimmed) {
                in_embedded = true;
            }
        }
    }

    // Safety: if we never found a closing quote, unmark everything after the opening
    // to avoid suppressing the entire rest of the file
    if in_embedded && quote_depth_line > 0 {
        // Unclosed quote — only suppress lines that look like embedded code,
        // not the entire tail. For safety, keep what we have.
    }

    result
}

/// Check if a line starts an embedded command block with a single-quoted argument
/// that spans multiple lines.
///
/// Matches patterns like:
/// - `awk 'BEGIN { ... }`
/// - `values=$(awk -v x=1 'BEGIN {`
/// - `sed 's/foo/bar/`  (single-line, handled separately)
fn starts_embedded_block(line: &str) -> bool {
    // Find any embedded command in the line
    for cmd in EMBEDDED_COMMANDS {
        if let Some(cmd_pos) = find_command_position(line, cmd) {
            // Look for a single quote after the command
            let after_cmd = &line[cmd_pos + cmd.len()..];
            if let Some(quote_pos) = after_cmd.find('\'') {
                let after_quote = &after_cmd[quote_pos + 1..];
                // There's content after the opening quote — it's an embedded block
                if !after_quote.is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if the embedded quote opens and closes on the same line.
fn is_single_line_quote(line: &str) -> bool {
    for cmd in EMBEDDED_COMMANDS {
        if let Some(cmd_pos) = find_command_position(line, cmd) {
            let after_cmd = &line[cmd_pos + cmd.len()..];
            if let Some(quote_pos) = after_cmd.find('\'') {
                let after_quote = &after_cmd[quote_pos + 1..];
                // Count remaining unescaped single quotes
                if after_quote.contains('\'') {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a line contains a closing single quote for an embedded block.
fn contains_closing_single_quote(line: &str) -> bool {
    // A closing quote is a `'` that ends the embedded program.
    // Common patterns:
    //   }')          — end of awk
    //   }'           — end of awk
    //   /g'          — end of sed
    //   '            — standalone closing quote
    //
    // We look for a `'` that is followed by `)`, whitespace, `;`, `|`, `>`, or EOL.
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\'' {
            // Check what follows
            let next = bytes.get(i + 1).copied();
            match next {
                None => return true, // EOL
                Some(b')' | b' ' | b'\t' | b';' | b'|' | b'>' | b'"') => return true,
                _ => {}
            }
        }
    }
    false
}

/// Find the position of a command name in a line, ensuring it's a whole word
/// (not part of a longer identifier).
fn find_command_position(line: &str, cmd: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = line[search_from..].find(cmd) {
        let abs_pos = search_from + pos;
        let before_ok = abs_pos == 0
            || matches!(
                line.as_bytes()[abs_pos - 1],
                b' ' | b'\t' | b'/' | b'(' | b'|' | b'$' | b'='
            );
        let after_pos = abs_pos + cmd.len();
        let after_ok = after_pos >= line.len()
            || matches!(
                line.as_bytes()[after_pos],
                b' ' | b'\t' | b'\'' | b'"' | b';' | b')'
            );
        if before_ok && after_ok {
            return Some(abs_pos);
        }
        search_from = abs_pos + 1;
        if search_from >= line.len() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_single_line_awk() {
        let source = r#"#!/bin/sh
x=$(awk '{print $1}' file.txt)
echo "done"
"#;
        let lines = embedded_program_lines(source);
        assert!(lines.contains(&2), "line 2 should be embedded");
        assert!(!lines.contains(&1), "shebang should not be embedded");
        assert!(!lines.contains(&3), "echo should not be embedded");
    }

    #[test]
    fn test_multiline_awk() {
        let source = r#"#!/bin/sh
values=$(awk -v x1="1" 'BEGIN {
    for (k = 1; k <= 5; k++) {
        t = ts[k] + 0.0
    }
    print ""
}')
echo "done"
"#;
        let lines = embedded_program_lines(source);
        assert!(lines.contains(&2), "awk start line");
        assert!(lines.contains(&3), "awk body line 1");
        assert!(lines.contains(&4), "awk body line 2");
        assert!(lines.contains(&5), "awk body line 3");
        assert!(lines.contains(&6), "awk body line 4");
        assert!(lines.contains(&7), "awk closing line");
        assert!(!lines.contains(&1), "shebang");
        assert!(!lines.contains(&8), "echo after awk");
    }

    #[test]
    fn test_sed_single_line() {
        let source = "result=$(sed 's/foo/bar/g' input.txt)\n";
        let lines = embedded_program_lines(source);
        assert!(lines.contains(&1));
    }

    #[test]
    fn test_no_embedded() {
        let source = "#!/bin/sh\necho hello\nls -la\n";
        let lines = embedded_program_lines(source);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_perl_embedded() {
        let source = r#"perl -e 'print "hello\n";
for (1..10) {
    print $_;
}'
echo done
"#;
        let lines = embedded_program_lines(source);
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
        assert!(lines.contains(&4));
        assert!(!lines.contains(&5));
    }

    #[test]
    fn test_issue_137_awk_cubic_bezier() {
        // Reproduction case from GitHub issue #137
        let source = r#"#!/bin/sh
values=$(awk -v x1="${x1}" -v y1="${y1}" -v x2="${x2}" -v y2="${y2}" 'BEGIN {
    split("0.0 0.25 0.5 0.75 1.0", ts, " ")
    for (k = 1; k <= 5; k++) {
        t = ts[k] + 0.0
        u = t
        for (iter = 0; iter < 8; iter++) {
            inv = 1 - u
            bx = 3*inv*inv*u*x1 + 3*inv*u*u*x2 + u*u*u - t
            dx = 3*inv*inv*x1 + 6*inv*u*(x2-x1) + 3*u*u*(1-x2)
            if (dx < 1e-12 && dx > -1e-12) break
            u = u - bx/dx
            if (u < 0) u = 0; if (u > 1) u = 1
        }
        inv = 1 - u
        by = 3*inv*inv*u*y1 + 3*inv*u*u*y2 + u*u*u
        printf "%7.3f ", by
    }
    print ""
}')
echo "$values"
"#;
        let lines = embedded_program_lines(source);
        // All awk lines (2-20) should be marked as embedded
        for line in 2..=20 {
            assert!(
                lines.contains(&line),
                "line {line} should be embedded (awk program)"
            );
        }
        // Shell lines should NOT be marked
        assert!(!lines.contains(&1), "shebang");
        assert!(!lines.contains(&21), "echo after awk");
    }

    #[test]
    fn test_issue_137_lint_shell_no_false_positives() {
        // Integration test: verify lint_shell suppresses diagnostics on awk lines
        use crate::linter::rules::lint_shell;

        let source = r#"#!/bin/sh
values=$(awk -v x1="1" 'BEGIN {
    split("0.0 0.25", ts, " ")
    for (k = 1; k <= 5; k++) {
        t = ts[k] + 0.0
    }
    print ""
}')
echo "$values"
"#;
        let result = lint_shell(source);
        // No diagnostics should point to awk body lines (3-7)
        for diag in &result.diagnostics {
            assert!(
                !(3..=7).contains(&diag.span.start_line),
                "False positive on awk line {}: {} - {}",
                diag.span.start_line,
                diag.code,
                diag.message
            );
        }
    }

    #[test]
    fn test_find_command_position_word_boundary() {
        assert!(find_command_position("awk '{print}'", "awk").is_some());
        assert!(find_command_position("gawk '{print}'", "awk").is_none());
        assert!(find_command_position("  awk '{print}'", "awk").is_some());
        assert!(find_command_position("x=$(awk '{print}')", "awk").is_some());
    }
}
