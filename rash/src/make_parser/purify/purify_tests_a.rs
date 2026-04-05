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
