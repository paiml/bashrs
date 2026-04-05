//! Comprehensive tests for bash_parser/codegen.rs
//!
//! EXTREME TDD coverage improvement: 26.5% → >90%
//!
//! Coverage targets:
//! - Unit tests: All 7 functions (generate_purified_bash, generate_statement, etc.)
//! - Property tests: Determinism, idempotency, shellcheck compliance
//! - Mutation tests: >90% kill rate

#![allow(clippy::expect_used)]

use super::ast::*;
use super::codegen::*;

// ===== RED PHASE: Unit Tests for generate_purified_bash() =====

#[test]
fn test_codegen_001_shebang_transformation() {
    // Task 1.1: Shebang transformation - #!/bin/bash → #!/bin/sh
    let ast = BashAst {
        statements: vec![],
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.starts_with("#!/bin/sh\n"),
        "Should emit POSIX sh shebang"
    );
}

#[test]
fn test_codegen_002_simple_command() {
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Literal("hello".to_string())],
            redirects: vec![],
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("echo hello"),
        "Should generate echo command"
    );
    assert!(output.starts_with("#!/bin/sh\n"), "Should have shebang");
}

#[test]
fn test_codegen_003_assignment_not_exported() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            index: None,
            value: BashExpr::Literal("value".to_string()),
            exported: false,
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(output.contains("VAR=value"), "Should generate assignment");
    assert!(!output.contains("export"), "Should not have export keyword");
}

#[test]
fn test_codegen_004_assignment_exported() {
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            index: None,
            value: BashExpr::Literal("value".to_string()),
            exported: true,
            span: Span::new(1, 1, 1, 10),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("export VAR=value"),
        "Should generate exported assignment"
    );
}

#[test]
fn test_codegen_005_comment_preserved() {
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: "This is a comment".to_string(),
            span: Span::new(1, 1, 1, 20),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    assert!(
        output.contains("# This is a comment"),
        "Should preserve comment"
    );
}

#[test]
fn test_codegen_006_shebang_comment_skipped() {
    // Shebangs in comments should be skipped to maintain idempotency
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: "!/bin/bash".to_string(),
            span: Span::new(1, 1, 1, 12),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let output = generate_purified_bash(&ast);

    // Should only have the #!/bin/sh shebang, not the comment
    assert_eq!(
        output.lines().count(),
        2,
        "Should have shebang + empty line"
    );
    assert!(output.starts_with("#!/bin/sh\n"));
}

#[test]

include!("codegen_tests_incl2.rs");
