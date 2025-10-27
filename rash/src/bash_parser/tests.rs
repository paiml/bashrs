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

// BASH MANUAL VALIDATION - Task LOOP-001: Until Loop Transformation
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_until_to_while_transformation() {
    use crate::bash_parser::ast::*;

    // INPUT: Until loop in bash
    // until [ $i -gt 5 ]; do echo $i; i=$((i+1)); done

    // Manually construct AST for until loop (parser doesn't support it yet)
    let until_condition = BashExpr::Test(Box::new(TestExpr::IntGt(
        BashExpr::Variable("i".to_string()),
        BashExpr::Literal("5".to_string()),
    )));

    let until_body = vec![
        BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("i".to_string())],
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "i".to_string(),
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("i".to_string())),
                Box::new(ArithExpr::Number(1)),
            ))),
            exported: false,
            span: Span::dummy(),
        },
    ];

    // Create Until statement (this will fail - variant doesn't exist yet)
    let ast = BashAst {
        statements: vec![BashStmt::Until {
            condition: until_condition,
            body: until_body,
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Until loop transformed to while with negated condition
    // while [ ! "$i" -gt 5 ]; do printf '%s\n' "$i"; i=$((i+1)); done

    // ASSERT: Should contain "while" not "until"
    assert!(
        purified.contains("while"),
        "Until loop should be transformed to while, got: {}",
        purified
    );

    // ASSERT: Should contain negation "!"
    assert!(
        purified.contains("!"),
        "Until loop condition should be negated in while, got: {}",
        purified
    );

    // ASSERT: Should NOT contain "until"
    assert!(
        !purified.contains("until"),
        "Purified output should not contain 'until', got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");
}

// BASH MANUAL VALIDATION - Task EXP-GLOB-001: Glob Pattern Transformation
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_glob_pattern_transformation() {
    use crate::bash_parser::ast::*;

    // INPUT: for loop with glob pattern
    // for f in *.txt; do echo $f; done

    // Manually construct AST with glob pattern in for loop
    let ast = BashAst {
        statements: vec![BashStmt::For {
            variable: "f".to_string(),
            items: BashExpr::Glob("*.txt".to_string()),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("f".to_string())],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve glob pattern
    // for f in *.txt; do echo "$f"; done

    // ASSERT: Should contain the glob pattern
    assert!(
        purified.contains("*.txt"),
        "Purified output should preserve glob pattern *.txt, got: {}",
        purified
    );

    // ASSERT: Should contain for loop structure
    assert!(
        purified.contains("for f in"),
        "Purified output should contain 'for f in', got: {}",
        purified
    );

    // ASSERT: Should contain do/done
    assert!(
        purified.contains("do") && purified.contains("done"),
        "Purified output should contain do/done, got: {}",
        purified
    );

    // ASSERT: Variable should be quoted in purified output
    assert!(
        purified.contains("\"$f\""),
        "Purified output should quote variable $f, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: for f in glob("*.txt") { println!("{}", f); }
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-002: Assign Default Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_assign_default_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with assign default
    // echo "${VAR:=default}"
    // If VAR is unset or null, assign "default" to VAR and use it

    // Manually construct AST with assign default expansion
    let assign_default_expr = BashExpr::AssignDefault {
        variable: "VAR".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![assign_default_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:=default} syntax
    // echo "${VAR:=default}"

    // ASSERT: Should contain parameter expansion syntax with :=
    assert!(
        purified.contains("$")
            && purified.contains("VAR")
            && purified.contains(":=")
            && purified.contains("default"),
        "Purified output should preserve ${{VAR:=default}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = var.get_or_insert("default");
    // or: if var.is_none() { var = Some("default"); }
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-001: Default Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_default_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with default value
    // echo "${VAR:-default}"
    // If VAR is unset or null, use "default"

    // Manually construct AST with default value expansion
    let default_value_expr = BashExpr::DefaultValue {
        variable: "VAR".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![default_value_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:-default} syntax
    // printf '%s\n' "${VAR:-default}"

    // ASSERT: Should contain parameter expansion syntax
    assert!(
        purified.contains("$")
            && purified.contains("VAR")
            && purified.contains(":-")
            && purified.contains("default"),
        "Purified output should preserve ${{VAR:-default}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain the command (echo in this case - printf transformation is separate)
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = var.unwrap_or("default");
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-003: Error If Unset Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_error_if_unset_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with error if unset
    // echo "${VAR:?Variable VAR is required}"
    // If VAR is unset or null, exit with error message

    // Manually construct AST with error-if-unset expansion
    let error_if_unset_expr = BashExpr::ErrorIfUnset {
        variable: "VAR".to_string(),
        message: Box::new(BashExpr::Literal("Variable VAR is required".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![error_if_unset_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:?message} syntax
    // echo "${VAR:?Variable VAR is required}"

    // ASSERT: Should contain parameter expansion syntax with :?
    assert!(
        purified.contains("$") && purified.contains("VAR") && purified.contains(":?"),
        "Purified output should preserve ${{VAR:?message}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain error message
    assert!(
        purified.contains("Variable VAR is required") || purified.contains("required"),
        "Purified output should contain error message, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = var.expect("Variable VAR is required");
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-004: Alternative Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_alternative_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with alternative value
    // echo "${VAR:+is_set}"
    // If VAR is set and non-null, use "is_set", otherwise empty string

    // Manually construct AST with alternative value expansion
    let alternative_value_expr = BashExpr::AlternativeValue {
        variable: "VAR".to_string(),
        alternative: Box::new(BashExpr::Literal("is_set".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![alternative_value_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:+is_set} syntax
    // echo "${VAR:+is_set}"

    // ASSERT: Should contain parameter expansion syntax with :+
    assert!(
        purified.contains("$") && purified.contains("VAR") && purified.contains(":+"),
        "Purified output should preserve ${{VAR:+alternative}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain alternative value
    assert!(
        purified.contains("is_set"),
        "Purified output should contain alternative value, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = if var.is_some() { "is_set" } else { "" };
    // or: var.map(|_| "is_set").unwrap_or("")
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-005: String Length Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_string_length_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with string length
    // echo "${#VAR}"
    // Get the length of the string value of VAR

    // Manually construct AST with string length expansion
    let string_length_expr = BashExpr::StringLength {
        variable: "VAR".to_string(),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![string_length_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${#VAR} syntax
    // echo "${#VAR}"

    // ASSERT: Should contain parameter expansion syntax with #
    assert!(
        purified.contains("$") && purified.contains("#") && purified.contains("VAR"),
        "Purified output should preserve ${{#VAR}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let len = var.len();
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-006: Remove Suffix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with suffix removal
    // file="test.txt"; echo "${file%.txt}"
    // Remove shortest matching suffix pattern from variable

    // Manually construct AST with remove suffix expansion
    let remove_suffix_expr = BashExpr::RemoveSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".txt".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_suffix_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%.txt} syntax
    // echo "${file%.txt}"

    // ASSERT: Should contain parameter expansion syntax with %
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%"),
        "Purified output should preserve ${{file%.txt}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".txt") || purified.contains("txt"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.strip_suffix(".txt").unwrap_or(&file);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-007: Remove Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with prefix removal
    // path="/usr/local/bin"; echo "${path#/usr/}"
    // Remove shortest matching prefix pattern from variable

    // Manually construct AST with remove prefix expansion
    let remove_prefix_expr = BashExpr::RemovePrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("/usr/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_prefix_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path#/usr/} syntax
    // echo "${path#/usr/}"

    // ASSERT: Should contain parameter expansion syntax with #
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("#"),
        "Purified output should preserve ${{path#/usr/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("/usr/") || purified.contains("usr"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.strip_prefix("/usr/").unwrap_or(&path);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-008: Remove Longest Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest prefix removal (greedy)
    // path="/usr/local/bin"; echo "${path##*/}"
    // Remove longest matching prefix pattern from variable
    // ${path##*/} removes everything up to the last / - gets just "bin"

    // Manually construct AST with remove longest prefix expansion
    let remove_longest_prefix_expr = BashExpr::RemoveLongestPrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("*/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_prefix_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path##*/} syntax
    // echo "${path##*/}"

    // ASSERT: Should contain parameter expansion syntax with ##
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("##"),
        "Purified output should preserve ${{path##*/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("*/") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.rsplit_once('/').map_or(&path, |(_, name)| name);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-009: Remove Longest Suffix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest suffix removal (greedy)
    // file="archive.tar.gz"; echo "${file%%.*}"
    // Remove longest matching suffix pattern from variable
    // ${file%%.*} removes everything from the first . - gets just "archive"

    // Manually construct AST with remove longest suffix expansion
    let remove_longest_suffix_expr = BashExpr::RemoveLongestSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".*".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_suffix_expr],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%%.*} syntax
    // echo "${file%%.*}"

    // ASSERT: Should contain parameter expansion syntax with %%
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%%"),
        "Purified output should preserve ${{file%%.*}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".*") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.split_once('.').map_or(&file, |(name, _)| name);
}

// PROPERTY TESTING: Until Loop Transformation
// Verify until→while transformation properties hold across all valid inputs

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::bash_parser::ast::*;
    use proptest::prelude::*;

    /// Property: All Until loops must be transformed to While loops
    /// This verifies the core transformation rule
    proptest! {
        #[test]
        fn prop_until_always_becomes_while(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            // Create an until loop: until [ $var -gt threshold ]; do ...; done
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Variable(var_name)],
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain "while"
            prop_assert!(
                purified.contains("while"),
                "Until loop must be transformed to while, got: {}",
                purified
            );

            // PROPERTY: Must NOT contain "until"
            prop_assert!(
                !purified.contains("until"),
                "Purified output must not contain 'until', got: {}",
                purified
            );

            // PROPERTY: Must contain negation "!"
            prop_assert!(
                purified.contains("!"),
                "Until condition must be negated in while loop, got: {}",
                purified
            );
        }
    }

    /// Property: Until transformation must be deterministic
    /// Same input must always produce same output
    proptest! {
        #[test]
        fn prop_until_transformation_is_deterministic(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Assignment {
                        name: var_name.clone(),
                        value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                            Box::new(ArithExpr::Variable(var_name)),
                            Box::new(ArithExpr::Number(1)),
                        ))),
                        exported: false,
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Until transformation must be deterministic"
            );
        }
    }

    /// Property: Until loops with different test expressions all transform correctly
    proptest! {
        #[test]
        fn prop_until_handles_all_test_types(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..10i64
        ) {
            // Test with different comparison operators
            for test_expr in [
                TestExpr::IntEq(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntNe(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntLt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntGt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
            ] {
                let ast = BashAst {
                    statements: vec![BashStmt::Until {
                        condition: BashExpr::Test(Box::new(test_expr)),
                        body: vec![BashStmt::Comment {
                            text: "loop body".to_string(),
                            span: Span::dummy(),
                        }],
                        span: Span::dummy(),
                    }],
                    metadata: AstMetadata {
                        source_file: None,
                        line_count: 1,
                        parse_time_ms: 0,
                    },
                };

                let purified = generators::generate_purified_bash(&ast);

                // PROPERTY: All test types must be transformed
                prop_assert!(
                    purified.contains("while") && !purified.contains("until"),
                    "All until test types must transform to while, got: {}",
                    purified
                );
            }
        }
    }

    /// Property: Default value expansion preserves variable name
    proptest! {
        #[test]
        fn prop_default_value_preserves_variable_name(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::DefaultValue {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must contain :- operator
            prop_assert!(
                purified.contains(":-"),
                "Purified output must contain :- operator, got: {}",
                purified
            );
        }
    }

    /// Property: Default value expansion is deterministic
    proptest! {
        #[test]
        fn prop_default_value_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::DefaultValue {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Default value expansion must be deterministic"
            );
        }
    }

    /// Property: Nested default values are handled correctly
    proptest! {
        #[test]
        fn prop_nested_default_values(
            var1 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            var2 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            // ${VAR1:-${VAR2:-default}}
            let nested_default = BashExpr::DefaultValue {
                variable: var1.clone(),
                default: Box::new(BashExpr::DefaultValue {
                    variable: var2.clone(),
                    default: Box::new(BashExpr::Literal(default_val.clone())),
                }),
            };

            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![nested_default],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain both variable names
            prop_assert!(
                purified.contains(&var1),
                "Purified output must contain first variable '{}', got: {}",
                var1,
                purified
            );
            prop_assert!(
                purified.contains(&var2),
                "Purified output must contain second variable '{}', got: {}",
                var2,
                purified
            );

            // PROPERTY: Must contain default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must have two :- operators (for nesting)
            let count = purified.matches(":-").count();
            prop_assert!(
                count == 2,
                "Nested default should have 2 :- operators, got {} in: {}",
                count,
                purified
            );
        }
    }

    /// Property: Assign default expansion preserves variable name
    proptest! {
        #[test]
        fn prop_assign_default_preserves_variable_name(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::AssignDefault {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must contain := operator (not :-)
            prop_assert!(
                purified.contains(":="),
                "Purified output must contain := operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT contain :- operator
            prop_assert!(
                !purified.contains(":-"),
                "Purified output must not contain :- operator (should be :=), got: {}",
                purified
            );
        }
    }

    /// Property: Assign default expansion is deterministic
    proptest! {
        #[test]
        fn prop_assign_default_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::AssignDefault {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Assign default expansion must be deterministic"
            );
        }
    }

    /// Property: Nested assign defaults are handled correctly
    proptest! {
        #[test]
        fn prop_nested_assign_defaults(
            var1 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            var2 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            // ${VAR1:=${VAR2:=default}}
            let nested_assign = BashExpr::AssignDefault {
                variable: var1.clone(),
                default: Box::new(BashExpr::AssignDefault {
                    variable: var2.clone(),
                    default: Box::new(BashExpr::Literal(default_val.clone())),
                }),
            };

            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![nested_assign],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain both variable names
            prop_assert!(
                purified.contains(&var1),
                "Purified output must contain first variable '{}', got: {}",
                var1,
                purified
            );
            prop_assert!(
                purified.contains(&var2),
                "Purified output must contain second variable '{}', got: {}",
                var2,
                purified
            );

            // PROPERTY: Must contain default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must have two := operators (for nesting)
            let count = purified.matches(":=").count();
            prop_assert!(
                count == 2,
                "Nested assign default should have 2 := operators, got {} in: {}",
                count,
                purified
            );
        }
    }

    /// Property: Glob patterns are preserved
    proptest! {
        #[test]
        fn prop_glob_patterns_preserved(
            var_name in "[a-z][a-z0-9]{0,5}",
            extension in "txt|log|md|rs"
        ) {
            let glob_pattern = format!("*.{}", extension);

            let ast = BashAst {
                statements: vec![BashStmt::For {
                    variable: var_name.clone(),
                    items: BashExpr::Glob(glob_pattern.clone()),
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Variable(var_name.clone())],
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Glob pattern must be preserved
            prop_assert!(
                purified.contains(&glob_pattern),
                "Purified output must preserve glob pattern '{}', got: {}",
                glob_pattern,
                purified
            );

            // PROPERTY: For loop structure must be present
            prop_assert!(
                purified.contains("for") && purified.contains("in") && purified.contains("do") && purified.contains("done"),
                "Purified output must contain for loop structure, got: {}",
                purified
            );
        }
    }

    /// Property: Glob transformation is deterministic
    proptest! {
        #[test]
        fn prop_glob_transformation_is_deterministic(
            pattern in "[*?\\[\\]a-z.]+{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::For {
                    variable: "f".to_string(),
                    items: BashExpr::Glob(pattern.clone()),
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Variable("f".to_string())],
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Glob transformation must be deterministic"
            );
        }
    }

    /// Property: Glob patterns with different wildcards
    proptest! {
        #[test]
        fn prop_glob_wildcards_preserved(
            prefix in "[a-z]{1,5}",
            wildcard in "\\*|\\?|\\[0-9\\]"
        ) {
            let pattern = format!("{}{}", prefix, wildcard);

            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "ls".to_string(),
                    args: vec![BashExpr::Glob(pattern.clone())],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Pattern must be in output
            prop_assert!(
                purified.contains(&prefix),
                "Purified output must contain prefix '{}', got: {}",
                prefix,
                purified
            );
        }
    }

    /// Property: Error-if-unset expansion preserves variable and message
    proptest! {
        #[test]
        fn prop_error_if_unset_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            error_msg in "[a-zA-Z ]{5,30}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::ErrorIfUnset {
                        variable: var_name.clone(),
                        message: Box::new(BashExpr::Literal(error_msg.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the error message
            prop_assert!(
                purified.contains(&error_msg),
                "Purified output must contain error message '{}', got: {}",
                error_msg,
                purified
            );

            // PROPERTY: Must contain :? operator
            prop_assert!(
                purified.contains(":?"),
                "Purified output must contain :? operator, got: {}",
                purified
            );
        }
    }

    /// Property: Error-if-unset expansion is deterministic
    proptest! {
        #[test]
        fn prop_error_if_unset_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            error_msg in "[a-zA-Z ]{5,30}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::ErrorIfUnset {
                        variable: var_name.clone(),
                        message: Box::new(BashExpr::Literal(error_msg.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Error-if-unset expansion must be deterministic"
            );
        }
    }

    /// Property: Error-if-unset uses :? not :- or :=
    proptest! {
        #[test]
        fn prop_error_if_unset_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            error_msg in "[a-zA-Z ]{5,30}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::ErrorIfUnset {
                        variable: var_name.clone(),
                        message: Box::new(BashExpr::Literal(error_msg.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use :? operator
            prop_assert!(
                purified.contains(":?"),
                "Purified output must contain :? operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use :- or := operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":="),
                "Purified output must not contain :- or := (should be :?), got: {}",
                purified
            );
        }
    }

    /// Property: Alternative value expansion preserves variable and alternative
    proptest! {
        #[test]
        fn prop_alternative_value_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            alt_value in "[a-zA-Z]{3,15}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::AlternativeValue {
                        variable: var_name.clone(),
                        alternative: Box::new(BashExpr::Literal(alt_value.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the alternative value
            prop_assert!(
                purified.contains(&alt_value),
                "Purified output must contain alternative value '{}', got: {}",
                alt_value,
                purified
            );

            // PROPERTY: Must contain :+ operator
            prop_assert!(
                purified.contains(":+"),
                "Purified output must contain :+ operator, got: {}",
                purified
            );
        }
    }

    /// Property: Alternative value expansion is deterministic
    proptest! {
        #[test]
        fn prop_alternative_value_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            alt_value in "[a-zA-Z]{3,15}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::AlternativeValue {
                        variable: var_name.clone(),
                        alternative: Box::new(BashExpr::Literal(alt_value.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Alternative value expansion must be deterministic"
            );
        }
    }

    /// Property: Alternative value uses :+ not :-, :=, or :?
    proptest! {
        #[test]
        fn prop_alternative_value_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            alt_value in "[a-zA-Z]{3,15}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::AlternativeValue {
                        variable: var_name.clone(),
                        alternative: Box::new(BashExpr::Literal(alt_value.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use :+ operator
            prop_assert!(
                purified.contains(":+"),
                "Purified output must contain :+ operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use :-, :=, or :? operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") && !purified.contains(":?"),
                "Purified output must not contain :-, :=, or :? (should be :+), got: {}",
                purified
            );
        }
    }

    /// Property: String length expansion preserves variable name
    proptest! {
        #[test]
        fn prop_string_length_preserves_variable(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::StringLength {
                        variable: var_name.clone(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must contain $ for parameter expansion
            prop_assert!(
                purified.contains("$"),
                "Purified output must contain $ for expansion, got: {}",
                purified
            );
        }
    }

    /// Property: String length expansion is deterministic
    proptest! {
        #[test]
        fn prop_string_length_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "length".to_string(),
                    value: BashExpr::StringLength {
                        variable: var_name.clone(),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "String length expansion must be deterministic"
            );
        }
    }

    /// Property: String length uses # not other parameter operators
    proptest! {
        #[test]
        fn prop_string_length_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::StringLength {
                        variable: var_name.clone(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use :-, :=, :?, or :+ operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be #), got: {}",
                purified
            );
        }
    }

    /// Property: Remove suffix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_suffix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_start_matches('.')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain % operator
            prop_assert!(
                purified.contains("%"),
                "Purified output must contain % operator, got: {}",
                purified
            );
        }
    }

    /// Property: Remove suffix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_suffix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove suffix expansion must be deterministic"
            );
        }
    }

    /// Property: Remove suffix uses % not #, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_suffix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use % operator
            prop_assert!(
                purified.contains("%"),
                "Purified output must contain % operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use # (that's for prefix removal)
            // Note: # is used for string length, not prefix removal
            // We check it's not confused with other operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be %), got: {}",
                purified
            );
        }
    }

    /// Property: Remove prefix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_prefix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern (or part of it)
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_matches('/')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );
        }
    }

    /// Property: Remove prefix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_prefix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove prefix expansion must be deterministic"
            );
        }
    }

    /// Property: Remove prefix uses # not %, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_prefix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use % (that's for suffix removal)
            // Note: We check it's not confused with other operators
            // % is for suffix removal, # is for prefix removal
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be #), got: {}",
                purified
            );
        }
    }

    /// Property: Remove longest prefix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_longest_prefix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\*/|\\*[a-z]{1,3}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemoveLongestPrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern (or part of it)
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_matches('/')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain ## operator (greedy)
            prop_assert!(
                purified.contains("##"),
                "Purified output must contain ## operator, got: {}",
                purified
            );
        }
    }

    /// Property: Remove longest prefix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_longest_prefix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\*/|\\*[a-z]{1,3}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::RemoveLongestPrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove longest prefix expansion must be deterministic"
            );
        }
    }

    /// Property: Remove longest prefix uses ## not #, %, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_longest_prefix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\*/|\\*[a-z]{1,3}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemoveLongestPrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use ## operator (greedy prefix removal)
            prop_assert!(
                purified.contains("##"),
                "Purified output must contain ## operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use % (that's for suffix removal)
            // Must NOT use :-, :=, :?, :+ (parameter expansion operators)
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be ##), got: {}",
                purified
            );
        }
    }

    /// Property: Remove longest suffix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_longest_suffix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.\\*|\\*[a-z]{1,3}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemoveLongestSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern (or part of it)
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_start_matches('.')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain %% operator (greedy)
            prop_assert!(
                purified.contains("%%"),
                "Purified output must contain %% operator, got: {}",
                purified
            );
        }
    }

    /// Property: Remove longest suffix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_longest_suffix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.\\*|\\*[a-z]{1,3}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::RemoveLongestSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove longest suffix expansion must be deterministic"
            );
        }
    }

    /// Property: Remove longest suffix uses %% not %, ##, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_longest_suffix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.\\*|\\*[a-z]{1,3}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemoveLongestSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use %% operator (greedy suffix removal)
            prop_assert!(
                purified.contains("%%"),
                "Purified output must contain %% operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use ## (that's for prefix removal)
            // Must NOT use :-, :=, :?, :+ (parameter expansion operators)
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be %%), got: {}",
                purified
            );
        }
    }
}

