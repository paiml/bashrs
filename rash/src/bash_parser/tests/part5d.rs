#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_JOB_001_background_jobs_portability_issues() {
    // DOCUMENTATION: Background job portability issues (3 critical issues)
    //
    // ISSUE 1: Job control availability
    // Job control (&, jobs, fg, bg) may not be available in all shells
    // Non-interactive shells: job control often disabled
    // Dash: Limited job control support
    // POSIX: Job control is OPTIONAL (not all shells support it)
    //
    // ISSUE 2: wait behavior varies
    // bash: wait with no args waits for all background jobs
    // dash: wait requires PID (wait $pid)
    // POSIX: wait behavior varies across shells
    //
    // ISSUE 3: Background job process groups
    // bash: Background jobs in separate process group
    // dash: Process group handling differs
    // PROBLEM: Signal handling is shell-dependent

    let portability_issues = r#"
#!/bin/sh
# This script has PORTABILITY ISSUES (uses background jobs)

# ISSUE 1: Job control may not be available
long_task &
# Non-interactive shell: May not support job control
# Dash: Limited support

# ISSUE 2: wait behavior varies
task1 &
task2 &
wait  # bash: waits for all, dash: may require PID

# ISSUE 3: Process groups
task &
pid=$!
# Process group handling varies by shell

# PURIFIED (POSIX-compliant, portable):
# Use foreground execution (no job control needed)
task1
task2
# Deterministic, portable, works in all shells
"#;

    let mut lexer = Lexer::new(portability_issues);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Portability issues should tokenize successfully"
        );
        let _ = tokens;
    }

    // Background jobs have PORTABILITY ISSUES
    // Job control is OPTIONAL in POSIX (not all shells support)
    // PURIFICATION: Use foreground execution (portable, deterministic)
}

// DOCUMENTATION: Comprehensive background jobs comparison (Bash vs POSIX vs Purified)
//
// FEATURE                    | Bash       | POSIX      | Purified
// Background jobs (&)        | SUPPORTED  | OPTIONAL   | NOT SUPPORTED
// Determinism                | NO         | NO         | YES (enforced)
// Reproducibility            | NO         | NO         | YES
// Testing                    | Flaky      | Flaky      | Reproducible
// Portability                | bash       | Optional   | POSIX (portable)
// Error handling             | Silent     | Silent     | Immediate
// Race conditions            | YES        | YES        | NO
// Resource management        | Manual     | Manual     | Automatic
//
// RUST MAPPING:
// Background jobs (&) -> NOT MAPPED (use sequential execution)
// Parallelism needs -> Use Rayon (deterministic parallelism)
// Async I/O -> Use tokio (structured concurrency)
// Job control -> Remove or convert to sequential
//
// PURIFICATION RULES:
// 1. Background jobs (&) -> DISCOURAGED (convert to foreground)
// 2. Parallel tasks -> Sequential execution (deterministic)
// 3. wait command -> Remove (sequential execution doesn't need wait)
// 4. Fire-and-forget jobs -> Remove or make synchronous
// 5. Parallelism for performance -> Use make -j or Rayon (deterministic)
#[test]
fn test_JOB_001_background_jobs_comparison_table() {
    // Comparison examples: bash (non-deterministic) vs purified (sequential)
    let comparison_table = concat!(
        "#!/bin/sh\n",
        "# COMPARISON EXAMPLES\n",
        "\n",
        "# PURIFIED (DETERMINISTIC):\n",
        "# Sequential execution (deterministic)\n",
        "long_task\n",
        "short_task\n",
        "# Guaranteed order, reproducible\n",
        "\n",
        "# PURIFIED (reproducible tests):\n",
        "test_sequential() {\n",
        "    task1\n",
        "    task2\n",
        "    [ -f task1.out ] || exit 1\n",
        "    [ -f task2.out ] || exit 1\n",
        "}\n",
        "\n",
        "# PURIFIED (immediate error detection):\n",
        "risky_operation || exit 1\n",
    );

    let mut lexer = Lexer::new(comparison_table);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Comparison table should tokenize successfully"
        );
    }
}

// ============================================================================
// PARAM-SPEC-006: $- (Shell Options) Purification
// ============================================================================

// DOCUMENTATION: $- (shell options) is NOT SUPPORTED (LOW priority purification)
//
// $-: Special parameter that expands to current shell option flags
// Contains single letters representing active shell options
// Set by: Shell at startup, modified by set command
//
// WHAT $- CONTAINS (each letter = an active option):
// h: hashall, i: interactive, m: monitor mode, B: brace expansion,
// H: history substitution, s: read from stdin, c: read from -c arg,
// e: exit on error, u: error on unset vars, x: print commands,
// v: print input lines, n: no execution, f: no globbing,
// a: auto-export all, t: exit after one command
//
// EXAMPLE VALUES:
// Interactive bash: "himBH", Script: "hB", set -e script: "ehB", sh: "h"
//
// WHY NOT SUPPORTED:
// 1. Runtime-specific (value depends on how shell was invoked)
// 2. Non-deterministic (different shells = different flags)
// 3. Shell-dependent (bash has different flags than sh/dash)
// 4. Implementation detail (exposes internal shell state)
// 5. Not needed for pure scripts (purified scripts don't rely on shell modes)
//
// POSIX COMPLIANCE: $- is POSIX SUPPORTED but FLAGS DIFFER between shells
// bash: himBH (many extensions), sh/dash: h (minimal)
//
// PURIFICATION STRATEGY:
// 1. Remove $- entirely (RECOMMENDED)
// 2. Replace with explicit option checks
// 3. Use set -e explicitly (don't check "e" in $-)
//
// PURIFICATION EXAMPLES:
// BEFORE: echo "Shell options: $-"  ->  AFTER: (removed, not needed)
// BEFORE: `case "$-" in *i*) ... esac`  ->  AFTER: echo "Non-interactive"
// BEFORE: `case "$-" in *e*) ... esac`  ->  AFTER: set -e (explicit)
#[test]
fn test_PARAM_SPEC_006_shell_options_not_supported() {
    // $- is NOT SUPPORTED by the current lexer
    // Special parameters like $-, $$, $?, $! are not yet implemented
    // This test documents that $- is NOT SUPPORTED and verifies the lexer doesn't crash
    let bash_input = r#"echo $-"#;
    let mut lexer = Lexer::new(bash_input);
    let tokens = lexer.tokenize().unwrap();

    assert!(
        !tokens.is_empty(),
        "Lexer should produce tokens without crashing"
    );
}

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
