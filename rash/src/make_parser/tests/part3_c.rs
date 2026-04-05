#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_SEMANTIC_CALL_005_shell_find_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) in call arguments
    let makefile = r#"
process_files = Processing: $(1)
OUTPUT := $(call process_files, $(shell find src -name '*.c'))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 67: Purification Engine Tests
// ============================================================================
//
// Goal: Implement purification engine that auto-fixes non-deterministic
//       patterns detected by semantic analysis.
//
// Approach: EXTREME TDD - Write RED tests first, then implement

