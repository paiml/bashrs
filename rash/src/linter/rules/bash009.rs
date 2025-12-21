//! BASH009: Inefficient Loops (Use Builtin Alternatives)
//!
//! **Rule**: Detect loops that could use bash builtins instead of external commands
//!
//! **Why this matters**:
//! External commands are slower and less portable than builtins:
//! - `seq` requires GNU coreutils (not always available)
//! - Spawning processes is expensive
//! - Builtins are faster and more portable
//! - Reduces dependencies
//!
//! **Examples**:
//!
//! ❌ **BAD** (inefficient - spawns seq process):
//! ```bash
//! for i in $(seq 1 10); do
//!   echo "$i"
//! done
//!
//! for i in $(seq 1 2 10); do  # step of 2
//!   echo "$i"
//! done
//! ```
//!
//! ✅ **GOOD** (efficient - bash builtin):
//! ```bash
//! for i in {1..10}; do
//!   echo "$i"
//! done
//!
//! for ((i=1; i<=10; i+=2)); do  # step of 2
//!   echo "$i"
//! done
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - `for i in $(seq ...)` - Use `{start..end}` or C-style for loop
//! - `cat file | while read` - Use `while read < file` or `while read; do ... done < file`
//!
//! ## Auto-fix
//!
//! Suggests:
//! - Replace `$(seq 1 10)` with `{1..10}`
//! - Replace `cat file | while read` with `while read; do ... done < file`

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for inefficient loop patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Strip comments from code
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Pattern 1: for i in $(seq ...) - inefficient
        if code_only.contains("for ") && code_only.contains("$(seq") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

            let diag = Diagnostic::new(
                "BASH009",
                Severity::Info,
                "Inefficient loop using $(seq ...) - use bash brace expansion {start..end} or C-style for loop for better performance and portability",
                span,
            );
            result.add(diag);
        }

        // Pattern 2: cat file | while read - inefficient (UUOC - Useless Use of Cat)
        if code_only.contains("cat ") && code_only.contains("| while read") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

            let diag = Diagnostic::new(
                "BASH009",
                Severity::Info,
                "Inefficient pattern 'cat file | while read' - use 'while read; do ... done < file' to avoid spawning cat process",
                span,
            );
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect for loop with seq
    #[test]
    fn test_BASH009_detects_seq_in_loop() {
        let script = r#"#!/bin/bash
for i in $(seq 1 10); do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH009");
        assert_eq!(diag.severity, Severity::Info);
        assert!(diag.message.contains("seq"));
        assert!(diag.message.contains("brace expansion"));
    }

    /// RED TEST 2: Pass with brace expansion
    #[test]
    fn test_BASH009_passes_brace_expansion() {
        let script = r#"#!/bin/bash
for i in {1..10}; do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Brace expansion should pass");
    }

    /// RED TEST 3: Pass with C-style for loop
    #[test]
    fn test_BASH009_passes_c_style_loop() {
        let script = r#"#!/bin/bash
for ((i=1; i<=10; i++)); do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "C-style loop should pass");
    }

    /// RED TEST 4: Detect cat | while read
    #[test]
    fn test_BASH009_detects_cat_while_read() {
        let script = r#"#!/bin/bash
cat file.txt | while read line; do
  echo "$line"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH009");
        assert!(diag.message.contains("cat"));
        assert!(diag.message.contains("while read"));
    }

    /// RED TEST 5: Pass with while read < file
    #[test]
    fn test_BASH009_passes_while_read_redirect() {
        let script = r#"#!/bin/bash
while read line; do
  echo "$line"
done < file.txt
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Input redirection should pass");
    }

    /// RED TEST 6: Detect seq with step
    #[test]
    fn test_BASH009_detects_seq_with_step() {
        let script = r#"#!/bin/bash
for i in $(seq 1 2 10); do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH009");
    }

    /// RED TEST 7: Ignore seq in comments
    #[test]
    fn test_BASH009_ignores_comments() {
        let script = r#"#!/bin/bash
# for i in $(seq 1 10); do
for i in {1..10}; do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Comments should be ignored");
    }

    /// RED TEST 8: Multiple violations
    #[test]
    fn test_BASH009_detects_multiple_violations() {
        let script = r#"#!/bin/bash
for i in $(seq 1 10); do
  echo "$i"
done
cat data.txt | while read line; do
  echo "$line"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].code, "BASH009");
        assert_eq!(result.diagnostics[1].code, "BASH009");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_bash009_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects seq in loop
        #[test]
        fn prop_bash009_detects_seq(
            start in 1u8..50,
            end in 51u8..100,
        ) {
            let script = format!("for i in $(seq {} {}); do echo $i; done", start, end);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH009");
        }

        /// PROPERTY TEST 3: Passes with brace expansion
        #[test]
        fn prop_bash009_passes_braces(
            start in 1u8..50,
            end in 51u8..100,
        ) {
            let script = format!("for i in {{{}..{}}}; do echo $i; done", start, end);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
