//! Additional coverage tests for make_parser/purify/ module
//!
//! Tests target three areas:
//! 1. report.rs — format_analysis_transformation with various transformation inputs
//! 2. reproducible_builds.rs — analyze_reproducible_builds with various Makefile patterns
//! 3. emitter/makefile.rs — analyze_makefile_line indirectly via emit_makefile public API

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::report;
use super::reproducible_builds;
use super::Transformation;
use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span, VarFlavor};

// =============================================================================
// Helper: build a MakeAst from a list of items
// =============================================================================

fn make_ast(items: Vec<MakeItem>) -> MakeAst {
    MakeAst {
        items,
        metadata: MakeMetadata::new(),
    }
}

fn var(name: &str, value: &str) -> MakeItem {
    MakeItem::Variable {
        name: name.to_string(),
        value: value.to_string(),
        flavor: VarFlavor::Simple,
        span: Span::dummy(),
    }
}

fn recursive_var(name: &str, value: &str) -> MakeItem {
    MakeItem::Variable {
        name: name.to_string(),
        value: value.to_string(),
        flavor: VarFlavor::Recursive,
        span: Span::dummy(),
    }
}

fn target(name: &str, recipe: Vec<&str>) -> MakeItem {
    MakeItem::Target {
        name: name.to_string(),
        prerequisites: vec![],
        recipe: recipe.into_iter().map(String::from).collect(),
        phony: false,
        recipe_metadata: None,
        span: Span::dummy(),
    }
}

// =============================================================================
// 1. report.rs — generate_report / is_safe_transformation / format_analysis_transformation
// =============================================================================

#[test]
fn test_purify_report_empty_transformations() {
    let result = report::generate_report(&[]);
    assert!(
        result.is_empty(),
        "Empty transformations should produce empty report"
    );
}

