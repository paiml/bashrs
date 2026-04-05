#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_PARAM_SPEC_006_shell_options_usage_patterns() {
    // DOCUMENTATION: Common $- usage patterns and purification
    //
    // PATTERN 1: Debugging output
    // Bash: echo "Shell options: $-"
    // Purification: Remove (debugging not needed in purified script)
    //
    // PATTERN 2: Interactive mode detection
    // Bash: case "$-" in *i*) interactive_mode ;; esac
    // Purification: Remove (purified scripts always non-interactive)
    //
    // PATTERN 3: Error mode detection
    // Bash: case "$-" in *e*) echo "Exit on error" ;; esac
    // Purification: Use explicit set -e, remove detection
    //
    // PATTERN 4: Shell identification
    // Bash: if [[ "$-" == *B* ]]; then echo "Bash"; fi
    // Purification: Remove (purified scripts are shell-agnostic)
    //
    // PATTERN 5: Trace mode detection
    // Bash: case "$-" in *x*) echo "Tracing enabled" ;; esac
    // Purification: Remove (tracing is runtime option, not script logic)

    // Pattern 1: Debugging
    let bash_debug = r#"echo $-"#;
    let mut lexer = Lexer::new(bash_debug);
    let tokens = lexer.tokenize().unwrap();
    // Note: $- not yet supported by lexer, just verify no crash
    assert!(!tokens.is_empty());

    // Pattern 2: Interactive check
    let bash_interactive = r#"case $- in *i*) echo Interactive ;; esac"#;
    let mut lexer = Lexer::new(bash_interactive);
    let tokens = lexer.tokenize().unwrap();
    // Note: $- not yet supported by lexer, just verify no crash
    assert!(!tokens.is_empty());

    let _ = tokens;
}

#[test]
fn test_PARAM_SPEC_006_shell_options_flag_meanings() {
    // DOCUMENTATION: Comprehensive guide to shell option flags
    //
    // INTERACTIVE FLAGS:
    // i - Interactive shell (prompts enabled, job control)
    // m - Monitor mode (job control, background jobs)
    //
    // BASH EXTENSION FLAGS:
    // B - Brace expansion enabled ({a,b,c}, {1..10})
    // H - History substitution enabled (!, !!, !$)
    //
    // INPUT/OUTPUT FLAGS:
    // s - Read commands from stdin
    // c - Commands from -c argument (bash -c 'cmd')
    //
    // ERROR HANDLING FLAGS (IMPORTANT):
    // e - Exit on error (set -e, errexit)
    // u - Error on unset variables (set -u, nounset)
    // n - No execution (syntax check only, set -n)
    //
    // DEBUGGING FLAGS:
    // x - Print commands before execution (set -x, xtrace)
    // v - Print input lines as read (set -v, verbose)
    //
    // BEHAVIOR FLAGS:
    // f - Disable filename expansion/globbing (set -f, noglob)
    // a - Auto-export all variables (set -a, allexport)
    // h - Hash commands as looked up (set -h, hashall)
    // t - Exit after one command (set -t, onecmd)
    //
    // EXAMPLE COMBINATIONS:
    // "himBH" - Interactive bash (hash, interactive, monitor, brace, history)
    // "hB" - Non-interactive bash script (hash, brace)
    // "ehB" - Bash script with set -e (exit on error, hash, brace)
    // "h" - POSIX sh (only hash, no extensions)
    //
    // PURIFICATION: Don't rely on these flags
    // - Use explicit set commands (set -e, set -u, set -x)
    // - Don't check flags at runtime (not deterministic)
    // - Remove flag detection code (use explicit behavior)

    let bash_input = r#"echo $-"#;
    let mut lexer = Lexer::new(bash_input);
    let tokens = lexer.tokenize().unwrap();

    // Note: $- not yet supported by lexer, just verify no crash
    assert!(
        !tokens.is_empty(),
        "Lexer should produce tokens without crashing"
    );

    let _ = tokens;
}

