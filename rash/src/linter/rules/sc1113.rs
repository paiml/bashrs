// SC1113: Use #! for the shebang, not just #
//
// The first line looks like a shebang but is missing the !.
// For example, `# /bin/sh` instead of `#!/bin/sh`.
//
// Examples:
// Bad:
//   # /bin/sh
//   # /usr/bin/env bash
//   # /bin/bash
//
// Good:
//   #!/bin/sh
//   #!/usr/bin/env bash
//   #!/bin/bash

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let first_line = match source.lines().next() {
        Some(line) => line,
        None => return result,
    };

    let trimmed = first_line.trim_start();

    // Must start with # but not #! (that's a real shebang)
    if !trimmed.starts_with('#') || trimmed.starts_with("#!") {
        return result;
    }

    // Strip the leading #
    let after_hash = &trimmed[1..];
    let after_hash_trimmed = after_hash.trim_start();

    // Check if remainder looks like a shebang path
    if after_hash_trimmed.starts_with("/bin/")
        || after_hash_trimmed.starts_with("/usr/bin/env ")
        || after_hash_trimmed.starts_with("/usr/bin/env\t")
        || after_hash_trimmed.starts_with("/usr/bin/")
        || after_hash_trimmed.starts_with("/sbin/")
    {
        let diagnostic = Diagnostic::new(
            "SC1113",
            Severity::Warning,
            "Use #! for the shebang, not just #. Missing the ! after #.".to_string(),
            Span::new(1, 1, 1, first_line.len() + 1),
        );
        result.add(diagnostic);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Detection tests
    #[test]
    fn test_sc1113_missing_bang() {
        let code = "# /bin/sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1113");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1113_missing_bang_bash() {
        let code = "# /bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1113_missing_bang_env() {
        let code = "# /usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1113_missing_bang_usr_bin() {
        let code = "# /usr/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1113_correct_shebang_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1113_regular_comment_ok() {
        let code = "# This is a regular comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1113_path_in_comment_ok() {
        // A comment mentioning a path but not looking like a shebang
        let code = "# See /etc/config for details\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1113_no_shebang_ok() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1113_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1113_hash_alone() {
        let code = "#\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1113_with_leading_spaces() {
        let code = "  # /bin/sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
