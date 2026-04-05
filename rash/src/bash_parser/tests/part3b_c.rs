#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_PARAM_SPEC_002_exit_status_comparison_table() {
    let comparison_example = r#"
# POSIX: $? fully supported
cmd
echo "Exit: $?"

# POSIX: Capture and use
cmd
STATUS=$?
if [ $STATUS -ne 0 ]; then
  echo "Failed with code $STATUS"
  exit $STATUS
fi

# POSIX: set -o pipefail (supported in bash, dash, ash)
set -o pipefail
cmd1 | cmd2 | cmd3
if [ $? -ne 0 ]; then
  echo "Pipeline failed"
fi

# Bash-only: PIPESTATUS (NOT SUPPORTED)
# cmd1 | cmd2 | cmd3
# echo "${PIPESTATUS[@]}"  # bashrs doesn't support this
"#;

    assert_parses_without_panic(comparison_example, "$? comparison documented");
}

// Summary:
// $? (exit status): FULLY SUPPORTED (POSIX)
// Range: 0-255 (0=success, non-zero=failure)
// Special codes: 126 (not executable), 127 (not found), 128+N (signal)
// Clobbering: Updated after every command
// Best practice: Capture immediately or use direct conditionals
// PIPESTATUS: NOT SUPPORTED (bash extension)
// pipefail: SUPPORTED (POSIX, available in bash/dash/ash)

// ============================================================================
// PARAM-SPEC-003: $$ Process ID (POSIX, but NON-DETERMINISTIC - PURIFY)
// ============================================================================

// DOCUMENTATION: $$ is POSIX but NON-DETERMINISTIC (must purify)
// $$ contains the process ID of the current shell. Changes every run.
// Purification: replace $$ with fixed identifier, use mktemp for temp files.