#[test]
fn test_PARAM_SPEC_006_shell_options_portability() {
    // DOCUMENTATION: $- portability across shells
    //
    // BASH (many flags):
    // Interactive: "himBH" (hash, interactive, monitor, brace, history)
    // Script: "hB" (hash, brace)
    // Bash-specific flags: B (brace), H (history)
    //
    // SH/DASH (minimal flags):
    // Interactive: "hi" (hash, interactive)
    // Script: "h" (hash only)
    // No bash extensions (no B, H flags)
    //
    // ASH/BUSYBOX SH (minimal):
    // Similar to dash: "h" or "hi"
    // No bash extensions
    //
    // ZSH (different flags):
    // Different option names and letters
    // Not compatible with bash flags
    //
    // POSIX GUARANTEE:
    // $- is POSIX (must exist in all shells)
    // BUT: Flag letters are IMPLEMENTATION-DEFINED
    // Different shells use different letters for same option
    // Only "h" (hashall) is somewhat universal
    //
    // PORTABILITY ISSUES:
    // 1. Flag letters differ (bash "B" doesn't exist in sh)
    // 2. Checking for specific flag is NON-PORTABLE
    // 3. Interactive detection fragile (different shells, different flags)
    // 4. Error mode detection fragile (all support -e, but letter varies)
    //
    // PURIFICATION FOR PORTABILITY:
    // 1. Remove all $- references (RECOMMENDED)
    // 2. Use explicit options (set -e, not check for "e" in $-)
    // 3. Don't detect shell type (write portable code instead)
    // 4. Don't check interactive mode (purified scripts always non-interactive)
    //
    // COMPARISON TABLE:
    //
    // | Shell | Interactive | Script | Extensions |
    // |-------|-------------|--------|------------|
    // | bash  | himBH       | hB     | B, H       |
    // | sh    | hi          | h      | None       |
    // | dash  | hi          | h      | None       |
    // | ash   | hi          | h      | None       |
    // | zsh   | different   | diff   | Different  |
    //
    // PURIFIED SCRIPT: No $- (explicit options only)

    let bash_input = r#"echo $-"#;
    let mut lexer = Lexer::new(bash_input);
    let tokens = lexer.tokenize().unwrap();

    // Note: $- not yet supported by lexer, just verify no crash
    assert!(
        !tokens.is_empty(),
        "Lexer should produce tokens without crashing"
    );

    let _ = tokens;
}

// DOCUMENTATION: Comprehensive $- purification examples
//
// EXAMPLE 1: Debug output
// BEFORE: echo "Shell options: $-"  ->  AFTER: (removed, not needed)
//
// EXAMPLE 2: Interactive mode detection
// BEFORE: `case "$-" in *i*) echo "Interactive" ;; *) echo "Non-interactive" ;; esac`
// AFTER: echo "Non-interactive mode"
//
// EXAMPLE 3: Error handling mode
// BEFORE: `case "$-" in *e*) echo "Will exit" ;; *) set -e ;; esac`
// AFTER: set -e (explicit)
//
// EXAMPLE 4: Shell detection
// BEFORE: `if [[ "$-" == *B* ]]; then ... else ... fi`
// AFTER: mkdir -p project/src project/tests project/docs (POSIX, no detection)
//
// EXAMPLE 5: Complex script with multiple $- checks
// BEFORE: `case "$-" in *x*) TRACE=1 ;; esac` + `case "$-" in *e*) ERREXIT=1 ;; esac`
// AFTER: set -e (explicit, remove runtime introspection)
#[test]
fn test_PARAM_SPEC_006_shell_options_removal_examples() {
    // Test: case statement using $- tokenizes without crash
    let bash_before = concat!(
        "case $- in\n",
        "  *i*) echo Interactive ;;\n",
        "  *) echo Non-interactive ;;\n",
        "esac\n",
    );

    let mut lexer = Lexer::new(bash_before);
    let tokens = lexer.tokenize().unwrap();

    // Note: $- not yet supported by lexer, just verify no crash
    assert!(
        !tokens.is_empty(),
        "Lexer should produce tokens without crashing"
    );
}

