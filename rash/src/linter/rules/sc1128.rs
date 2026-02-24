// SC1128: Shebang must be on the first line
//
// A shebang (#! followed by an interpreter path) is found on a non-first line.
// The kernel only recognizes shebangs on line 1. A shebang on any other line
// is likely misplaced and will not have the intended effect.
//
// Examples:
// Bad:
//   # Config script
//   #!/bin/bash                  // Shebang on line 2
//   echo hello
//
//   echo hello
//   #!/usr/bin/env sh            // Shebang on line 2
//
// Good:
//   #!/bin/bash
//   # Config script
//   echo hello

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Skip the first line
        if line_num == 1 {
            continue;
        }

        let trimmed = line.trim_start();

        // Look for shebang-like patterns on non-first lines
        if let Some(after) = trimmed.strip_prefix("#!") {
            // Only flag if it looks like a real shebang (has a path)
            if after.starts_with("/bin/")
                || after.starts_with("/usr/bin/")
                || after.starts_with("/usr/local/bin/")
                || after.starts_with("/sbin/")
            {
                let diagnostic = Diagnostic::new(
                    "SC1128",
                    Severity::Error,
                    format!(
                        "The shebang must be on the first line. Move it from line {} to line 1.",
                        line_num
                    ),
                    Span::new(line_num, 1, line_num, line.len() + 1),
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

    // Detection tests
    #[test]
    fn test_sc1128_shebang_on_line_2() {
        let code = "# comment\n#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1128");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("line 2"));
    }

    #[test]
    fn test_sc1128_shebang_on_line_3() {
        let code = "# header\n# more header\n#!/usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("line 3"));
    }

    #[test]
    fn test_sc1128_shebang_after_code() {
        let code = "echo hello\n#!/bin/sh\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1128_correct_shebang_ok() {
        let code = "#!/bin/bash\n# comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1128_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1128_hash_bang_in_comment_ok() {
        // #! without a path should not trigger
        let code = "#!/bin/bash\n# Note: #! is the shebang\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1128_hash_bang_in_string_context() {
        // Lines with #! but no interpreter path
        let code = "#!/bin/bash\necho '#! is special'\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1128_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1128_single_line() {
        let code = "#!/bin/bash";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1128_usr_local_bin() {
        let code = "echo hello\n#!/usr/local/bin/bash\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1128_multiple_shebangs() {
        let code = "#!/bin/bash\n#!/bin/sh\necho hello";
        let result = check(code);
        // The second one on line 2 should be flagged
        assert_eq!(result.diagnostics.len(), 1);
    }
}