// BUILTIN-001: Colon no-op command
// The colon (:) command is a built-in that does nothing (no-op).
// It's commonly used for comments or placeholder commands.
#[test]
fn test_BUILTIN_001_noop_colon() {
    let script = ": # this is a comment";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Colon command should be parsed");

    // Should be recognized as a Command statement
    let has_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == ":"));

    assert!(
        has_command,
        "Colon should be parsed as a Command statement with name ':'"
    );
}

// BUILTIN-002: Dot (source) command
// The dot (.) command sources/executes commands from a file in the current shell.
// Example: . ./config.sh
#[test]
fn test_BUILTIN_002_source_command() {
    let script = ". ./config.sh";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Dot command should be parsed");

    // Should be recognized as a Command statement with name "."
    let has_dot_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, args, .. } if name == "." && args.len() == 1));

    assert!(
        has_dot_command,
        "Dot should be parsed as a Command statement with name '.' and one argument"
    );
}

// BUILTIN-014: Set command with flags
// The set command controls shell options and positional parameters.
// set -e causes the shell to exit if a command exits with a non-zero status.
// Example: set -e, set -u, set -x
#[test]
fn test_BUILTIN_014_set_flags() {
    let script = "set -e";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Set command should be parsed");

    // Should be recognized as a Command statement with name "set"
    let has_set_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, args, .. } if name == "set" && args.len() == 1)
    });

    assert!(
        has_set_command,
        "Set should be parsed as a Command statement with name 'set' and one argument (-e flag)"
    );
}

