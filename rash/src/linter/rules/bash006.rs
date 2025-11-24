//! BASH006: Missing Function Documentation
//!
//! **Rule**: Detect functions without docstring comments
//!
//! **Why this matters**:
//! Undocumented functions reduce code maintainability:
//! - Harder to understand function purpose
//! - Parameters and return values unclear
//! - Team collaboration suffers
//! - Onboarding new developers takes longer
//!
//! **Examples**:
//!
//! ❌ **BAD** (missing documentation):
//! ```bash
//! process_data() {
//!   local input="$1"
//!   echo "$input" | jq '.items[]'
//! }
//! ```
//!
//! ✅ **GOOD** (with documentation):
//! ```bash
//! # Process JSON data and extract items array
//! # Arguments:
//! #   $1 - JSON input string
//! # Returns:
//! #   Items array, one per line
//! process_data() {
//!   local input="$1"
//!   echo "$input" | jq '.items[]'
//! }
//! ```
//!
//! ## Detection Logic
//!
//! This rule detects:
//! - Function definitions: `function_name()` or `function function_name()`
//! - Missing comment block immediately before function
//!
//! ## Auto-fix
//!
//! Suggests adding documentation template:
//! ```bash
//! # Brief description of function
//! # Arguments:
//! #   $1 - Description of first argument
//! # Returns:
//! #   Description of return value
//! ```

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Check for functions without documentation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detect function definitions
        // Pattern 1: function_name() {
        // Pattern 2: function function_name() {
        if is_function_definition(trimmed) {
            // Check if previous line is a comment
            let has_doc = if line_num > 0 {
                has_documentation_comment(&lines, line_num)
            } else {
                false
            };

            if !has_doc {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

                let function_name = extract_function_name(trimmed);
                let diag = Diagnostic::new(
                    "BASH006",
                    Severity::Info,
                    format!(
                        "Function '{}' lacks documentation - add comment describing purpose, arguments, and return value for better maintainability",
                        function_name
                    ),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

/// Check if line is a function definition
fn is_function_definition(line: &str) -> bool {
    // Pattern 1: function_name() {
    if line.contains("()") && (line.contains('{') || line.ends_with("()")) {
        // Exclude common non-function patterns (control flow statements)
        // Use word boundaries to avoid excluding function names like "if_", "while_", "for_loop"
        if line.starts_with("if ") || line.starts_with("while ") || line.starts_with("for ") {
            return false;
        }
        return true;
    }

    // Pattern 2: function function_name() {
    if line.starts_with("function ") && line.contains("()") {
        return true;
    }

    false
}

/// Extract function name from definition
fn extract_function_name(line: &str) -> String {
    // Remove "function " prefix if present
    let without_keyword = line.strip_prefix("function ").unwrap_or(line);

    // Extract name before ()
    if let Some(pos) = without_keyword.find('(') {
        without_keyword[..pos].trim().to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if function has documentation comment
fn has_documentation_comment(lines: &[&str], func_line: usize) -> bool {
    // Look for comment immediately before function (allowing blank lines)
    let mut check_line = func_line;

    // Skip back over blank lines
    while check_line > 0 {
        check_line -= 1;
        let trimmed = lines[check_line].trim();

        if trimmed.is_empty() {
            continue; // Skip blank lines
        }

        // Check if it's a comment (but not a shebang)
        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            return true; // Found documentation comment
        }

        // Non-comment, non-blank line (or shebang)
        return false;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect function without documentation
    #[test]
    fn test_BASH006_detects_undocumented_function() {
        let script = r#"#!/bin/bash
process_data() {
  echo "Processing"
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH006");
        assert_eq!(diag.severity, Severity::Info);
        assert!(diag.message.contains("process_data"));
        assert!(diag.message.contains("documentation"));
    }

    /// RED TEST 2: Pass when function has documentation
    #[test]
    fn test_BASH006_passes_with_documentation() {
        let script = r#"#!/bin/bash
# Process data from input
# Arguments: $1 - input file
process_data() {
  echo "Processing"
}
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Should pass with documentation"
        );
    }

    /// RED TEST 3: Detect function keyword syntax
    #[test]
    fn test_BASH006_detects_function_keyword() {
        let script = r#"#!/bin/bash
function build() {
  make all
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "BASH006");
        assert!(diag.message.contains("build"));
    }

    /// RED TEST 4: Pass with single-line comment
    #[test]
    fn test_BASH006_passes_with_single_comment() {
        let script = r#"#!/bin/bash
# Build the project
function build() {
  make all
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass with comment");
    }

    /// RED TEST 5: Detect multiple undocumented functions
    #[test]
    fn test_BASH006_detects_multiple_functions() {
        let script = r#"#!/bin/bash
build() {
  make all
}

test() {
  make test
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].code, "BASH006");
        assert_eq!(result.diagnostics[1].code, "BASH006");
    }

    /// RED TEST 6: Pass with blank line between comment and function
    #[test]
    fn test_BASH006_passes_with_blank_line() {
        let script = r#"#!/bin/bash
# Deploy application

deploy() {
  echo "Deploying"
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should pass with blank line");
    }

    /// RED TEST 7: Ignore non-function patterns
    #[test]
    fn test_BASH006_ignores_non_functions() {
        let script = r#"#!/bin/bash
if [ -f file ]; then
  echo "exists"
fi

for i in $(seq 1 10); do
  echo "$i"
done
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Should ignore if/for");
    }

    /// RED TEST 8: Detect function with multi-line definition
    #[test]
    fn test_BASH006_detects_multiline_function() {
        let script = r#"#!/bin/bash
complex_function() {
  local var="value"
  echo "$var"
}
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("complex_function"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_bash006_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects undocumented function
        #[test]
        fn prop_bash006_detects_undocumented(
            func_name in "[a-z_]{3,15}",
        ) {
            let script = format!("{}() {{\n  echo test\n}}", func_name);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "BASH006");
        }

        /// PROPERTY TEST 3: Passes with documentation
        #[test]
        fn prop_bash006_passes_with_doc(
            func_name in "[a-z_]{3,15}",
        ) {
            let script = format!("# Function doc\n{}() {{\n  echo test\n}}", func_name);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
