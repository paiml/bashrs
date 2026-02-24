//! Coverage tests for codegen.rs uncovered branches (~9%, 247 lines)
//!
//! Targets: generate_declare_posix, Select, Negated, subshell brace group,
//! literal shell keyword quoting, multi-elif, multi-pattern case, pipeline,
//! nested indentation, until with non-test condition, declare+redirect combos.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::codegen::generate_purified_bash;

fn ast(stmts: Vec<BashStmt>) -> BashAst {
    BashAst {
        statements: stmts,
        metadata: AstMetadata { source_file: None, line_count: 0, parse_time_ms: 0 },
    }
}

fn cmd(name: &str, args: Vec<BashExpr>) -> BashStmt {
    BashStmt::Command { name: name.into(), args, redirects: vec![], span: Span::dummy() }
}

fn decl(name: &str, flags: &[&str], assigns: &[&str], redirects: Vec<Redirect>) -> BashStmt {
    let mut args: Vec<BashExpr> = flags.iter().map(|f| BashExpr::Literal(f.to_string())).collect();
    args.extend(assigns.iter().map(|a| BashExpr::Literal(a.to_string())));
    BashStmt::Command { name: name.into(), args, redirects, span: Span::dummy() }
}

fn gen(stmts: Vec<BashStmt>) -> String { generate_purified_bash(&ast(stmts)) }

// --- declare/typeset POSIX conversion ---

#[test]
fn test_CODEGEN_COV_001_declare_readonly() {
    let o = gen(vec![decl("declare", &["-r"], &["MAX=100"], vec![])]);
    assert!(o.contains("readonly") && o.contains("MAX=100"), "{o}");
}

#[test]
fn test_CODEGEN_COV_002_declare_export() {
    let o = gen(vec![decl("declare", &["-x"], &["PATH=/usr/bin"], vec![])]);
    assert!(o.contains("export") && o.contains("PATH"), "{o}");
}

#[test]
fn test_CODEGEN_COV_003_declare_readonly_export() {
    let o = gen(vec![decl("declare", &["-rx"], &["KEY=val"], vec![])]);
    assert!(o.contains("export") && o.contains("readonly") && o.contains("KEY=val"), "{o}");
}

#[test]
fn test_CODEGEN_COV_004_declare_array() {
    let o = gen(vec![decl("declare", &["-a"], &["arr"], vec![])]);
    assert!(o.contains("not POSIX"), "{o}");
}

#[test]
fn test_CODEGEN_COV_005_declare_assoc_array() {
    let o = gen(vec![decl("declare", &["-A"], &["hash"], vec![])]);
    assert!(o.contains("not POSIX"), "{o}");
}

#[test]
fn test_CODEGEN_COV_006_declare_plain() {
    let o = gen(vec![decl("declare", &[], &["x=42"], vec![])]);
    assert!(o.contains("x=42") && !o.contains("export") && !o.contains("readonly"), "{o}");
}

#[test]
fn test_CODEGEN_COV_007_declare_integer_flag() {
    let o = gen(vec![decl("declare", &["-i"], &["count=0"], vec![])]);
    assert!(o.contains("count=0"), "{o}");
}

#[test]
fn test_CODEGEN_COV_008_typeset_as_declare() {
    let o = gen(vec![decl("typeset", &["-r"], &["CONST=abc"], vec![])]);
    assert!(o.contains("readonly"), "{o}");
}

#[test]
fn test_CODEGEN_COV_009_declare_with_redirect() {
    let o = gen(vec![decl("declare", &["-r"], &["LOG=info"], vec![
        Redirect::Output { target: BashExpr::Literal("/dev/null".into()) },
    ])]);
    assert!(o.contains("readonly") && o.contains("> /dev/null"), "{o}");
}

#[test]
fn test_CODEGEN_COV_010_declare_rx_with_redirect() {
    let o = gen(vec![decl("declare", &["-rx"], &["CONF=yes"], vec![
        Redirect::Output { target: BashExpr::Literal("/dev/null".into()) },
    ])]);
    assert!(o.contains("export") && o.contains("readonly") && o.contains("> /dev/null"), "{o}");
}

#[test]
fn test_CODEGEN_COV_011_declare_array_with_assign() {
    let o = gen(vec![decl("declare", &["-a"], &["arr=(one two)"], vec![])]);
    assert!(o.contains("not POSIX"), "{o}");
}

// --- Select, Negated ---

#[test]
fn test_CODEGEN_COV_012_select_stmt() {
    let o = gen(vec![BashStmt::Select {
        variable: "opt".into(),
        items: BashExpr::Array(vec![BashExpr::Literal("yes".into()), BashExpr::Literal("no".into())]),
        body: vec![cmd("echo", vec![BashExpr::Variable("opt".into())])],
        span: Span::dummy(),
    }]);
    // select is now converted to POSIX while-loop menu
    assert!(o.contains("while") && o.contains("read REPLY") && o.contains("done"), "{o}");
}