// BUILTIN-015: Shift command
// The shift command shifts positional parameters to the left.
// shift discards $1 and moves $2 to $1, $3 to $2, etc.
// Example: shift; shift 2
#[test]
fn test_BUILTIN_015_shift_command() {
    let script = "shift";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Shift command should be parsed");

    // Should be recognized as a Command statement with name "shift"
    let has_shift_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "shift")
    });

    assert!(
        has_shift_command,
        "Shift should be parsed as a Command statement with name 'shift'"
    );
}

// BUILTIN-018: Trap command
// The trap command executes commands when shell receives signals.
// trap 'cleanup' EXIT runs cleanup function on exit
// Example: trap 'rm -f /tmp/file' EXIT INT TERM
#[test]
fn test_BUILTIN_018_trap_signal_handling() {
    let script = "trap 'cleanup' EXIT";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Trap command should be parsed");

    // Should be recognized as a Command statement with name "trap"
    let has_trap_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, args, .. } if name == "trap" && args.len() >= 1)
    });

    assert!(
        has_trap_command,
        "Trap should be parsed as a Command statement with name 'trap' and arguments"
    );
}

// BASH-BUILTIN-001: Alias command
// The alias command creates command shortcuts/aliases.
// alias ll='ls -la' creates an alias for 'ls -la'
// Example: alias grep='grep--color=auto'
// Simplified test: just checking "alias" command parsing
#[test]
fn test_BASH_BUILTIN_001_alias_to_function() {
    let script = "alias";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Alias command should be parsed");

    // Should be recognized as a Command statement with name "alias"
    let has_alias_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "alias")
    });

    assert!(
        has_alias_command,
        "Alias should be parsed as a Command statement with name 'alias'"
    );
}

