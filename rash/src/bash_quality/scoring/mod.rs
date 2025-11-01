//! Bash Quality Scoring
//!
//! TDG-style quality scoring for bash scripts (A+ to F).
//!
//! ## Scoring Dimensions
//!
//! 1. **Complexity**: Cyclomatic complexity, nesting depth
//! 2. **Safety**: Proper quoting, injection prevention, error handling
//! 3. **Maintainability**: Function size, variable naming, comments
//! 4. **Testing**: Test coverage, test quality
//! 5. **Documentation**: Comments, inline docs, examples
//!
//! ## Grade Scale
//!
//! - A+ (9.5-10): Perfect
//! - A  (9.0-9.5): Excellent
//! - B+ (8.5-9.0): Very Good
//! - B  (8.0-8.5): Good
//! - C+ (7.5-8.0): Above Average
//! - C  (7.0-7.5): Average
//! - D  (6.0-7.0): Below Average
//! - F  (<6.0): Poor

/// Quality score for a bash script
#[derive(Debug, Clone)]
pub struct QualityScore {
    /// Overall grade (A+ to F)
    pub grade: String,

    /// Numeric score (0.0 - 10.0)
    pub score: f64,

    /// Complexity score (0.0 - 10.0)
    pub complexity: f64,

    /// Safety score (0.0 - 10.0)
    pub safety: f64,

    /// Maintainability score (0.0 - 10.0)
    pub maintainability: f64,

    /// Testing score (0.0 - 10.0)
    pub testing: f64,

    /// Documentation score (0.0 - 10.0)
    pub documentation: f64,

    /// Improvement suggestions
    pub suggestions: Vec<String>,
}

impl QualityScore {
    /// Create new quality score
    pub fn new() -> Self {
        Self {
            grade: String::from("F"),
            score: 0.0,
            complexity: 0.0,
            safety: 0.0,
            maintainability: 0.0,
            testing: 0.0,
            documentation: 0.0,
            suggestions: Vec::new(),
        }
    }
}

impl Default for QualityScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a bash script for quality
///
/// Returns quality score with grade, numeric score, and improvement suggestions.
pub fn score_script(source: &str) -> Result<QualityScore, String> {
    score_script_with_file_type(source, None)
}

/// Score a bash script with file type detection
///
/// Returns quality score with file-type aware weights and thresholds.
pub fn score_script_with_file_type(
    source: &str,
    file_path: Option<&std::path::Path>,
) -> Result<QualityScore, String> {
    use crate::bash_quality::linter::suppressions::FileType;
    use crate::bash_quality::scoring_config::calculate_grade as calculate_grade_by_type;

    let mut score = QualityScore::new();

    // Detect file type if path provided
    let file_type = file_path.map(FileType::from_path).unwrap_or_default();

    // Calculate each dimension
    score.complexity = calculate_complexity_score(source);
    score.safety = calculate_safety_score(source);
    score.maintainability = calculate_maintainability_score(source);
    score.testing = calculate_testing_score(source);
    score.documentation = calculate_documentation_score(source);

    // Calculate overall score (weighted average)
    // Use standard weights (same as original) - file type only affects grade thresholds
    score.score = (score.complexity * 0.25)
        + (score.safety * 0.30)
        + (score.maintainability * 0.20)
        + (score.testing * 0.15)
        + (score.documentation * 0.10);

    // Assign grade using file-type aware thresholds
    score.grade = calculate_grade_by_type(score.score, file_type);

    // Generate suggestions
    score.suggestions = generate_suggestions(source, &score);

    Ok(score)
}

/// Calculate complexity score (0.0-10.0)
/// Check if line starts a control structure (if/for/while/case)
fn is_control_structure_start(trimmed: &str) -> bool {
    trimmed.starts_with("if ")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("case ")
}

/// Check if line ends a control structure (fi/done/esac)
fn is_control_structure_end(trimmed: &str) -> bool {
    trimmed == "fi" || trimmed == "done" || trimmed == "esac"
}

/// Calculate score based on nesting depth
fn calculate_nesting_score(max_nesting: i32) -> f64 {
    match max_nesting {
        0..=1 => 10.0,
        2 => 9.0,
        3 => 7.0,
        4 => 5.0,
        _ => 3.0,
    }
}

/// Calculate score based on maximum function length
fn calculate_length_score(max_function_length: usize) -> f64 {
    match max_function_length {
        0..=10 => 10.0,
        11..=20 => 9.0,
        21..=30 => 7.0,
        31..=50 => 5.0,
        _ => 3.0,
    }
}