#[test]
fn test_purify_report_single_wrap_with_sort() {
    let transformations = vec![Transformation::WrapWithSort {
        variable_name: "SOURCES".to_string(),
        pattern: "$(wildcard".to_string(),
        safe: true,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Wrapped"));
    assert!(result[0].contains("$(wildcard"));
    assert!(result[0].contains("SOURCES"));
    assert!(result[0].contains("$(sort"));
}

#[test]
fn test_purify_report_single_add_comment() {
    let transformations = vec![Transformation::AddComment {
        variable_name: "STAMP".to_string(),
        rule: "NO_TIMESTAMPS".to_string(),
        suggestion: "Use SOURCE_DATE_EPOCH".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Manual fix needed"));
    assert!(result[0].contains("STAMP"));
    assert!(result[0].contains("NO_TIMESTAMPS"));
}

#[test]
fn test_purify_report_multiple_mixed_transformations() {
    let transformations = vec![
        Transformation::WrapWithSort {
            variable_name: "FILES".to_string(),
            pattern: "$(wildcard".to_string(),
            safe: true,
        },
        Transformation::DetectTimestamp {
            variable_name: "BUILD_DATE".to_string(),
            pattern: "$(shell date)".to_string(),
            safe: false,
        },
        Transformation::DetectBashism {
            target_name: "test".to_string(),
            construct: "[[".to_string(),
            posix_alternative: "Use [ instead".to_string(),
            safe: false,
        },
    ];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 3);
    assert!(result[0].contains("Wrapped"));
    assert!(result[1].contains("timestamp"));
    assert!(result[2].contains("bashism"));
}

#[test]
fn test_purify_is_safe_transformation_safe_wrap() {
    let t = Transformation::WrapWithSort {
        variable_name: "X".to_string(),
        pattern: "$(wildcard".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_transformation_unsafe_comment() {
    let t = Transformation::AddComment {
        variable_name: "X".to_string(),
        rule: "NO_TIMESTAMPS".to_string(),
        suggestion: String::new(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_race_condition_unsafe() {
    let t = Transformation::DetectRaceCondition {
        target_names: vec!["a".to_string(), "b".to_string()],
        conflicting_file: "out.txt".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_not_parallel() {
    let t = Transformation::RecommendNotParallel {
        reason: "shared state".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_timestamp() {
    let t = Transformation::DetectTimestamp {
        variable_name: "V".to_string(),
        pattern: "date".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_combine_shell_safe() {
    let t = Transformation::SuggestCombineShellInvocations {
        target_name: "build".to_string(),
        recipe_count: 3,
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_report_format_recommend_not_parallel() {
    let transformations = vec![Transformation::RecommendNotParallel {
        reason: "shared state between targets".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Parallel safety"));
    assert!(result[0].contains(".NOTPARALLEL"));
}

#[test]
fn test_purify_report_format_detect_race_condition() {
    let transformations = vec![Transformation::DetectRaceCondition {
        target_names: vec!["target1".to_string(), "target2".to_string()],
        conflicting_file: "shared.txt".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Race condition"));
    assert!(result[0].contains("shared.txt"));
}

#[test]
fn test_purify_report_format_recommend_order_only_prereq() {
    let transformations = vec![Transformation::RecommendOrderOnlyPrereq {
        target_name: "build".to_string(),
        prereq_name: "output_dir".to_string(),
        reason: "directory must exist".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("order-only prerequisite"));
    assert!(result[0].contains("build"));
    assert!(result[0].contains("output_dir"));
}

#[test]
fn test_purify_report_format_detect_missing_dependency() {
    let transformations = vec![Transformation::DetectMissingDependency {
        target_name: "process".to_string(),
        missing_file: "data.txt".to_string(),
        provider_target: "generate".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Missing dependency"));
    assert!(result[0].contains("process"));
    assert!(result[0].contains("data.txt"));
    assert!(result[0].contains("generate"));
}

#[test]
fn test_purify_report_format_detect_output_conflict() {
    let transformations = vec![Transformation::DetectOutputConflict {
        target_names: vec!["debug".to_string(), "release".to_string()],
        output_file: "app".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Output conflict"));
    assert!(result[0].contains("app"));
}

#[test]
fn test_purify_report_format_recommend_recursive_make_handling() {
    let transformations = vec![Transformation::RecommendRecursiveMakeHandling {
        target_name: "all".to_string(),
        subdirs: vec!["subdir1".to_string(), "subdir2".to_string()],
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Recursive make"));
    assert!(result[0].contains("$(MAKE)"));
}

#[test]
fn test_purify_report_format_detect_directory_race() {
    let transformations = vec![Transformation::DetectDirectoryRace {
        target_names: vec!["a".to_string(), "b".to_string()],
        directory: "obj".to_string(),
        safe: false,
    }];
    let result = report::generate_report(&transformations);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("Directory creation race"));
    assert!(result[0].contains("obj"));
}

// =============================================================================
// 2. reproducible_builds.rs — analyze_reproducible_builds
// =============================================================================

#[test]
fn test_purify_reproducible_empty_ast() {
    let ast = make_ast(vec![]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.is_empty(),
        "Empty AST should produce no reproducibility issues"
    );
}

#[test]
fn test_purify_reproducible_clean_variable_no_issues() {
    let ast = make_ast(vec![var("CC", "gcc")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.is_empty(),
        "Simple constant variable should have no reproducibility issues"
    );
}

#[test]
fn test_purify_reproducible_detect_shell_date() {
    let ast = make_ast(vec![var("BUILD_TIME", "$(shell date +%Y%m%d)")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectTimestamp { .. })),
        "Should detect $(shell date) as timestamp"
    );
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::SuggestSourceDateEpoch { .. })),
        "Should suggest SOURCE_DATE_EPOCH for date patterns"
    );
}

#[test]
fn test_purify_reproducible_detect_shell_date_with_brace_syntax() {
    let ast = make_ast(vec![var("STAMP", "${shell date +%s}")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectTimestamp { .. })),
        "Should detect ${{shell date}} brace syntax as timestamp"
    );
}

#[test]
fn test_purify_reproducible_detect_dollar_random() {
    let ast = make_ast(vec![var("SEED", "$$RANDOM")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectRandom { .. })),
        "Should detect $$RANDOM"
    );
}

#[test]
fn test_purify_reproducible_detect_bare_random() {
    let ast = make_ast(vec![var("SEED", "$RANDOM")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectRandom { .. })),
        "Should detect $RANDOM without double dollar"
    );
}

#[test]

include!("purify_tests_incl2.rs");