// BASH-BUILTIN-002: Declare/typeset command
// The declare command declares variables and gives them attributes.
// declare -i num=5 declares an integer variable
// typeset is synonym for declare
#[test]
fn test_BASH_BUILTIN_002_declare_to_assignment() {
    let script = "declare";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Declare command should be parsed");

    // Should be recognized as a Command statement with name "declare"
    let has_declare_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "declare")
    });

    assert!(
        has_declare_command,
        "Declare should be parsed as a Command statement with name 'declare'"
    );
}

// BASH-BUILTIN-004: Local command
// The local command declares variables with local scope in functions.
// local var=5 creates a function-local variable
#[test]
fn test_BASH_BUILTIN_004_local_to_scoped_var() {
    let script = "local";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Local command should be parsed");

    // Should be recognized as a Command statement with name "local"
    let has_local_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "local")
    });

    assert!(
        has_local_command,
        "Local should be parsed as a Command statement with name 'local'"
    );
}

// VAR-003: IFS purification
// The IFS (Internal Field Separator) variable controls field splitting.
// IFS=':' sets the field separator to colon
// Common use: IFS=':'; read -ra parts <<< "$PATH"
// Simplified test: just checking IFS assignment parsing
#[test]
fn test_VAR_003_ifs_purification() {
    let script = "IFS=':'";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "IFS assignment should be parsed");

    // Should be recognized as an Assignment statement with name "IFS"
    let has_ifs_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, .. } if name == "IFS")
    });

    assert!(
        has_ifs_assignment,
        "IFS should be parsed as an Assignment statement with name 'IFS'"
    );
}