fn calculate_complexity_score(source: &str) -> f64 {
    let lines: Vec<&str> = source.lines().collect();

    // Empty script gets 0.0
    if lines.is_empty() {
        return 0.0;
    }

    let mut max_nesting: i32 = 0;
    let mut current_nesting: i32 = 0;
    let mut max_function_length: usize = 0;
    let mut current_function_length: usize = 0;
    let mut in_function = false;

    for line in &lines {
        let trimmed = line.trim();

        // Track function start
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            in_function = true;
            current_function_length = 0;
        }

        // Track function end
        if in_function && trimmed == "}" {
            if current_function_length > max_function_length {
                max_function_length = current_function_length;
            }
            in_function = false;
        }

        if in_function {
            current_function_length += 1;
        }

        // Track nesting depth
        if is_control_structure_start(trimmed) {
            current_nesting += 1;
            if current_nesting > max_nesting {
                max_nesting = current_nesting;
            }
        }

        if is_control_structure_end(trimmed) {
            current_nesting = current_nesting.saturating_sub(1);
        }
    }

    // Score based on complexity metrics
    let nesting_score = calculate_nesting_score(max_nesting);
    let length_score = calculate_length_score(max_function_length);

    (nesting_score + length_score) / 2.0
}

/// Check if line is empty or a comment
fn is_empty_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Check if line has unquoted variable and count issues
fn count_unquoted_vars(line: &str) -> usize {
    let trimmed = line.trim();

    // Skip if no variables or already quoted
    if !trimmed.contains('$') || trimmed.contains("\"$") {
        return 0;
    }

    // Simple heuristic: look for $VAR without quotes
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let mut count = 0;
    for part in parts {
        if part.starts_with('$') && !part.starts_with("\"$") && !part.starts_with("$(") {
            count += 1;
        }
    }
    count
}

/// Check if line has strict mode settings (set -e, set -u, set -o pipefail)
fn has_strict_mode_setting(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("set -e") || trimmed.contains("set -u") || trimmed.contains("set -o pipefail")
}

/// Check if line has quoted variables
fn has_quoted_variable(line: &str) -> bool {
    line.trim().contains("\"$")
}

/// Check if line has error handling
fn has_error_handling(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("|| return") || trimmed.contains("|| exit")
}

/// Calculate safety ratio from issues and good practices counts
fn calculate_safety_ratio(issues: usize, good_practices: usize) -> f64 {
    let safety_ratio = if issues == 0 {
        10.0
    } else if good_practices > issues * 2 {
        8.0
    } else if good_practices > issues {
        6.0
    } else {
        4.0 - (issues as f64 * 0.5).min(3.0)
    };

    safety_ratio.clamp(0.0, 10.0)
}

/// Calculate safety score (0.0-10.0)
fn calculate_safety_score(source: &str) -> f64 {
    // Empty script gets 0.0
    if source.trim().is_empty() {
        return 0.0;
    }

    let mut issues = 0;
    let mut good_practices = 0;
    let mut has_code = false;

    for line in source.lines() {
        // Skip comments and empty lines
        if is_empty_or_comment(line) {
            continue;
        }

        has_code = true;

        // Count unquoted variables (bad practice)
        issues += count_unquoted_vars(line);

        // Check for good practices
        if has_strict_mode_setting(line) {
            good_practices += 2;
        }

        if has_quoted_variable(line) {
            good_practices += 1;
        }

        if has_error_handling(line) {
            good_practices += 1;
        }
    }

    // If no actual code, return 0.0
    if !has_code {
        return 0.0;
    }

    // Calculate score based on issues vs good practices
    calculate_safety_ratio(issues, good_practices)
}

/// Calculate maintainability score (0.0-10.0)
fn calculate_maintainability_score(source: &str) -> f64 {
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len() as f64;

    if total_lines == 0.0 {
        return 0.0;
    }

    let mut function_count = 0;
    let mut comment_lines = 0;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            comment_lines += 1;
        }

        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            function_count += 1;
        }
    }

    // Maintainability factors
    let comment_ratio = comment_lines as f64 / total_lines;
    let has_functions = function_count > 0;

    let mut score: f64 = 0.0;

    // Base score for having any content
    if total_lines > 3.0 {
        score += 4.0;
    }

    // Good comment ratio (10-30%)
    if (0.10..=0.30).contains(&comment_ratio) {
        score += 3.0;
    } else if comment_ratio > 0.05 {
        score += 1.0;
    }

    // Has modular functions
    if has_functions {
        score += 2.0;
    }

    // Good function count (not too many, not too few)
    if (2..=10).contains(&function_count) {
        score += 1.0;
    }

    score.min(10.0)
}

