//! Integration tests for bash-to-rash transpiler

use super::*;
use crate::bash_parser::parser::BashParser;
use codegen::{BashToRashTranspiler, TranspileOptions};

#[test]
fn test_end_to_end_simple_script() {
    let bash_script = r#"
#!/bin/bash
FOO=bar
echo $FOO
"#;

    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("let FOO"));
    assert!(rash_code.contains("echo"));
}

#[test]
fn test_end_to_end_function() {
    let bash_script = r#"
function deploy() {
    echo "Deploying..."
}

deploy
"#;

    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("fn deploy()"));
    assert!(rash_code.contains("deploy()"));
}

#[test]
fn test_end_to_end_conditionals() {
    let bash_script = r#"
if [ $status == 0 ]; then
    echo "Success"
else
    echo "Failed"
fi
"#;

    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("if status == 0"));
    assert!(rash_code.contains("else"));
}
