// SC1008: Unrecognized shebang interpreter
//
// The shebang line specifies an interpreter that is not a recognized shell.
// This may indicate a typo or an unsupported interpreter.
//
// Examples:
// Bad:
//   #!/usr/bin/bsh              // Typo for bash
//   #!/usr/bin/env bsh          // Typo via env
//   #!/bin/bassh                // Misspelling
//
// Good:
//   #!/bin/bash
//   #!/usr/bin/env zsh
//   #!/bin/sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Known shell interpreters
const KNOWN_SHELLS: &[&str] = &[
    "bash", "sh", "dash", "zsh", "ksh", "csh", "tcsh", "fish", "ash",
];

/// Extract the interpreter name from a shebang line.
/// Handles both `#!/path/to/shell` and `#!/usr/bin/env shell` forms.
fn extract_interpreter(shebang: &str) -> Option<&str> {
    let after_hash_bang = shebang.strip_prefix("#!")?;
    let trimmed = after_hash_bang.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Split into parts: path and optional arguments
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    // Check for #!/usr/bin/env SHELL form
    if parts[0].ends_with("/env") && parts.len() > 1 {
        // The interpreter is the next argument after env
        // Strip any leading flags like -S
        for part in &parts[1..] {
            if !part.starts_with('-') {
                return Some(part);
            }
        }
        return None;
    }

    // Direct path form: extract basename
    let path = parts[0];
    path.rsplit('/').next()
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let first_line = match source.lines().next() {
        Some(line) => line,
        None => return result,
    };

    // Only check lines that start with #!
    if !first_line.trim_start().starts_with("#!") {
        return result;
    }

    let interpreter = match extract_interpreter(first_line) {
        Some(name) => name,
        None => return result,
    };

    // Check if the interpreter is a known shell
    if !KNOWN_SHELLS.contains(&interpreter) {
        let diagnostic = Diagnostic::new(
            "SC1008",
            Severity::Warning,
            format!(
                "Unrecognized shebang interpreter '{}'. Expected one of: {}",
                interpreter,
                KNOWN_SHELLS.join(", ")
            ),
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
    fn test_sc1008_unrecognized_interpreter() {
        let code = "#!/usr/bin/bsh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1008");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("bsh"));
    }

    #[test]
    fn test_sc1008_unrecognized_via_env() {
        let code = "#!/usr/bin/env bsh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("bsh"));
    }

    #[test]
    fn test_sc1008_python_interpreter() {
        let code = "#!/usr/bin/python3\nprint('hello')";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("python3"));
    }

    #[test]
    fn test_sc1008_perl_via_env() {
        let code = "#!/usr/bin/env perl\nprint 'hello';";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("perl"));
    }

    // False-positive avoidance tests
    #[test]
    fn test_sc1008_bash_ok() {
        let code = "#!/bin/bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_sh_ok() {
        let code = "#!/bin/sh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_env_bash_ok() {
        let code = "#!/usr/bin/env bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_zsh_ok() {
        let code = "#!/usr/bin/zsh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_dash_ok() {
        let code = "#!/usr/bin/env dash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_fish_ok() {
        let code = "#!/usr/bin/fish\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_ksh_ok() {
        let code = "#!/bin/ksh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_no_shebang() {
        let code = "echo hello\necho world";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_comment_not_shebang() {
        let code = "# just a comment\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Edge case tests
    #[test]
    fn test_sc1008_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_env_with_flags() {
        let code = "#!/usr/bin/env -S bash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_ash_ok() {
        let code = "#!/usr/bin/env ash\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_csh_ok() {
        let code = "#!/usr/bin/csh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1008_tcsh_ok() {
        let code = "#!/usr/bin/tcsh\necho hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
