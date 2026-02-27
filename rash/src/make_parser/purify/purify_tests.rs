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
fn test_purify_reproducible_detect_process_id() {
    let ast = make_ast(vec![var("TMP", "/tmp/build_$$$$")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectProcessId { .. })),
        "Should detect $$$$ as process ID"
    );
}

#[test]
fn test_purify_reproducible_detect_hostname() {
    let ast = make_ast(vec![var("HOST", "$(shell hostname)")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "hostname")
        ),
        "Should detect $(shell hostname) as non-deterministic"
    );
}

#[test]
fn test_purify_reproducible_detect_git_log_timestamp() {
    let ast = make_ast(vec![var(
        "GIT_DATE",
        "$(shell git log -1 --format=%cd --date=short)",
    )]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "git log timestamp")
        ),
        "Should detect git log timestamp"
    );
}

#[test]
fn test_purify_reproducible_detect_mktemp_in_recipe() {
    let ast = make_ast(vec![target(
        "build",
        vec!["TMP=$(mktemp -d)", "echo building"],
    )]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "mktemp")
        ),
        "Should detect mktemp in recipe"
    );
}

#[test]
fn test_purify_reproducible_no_mktemp_in_clean_recipe() {
    let ast = make_ast(vec![target("clean", vec!["rm -f *.o", "rm -f app"])]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        !result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "mktemp")
        ),
        "Clean recipe without mktemp should not flag mktemp"
    );
}

#[test]
fn test_purify_reproducible_multiple_issues_in_single_ast() {
    let ast = make_ast(vec![
        var("BUILD_TIME", "$(shell date +%s)"),
        var("RAND_ID", "$$RANDOM"),
        var("HOST", "$(shell hostname)"),
        var("TMP", "/tmp/build_$$$$"),
    ]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    // Expected: DetectTimestamp, SuggestSourceDateEpoch, DetectRandom, DetectNonDeterministicCommand (hostname), DetectProcessId
    assert!(
        result.len() >= 5,
        "Should detect at least 5 issues from 4 problematic variables (date generates 2)"
    );
}

#[test]
fn test_purify_reproducible_recursive_var_with_date() {
    let ast = make_ast(vec![recursive_var("VERSION", "$(shell date)")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectTimestamp { .. })),
        "Recursive variable with date should also be detected"
    );
}

#[test]
fn test_purify_reproducible_comment_items_ignored() {
    let ast = make_ast(vec![MakeItem::Comment {
        text: "$(shell date) is used for versioning".to_string(),
        span: Span::dummy(),
    }]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.is_empty(),
        "Comments should not trigger reproducibility analysis"
    );
}

#[test]
fn test_purify_reproducible_git_log_without_date_not_flagged() {
    // git log without %cd or --date should NOT trigger
    let ast = make_ast(vec![var(
        "GIT_HASH",
        "$(shell git log -1 --format=%H)",
    )]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        !result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "git log timestamp")
        ),
        "git log without --date/%%cd should not be flagged as timestamp"
    );
}

#[test]
fn test_purify_reproducible_hostname_in_brace_syntax() {
    let ast = make_ast(vec![var("HOST", "${shell hostname}")]);
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(
            |t| matches!(t, Transformation::DetectNonDeterministicCommand { command, .. } if command == "hostname")
        ),
        "Should detect hostname in ${{shell ...}} brace syntax"
    );
}

// =============================================================================
// 3. emitter/makefile.rs — analyze_makefile_line (tested indirectly via emit_makefile)
// =============================================================================

use crate::ast::restricted::{Function, Literal, Type};
use crate::ast::{Expr, RestrictedAst, Stmt};

/// Helper to build a RestrictedAst that emits a single raw line via rash_println
fn ast_with_println(line: &str) -> RestrictedAst {
    RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "rash_println".to_string(),
                args: vec![Expr::Literal(Literal::Str(line.to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    }
}

/// Helper to build a RestrictedAst that emits a single raw line via exec()
fn ast_with_exec(line: &str) -> RestrictedAst {
    RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "exec".to_string(),
                args: vec![Expr::Literal(Literal::Str(line.to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    }
}

#[test]
fn test_purify_emit_makefile_target_line() {
    let ast = ast_with_println("build: main.o util.o");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("build"),
        "Should emit target name from line with colon"
    );
}

#[test]
fn test_purify_emit_makefile_simple_variable_line() {
    let ast = ast_with_println("CC := gcc");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("CC"),
        "Should emit variable name from := assignment"
    );
}

#[test]
fn test_purify_emit_makefile_recursive_variable_line() {
    let ast = ast_with_println("CFLAGS = -Wall -O2");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("CFLAGS"),
        "Should emit variable from = assignment"
    );
}

#[test]
fn test_purify_emit_makefile_comment_fallback() {
    // A line that matches no pattern should be emitted as-is in raw mode
    let ast = ast_with_println("just a plain text line");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("just a plain text line"),
        "Plain text should be emitted in raw output mode"
    );
}

#[test]
fn test_purify_emit_makefile_phony_line() {
    let ast = ast_with_println(".PHONY: clean test build");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains(".PHONY"),
        "Should preserve .PHONY directive in output"
    );
}

#[test]
fn test_purify_emit_makefile_tab_prefixed_recipe_line() {
    let ast = ast_with_println("\tgcc -o app main.c");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("gcc -o app main.c"),
        "Tab-prefixed recipe line should be preserved"
    );
}

