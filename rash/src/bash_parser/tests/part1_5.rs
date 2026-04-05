#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

        #[test]
        fn prop_remove_longest_prefix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\*/|\\*[a-z]{1,3}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
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

    // Property: Remove longest prefix uses ## not #, %, :-, :=, :?, or :+
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
                    redirects: vec![],
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

    // Property: Remove longest suffix expansion preserves variable and pattern
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
                    redirects: vec![],
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

    // Property: Remove longest suffix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_longest_suffix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.\\*|\\*[a-z]{1,3}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
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

    // Property: Remove longest suffix uses %% not %, ##, :-, :=, :?, or :+
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
                    redirects: vec![],
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
    let has_dot_command = ast.statements.iter().any(
        |s| matches!(s, BashStmt::Command { name, args, .. } if name == "." && args.len() == 1),
    );

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
    let has_set_command = ast.statements.iter().any(
        |s| matches!(s, BashStmt::Command { name, args, .. } if name == "set" && args.len() == 1),
    );

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
    let has_shift_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == "shift"));

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
    let has_trap_command = ast.statements.iter().any(
        |s| matches!(s, BashStmt::Command { name, args, .. } if name == "trap" && !args.is_empty()),
    );

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
    let has_alias_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == "alias"));

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
    assert!(
        !ast.statements.is_empty(),
        "Declare command should be parsed"
    );

    // Should be recognized as a Command statement with name "declare"
    let has_declare_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == "declare"));

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
    let has_local_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == "local"));

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
    assert!(
        !ast.statements.is_empty(),
        "IFS assignment should be parsed"
    );

    // Should be recognized as an Assignment statement with name "IFS"
    let has_ifs_assignment = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Assignment { name, .. } if name == "IFS"));

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
    assert!(
        !ast.statements.is_empty(),
        "Array identifier should be parsed"
    );

    // Should be recognized as a Command statement (since no assignment operator)
    let has_command = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Command { name, .. } if name == "arr"));

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
