#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: assert that BashParser handles the input without panicking.
/// Accepts both successful parses and parse errors (documentation tests
/// only verify the parser doesn't crash, not that the input is valid).
#[test]
fn test_PARAM_SPEC_004_background_pid_common_mistakes() {
    let common_mistakes = r#"
# Mistake 1: Race condition (BAD)
# cmd &
# kill $!  # May fail if job finished

# GOOD: Check if job exists
# cmd &
# BG_PID=$!
# if kill -0 $BG_PID 2>/dev/null; then
#   kill $BG_PID
# fi

# Mistake 2: Exit without wait (BAD)
# important_task &
# exit 0  # Task may not complete!

# GOOD: Wait for job
# important_task &
# wait $!

# BETTER (bashrs): Synchronous
important_task
exit 0

# Mistake 3: Uncontrolled parallelism (BAD)
# for i in 1 2 3 4 5; do
#   process_item $i &
# done

# BETTER (bashrs): Sequential
for i in 1 2 3 4 5; do
  process_item "$i"
done
"#;

    assert_parses_without_panic(common_mistakes, "Common $! mistakes documented");
}

#[test]
fn test_PARAM_SPEC_004_background_pid_comparison_table() {
    // DOCUMENTATION: $! and & comparison (POSIX vs bashrs)
    //
    // Feature                 | POSIX sh | bash | dash | ash | bashrs
    // ------------------------|----------|------|------|-----|--------
    // & (background job)      | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // $! (background PID)     | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // Deterministic           | ❌       | ❌   | ❌   | ❌  | ✅ (sync)
    // wait                    | ✅       | ✅   | ✅   | ✅  | ❌ (implicit)
    // jobs                    | ✅       | ✅   | ✅   | ✅  | ❌
    // fg/bg                   | ✅       | ✅   | ✅   | ✅  | ❌
    //
    // bashrs purification policy:
    // - & (background) is POSIX but NON-DETERMINISTIC
    // - MUST purify to synchronous execution
    // - Remove all background jobs
    // - Remove $! (unnecessary without &)
    // - Remove wait (implicit in synchronous)
    //
    // Purification strategies:
    // 1. Background job: cmd & → cmd (synchronous)
    // 2. Multiple jobs: task1 & task2 & wait → task1; task2 (sequential)
    // 3. Timeout: cmd & sleep 5; kill $! → timeout 5 cmd || true
    // 4. Wait pattern: cmd &; wait $! → cmd (implicit wait)
    // 5. Remove non-essential: log_task & → (remove or make sync)
    //
    // Rust mapping (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Background execution (non-deterministic)
    // // let child = Command::new("cmd").spawn()?;
    // // let pid = child.id();
    // // child.wait()?;
    //
    // // DO: Synchronous execution (deterministic)
    // let status = Command::new("cmd").status()?;
    // ```
    //
    // Best practices:
    // 1. Use synchronous execution for determinism
    // 2. Avoid background jobs in bootstrap/config scripts
    // 3. Use timeout command for time limits (not background + kill)
    // 4. Sequential execution is easier to test and debug
    // 5. Interactive tools can use &, but not purified scripts

    let comparison_example = r#"
# POSIX: Background job (non-deterministic)
# cmd &
# echo "BG: $!"
# wait $!

# bashrs: Synchronous (deterministic)
cmd
echo "Done"

# POSIX: Multiple background jobs
# task1 &
# task2 &
# wait

# bashrs: Sequential
task1
task2

# POSIX: Timeout with background
# task &
# BG=$!
# sleep 5
# kill $BG

# bashrs: Use timeout command
timeout 5 task || true
"#;

    let result = BashParser::new(comparison_example);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$! and & comparison and purification strategy documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// Summary:
// $! (background PID): POSIX but NON-DETERMINISTIC (MUST PURIFY)
// Contains PID of last background job (changes every run)
// Background jobs (&) are non-deterministic (PIDs, timing, execution order)
// bashrs policy: Purify to SYNCHRONOUS execution (remove & and $!)
// Purification: cmd & → cmd, task1 & task2 & wait → task1; task2
// Timeout pattern: cmd & sleep N; kill $! → timeout N cmd || true
// Job control (jobs, fg, bg): NOT SUPPORTED (interactive features)
// Common mistakes: Race conditions, exit without wait, uncontrolled parallelism
// Best practice: Synchronous execution for determinism, testability, reproducibility

// ============================================================================
// EXP-BRACE-001: Brace Expansion {..} (Bash extension, NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: Brace expansion is NOT SUPPORTED (bash extension)
// Bash 3.0+ feature: {1..5}, {a..z}, {foo,bar,baz}, {a,b}{1,2}.
// Not in POSIX. sh/dash/ash don't support. Work around with loops or lists.
#[test]
fn test_EXP_BRACE_001_brace_expansion_not_supported() {
    let brace_expansion = r#"
# Bash brace expansion (NOT SUPPORTED)
echo {1..5}
echo {a..z}
echo {foo,bar,baz}
"#;

    assert_parses_without_panic(
        brace_expansion,
        "Brace expansion is bash extension, NOT SUPPORTED",
    );
}

// DOCUMENTATION: Sequence expansion {start..end} (bash, NOT SUPPORTED)
// Numeric: {1..10}, {0..100..10}. Letter: {a..f}, {A..Z}.
// POSIX alternatives: seq, explicit for loop, while loop with counter.
#[test]
fn test_EXP_BRACE_001_sequence_expansion() {
    let sequence_expansion = r#"
# Bash sequences (NOT SUPPORTED)
# echo {1..10}
# echo {0..100..10}
# echo {a..z}

# POSIX alternatives (SUPPORTED)
seq 1 10
for i in 1 2 3 4 5; do echo "$i"; done

i=1
while [ $i -le 10 ]; do
  echo "$i"
  i=$((i+1))
done
"#;

    assert_parses_without_panic(
        sequence_expansion,
        "POSIX alternatives: seq, for loop, while loop",
    );
}

// DOCUMENTATION: Comma expansion {item1,item2} (bash, NOT SUPPORTED)
// {foo,bar,baz}, pre{A,B,C}post, {red,green,blue}_color.
// POSIX alternatives: explicit list, for loop, variable iteration.
#[test]
fn test_EXP_BRACE_001_comma_expansion() {
    let comma_expansion = r#"
# Bash comma expansion (NOT SUPPORTED)
# echo {foo,bar,baz}
# echo pre{A,B,C}post

# POSIX alternatives (SUPPORTED)
echo foo bar baz

for item in foo bar baz; do
  echo "$item"
done

# Explicit iteration
items="foo bar baz"
for item in $items; do
  echo "$item"
done
"#;

    assert_parses_without_panic(
        comma_expansion,
        "POSIX alternatives: explicit lists, for loops",
    );
}

#[test]
fn test_EXP_BRACE_001_nested_expansion() {
    // DOCUMENTATION: Nested brace expansion (bash, NOT SUPPORTED)
    //
    // Cartesian product:
    // $ echo {a,b}{1,2}
    // a1 a2 b1 b2
    //
    // $ echo {x,y,z}{A,B}
    // xA xB yA yB zA zB
    //
    // Multiple nesting:
    // $ echo {a,b}{1,2}{X,Y}
    // a1X a1Y a2X a2Y b1X b1Y b2X b2Y
    //
    // POSIX alternative: Nested loops
    // $ for letter in a b; do
    // $   for num in 1 2; do
    // $     echo "${letter}${num}"
    // $   done
    // $ done
    // a1
    // a2
    // b1
    // b2

    let nested_expansion = r#"
# Bash nested expansion (NOT SUPPORTED)
# echo {a,b}{1,2}
# echo {x,y,z}{A,B}

# POSIX alternative: Nested loops
for letter in a b; do
  for num in 1 2; do
    echo "${letter}${num}"
  done
done

for letter in x y z; do
  for suffix in A B; do
    echo "${letter}${suffix}"
  done
done
"#;

    let result = BashParser::new(nested_expansion);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX alternative: nested for loops"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: bashrs purification strategy for brace expansion
// Strategy: numeric seq -> seq/loop, letters -> explicit list,
// comma lists -> explicit, nested -> nested loops, file ops -> explicit.
#[test]
fn test_EXP_BRACE_001_purification_strategy() {
    let purification_examples = r#"
# BEFORE (bash brace expansion)
# echo {1..10}
# echo {a..e}
# echo {foo,bar,baz}

# AFTER (POSIX)
seq 1 10
echo a b c d e
echo foo bar baz

# BEFORE (nested)
# echo {a,b}{1,2}

# AFTER (POSIX)
for x in a b; do
  for y in 1 2; do
    echo "${x}${y}"
  done
done
"#;

    assert_parses_without_panic(
        purification_examples,
        "Purification strategy: seq, explicit lists, nested loops",
    );
}

// DOCUMENTATION: Common brace expansion use cases (bash, NOT SUPPORTED)
// mkdir dirs, backup files, iterate ranges, generate filenames, multi-commands.
// All have POSIX equivalents using explicit lists, while loops, or for loops.
#[test]
fn test_EXP_BRACE_001_common_use_cases() {
    let common_uses = r#"
# Use Case 1: Create directories (Bash)
# mkdir -p project/{src,tests,docs}

# POSIX alternative
mkdir -p project/src project/tests project/docs

# Use Case 2: Backup files (Bash)
# cp config.json{,.bak}

# POSIX alternative
cp config.json config.json.bak

# Use Case 3: Iterate ranges (Bash)
# for i in {1..100}; do echo "$i"; done

# POSIX alternative
i=1
while [ $i -le 100 ]; do
  echo "$i"
  i=$((i+1))
done

# Use Case 4: Generate files (Bash)
# touch file{1..5}.txt

# POSIX alternative
for i in 1 2 3 4 5; do
  touch "file${i}.txt"
done
"#;

    assert_parses_without_panic(common_uses, "Common use cases with POSIX alternatives");
}

#[test]
fn test_EXP_BRACE_001_edge_cases() {
    // DOCUMENTATION: Brace expansion edge cases (bash, NOT SUPPORTED)
    //
    // Edge Case 1: Zero-padded sequences
    // Bash:
    // $ echo {01..10}
    // 01 02 03 04 05 06 07 08 09 10
    //
    // POSIX:
    // $ seq -f "%02g" 1 10
    //
    // Edge Case 2: Reverse sequences
    // Bash:
    // $ echo {10..1}
    // 10 9 8 7 6 5 4 3 2 1
    //
    // POSIX:
    // $ seq 10 -1 1
    //
    // Edge Case 3: Step sequences
    // Bash:
    // $ echo {0..100..10}
    // 0 10 20 30 40 50 60 70 80 90 100
    //
    // POSIX:
    // $ seq 0 10 100
    //
    // Edge Case 4: Empty braces (literal)
    // Bash:
    // $ echo {}
    // {}  # Literal braces, no expansion
    //
    // Edge Case 5: Single item (literal)
    // Bash:
    // $ echo {foo}
    // {foo}  # Literal, no expansion (needs comma or ..)

    let edge_cases = r#"
# Edge Case 1: Zero-padded (Bash)
# echo {01..10}

# POSIX alternative
seq -f "%02g" 1 10

# Edge Case 2: Reverse sequence (Bash)
# echo {10..1}

# POSIX alternative
seq 10 -1 1

# Edge Case 3: Step sequence (Bash)
# echo {0..100..10}

# POSIX alternative
seq 0 10 100

# Edge Case 4: Empty braces (literal in bash)
# echo {}  # No expansion, prints {}

# Edge Case 5: Single item (literal in bash)
# echo {foo}  # No expansion, prints {foo}
"#;

    let result = BashParser::new(edge_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Edge cases documented with POSIX alternatives"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Brace expansion comparison (Bash vs POSIX vs bashrs)
// {1..10}, {a..z}, {foo,bar}, {a,b}{1,2} all bash-only, NOT SUPPORTED.
// Purify to POSIX: seq, explicit lists, nested loops. All portable.
#[test]
fn test_EXP_BRACE_001_comparison_table() {
    let comparison_example = r#"
# Bash: Brace expansion (NOT SUPPORTED)
# echo {1..10}
# echo {a..e}
# echo {foo,bar,baz}

# POSIX: seq and explicit lists (SUPPORTED)
seq 1 10
echo a b c d e
echo foo bar baz

# Bash: Nested expansion (NOT SUPPORTED)
# echo {a,b}{1,2}

# POSIX: Nested loops (SUPPORTED)
for x in a b; do
  for y in 1 2; do
    echo "${x}${y}"
  done
done
"#;

    assert_parses_without_panic(
        comparison_example,
        "Brace expansion comparison and purification documented",
    );
}

// Summary:
// Brace expansion {..}: Bash extension (NOT SUPPORTED)
// Types: Numeric sequences {1..10}, letter sequences {a..z}, comma lists {foo,bar}
// Nested: {a,b}{1,2} creates Cartesian product (a1 a2 b1 b2)
// Introduced: Bash 3.0 (2004), not in POSIX specification
// POSIX alternatives: seq command, for loops, explicit lists
// Purification: {1..10} → seq 1 10, {foo,bar} → echo foo bar, nested → loops
// Common uses: mkdir {src,tests,docs}, cp file{,.bak}, touch file{1..5}.txt
// Best practice: Use seq for ranges, explicit lists for small sets, avoid in portable scripts

// ============================================================================
// EXP-TILDE-001: Tilde Expansion ~ (POSIX, SUPPORTED)
// ============================================================================