#[test]
fn test_purify_emit_makefile_exec_target_line() {
    let ast = ast_with_exec("all: build test");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("all"),
        "exec() with target line should produce output containing target"
    );
}

#[test]
fn test_purify_emit_makefile_multiple_lines() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Expr(Expr::FunctionCall {
                    name: "rash_println".to_string(),
                    args: vec![Expr::Literal(Literal::Str("CC := gcc".to_string()))],
                }),
                Stmt::Expr(Expr::FunctionCall {
                    name: "rash_println".to_string(),
                    args: vec![Expr::Literal(Literal::Str(
                        "CFLAGS := -Wall -O2".to_string(),
                    ))],
                }),
                Stmt::Expr(Expr::FunctionCall {
                    name: "rash_println".to_string(),
                    args: vec![Expr::Literal(Literal::Str("build: main.c".to_string()))],
                }),
            ],
        }],
        entry_point: "main".to_string(),
    };
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(result.contains("CC"), "Should contain CC variable");
    assert!(result.contains("CFLAGS"), "Should contain CFLAGS variable");
    assert!(result.contains("build"), "Should contain build target");
}

#[test]
fn test_purify_emit_makefile_conditional_line_as_comment() {
    // Lines starting with 'ifeq' or 'ifdef' are not directly parsed as
    // targets or variables, so they get emitted as comments in raw mode
    let ast = ast_with_println("ifeq ($(DEBUG),1)");
    let result = crate::emitter::makefile::emit_makefile(&ast).unwrap();
    assert!(
        result.contains("ifeq"),
        "Conditional line should be preserved in raw output"
    );
}

// =============================================================================
// 4. report.rs — is_safe_transformation for all remaining variants
// =============================================================================

#[test]
fn test_purify_is_safe_detect_random() {
    let t = Transformation::DetectRandom {
        variable_name: "X".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_process_id() {
    let t = Transformation::DetectProcessId {
        variable_name: "TMP".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_source_date_epoch() {
    let t = Transformation::SuggestSourceDateEpoch {
        variable_name: "BUILD_DATE".to_string(),
        original_pattern: "$(shell date)".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_non_deterministic_command() {
    let t = Transformation::DetectNonDeterministicCommand {
        variable_name: "HOST".to_string(),
        command: "hostname".to_string(),
        reason: "env-dependent".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_order_only_prereq() {
    let t = Transformation::RecommendOrderOnlyPrereq {
        target_name: "build".to_string(),
        prereq_name: "dir".to_string(),
        reason: "dir must exist".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_missing_dependency() {
    let t = Transformation::DetectMissingDependency {
        target_name: "link".to_string(),
        missing_file: "main.o".to_string(),
        provider_target: "compile".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_output_conflict() {
    let t = Transformation::DetectOutputConflict {
        target_names: vec!["a".to_string()],
        output_file: "out".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_recursive_make_handling() {
    let t = Transformation::RecommendRecursiveMakeHandling {
        target_name: "all".to_string(),
        subdirs: vec!["sub1".to_string()],
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_directory_race() {
    let t = Transformation::DetectDirectoryRace {
        target_names: vec!["a".to_string()],
        directory: "obj".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_simple_expansion() {
    let t = Transformation::SuggestSimpleExpansion {
        variable_name: "CC".to_string(),
        reason: "constant".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_suffixes() {
    let t = Transformation::RecommendSuffixes {
        reason: "disable builtin rules".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_sequential_recipes() {
    let t = Transformation::DetectSequentialRecipes {
        target_name: "install".to_string(),
        recipe_count: 5,
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_pattern_rule() {
    let t = Transformation::SuggestPatternRule {
        pattern: "%.o: %.c".to_string(),
        target_count: 3,
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_missing_error_handling() {
    let t = Transformation::DetectMissingErrorHandling {
        target_name: "build".to_string(),
        command: "gcc".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_silent_failure() {
    let t = Transformation::DetectSilentFailure {
        target_name: "test".to_string(),
        command: "@echo".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_delete_on_error() {
    let t = Transformation::RecommendDeleteOnError {
        reason: "safety".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_oneshell() {
    let t = Transformation::RecommendOneshell {
        target_name: "deploy".to_string(),
        reason: "multiline recipe".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_missing_set_e() {
    let t = Transformation::DetectMissingSetE {
        target_name: "test".to_string(),
        command: "bash -c".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_loop_without_error_handling() {
    let t = Transformation::DetectLoopWithoutErrorHandling {
        target_name: "deploy".to_string(),
        loop_command: "for f in *.sh; do sh $f; done".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_bashism() {
    let t = Transformation::DetectBashism {
        target_name: "test".to_string(),
        construct: "[[".to_string(),
        posix_alternative: "use [".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_platform_specific() {
    let t = Transformation::DetectPlatformSpecific {
        target_name: "check".to_string(),
        command: "uname".to_string(),
        reason: "OS-specific".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_shell_specific() {
    let t = Transformation::DetectShellSpecific {
        target_name: "run".to_string(),
        feature: "source".to_string(),
        posix_alternative: "use .".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_non_portable_flags() {
    let t = Transformation::DetectNonPortableFlags {
        target_name: "copy".to_string(),
        command: "cp --preserve=all".to_string(),
        flag: "--preserve".to_string(),
        reason: "GNU-only".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_non_portable_echo() {
    let t = Transformation::DetectNonPortableEcho {
        target_name: "info".to_string(),
        command: "echo -e".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}
