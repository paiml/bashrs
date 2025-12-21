//! BASH002: Missing `set -o pipefail` in Pipelines
//!
//! **Rule**: Detect pipelines without `set -o pipefail`
//!
//! **Why this matters**:
//! Without `pipefail`, pipelines only check the last command's exit code:
//! - `failing_cmd | succeeding_cmd` returns 0 (hides failure!)
//! - Errors in early pipeline stages go unnoticed
//! - Silent data corruption or incomplete processing
//! - Critical for production script reliability
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (missing pipefail):
//! ```bash
//! #!/bin/bash
//! set -e  # Only catches last command in pipeline!
//! curl https://example.com/data.json | jq '.items[]'
//! # If curl fails, jq succeeds with empty input → exit 0 (wrong!)
//! ```
//!
//! ✅ **SAFE** (with pipefail):
//! ```bash
//! #!/bin/bash
//! set -eo pipefail  # Catches failures anywhere in pipeline
//! curl https://example.com/data.json | jq '.items[]'
//! # If curl fails, script exits with error (correct!)
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects scripts that:
//! 1. Contain pipelines (commands with `|`)
//! 2. Do NOT have `set -o pipefail` or `set -euo pipefail`
//!
//! ## Auto-fix
//!
//! Suggests adding `set -o pipefail` after shebang:
//! ```bash
//! #!/bin/bash
//! set -eo pipefail  # Recommended: combine with set -e
//! ```

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for missing `set -o pipefail` in scripts with pipelines
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut has_pipefail = false;
    let mut has_pipeline = false;
    let mut first_pipeline_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Strip comments
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Check for pipefail setting
        if code_only.contains("set") && code_only.contains("pipefail") {
            has_pipefail = true;
        }

        // Check for pipeline (but not in comments, strings, or certain contexts)
        if code_only.contains('|') &&
           !code_only.contains("||") &&  // Not logical OR
           !is_in_string_or_regex(code_only)
        {
            if !has_pipeline {
                first_pipeline_line = line_num;
            }
            has_pipeline = true;
        }
    }

    // If script has pipelines but no pipefail, warn
    if has_pipeline && !has_pipefail {
        let span = Span::new(
            first_pipeline_line + 1,
            1,
            first_pipeline_line + 1,
            source.lines().nth(first_pipeline_line).unwrap_or("").len(),
        );

        let diag = Diagnostic::new(
            "BASH002",
            Severity::Warning,
            "Script uses pipelines without 'set -o pipefail' - pipeline failures may be hidden (only last command's exit code is checked)",
            span,
        );
        result.add(diag);
    }

    result
}

/// Check if pipe character is in a string or regex
fn is_in_string_or_regex(line: &str) -> bool {
    // Simple heuristic: if line contains quotes and pipe is between them
    // This is a simplified check - full parsing would be more accurate
    let single_quote_count = line.matches('\'').count();
    let double_quote_count = line.matches('"').count();

    // If odd number of quotes, pipe might be in string
    !single_quote_count.is_multiple_of(2) || !double_quote_count.is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect pipeline without pipefail
    #[test]
    fn test_BASH002_detects_pipeline_without_pipefail() {
        let script = r#"#!/bin/bash
set -e
curl https://example.com/data.json | jq '.items[]'
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH002");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("pipefail"));
        assert!(diag.message.contains("pipeline"));
    }

    /// RED TEST 2: Pass when pipefail is set
    #[test]
    fn test_BASH002_passes_with_pipefail() {
        let script = r#"#!/bin/bash
set -eo pipefail
curl https://example.com/data.json | jq '.items[]'
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass with pipefail");
    }

    /// RED TEST 3: Pass when set -o pipefail is used (alternative syntax)
    #[test]
    fn test_BASH002_passes_with_set_o_pipefail() {
        let script = r#"#!/bin/bash
set -e
set -o pipefail
cat file.txt | grep pattern | sort
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should pass with set -o pipefail"
        );
    }

    /// RED TEST 4: Pass when no pipelines exist
    #[test]
    fn test_BASH002_passes_without_pipelines() {
        let script = r#"#!/bin/bash
set -e
echo "Hello"
curl https://example.com/data.json
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass without pipelines");
    }

    /// RED TEST 5: Detect multiple pipelines without pipefail
    #[test]
    fn test_BASH002_detects_multiple_pipelines() {
        let script = r#"#!/bin/bash
cat file1.txt | grep foo
cat file2.txt | grep bar | sort
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH002");
    }

    /// RED TEST 6: Ignore logical OR (||)
    #[test]
    fn test_BASH002_ignores_logical_or() {
        let script = r#"#!/bin/bash
set -e
command1 || command2
[ -f file ] || exit 1
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should ignore logical OR");
    }

    /// RED TEST 7: Detect pipeline in function
    #[test]
    fn test_BASH002_detects_pipeline_in_function() {
        let script = r#"#!/bin/bash
process_data() {
  cat data.json | jq '.items[]'
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH002");
    }

    /// RED TEST 8: Pass with set -euo pipefail (combined flags)
    #[test]
    fn test_BASH002_passes_with_combined_flags() {
        let script = r#"#!/bin/bash
set -euo pipefail
find . -name "*.txt" | xargs grep pattern
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should pass with combined flags"
        );
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
        fn prop_bash002_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects pipeline without pipefail
        #[test]
        fn prop_bash002_detects_pipeline(
            cmd1 in "[a-z]{3,10}",
            cmd2 in "[a-z]{3,10}",
        ) {
            let script = format!("#!/bin/bash\n{} | {}", cmd1, cmd2);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH002");
        }

        /// PROPERTY TEST 3: Passes when pipefail is set
        #[test]
        fn prop_bash002_passes_with_pipefail(
            cmd1 in "[a-z]{3,10}",
            cmd2 in "[a-z]{3,10}",
        ) {
            let script = format!("#!/bin/bash\nset -o pipefail\n{} | {}", cmd1, cmd2);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