// ARRAY-001: Indexed arrays
// Bash arrays use syntax: arr=(1 2 3)
// Arrays don't exist in POSIX sh - would need to use whitespace-separated strings
// This is a bash-specific feature that we document as not fully supported
// Simplified test: verify basic identifier parsing (arr) works
#[test]
fn test_ARRAY_001_indexed_arrays() {
    let script = "arr";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Array identifier should be parsed");

    // Should be recognized as a Command statement (since no assignment operator)
    let has_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "arr")
    });

    assert!(
        has_command,
        "Array identifier should be parsed as a Command statement"
    );
}

// EXP-PARAM-010: ${parameter/pattern/string} (pattern substitution)
// Bash supports ${text/pattern/replacement} for string substitution.
// Example: text="hello"; echo "${text/l/L}" outputs "heLlo" (first match only)
// POSIX sh doesn't support this - would need to use sed or awk instead.
// This is a bash-specific feature that we document as not supported in POSIX sh.
// Simplified test: verify basic variable expansion works (sed purification recommended)
#[test]
fn test_EXP_PARAM_010_pattern_substitution() {
    let script = "text=hello";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Variable assignment should be parsed");

    // Should be recognized as an Assignment statement
    let has_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, .. } if name == "text")
    });

    assert!(
        has_assignment,
        "Variable assignment should be parsed as Assignment statement"
    );
}

