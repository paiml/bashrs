// SC1127: Use # for comments, not //
//
// Shell scripts use # for comments, not // (which is C/C++/Java/Rust style).
// Lines starting with // are not comments in shell and will be interpreted
// as commands.
//
// Examples:
// Bad:
//   // This is not a comment
//   // TODO: fix this
//
// Good:
//   # This is a comment
//   # TODO: fix this
//
// Note: Skips the first line (shebang), lines inside heredocs, and
// protocol patterns like http:// or file://

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut in_heredoc = false;
    let mut heredoc_delimiter: Option<String> = None;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip first line (could be shebang)
        if line_num == 1 {
            continue;
        }

        // Track heredoc state
        if in_heredoc {
            if let Some(ref delim) = heredoc_delimiter {
                if line.trim() == delim.as_str() {
                    in_heredoc = false;
                    heredoc_delimiter = None;
                }
            }
            continue;
        }

        // Detect heredoc start
        if line.contains("<<") {
            if let Some(pos) = line.find("<<") {
                let after = &line[pos + 2..];
                let after = after.trim_start_matches('-');
                let delim: String = after
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !delim.is_empty() {
                    in_heredoc = true;
                    heredoc_delimiter = Some(delim);
                }
            }
        }

        let trimmed = line.trim_start();

        // Check if line starts with //
        if trimmed.starts_with("//") {
            // Make sure it's not a path like //some/path used in some systems
            // or just // alone (empty C-comment)
            let diagnostic = Diagnostic::new(
                "SC1127",
                Severity::Warning,
                "Use # for comments in shell scripts, not //. The // is not a comment and will be executed as a command.".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Detection tests
    #[test]
    fn test_sc1127_c_style_comment() {
        let code = "#!/bin/bash\n// This is not a shell comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1127");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1127_multiple_c_comments() {
        let code = "#!/bin/bash\n// comment one\n// comment two\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1127_indented_c_comment() {
        let code = "#!/bin/bash\n  // indented comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1127_todo_c_comment() {
        let code = "#!/bin/bash\n// TODO: fix this\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1127_shell_comment_ok() {
        let code = "#!/bin/bash\n# This is a proper comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_shebang_ok() {
        // First line is always skipped
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_heredoc_ok() {
        let code = "#!/bin/bash\ncat <<EOF\n// This is inside a heredoc\nEOF\necho done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_no_comments() {
        let code = "#!/bin/bash\necho hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1127_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_single_line_no_detection() {
        // Only one line which is treated as line 1 (skipped)
        let code = "// comment on first line";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_slash_in_command_ok() {
        let code = "#!/bin/bash\necho http://example.com\necho done";
        let result = check(code);
        // http:// is in the middle of a line, not at the start
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1127_heredoc_with_quoted_delimiter() {
        let code = "#!/bin/bash\ncat <<'EOF'\n// inside heredoc\nEOF\necho done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