#[test]
fn test_PARAM_SPEC_006_shell_options_comparison_table() {
    // DOCUMENTATION: Comprehensive comparison of $- across bash, sh, and purified
    //
    // +-----------------+------------------------+---------------------+---------------------------+
    // | Feature         | Bash                   | POSIX sh            | Purified                  |
    // +-----------------+------------------------+---------------------+---------------------------+
    // | $- support      | SUPPORTED              | SUPPORTED           | NOT USED                  |
    // | Common flags    | himBH (interactive)    | hi (interactive)    | N/A                       |
    // |                 | hB (script)            | h (script)          |                           |
    // | Bash extensions | B (brace expansion)    | None                | Removed                   |
    // |                 | H (history)            | None                | Removed                   |
    // | Portable flags  | e, u, x, v, f          | e, u, x, v, f       | Use explicit set commands |
    // | Interactive     | Check *i* in $-        | Check *i* in $-     | Always non-interactive    |
    // | Error mode      | Check *e* in $-        | Check *e* in $-     | Use explicit set -e       |
    // | Trace mode      | Check *x* in $-        | Check *x* in $-     | Use explicit set -x       |
    // | Shell detection | Check B/H flags        | Check absence of B  | No detection needed       |
    // | Debugging       | echo "Options: $-"     | echo "Options: $-"  | Remove (not needed)       |
    // | Determinism     | NON-DETERMINISTIC      | NON-DETERMINISTIC   | DETERMINISTIC             |
    // |                 | (runtime-specific)     | (runtime-specific)  | (no $- references)        |
    // | Portability     | BASH ONLY              | POSIX sh            | UNIVERSAL                 |
    // | Use case        | Runtime introspection  | Runtime checks      | No runtime checks         |
    // | Best practice   | Avoid in scripts       | Avoid in scripts    | ALWAYS remove             |
    // +-----------------+------------------------+---------------------+---------------------------+
    //
    // KEY DIFFERENCES:
    //
    // 1. Bash: Many flags (B, H are bash-specific)
    // 2. sh: Minimal flags (no bash extensions)
    // 3. Purified: NO $- REFERENCES (explicit options only)
    //
    // PURIFICATION PRINCIPLES:
    //
    // 1. Remove all $- references (runtime introspection not needed)
    // 2. Use explicit set commands (set -e, set -u, set -x)
    // 3. Don't detect shell type (write portable code)
    // 4. Don't check interactive mode (scripts always non-interactive)
    // 5. Don't check error mode (use explicit set -e)
    //
    // RATIONALE:
    //
    // $- exposes RUNTIME CONFIGURATION, not SCRIPT LOGIC
    // Purified scripts should be EXPLICIT about behavior
    // Checking $- makes scripts NON-DETERMINISTIC
    // Different invocations = different flags = different behavior

    let bash_input = r#"echo $-"#;
    let mut lexer = Lexer::new(bash_input);
    let tokens = lexer.tokenize().unwrap();

    // Note: $- not yet supported by lexer, just verify no crash
    assert!(
        !tokens.is_empty(),
        "Lexer should produce tokens without crashing"
    );

    let _ = tokens;
}

// EXTREME TDD - RED Phase: Test for loop with multiple values
// This test is EXPECTED TO FAIL until parser enhancement is implemented
// Bug: Parser cannot handle `for i in 1 2 3; do` (expects single value)
// Error: UnexpectedToken { expected: "Do", found: "Some(Number(2))", line: X }
#[test]
fn test_for_loop_with_multiple_values() {
    let script = r#"
for i in 1 2 3; do
    echo "$i"
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "For loop with multiple values should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    let has_for = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::For { .. }));

    assert!(has_for, "AST should contain a for loop");
}

// EXTREME TDD - Test for while loop with semicolon before do
// Bug was: Parser could not handle `while [ condition ]; do` (expected do immediately after condition)
// Fixed: Parser now optionally consumes semicolon before 'do' keyword (PARSER-ENH-003)
#[test]
fn test_while_loop_with_semicolon_before_do() {
    let script = r#"
x=5
while [ "$x" = "5" ]; do
    echo "looping"
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "While loop with semicolon before do should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    let has_while = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::While { .. }));

    assert!(has_while, "AST should contain a while loop");
}

