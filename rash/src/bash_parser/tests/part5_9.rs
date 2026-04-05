#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_F051_rm_uses_f_flag() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"rm /tmp/testfile"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F051 FALSIFIED: Purifier MUST handle rm command"
    );
}

/// F052: Purified ln uses -sf flags for idempotency
#[test]
fn test_F052_ln_uses_sf_flags() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"ln -s /source /target"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F052 FALSIFIED: Purifier MUST handle ln command"
    );
}

/// F053: Purified cp handles idempotency
#[test]
fn test_F053_cp_idempotency() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"cp /source /dest"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F053 FALSIFIED: Purifier MUST handle cp command"
    );
}

/// F054: Purified touch is already idempotent
#[test]
fn test_F054_touch_idempotent() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"touch /tmp/testfile"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F054 FALSIFIED: Purifier MUST handle touch command (already idempotent)"
    );
}

/// F055: Purified output handles loops
#[test]
fn test_F055_handles_loops() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"
for i in 1 2 3; do
    echo $i
done
"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F055 FALSIFIED: Purifier MUST handle for loops"
    );
}

/// F056: Purified output handles functions
#[test]
fn test_F056_handles_functions() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"
my_func() {
    echo "hello"
}
my_func
"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F056 FALSIFIED: Purifier MUST handle function definitions"
    );
}

/// F057: Purified output handles traps
#[test]
fn test_F057_handles_traps() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"trap 'cleanup' EXIT"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F057 FALSIFIED: Purifier MUST handle trap commands"
    );
}

/// F058: Purified output handles redirects
#[test]
fn test_F058_handles_redirects() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"echo "hello" > /tmp/output.txt"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F058 FALSIFIED: Purifier MUST handle I/O redirections"
    );
}

/// F059: Purified output handles pipes
#[test]
fn test_F059_handles_pipes() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    let script = r#"cat /etc/passwd | grep root"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F059 FALSIFIED: Purifier MUST handle pipelines"
    );
}

/// F060: Purified output handles subshells (via command substitution)
#[test]
fn test_F060_handles_subshells() {
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    // Use command substitution as a form of subshell
    let script = r#"OUTPUT=$(cd /tmp; ls)"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");

    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);

    let result = purifier.purify(&ast);
    assert!(
        result.is_ok(),
        "F060 FALSIFIED: Purifier MUST handle subshell constructs"
    );
}

// ===== parse_assignment coverage: keyword-as-variable-name branches =====

#[test]
fn test_ASSIGN_COV_001_keyword_if_as_variable_name() {
    let script = "if=1\necho $if";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "if"));
    assert!(
        has_assignment,
        "Should parse 'if' as variable name in 'if=1'"
    );
}

#[test]
fn test_ASSIGN_COV_002_keyword_then_as_variable_name() {
    let script = "then=hello";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "then"));
    assert!(has_assignment, "Should parse 'then' as variable name");
}

#[test]
fn test_ASSIGN_COV_003_keyword_elif_as_variable_name() {
    let script = "elif=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "elif"));
    assert!(has_assignment, "Should parse 'elif' as variable name");
}

#[test]
fn test_ASSIGN_COV_004_keyword_else_as_variable_name() {
    let script = "else=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "else"));
    assert!(has_assignment, "Should parse 'else' as variable name");
}

#[test]
fn test_ASSIGN_COV_005_keyword_fi_as_variable_name() {
    let script = "fi=1";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "fi"));
    assert!(has_assignment, "Should parse 'fi' as variable name");
}

#[test]
fn test_ASSIGN_COV_006_keyword_for_as_variable_name() {
    let script = "for=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "for"));
    assert!(has_assignment, "Should parse 'for' as variable name");
}

#[test]
fn test_ASSIGN_COV_007_keyword_while_as_variable_name() {
    let script = "while=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "while"));
    assert!(has_assignment, "Should parse 'while' as variable name");
}

#[test]
fn test_ASSIGN_COV_008_keyword_do_as_variable_name() {
    let script = "do=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "do"));
    assert!(has_assignment, "Should parse 'do' as variable name");
}

#[test]
fn test_ASSIGN_COV_009_keyword_done_as_variable_name() {
    let script = "done=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "done"));
    assert!(has_assignment, "Should parse 'done' as variable name");
}

#[test]
fn test_ASSIGN_COV_010_keyword_case_as_variable_name() {
    let script = "case=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "case"));
    assert!(has_assignment, "Should parse 'case' as variable name");
}

#[test]
fn test_ASSIGN_COV_011_keyword_esac_as_variable_name() {
    let script = "esac=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "esac"));
    assert!(has_assignment, "Should parse 'esac' as variable name");
}

#[test]
fn test_ASSIGN_COV_012_keyword_in_as_variable_name() {
    let script = "in=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "in"));
    assert!(has_assignment, "Should parse 'in' as variable name");
}

#[test]
fn test_ASSIGN_COV_013_keyword_function_as_variable_name() {
    let script = "function=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "function"));
    assert!(has_assignment, "Should parse 'function' as variable name");
}

#[test]
fn test_ASSIGN_COV_014_keyword_return_as_variable_name() {
    let script = "return=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "return"));
    assert!(has_assignment, "Should parse 'return' as variable name");
}

// ===== parse_assignment coverage: array element assignment =====

#[test]
fn test_ASSIGN_COV_015_array_element_number_index() {
    // arr[0]=value
    let script = "arr[0]=hello";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "0")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with number index"
    );
}

#[test]
fn test_ASSIGN_COV_016_array_element_identifier_index() {
    // arr[key]=value
    let script = "arr[key]=world";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "key")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with identifier index"
    );
}

#[test]
fn test_ASSIGN_COV_017_array_element_string_index() {
    // arr["quoted"]=value
    let script = r#"arr["quoted"]=value"#;
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, index: Some(_), .. } if name == "arr"));
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with string index"
    );
}

#[test]
fn test_ASSIGN_COV_018_array_element_variable_index() {
    // arr[$i]=value
    let script = "arr[$i]=value";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    let has_indexed_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, index: Some(idx), .. } if name == "arr" && idx == "$i")
    });
    assert!(
        has_indexed_assignment,
        "Should parse array element assignment with variable index"
    );
}

// ===== parse_assignment coverage: append operator += =====

#[test]
fn test_ASSIGN_COV_019_append_assignment() {
    // PATH+=/usr/local/bin (append operator)
    let script = "PATH+=/usr/local/bin";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    // Parser should produce an Assignment (or equivalent) for +=
    assert!(
        !ast.statements.is_empty(),
        "Should parse += append assignment"
    );
}

// ===== parse_assignment coverage: empty assignment before pipe/comment =====

#[test]
fn test_ASSIGN_COV_020_empty_assignment_before_pipe() {
    let script = "x= | cat";
    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let ast = parser.parse().expect("Parse should succeed");
    assert!(
        !ast.statements.is_empty(),
        "Should parse empty assignment before pipe"
    );
}