/// Calculate testing score (0.0-10.0)
fn calculate_testing_score(source: &str) -> f64 {
    let mut test_count: i32 = 0;
    let mut function_count: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.contains("test_")
            && (trimmed.contains("() {") || trimmed.starts_with("function test_"))
        {
            test_count += 1;
        }

        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            function_count += 1;
        }
    }

    // Subtract test functions from total to get regular functions
    let regular_functions = function_count.saturating_sub(test_count);

    if test_count == 0 {
        return 0.0;
    }

    // Score based on test coverage
    let coverage_ratio = if regular_functions > 0 {
        test_count as f64 / regular_functions as f64
    } else {
        1.0
    };

    match coverage_ratio {
        r if r >= 1.0 => 10.0, // 100% coverage or better
        r if r >= 0.8 => 8.0,  // 80%+
        r if r >= 0.5 => 6.0,  // 50%+
        r if r >= 0.3 => 4.0,  // 30%+
        _ => 2.0,              // Some tests
    }
}

/// Calculate documentation score (0.0-10.0)
fn calculate_documentation_score(source: &str) -> f64 {
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len() as f64;

    if total_lines == 0.0 {
        return 0.0;
    }

    let mut comment_lines = 0;
    let mut header_comment = false;
    let mut function_docs = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Count comments
        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            comment_lines += 1;

            // Check for header comments (first 10 lines)
            if i < 10 {
                header_comment = true;
            }
        }

        // Check for function documentation
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            // Look at previous line for comment
            if i > 0 {
                if let Some(prev_line) = lines.get(i - 1) {
                    if prev_line.trim().starts_with('#') {
                        function_docs += 1;
                    }
                }
            }
        }
    }

    let comment_ratio = comment_lines as f64 / total_lines;

    let mut score: f64 = 0.0;

    // Good comment ratio (more granular scoring)
    if comment_ratio >= 0.20 {
        score += 5.0;
    } else if comment_ratio >= 0.15 {
        score += 4.0;
    } else if comment_ratio >= 0.10 {
        score += 3.0;
    } else if comment_ratio >= 0.05 {
        score += 1.5;
    } else if comment_ratio > 0.0 {
        score += 0.5;
    }

    // Has header comment
    if header_comment {
        score += 3.0;
    }

    // Has function documentation (scale by number of documented functions)
    if function_docs > 0 {
        score += (function_docs as f64 * 0.5).min(2.0);
    }

    score.min(10.0)
}

/// Generate improvement suggestions
fn generate_suggestions(source: &str, score: &QualityScore) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Safety suggestions
    if score.safety < 7.0 {
        let mut has_unquoted = false;
        for line in source.lines() {
            if line.contains("$") && !line.contains("\"$") {
                has_unquoted = true;
                break;
            }
        }

        if has_unquoted {
            suggestions.push(
                "Add quotes around variable expansions (\"$var\") to prevent word splitting"
                    .to_string(),
            );
        }

        if !source.contains("set -e") && !source.contains("set -u") {
            suggestions.push("Add 'set -euo pipefail' for better error handling".to_string());
        }
    }

    // Complexity suggestions
    if score.complexity < 7.0 {
        suggestions.push(
            "Reduce function complexity by extracting nested logic into separate functions"
                .to_string(),
        );
        suggestions.push(
            "Consider breaking down large functions (>20 lines) into smaller, focused functions"
                .to_string(),
        );
    }

    // Testing suggestions
    if score.testing < 5.0 {
        suggestions.push("Add test functions (test_*) to verify script behavior".to_string());
        suggestions.push("Aim for at least 50% test coverage of your functions".to_string());
    }

    // Documentation suggestions
    if score.documentation < 5.0 {
        suggestions.push("Add header comments describing script purpose and usage".to_string());
        suggestions
            .push("Document each function with a comment describing its purpose".to_string());
    }

    // Maintainability suggestions
    if score.maintainability < 7.0 {
        suggestions.push("Extract repeated code into reusable functions".to_string());
        suggestions.push("Add comments to explain complex logic".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
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
}
