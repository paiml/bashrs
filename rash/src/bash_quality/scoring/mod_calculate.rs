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

/// Score comment ratio on a 0-5 scale
fn score_comment_ratio(ratio: f64) -> f64 {
    if ratio >= 0.20 {
        5.0
    } else if ratio >= 0.15 {
        4.0
    } else if ratio >= 0.10 {
        3.0
    } else if ratio >= 0.05 {
        1.5
    } else if ratio > 0.0 {
        0.5
    } else {
        0.0
    }
}

/// Count comments, header comment presence, and documented functions
fn analyze_comments(lines: &[&str]) -> (usize, bool, usize) {
    let mut comment_lines = 0;
    let mut header_comment = false;
    let mut function_docs = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            comment_lines += 1;
            if i < 10 {
                header_comment = true;
            }
        }
        if (trimmed.contains("() {") || trimmed.starts_with("function ")) && i > 0 {
            if let Some(prev_line) = lines.get(i - 1) {
                if prev_line.trim().starts_with('#') {
                    function_docs += 1;
                }
            }
        }
    }

    (comment_lines, header_comment, function_docs)
}

/// Calculate documentation score (0.0-10.0)
fn calculate_documentation_score(source: &str) -> f64 {
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len() as f64;

    if total_lines == 0.0 {
        return 0.0;
    }

    let (comment_lines, header_comment, function_docs) = analyze_comments(&lines);
    let comment_ratio = comment_lines as f64 / total_lines;

    let mut score = score_comment_ratio(comment_ratio);

    if header_comment {
        score += 3.0;
    }
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
            if line.contains('$') && !line.contains("\"$") {
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
#[path = "mod_tests_score_empty.rs"]
mod tests_extracted;