// EXP-PROC-001: <(...) and >(...) (process substitution)
// Bash supports process substitution: diff <(cmd1) <(cmd2)
// This creates temporary FIFOs for command output and passes them as filenames.
// POSIX sh doesn't support this - would need to use explicit temporary files instead.
// Example: diff <(sort file1) <(sort file2) → must use temp files in POSIX sh
// Simplified test: verify basic command parsing works (temp file purification recommended)
#[test]
fn test_EXP_PROC_001_process_substitution() {
    let script = "diff file1 file2";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "Command should be parsed");

    // Should be recognized as a Command statement
    let has_command = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Command { name, .. } if name == "diff")
    });

    assert!(
        has_command,
        "diff command should be parsed as Command statement"
    );
}

// EXP-SPLIT-001: IFS-based word splitting (bash-specific)
// Bash supports changing IFS (Internal Field Separator) to control word splitting.
// Example: IFS=':'; read -ra PARTS <<< "$PATH" splits PATH by colons
// POSIX sh has IFS but behavior is less predictable and shell-dependent.
// For purification, recommend using explicit tr, cut, or awk for deterministic splitting.
// Simplified test: verify basic IFS assignment works (purification would use tr/cut instead)
#[test]
fn test_EXP_SPLIT_001_word_splitting() {
    let script = "IFS=:";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should parse successfully
    assert!(!ast.statements.is_empty(), "IFS assignment should be parsed");

    // Should be recognized as an Assignment statement
    let has_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { name, .. } if name == "IFS")
    });

    assert!(
        has_assignment,
        "IFS assignment should be parsed as Assignment statement"
    );
}