#[test]
fn test_CODEGEN_COV_013_negated_command() {
    let o = gen(vec![BashStmt::Negated {
        command: Box::new(cmd("grep", vec![BashExpr::Literal("-q".into()), BashExpr::Literal("pat".into())])),
        span: Span::dummy(),
    }]);
    assert!(o.contains("! grep"), "{o}");
}

// --- Brace group: subshell vs non-subshell ---

#[test]
fn test_CODEGEN_COV_014_subshell() {
    let o = gen(vec![BashStmt::BraceGroup {
        body: vec![cmd("echo", vec![BashExpr::Literal("sub".into())]), cmd("pwd", vec![])],
        subshell: true, span: Span::dummy(),
    }]);
    assert!(o.contains('(') && o.contains(')'), "{o}");
}

#[test]
fn test_CODEGEN_COV_015_brace_group_multi() {
    let o = gen(vec![BashStmt::BraceGroup {
        body: vec![cmd("echo", vec![BashExpr::Literal("a".into())]), cmd("echo", vec![BashExpr::Literal("b".into())])],
        subshell: false, span: Span::dummy(),
    }]);
    assert!(o.contains("{ echo") && o.contains("; }"), "{o}");
}

// --- Literal quoting: shell keywords, empty, dollar with inner quotes ---

#[test]
fn test_CODEGEN_COV_016_keyword_quoted() {
    let o = gen(vec![cmd("echo", vec![BashExpr::Literal("if".into())])]);
    assert!(o.contains("\"if\""), "{o}");
}

#[test]
fn test_CODEGEN_COV_017_keyword_done_quoted() {
    let o = gen(vec![cmd("echo", vec![BashExpr::Literal("done".into())])]);
    assert!(o.contains("\"done\""), "{o}");
}

#[test]
fn test_CODEGEN_COV_018_empty_literal() {
    let o = gen(vec![cmd("echo", vec![BashExpr::Literal(String::new())])]);
    assert!(o.contains("echo ''"), "{o}");
}

#[test]
fn test_CODEGEN_COV_019_dollar_with_inner_quotes() {
    let o = gen(vec![cmd("echo", vec![BashExpr::Literal("$HOME says \"hi\"".into())])]);
    assert!(o.contains("\\\""), "{o}");
}

// --- Multi-elif with else ---

#[test]
fn test_CODEGEN_COV_020_multi_elif() {
    let o = gen(vec![BashStmt::If {
        condition: BashExpr::Test(Box::new(TestExpr::IntEq(BashExpr::Variable("x".into()), BashExpr::Literal("1".into())))),
        then_block: vec![cmd("echo", vec![BashExpr::Literal("one".into())])],
        elif_blocks: vec![
            (BashExpr::Test(Box::new(TestExpr::IntEq(BashExpr::Variable("x".into()), BashExpr::Literal("2".into())))),
             vec![cmd("echo", vec![BashExpr::Literal("two".into())])]),
            (BashExpr::Test(Box::new(TestExpr::IntEq(BashExpr::Variable("x".into()), BashExpr::Literal("3".into())))),
             vec![cmd("echo", vec![BashExpr::Literal("three".into())])]),
        ],
        else_block: Some(vec![cmd("echo", vec![BashExpr::Literal("other".into())])]),
        span: Span::dummy(),
    }]);
    assert_eq!(o.matches("elif").count(), 2, "{o}");
    assert!(o.contains("else") && o.contains("fi"), "{o}");
}

// --- Case with multi-pattern arm ---

#[test]
fn test_CODEGEN_COV_021_case_multi_pattern() {
    let o = gen(vec![BashStmt::Case {
        word: BashExpr::Variable("ext".into()),
        arms: vec![
            CaseArm { patterns: vec!["*.c".into(), "*.h".into()],
                       body: vec![cmd("echo", vec![BashExpr::Literal("C".into())])] },
            CaseArm { patterns: vec!["*".into()],
                       body: vec![cmd("echo", vec![BashExpr::Literal("other".into())])] },
        ],
        span: Span::dummy(),
    }]);
    assert!(o.contains("*.c|*.h)") && o.contains("esac"), "{o}");
}

// --- Pipeline 3 commands ---

#[test]
fn test_CODEGEN_COV_022_pipeline() {
    let o = gen(vec![BashStmt::Pipeline {
        commands: vec![
            cmd("cat", vec![BashExpr::Literal("f".into())]),
            cmd("sort", vec![]),
            cmd("uniq", vec![BashExpr::Literal("-c".into())]),
        ],
        span: Span::dummy(),
    }]);
    assert_eq!(o.matches(" | ").count(), 2, "{o}");
}

// --- Command with multiple redirects ---