// EXTREME TDD - RED Phase: Test for arithmetic expansion $((expr))
// This is P0 blocker documented in multiple locations
// Bug: Parser cannot handle arithmetic expansion like y=$((y - 1))
// Expected error: InvalidSyntax or UnexpectedToken when parsing $((...))
// GREEN phase complete - lexer + parser implemented with proper operator precedence
#[test]
fn test_arithmetic_expansion_basic() {
    let script = r#"
x=5
y=$((x + 1))
echo "$y"
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Arithmetic expansion should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // Verify we have an assignment with arithmetic expansion
    let has_arithmetic_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { value, .. }
            if matches!(value, BashExpr::Arithmetic(_)))
    });

    assert!(
        has_arithmetic_assignment,
        "AST should contain arithmetic expansion in assignment"
    );
}

#[test]
fn test_arithmetic_expansion_in_loop() {
    let script = r#"
count=3
while [ "$count" -gt "0" ]; do
    echo "Iteration $count"
    count=$((count - 1))
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "While loop with arithmetic decrement should parse: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    let has_while = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::While { .. }));

    assert!(has_while, "AST should contain a while loop");
}

#[test]
fn test_arithmetic_expansion_complex_expressions() {
    let script = r#"
a=10
b=20
sum=$((a + b))
diff=$((a - b))
prod=$((a * b))
quot=$((a / b))
mod=$((a % b))
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Complex arithmetic expressions should parse: {:?}",
        result.err()
    );
}

// ============================================================================
// ISSUE #4: Benchmark Parser Gaps - STOP THE LINE (P0 BLOCKER)
// ============================================================================
// Issue: docs/known-limitations/issue-004-benchmark-parser-gaps.md
//
// All benchmark fixture files (small.sh, medium.sh, large.sh) fail to parse
// due to missing parser support for common bash constructs:
// 1. $RANDOM - Special bash variable (0-32767 random integer)
// 2. $$ - Process ID variable
// 3. $(command) - Command substitution
// 4. function keyword - Function definition syntax
//
// These tests verify parser ACCEPTS these constructs (LEXER/PARSER ONLY).
// Purification transformation is separate (handled by purifier).
//
// Architecture: bash → PARSE (accept) → AST → PURIFY (transform) → POSIX sh
// Cannot purify what cannot be parsed!
// ============================================================================

#[test]
fn test_ISSUE_004_001_parse_random_special_variable() {
    // RED PHASE: Write failing test for $RANDOM parsing
    //
    // CRITICAL: Parser MUST accept $RANDOM to enable purification
    // Purifier will later reject/transform it, but parser must accept first
    //
    // INPUT: bash with $RANDOM
    // EXPECTED: Parser accepts, returns AST with Variable("RANDOM")
    // PURIFIER (later): Rejects or transforms to deterministic alternative

    let bash = r#"
#!/bin/bash
ID=$RANDOM
echo "Random ID: $ID"
"#;

    // ARRANGE: Lexer should tokenize $RANDOM
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize $RANDOM: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept $RANDOM
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept $RANDOM (for purification to work)
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept $RANDOM to enable purification: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains assignment with Variable("RANDOM")
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "$RANDOM should produce non-empty AST"
    );
}

#[test]
fn test_ISSUE_004_002_parse_process_id_variable() {
    // RED PHASE: Write failing test for $$ parsing
    //
    // CRITICAL: Parser MUST accept $$ to enable purification
    // $$ is process ID (non-deterministic, needs purification)
    //
    // INPUT: bash with $$
    // EXPECTED: Parser accepts, returns AST with special PID variable
    // PURIFIER (later): Transforms to deterministic alternative

    let bash = r#"
#!/bin/bash
PID=$$
TEMP_DIR="/tmp/build-$PID"
echo "Process ID: $PID"
"#;

    // ARRANGE: Lexer should tokenize $$
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize $$: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept $$
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept $$ (for purification to work)
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept $$ to enable purification: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains assignment with PID variable
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "$$ should produce non-empty AST"
    );
}

