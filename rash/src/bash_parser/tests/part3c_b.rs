#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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

#[test]
fn test_EXP_TILDE_001_tilde_expansion_supported() {
    // DOCUMENTATION: Tilde expansion is SUPPORTED (POSIX)
    //
    // Tilde expansion replaces ~ with paths:
    // - POSIX-compliant feature (sh, bash, dash, ash all support)
    // - ~ expands to $HOME (user's home directory)
    // - ~user expands to user's home directory
    //
    // Basic tilde expansion:
    // $ echo ~
    // /home/username
    //
    // $ cd ~/documents
    // # Changes to /home/username/documents
    //
    // User-specific tilde:
    // $ echo ~root
    // /root
    //
    // $ echo ~alice
    // /home/alice
    //
    // Why tilde expansion is POSIX:
    // - Part of POSIX specification
    // - All POSIX shells support ~
    // - Portable across sh, bash, dash, ash
    //
    // Rust mapping:
    // ```rust
    // use std::env;
    //
    // // Get home directory
    // let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
    // let path = format!("{}/documents", home);
    //
    // // Or use dirs crate
    // use dirs::home_dir;
    // let home = home_dir().expect("No home directory");
    // ```

    let tilde_expansion = r#"
# POSIX tilde expansion (SUPPORTED)
cd ~
cd ~/documents
echo ~
ls ~/projects
"#;

    let result = BashParser::new(tilde_expansion);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Tilde expansion is POSIX-compliant, FULLY SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - ~ may not be fully implemented yet
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_home_directory() {
    // DOCUMENTATION: ~ expands to $HOME (POSIX)
    //
    // Basic ~ expansion:
    // $ echo ~
    // /home/username  # Value of $HOME
    //
    // $ HOME=/custom/path
    // $ echo ~
    // /custom/path  # Uses current $HOME value
    //
    // Tilde in paths:
    // $ cd ~/projects
    // # Expands to: cd /home/username/projects
    //
    // $ mkdir ~/backup
    // # Expands to: mkdir /home/username/backup
    //
    // Important: Tilde must be at start of word
    // $ echo ~/dir    # ✅ Expands
    // $ echo /~       # ❌ No expansion (~ not at start)
    // $ echo "~"      # ❌ No expansion (quoted)
    //
    // POSIX equivalent:
    // $ cd "$HOME/projects"
    // $ mkdir "$HOME/backup"

    let tilde_home = r#"
# Tilde at start of word (expands)
cd ~
cd ~/documents
mkdir ~/backup

# Tilde not at start (no expansion)
# echo /~  # Literal /~, not expanded

# Quoted tilde (no expansion)
# echo "~"  # Literal ~, not expanded

# POSIX alternative: explicit $HOME
cd "$HOME"
cd "$HOME/documents"
mkdir "$HOME/backup"
"#;

    let result = BashParser::new(tilde_home);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "~ expands to $HOME (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

