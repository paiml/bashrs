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
use once_cell::sync::Lazy;
use regex::Regex;

static MULTIPLE_STDOUT_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
    // Match: > file1 ... > file2  (without 2> in between)
    Regex::new(r">\s*[^\s;&|]+[^2>]*>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_STDERR_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
    // Match: 2> file1 ... 2> file2
    Regex::new(r"2>\s*[^\s;&|]+.*2>\s*[^\s;&|]+").unwrap()
});

static MULTIPLE_APPEND_REDIRECTS: Lazy<Regex> = Lazy::new(|| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    use proptest::prelude::*;

    // ===== Manual Property Tests =====
    // Establish invariants before refactoring

    #[test]
    fn prop_sc2096_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# command > file1.txt > file2.txt",
            "  # 2> err1.log 2> err2.log",
            "\t# >> a.txt >> b.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Comments should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2096_single_redirects_always_ok() {
        // Property: Single redirects of any type should never be diagnosed
        let test_cases = vec![
            "command > output.txt",
            "command 2> error.log",
            "command >> append.txt",
            "command &> combined.log",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Single redirects should be OK: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2096_different_streams_always_ok() {
        // Property: Redirecting different streams (stdout vs stderr) is always OK
        let test_cases = vec![
            "command > out.txt 2> err.txt",
            "command 2> err.txt > out.txt",
            "cmd > /dev/null 2> /tmp/err",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Different streams should be OK: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2096_multiple_same_stream_diagnosed() {
        // Property: Multiple redirects of the same stream should be diagnosed
        let test_cases = vec![
            ("command > file1 > file2", "stdout"),
            ("command 2> err1 2> err2", "stderr"),
            ("echo a >> f1 >> f2", "append"),
        ];

        for (code, stream_type) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                1,
                "Multiple {} redirects should be diagnosed: {}",
                stream_type,
                code
            );
            assert_eq!(result.diagnostics[0].code, "SC2096");
            assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2096_heredocs_never_diagnosed() {
        // Property: Heredocs should never be diagnosed as duplicate redirects
        let test_cases = vec![
            "cat <<EOF > output.txt",
            "cat <<-EOF > output.txt",
            "cat <<<STRING > output.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Heredocs should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2096_chained_commands_independent() {
        // Property: Redirects in separate commands should be independent
        let test_cases = vec![
            "cmd1 > file1.txt && cmd2 > file2.txt",
            "cmd1 > out1 ; cmd2 > out2",
            "cmd1 > f1 | cmd2 > f2",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Separate commands should have independent redirects: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2096_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2096_diagnostic_code_always_sc2096() {
        // Property: All diagnostics must have code "SC2096"
        let code = "cmd > f1 > f2\ncmd 2> e1 2> e2\necho >> a >> b";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.code, "SC2096");
        }
    }

    #[test]
    fn prop_sc2096_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "cmd > f1 > f2\ncmd 2> e1 2> e2";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2096_multiple_stdout() {
        let code = r#"command > file1.txt > file2.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2096");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("stdout"));
    }

    #[test]
    fn test_sc2096_multiple_stderr() {
        let code = r#"command 2> err1.log 2> err2.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("stderr"));
    }

    #[test]
    fn test_sc2096_multiple_append() {
        let code = r#"echo "a" >> file1.txt >> file2.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("append"));
    }

    #[test]
    fn test_sc2096_stdout_and_stderr_ok() {
        let code = r#"command > stdout.txt 2> stderr.txt"#;
        let result = check(code);
        // Different streams, this is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_single_redirect_ok() {
        let code = r#"command > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_pipe_ok() {
        let code = r#"command | grep pattern > output.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_both_redirect_ok() {
        let code = r#"command &> all.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_heredoc_ok() {
        let code = r#"cat <<EOF > output.txt"#;
        let result = check(code);
        // Heredoc is not a duplicate redirect
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2096_three_redirects() {
        let code = r#"echo test > a.txt > b.txt > c.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2096_chained_commands_ok() {
        let code = r#"cmd1 > file1.txt && cmd2 > file2.txt"#;
        let result = check(code);
        // Different commands, not duplicate redirects
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Generative Property Tests =====
    // Using proptest for random input generation (100 cases each)

    proptest! {
        #[test]
        fn prop_gen_comments_never_diagnosed(comment in r"#[^\n]{0,50}") {
            // Property: Any line starting with # should never be diagnosed
            let result = check(&comment);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_single_stdout_always_ok(
            cmd in r"[a-z]{1,10}",
            file in r"[a-z]{1,10}\.(txt|log)"
        ) {
            // Property: Single stdout redirect should never be diagnosed
            let code = format!("{} > {}", cmd, file);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_single_stderr_always_ok(
            cmd in r"[a-z]{1,10}",
            file in r"[a-z]{1,10}\.(txt|log)"
        ) {
            // Property: Single stderr redirect should never be diagnosed
            let code = format!("{} 2> {}", cmd, file);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_single_append_always_ok(
            cmd in r"[a-z]{1,10}",
            file in r"[a-z]{1,10}\.(txt|log)"
        ) {
            // Property: Single append redirect should never be diagnosed
            let code = format!("{} >> {}", cmd, file);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_stdout_stderr_mix_always_ok(
            cmd in r"[a-z]{1,10}",
            out_file in r"[a-z]{1,10}\.txt",
            err_file in r"[a-z]{1,10}\.log"
        ) {
            // Property: Mixing stdout and stderr redirects is always OK
            let code = format!("{} > {} 2> {}", cmd, out_file, err_file);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_double_stdout_always_diagnosed(
            cmd in r"[a-z]{1,10}",
            file1 in r"[a-z]{1,5}\.txt",
            file2 in r"[a-z]{1,5}\.log"
        ) {
            // Property: Double stdout redirects should always be diagnosed
            let code = format!("{} > {} > {}", cmd, file1, file2);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
            prop_assert!(result.diagnostics[0].message.contains("stdout"));
        }

        #[test]
        fn prop_gen_double_stderr_always_diagnosed(
            cmd in r"[a-z]{1,10}",
            file1 in r"[a-z]{1,5}\.txt",
            file2 in r"[a-z]{1,5}\.log"
        ) {
            // Property: Double stderr redirects should always be diagnosed
            let code = format!("{} 2> {} 2> {}", cmd, file1, file2);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
            prop_assert!(result.diagnostics[0].message.contains("stderr"));
        }

        #[test]
        fn prop_gen_double_append_always_diagnosed(
            cmd in r"[a-z]{1,10}",
            file1 in r"[a-z]{1,5}\.txt",
            file2 in r"[a-z]{1,5}\.log"
        ) {
            // Property: Double append redirects should always be diagnosed
            let code = format!("{} >> {} >> {}", cmd, file1, file2);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
            prop_assert!(result.diagnostics[0].message.contains("append"));
        }

        #[test]
        fn prop_gen_heredoc_never_diagnosed(
            cmd in r"[a-z]{1,10}",
            file in r"[a-z]{1,10}\.txt"
        ) {
            // Property: Heredocs should never be diagnosed
            let code = format!("{} <<EOF > {}", cmd, file);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_chained_commands_independent(
            cmd1 in r"[a-z]{1,8}",
            cmd2 in r"[a-z]{1,8}",
            file1 in r"[a-z]{1,8}\.txt",
            file2 in r"[a-z]{1,8}\.log",
            separator in r"(&&|\|\||;)"
        ) {
            // Property: Redirects in chained commands are independent
            let code = format!("{} > {} {} {} > {}", cmd1, file1, separator, cmd2, file2);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_gen_diagnostic_severity_always_warning(
            cmd in r"[a-z]{1,10}",
            file1 in r"[a-z]{1,5}\.txt",
            file2 in r"[a-z]{1,5}\.log"
        ) {
            // Property: All diagnostics must be Warning severity
            let test_cases = vec![
                format!("{} > {} > {}", cmd, file1, file2),
                format!("{} 2> {} 2> {}", cmd, file1, file2),
                format!("{} >> {} >> {}", cmd, file1, file2),
            ];

            for code in test_cases {
                let result = check(&code);
                if !result.diagnostics.is_empty() {
                    for diagnostic in &result.diagnostics {
                        prop_assert_eq!(diagnostic.severity, Severity::Warning);
                    }
                }
            }
        }

        #[test]
        fn prop_gen_empty_lines_never_diagnosed(whitespace in r"\s{0,20}") {
            // Property: Empty or whitespace-only lines never diagnosed
            let result = check(&whitespace);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
