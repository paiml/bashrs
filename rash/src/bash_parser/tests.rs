//! Integration tests for bash parser

use super::*;
use parser::BashParser;
use semantic::SemanticAnalyzer;

#[test]
fn test_parse_and_analyze_simple_script() {
    let script = r#"
#!/bin/bash
FOO=bar
echo $FOO
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    assert!(!ast.statements.is_empty());

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(report.scope_info.variables.contains_key("FOO"));
}

#[test]
fn test_parse_function_definition() {
    let script = r#"
function greet() {
    echo "Hello, World!"
}

greet
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_function = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Function { .. }));

    assert!(has_function);
}

#[test]
fn test_parse_if_statement() {
    let script = r#"
if [ $x == 1 ]; then
    echo "one"
elif [ $x == 2 ]; then
    echo "two"
else
    echo "other"
fi
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_if = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. }));

    assert!(has_if);
}

#[test]
fn test_parse_for_loop() {
    let script = r#"
for file in *.txt; do
    echo $file
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_for = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::For { .. }));

    assert!(has_for);
}

#[test]
fn test_semantic_analysis_detects_exports() {
    let script = "export PATH=/usr/bin";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(report.effects.env_modifications.contains("PATH"));
}

#[test]
fn test_semantic_analysis_detects_file_operations() {
    let script = "cat /etc/passwd";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(!report.effects.file_reads.is_empty());
}

// BASH MANUAL VALIDATION - Task 1.1: Shebang Transformation
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_shebang_transformation() {
    // INPUT: Bash script with bash shebang
    let bash_script = "#!/bin/bash\necho 'Hello'";

    // Parse bash
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // ASSERT: Shebang should be transformed to POSIX sh
    assert!(
        purified.starts_with("#!/bin/sh"),
        "Purified bash must use POSIX sh shebang, got: {}",
        purified.lines().next().unwrap_or("")
    );

    // PROPERTY: Purified output must be deterministic
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");
}
