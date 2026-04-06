
use super::*;

// RED PHASE: Write failing tests first

#[test]
fn test_score_empty_script() {
    let source = "";
    let score = score_script(source).unwrap();
    assert_eq!(score.grade, "F");
    assert_eq!(score.score, 0.0);
}

#[test]
fn test_score_perfect_script() {
    let source = r#"#!/bin/bash
# Perfect script example
# This script demonstrates best practices

set -euo pipefail

# TEST: main function works
test_main() {
result=$(main)
[ "$result" = "success" ]
}

# Main function with proper error handling
main() {
local input="${1:-}"

if [ -z "$input" ]; then
    echo "Error: input required" >&2
    return 1
fi

echo "success"
}

main "$@"
"#;
    let score = score_script(source).unwrap();
    assert!(score.score >= 9.0, "Perfect script should score A or A+");
    assert!(matches!(score.grade.as_str(), "A" | "A+"));
}

#[test]
fn test_score_unsafe_script() {
    let source = r#"#!/bin/bash
# Unsafe script with many issues

FILES=$(ls *.txt)
for f in $FILES; do
rm $f
done
"#;
    let score = score_script(source).unwrap();
    assert!(score.score < 6.0, "Unsafe script should score D or F");
    assert!(
        matches!(score.grade.as_str(), "D" | "F"),
        "Should get D or F grade"
    );
    assert!(!score.suggestions.is_empty(), "Should provide suggestions");
}

#[test]
fn test_score_dimensions_calculated() {
    let source = r#"#!/bin/bash
function example() {
echo "test"
}
"#;
    let score = score_script(source).unwrap();

    // All dimensions should be calculated (0.0-10.0)
    assert!(score.complexity >= 0.0 && score.complexity <= 10.0);
    assert!(score.safety >= 0.0 && score.safety <= 10.0);
    assert!(score.maintainability >= 0.0 && score.maintainability <= 10.0);
    assert!(score.testing >= 0.0 && score.testing <= 10.0);
    assert!(score.documentation >= 0.0 && score.documentation <= 10.0);
}

#[test]
fn test_score_with_tests_higher() {
    let script_with_tests = r#"#!/bin/bash
function add() {
echo $(( $1 + $2 ))
}

# TEST: add function works
test_add() {
result=$(add 2 3)
[ "$result" -eq 5 ]
}
"#;
    let script_without_tests = r#"#!/bin/bash
function add() {
echo $(( $1 + $2 ))
}
"#;

    let score_with = score_script(script_with_tests).unwrap();
    let score_without = score_script(script_without_tests).unwrap();

    assert!(score_with.testing > score_without.testing);
    assert!(score_with.score > score_without.score);
}

#[test]
fn test_score_with_documentation_higher() {
    let script_with_docs = r#"#!/bin/bash
# This script does something useful
# Author: Test
# Usage: script.sh <input>

# Main function
# Args: $1 - input value
function main() {
echo "test"
}
"#;
    let script_without_docs = r#"#!/bin/bash
function main() {
echo "test"
}
"#;

    let score_with = score_script(script_with_docs).unwrap();
    let score_without = score_script(script_without_docs).unwrap();

    assert!(score_with.documentation > score_without.documentation);
    assert!(score_with.score > score_without.score);
}

#[test]
fn test_score_safety_quoting() {
    let safe_script = r#"#!/bin/bash
FILES="$(ls *.txt)"
for f in "$FILES"; do
echo "$f"
done
"#;
    let unsafe_script = r#"#!/bin/bash
FILES=$(ls *.txt)
for f in $FILES; do
echo $f
done
"#;

    let score_safe = score_script(safe_script).unwrap();
    let score_unsafe = score_script(unsafe_script).unwrap();

    assert!(score_safe.safety > score_unsafe.safety);
    assert!(score_safe.score > score_unsafe.score);
}

#[test]
fn test_score_provides_suggestions() {
    let source = r#"#!/bin/bash
rm $FILE
cp $SRC $DST
"#;
    let score = score_script(source).unwrap();

    assert!(!score.suggestions.is_empty());
    assert!(score.suggestions.iter().any(|s| s.contains("quote")));
}

// NOTE: Grade calculation tests moved to scoring_config.rs (26 comprehensive tests)
// Old test_calculate_grade_boundaries removed - now using file type-aware grading

#[test]
fn test_score_complexity_long_functions() {
    let simple_script = r#"#!/bin/bash
function simple() {
echo "test"
}
"#;
    let complex_script = r#"#!/bin/bash
function complex() {
if [ "$1" = "a" ]; then
    if [ "$2" = "b" ]; then
        if [ "$3" = "c" ]; then
            for i in 1 2 3; do
                while [ "$i" -lt 10 ]; do
                    echo "$i"
                    i=$((i + 1))
                done
            done
        fi
    fi
fi
}
"#;

    let score_simple = score_script(simple_script).unwrap();
    let score_complex = score_script(complex_script).unwrap();

    assert!(score_simple.complexity > score_complex.complexity);
    assert!(score_simple.score > score_complex.score);
}
