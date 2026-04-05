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
    let ast = make_ast(vec![var("GIT_HASH", "$(shell git log -1 --format=%H)")]);
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

include!("purify_tests_incl2_incl2.rs");
