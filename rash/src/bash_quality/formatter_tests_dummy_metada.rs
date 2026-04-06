
use super::*;
use crate::bash_parser::ast::{AstMetadata, BashExpr, BashStmt, CaseArm, Span};

fn dummy_metadata() -> AstMetadata {
    AstMetadata {
        source_file: None,
        line_count: 1,
        parse_time_ms: 0,
    }
}

#[test]
fn test_formatter_new() {
    let formatter = Formatter::new();
    assert_eq!(formatter.config.indent_width, 2);
    assert!(!formatter.config.use_tabs);
}

#[test]
fn test_formatter_default() {
    let formatter = Formatter::default();
    assert_eq!(formatter.config.indent_width, 2);
}

#[test]
fn test_formatter_with_config() {
    let config = FormatterConfig {
        indent_width: 4,
        ..Default::default()
    };
    let formatter = Formatter::with_config(config);
    assert_eq!(formatter.config.indent_width, 4);
}

#[test]
fn test_set_source() {
    let mut formatter = Formatter::new();
    assert!(formatter.source.is_none());
    formatter.set_source("echo hello");
    assert!(formatter.source.is_some());
    assert_eq!(formatter.source.unwrap(), "echo hello");
}

#[test]
fn test_format_assignment() {
    let formatter = Formatter::new();
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            index: None,
            value: BashExpr::Literal("value".to_string()),
            exported: false,
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert_eq!(result, "VAR=value");
}

#[test]
fn test_format_exported_assignment() {
    let formatter = Formatter::new();
    let ast = BashAst {
        statements: vec![BashStmt::Assignment {
            name: "VAR".to_string(),
            index: None,
            value: BashExpr::Literal("value".to_string()),
            exported: true,
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert!(result.contains("export "));
    assert!(result.contains("VAR=value"));
}

#[test]
fn test_format_comment() {
    let formatter = Formatter::new();
    let ast = BashAst {
        statements: vec![BashStmt::Comment {
            text: " This is a comment".to_string(),
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert_eq!(result, "# This is a comment");
}

#[test]
fn test_format_command() {
    let formatter = Formatter::new();
    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![
                BashExpr::Literal("hello".to_string()),
                BashExpr::Variable("name".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("hello"));
}

#[test]
fn test_format_function() {
    let formatter = Formatter::new();
    let ast = BashAst {
        statements: vec![BashStmt::Function {
            name: "greet".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert!(result.contains("greet() {"));
    assert!(result.contains("  echo hello"));
    assert!(result.contains("}"));
}

#[test]
fn test_format_function_not_normalized() {
    let config = FormatterConfig {
        normalize_functions: false,
        ..Default::default()
    };
    let formatter = Formatter::with_config(config);

    let ast = BashAst {
        statements: vec![BashStmt::Function {
            name: "test".to_string(),
            body: vec![],
            span: Span::dummy(),
        }],
        metadata: dummy_metadata(),
    };

    let result = formatter.format(&ast).unwrap();
    assert!(result.contains("function test()"));
}

#[cfg(test)]
mod formatter_tests_ext_test_format_fun {
    use super::*;
    // FIXME(PMAT-238): include!("formatter_tests_ext_test_format_fun.rs");
}