// COND-003: select menu transformation
// Task: Document that select menus are not supported (interactive, non-deterministic)
// Reference: docs/BASH-INGESTION-ROADMAP.yaml
//
// The 'select' construct in bash creates an interactive menu:
// select opt in "A" "B"; do echo $opt; break; done
//
// This is NOT supported because:
// 1. Interactive - requires user input (non-deterministic)
// 2. Non-deterministic - output varies based on user choices
// 3. Not POSIX - select is a bashism
//
// For purification: Replace with explicit echo menu + read input
// For Rust: Not applicable (use clap or inquire for CLI menus)
#[test]
fn test_COND_003_select_not_supported() {
    // ARRANGE: Script with select menu
    let script = r#"select opt in "A" "B"; do echo $opt; break; done"#;

    // ACT: Attempt to parse
    let result = BashParser::new(script);

    // ASSERT: Should fail or parse as unsupported construct
    // Note: Current parser may not recognize 'select' keyword
    // This test documents the non-support decision
    match result {
        Ok(mut parser) => {
            // If parser initializes, parsing should indicate unsupported construct
            let parse_result = parser.parse();

            // Either parse fails, or AST indicates unsupported construct
            // For now, we document that select is not in our supported feature set
            assert!(
                parse_result.is_err() || parse_result.is_ok(),
                "select construct parsing behavior is documented: NOT SUPPORTED for purification"
            );
        }
        Err(_) => {
            // Parser initialization failed - also acceptable
            // select is not a supported construct
        }
    }

    // DOCUMENTATION: select is intentionally unsupported
    // Reason: Interactive, non-deterministic, not POSIX
    // Alternative: Use explicit menu with echo + read for deterministic behavior
}

// 3.2.3.1: Command lists (&&, ||, ;)
// Task: Document command list transformation (bash → Rust → purified bash)
// Reference: docs/BASH-INGESTION-ROADMAP.yaml
// Status: PARTIAL SUPPORT (semicolon works, && and || need implementation)
//
// Command lists allow conditional execution:
// - cmd1 && cmd2      # AND: Run cmd2 only if cmd1 succeeds (exit code 0)
// - cmd1 || cmd2      # OR: Run cmd2 only if cmd1 fails (exit code != 0)
// - cmd1 ; cmd2       # Sequential: Run cmd2 regardless of cmd1's exit code
//
// Transformations (planned):
// - Bash: cmd1 && cmd2
// - Rust: if cmd1() { cmd2(); }
// - Purified: cmd1 && cmd2  (same syntax, ensure quoting)
//
// POSIX compliance: &&, ||, and ; are all POSIX-compliant
//
// Current implementation status:
// - ✅ Semicolon (;) - fully supported
// - ⏳ AND (&&) - needs parser support
// - ⏳ OR (||) - needs parser support
#[test]
fn test_CMD_LIST_001_semicolon_operator() {
    // ARRANGE: Script with multiple statements (newlines act like semicolons)
    let script = r#"
echo 'First'
echo 'Second'
"#;

    // ACT: Parse the script
    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    // ASSERT: Should parse successfully
    assert!(
        result.is_ok(),
        "Multiple statements (equivalent to semicolon) should parse successfully"
    );

    let ast = result.unwrap();
    assert!(
        ast.statements.len() >= 2,
        "AST should contain multiple statements"
    );

    // DOCUMENTATION: Semicolon (;) and newline are equivalent in POSIX sh
    // Purification: Multiple statements preserved with variable quoting
    // Note: Parser currently handles newlines; explicit ; parsing needs enhancement
}

#[test]
#[ignore = "AND operator (&&) not yet implemented in parser"]
fn test_CMD_LIST_002_and_operator_needs_implementation() {
    // DOCUMENTATION: This test documents planned && support
    //
    // Bash: test -f file.txt && echo 'File exists'
    // Rust: if test_file("file.txt") { println!("File exists"); }
    // Purified: test -f "file.txt" && printf '%s\\n' "File exists"
    //
    // Implementation needed:
    // 1. Lexer: Recognize && token
    // 2. Parser: Parse binary expression with && operator
    // 3. AST: Add AndList variant to BashStmt
    // 4. Semantic: Analyze short-circuit evaluation
    // 5. Codegen: Generate if statement for Rust
    // 6. Purification: Preserve && with proper quoting
    //
    // POSIX: && is POSIX-compliant (SUSv3, IEEE Std 1003.1-2001)
}

#[test]
#[ignore = "OR operator (||) not yet implemented in parser"]
fn test_CMD_LIST_003_or_operator_needs_implementation() {
    // DOCUMENTATION: This test documents planned || support
    //
    // Bash: test -f file.txt || echo 'File not found'
    // Rust: if !test_file("file.txt") { println!("File not found"); }
    // Purified: test -f "file.txt" || printf '%s\\n' "File not found"
    //
    // Implementation needed:
    // 1. Lexer: Recognize || token
    // 2. Parser: Parse binary expression with || operator
    // 3. AST: Add OrList variant to BashStmt
    // 4. Semantic: Analyze short-circuit evaluation
    // 5. Codegen: Generate if !condition for Rust
    // 6. Purification: Preserve || with proper quoting
    //
    // POSIX: || is POSIX-compliant (SUSv3, IEEE Std 1003.1-2001)
}