#[test]
fn test_CODEGEN_COV_023_multi_redirects() {
    let o = gen(vec![BashStmt::Command {
        name: "cmd".into(), args: vec![], span: Span::dummy(),
        redirects: vec![
            Redirect::Output { target: BashExpr::Literal("out.log".into()) },
            Redirect::Error { target: BashExpr::Literal("err.log".into()) },
        ],
    }]);
    assert!(o.contains("> out.log") && o.contains("2> err.log"), "{o}");
}

// --- Until with non-test condition (exercises negate_condition non-test path) ---

#[test]
fn test_CODEGEN_COV_024_until_non_test() {
    let o = gen(vec![BashStmt::Until {
        condition: BashExpr::CommandCondition(Box::new(
            cmd("grep", vec![BashExpr::Literal("-q".into()), BashExpr::Literal("ready".into())])
        )),
        body: vec![cmd("sleep", vec![BashExpr::Literal("1".into())])],
        span: Span::dummy(),
    }]);
    assert!(o.contains("while ! grep"), "{o}");
}

// --- Return with/without code ---

#[test]
fn test_CODEGEN_COV_025_return_with_code() {
    let o = gen(vec![BashStmt::Return { code: Some(BashExpr::Literal("1".into())), span: Span::dummy() }]);
    assert!(o.contains("return 1"), "{o}");
}

#[test]
fn test_CODEGEN_COV_026_return_bare() {
    let o = gen(vec![BashStmt::Return { code: None, span: Span::dummy() }]);
    let last = o.trim().lines().last().unwrap_or("");
    assert_eq!(last.trim(), "return", "{o}");
}

// --- Function, while, and/or list, coproc ---

#[test]
fn test_CODEGEN_COV_027_function_multi_body() {
    let o = gen(vec![BashStmt::Function {
        name: "setup".into(), span: Span::dummy(),
        body: vec![
            cmd("mkdir", vec![BashExpr::Literal("-p".into()), BashExpr::Literal("/tmp/w".into())]),
            cmd("cd", vec![BashExpr::Literal("/tmp/w".into())]),
        ],
    }]);
    assert!(o.contains("setup()") && o.contains("    mkdir"), "{o}");
}

#[test]
fn test_CODEGEN_COV_028_while_test_condition() {
    let o = gen(vec![BashStmt::While {
        condition: BashExpr::Test(Box::new(TestExpr::IntLt(
            BashExpr::Variable("i".into()), BashExpr::Literal("10".into()),
        ))),
        body: vec![cmd("echo", vec![BashExpr::Variable("i".into())])],
        span: Span::dummy(),
    }]);
    assert!(o.contains("while [ \"$i\" -lt 10 ]"), "{o}");
}

#[test]
fn test_CODEGEN_COV_029_coproc_multi_body() {
    let o = gen(vec![BashStmt::Coproc {
        name: Some("BG".into()), span: Span::dummy(),
        body: vec![cmd("sleep", vec![BashExpr::Literal("1".into())]),
                   cmd("echo", vec![BashExpr::Literal("done".into())])],
    }]);
    assert!(o.contains("coproc BG {") && o.contains("; echo"), "{o}");
}

// --- Nested indentation (for inside if) ---

#[test]
fn test_CODEGEN_COV_030_nested_indent() {
    let o = gen(vec![BashStmt::If {
        condition: BashExpr::Test(Box::new(TestExpr::FileDirectory(BashExpr::Literal("/tmp".into())))),
        then_block: vec![BashStmt::For {
            variable: "f".into(), items: BashExpr::Glob("*.txt".into()),
            body: vec![cmd("echo", vec![BashExpr::Variable("f".into())])],
            span: Span::dummy(),
        }],
        elif_blocks: vec![], else_block: None, span: Span::dummy(),
    }]);
    assert!(o.contains("    for f in") && o.contains("        echo"), "{o}");
}

// --- Arithmetic expressions ---

#[test]
fn test_CODEGEN_COV_031_arithmetic_nested() {
    let o = gen(vec![BashStmt::Assignment {
        name: "r".into(), index: None, exported: false, span: Span::dummy(),
        value: BashExpr::Arithmetic(Box::new(ArithExpr::Mod(
            Box::new(ArithExpr::Mul(Box::new(ArithExpr::Number(6)), Box::new(ArithExpr::Number(7)))),
            Box::new(ArithExpr::Div(Box::new(ArithExpr::Number(10)), Box::new(ArithExpr::Number(3)))),
        ))),
    }]);
    assert!(o.contains("6 * 7") && o.contains("10 / 3") && o.contains('%'), "{o}");
}

// --- FileDirectory test ---

#[test]
fn test_CODEGEN_COV_032_file_directory_test() {
    let o = gen(vec![BashStmt::If {
        condition: BashExpr::Test(Box::new(TestExpr::FileDirectory(BashExpr::Literal("/tmp".into())))),
        then_block: vec![cmd("echo", vec![BashExpr::Literal("d".into())])],
        elif_blocks: vec![], else_block: None, span: Span::dummy(),
    }]);
    assert!(o.contains("-d /tmp"), "{o}");
}
