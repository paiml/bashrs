#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_SEMANTIC_RECURSIVE_014_detect_multiple_nested_issues() {
    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all three types of issues detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            // Should detect wildcard, $RANDOM, and shell find
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("RANDOM"));
            assert!(value.contains("$(shell find"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_015_pattern_rule_with_nested_wildcard() {
    // ARRANGE: pattern rule with wildcard in prerequisites
    let makefile = r#"
%.o: $(filter %.c, $(wildcard src/*.c))
	gcc -c $< -o $@
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify pattern rule contains nested wildcard
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            // Prerequisites should contain nested wildcard
            let prereqs = prereq_patterns.join(" ");
            assert!(prereqs.contains("$(wildcard"));
            assert!(prereqs.contains("$(filter"));
        }
        _ => panic!("Expected PatternRule, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Integration Tests for analyze_makefile() with Nested Patterns
// ============================================================================
// These tests verify that analyze_makefile() detects non-deterministic patterns
// even when nested inside function arguments

#[test]
fn test_SEMANTIC_ANALYZE_001_detect_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested wildcard
    assert!(
        !issues.is_empty(),
        "Expected to detect nested wildcard, but got no issues"
    );
    assert_eq!(
        issues.len(),
        1,
        "Expected exactly 1 issue for nested wildcard"
    );
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FILES"));
    assert!(issues[0].message.contains("wildcard"));
}

#[test]
fn test_SEMANTIC_ANALYZE_002_detect_nested_shell_date_in_addsuffix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell date
    assert!(!issues.is_empty(), "Expected to detect nested shell date");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
    assert!(issues[0].message.contains("TIMESTAMPED"));
}

#[test]
fn test_SEMANTIC_ANALYZE_003_detect_nested_random_in_word() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested $RANDOM
    assert!(!issues.is_empty(), "Expected to detect nested $RANDOM");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_RANDOM");
    assert!(issues[0].message.contains("PICK"));
}

#[test]
fn test_SEMANTIC_ANALYZE_004_no_issue_for_safe_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect any issues (no wildcard, no shell, no random)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for safe filter, but got: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_005_purified_wildcard_not_detected() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: PURIFIED wildcard wrapped with sort
    // Enhancement IMPLEMENTED: detect $(sort $(wildcard)) as "already purified"
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Purified wildcards should NOT be detected
    assert_eq!(
        issues.len(),
        0,
        "Purified wildcard should not be detected: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_006_deeply_nested_unpurified_wildcard() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: deeply nested - wildcard in filter in sort (NOT PURIFIED PROPERLY)
    // This is NOT purified because the wildcard itself is not wrapped with sort
    // The outer sort only sorts the filter results, not the wildcard results
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard because it's not directly wrapped with sort
    assert!(
        !issues.is_empty(),
        "Wildcard should be detected when not directly wrapped with sort"
    );
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_ANALYZE_007_multiple_nested_wildcards() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (may report as 1 issue for the variable)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("MULTI"));
}

#[test]
fn test_SEMANTIC_ANALYZE_008_nested_shell_find_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell find
    assert!(!issues.is_empty());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_UNORDERED_FIND");
    assert!(issues[0].message.contains("FOUND"));
}

#[test]
fn test_SEMANTIC_ANALYZE_009_multiple_different_nested_issues() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect all three types of issues
    // Current implementation detects all patterns in the value string
    assert!(
        issues.len() >= 3,
        "Expected at least 3 issues (wildcard, random, shell find), got {}",
        issues.len()
    );

    // Verify all three rule types are detected
    let rules: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
    assert!(rules.contains(&"NO_WILDCARD"), "Should detect wildcard");
    assert!(rules.contains(&"NO_RANDOM"), "Should detect $RANDOM");
    assert!(
        rules.contains(&"NO_UNORDERED_FIND"),
        "Should detect shell find"
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_010_nested_wildcard_in_firstword() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard in firstword (HIGH RISK - different results based on order)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (critical case for firstword)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FIRST"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - FOREACH Detection Tests
// ============================================================================
//
// Goal: Verify semantic analysis detects non-deterministic patterns in
//       $(foreach) loops where iteration order matters.
//
// Hypothesis (based on Sprint 64-65): Existing .contains() approach
// already detects these patterns at any nesting level.
//
// Test Strategy: Write verification tests first (EXTREME TDD RED phase)

#[test]
fn test_SEMANTIC_FOREACH_001_detect_wildcard_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) iterating over $(wildcard) - ORDER MATTERS!
    // This is CRITICAL because foreach processes items in iteration order
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in foreach list (non-deterministic order)
    // Based on Sprint 65 discovery: .contains("$(wildcard") should catch this!
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in foreach list"
    );

    // Verify it's detected as NO_WILDCARD
    let wildcard_issues: Vec<_> = issues.iter().filter(|i| i.rule == "NO_WILDCARD").collect();
    assert!(!wildcard_issues.is_empty(), "Should detect as NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_FOREACH_002_safe_foreach_with_explicit_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) with explicit list (SAFE - deterministic order)
    let makefile = "OBJS := $(foreach file, foo.c bar.c baz.c, $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit list is deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit list: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_FOREACH_003_nested_shell_date_in_foreach() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell date) nested in foreach body
    let makefile = "TIMESTAMPED := $(foreach f, foo bar, $(f)-$(shell date +%s))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_FOREACH_004_random_in_foreach_body() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in foreach body
    let makefile = "IDS := $(foreach item, a b c, id-$RANDOM)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

#[test]
fn test_SEMANTIC_FOREACH_005_shell_find_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) as foreach list source
    let makefile = "PROCESSED := $(foreach f, $(shell find src -name '*.c'), process-$(f))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - CALL Detection Tests
// ============================================================================

#[test]
fn test_SEMANTIC_CALL_001_detect_wildcard_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(wildcard) in arguments
    // Define a function and call it with wildcard
    let makefile = r#"
reverse = $(2) $(1)
FILES := $(call reverse, $(wildcard *.c), foo.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in call arguments
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in call args"
    );

    // Check that FILES variable has wildcard issue
    let files_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("FILES"))
        .collect();
    assert!(
        !files_issues.is_empty(),
        "Should detect wildcard in FILES variable"
    );
}

#[test]
fn test_SEMANTIC_CALL_002_safe_call_with_explicit_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with explicit arguments (SAFE)
    let makefile = r#"
reverse = $(2) $(1)
RESULT := $(call reverse, foo.c, bar.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit args are deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit args: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_CALL_003_shell_date_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(shell date) in arguments
    let makefile = r#"
timestamp = build-$(1)-$(2)
RELEASE := $(call timestamp, v1.0, $(shell date +%s))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_CALL_004_random_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in call arguments
    let makefile = r#"
generate_id = id-$(1)-$(2)
SESSION := $(call generate_id, sess, $RANDOM)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