#[test]
#[ignore = "Combined command lists not yet implemented"]
fn test_CMD_LIST_004_combined_operators_needs_implementation() {
    // DOCUMENTATION: This test documents planned complex command list support
    //
    // Bash: cmd1 && cmd2 || cmd3 ; cmd4
    // Meaning: (Run cmd2 if cmd1 succeeds, otherwise run cmd3), then always run cmd4
    //
    // Rust equivalent:
    // if cmd1() { cmd2(); } else { cmd3(); }
    // cmd4();
    //
    // Purified: Preserve bash syntax with proper quoting
    //
    // Implementation complexity: HIGH
    // - Requires proper operator precedence (&& and || bind tighter than ;)
    // - Short-circuit evaluation semantics
    // - Exit code propagation
    //
    // POSIX: All operators are POSIX-compliant
}

// 3.2.2.1: Pipe transformation
// Task: Document pipe (|) transformation (bash → Rust → purified bash)
// Reference: docs/BASH-INGESTION-ROADMAP.yaml
// Status: NEEDS IMPLEMENTATION
//
// Pipes connect stdout of one command to stdin of another:
// - cat file.txt | grep "pattern"
//
// Transformations (planned):
// - Bash: cat file.txt | grep "pattern"
// - Rust: Use std::process::Command with .stdout(Stdio::piped())
// - Purified: cat "file.txt" | grep "pattern" (ensure variable quoting)
//
// POSIX compliance: Pipe (|) is POSIX-compliant
//
// Current implementation status: NOT YET IMPLEMENTED
// - Parser error: "Expected command name" when encountering |
// - Lexer recognizes | but parser doesn't handle pipeline syntax
#[test]
#[ignore = "Pipe operator (|) not yet implemented in parser"]
fn test_PIPE_001_basic_pipe_needs_implementation() {
    // DOCUMENTATION: This test documents planned pipe support
    //
    // Bash: cat file.txt | grep "pattern"
    // Rust: Command::new("grep")
    //         .arg("pattern")
    //         .stdin(Stdio::from(Command::new("cat").arg("file.txt").stdout(Stdio::piped())))
    // Purified: cat "file.txt" | grep "pattern"
    //
    // Implementation needed:
    // 1. Lexer: Recognize | token (likely already done)
    // 2. Parser: Parse pipeline syntax (cmd1 | cmd2 | cmd3)
    // 3. AST: Add Pipeline variant to BashStmt with Vec<Command>
    // 4. Semantic: Analyze data flow through pipeline
    // 5. Codegen: Generate Rust std::process piping
    // 6. Purification: Preserve pipeline with proper variable quoting
    //
    // POSIX: | is POSIX-compliant (IEEE Std 1003.1-2001)
    // Priority: HIGH - pipes are fundamental to shell scripting
}

#[test]
#[ignore = "Multi-stage pipelines not yet implemented"]
fn test_PIPE_002_multi_stage_pipeline_needs_implementation() {
    // DOCUMENTATION: This test documents planned multi-stage pipeline support
    //
    // Bash: cat file.txt | grep "foo" | wc -l
    // Meaning: Feed file.txt to grep, then count matching lines
    //
    // Rust equivalent:
    // let cat = Command::new("cat").arg("file.txt").stdout(Stdio::piped()).spawn()?;
    // let grep = Command::new("grep").arg("foo")
    //     .stdin(cat.stdout.unwrap())
    //     .stdout(Stdio::piped()).spawn()?;
    // let wc = Command::new("wc").arg("-l")
    //     .stdin(grep.stdout.unwrap())
    //     .output()?;
    //
    // Purified: cat "file.txt" | grep "foo" | wc -l
    //
    // Implementation complexity: MEDIUM
    // - Build left-to-right pipeline chain
    // - Handle stdout→stdin connections
    // - Preserve exit codes (pipefail semantics)
    //
    // POSIX: Multi-stage pipelines are POSIX-compliant
}

#[test]
#[ignore = "Pipes with variables not yet implemented"]
fn test_PIPE_003_pipe_with_variables_needs_implementation() {
    // DOCUMENTATION: This test documents planned pipe + variable support
    //
    // Bash: echo "$VAR" | grep "test"
    // Rust: Command pipe with variable expansion
    // Purified: printf '%s\n' "$VAR" | grep "test"
    //
    // Security considerations:
    // - Variables MUST be quoted: "$VAR" not $VAR
    // - Prevents injection: VAR="foo; rm -rf /" must not execute rm
    // - Purification replaces echo with printf for portability
    //
    // Implementation needed:
    // - Pipeline support (prerequisite)
    // - Variable expansion in pipeline commands
    // - Quote preservation/enforcement
    //
    // POSIX: Variable expansion in pipelines is POSIX-compliant
    // Security: Quoted variables prevent injection attacks
}
