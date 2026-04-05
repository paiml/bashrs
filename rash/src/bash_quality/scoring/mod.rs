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

    f64::midpoint(nesting_score, length_score)
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

include!("mod_incl2.rs");
