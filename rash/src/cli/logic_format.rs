// CLI Logic - Formatting and Display Utilities
//
// Functions for formatting output, scores, reports, and human-readable text.

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Hex encode bytes to string
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Format timestamp as relative time
pub fn format_timestamp(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Truncate string to max length with ellipsis (delegates to batuta-common).
pub fn truncate_str(s: &str, max_len: usize) -> String {
    batuta_common::display::truncate_str(s, max_len)
}

/// Generate diff lines between original and purified content
pub fn generate_diff_lines(original: &str, purified: &str) -> Vec<(usize, String, String)> {
    let original_lines: Vec<&str> = original.lines().collect();
    let purified_lines: Vec<&str> = purified.lines().collect();

    original_lines
        .iter()
        .zip(purified_lines.iter())
        .enumerate()
        .filter_map(|(i, (orig, pure))| {
            if orig != pure {
                Some((i + 1, orig.to_string(), pure.to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Helper to get status emoji for dimension score
pub fn score_status(score: f64) -> &'static str {
    if score >= 8.0 {
        "✅"
    } else if score >= 6.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Helper to get status emoji for coverage percent
pub fn coverage_status(percent: f64) -> &'static str {
    if percent >= 80.0 {
        "✅"
    } else if percent >= 50.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Helper to get CSS class for coverage percent
pub fn coverage_class(percent: f64) -> &'static str {
    if percent >= 90.0 {
        "excellent"
    } else if percent >= 80.0 {
        "good"
    } else if percent >= 70.0 {
        "fair"
    } else {
        "poor"
    }
}

/// Calculate percentage with bounds
pub fn calculate_percentage(value: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (value as f64 / total as f64) * 100.0
    }
}

/// Format bytes as human readable size
pub fn format_bytes_human(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration in seconds as human readable
pub fn format_duration_human(seconds: u64) -> String {
    if seconds >= 3600 {
        format!(
            "{}h {}m {}s",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

// =============================================================================
// GRADE INTERPRETATION
// =============================================================================

/// Get human-readable grade interpretation
pub fn grade_interpretation(grade: &str) -> &'static str {
    match grade {
        "A+" => "Excellent! Near-perfect code quality.",
        "A" => "Great! Very good code quality.",
        "B+" | "B" => "Good code quality with room for improvement.",
        "C+" | "C" => "Average code quality. Consider addressing suggestions.",
        "D" => "Below average. Multiple improvements needed.",
        "F" => "Poor code quality. Significant improvements required.",
        _ => "Unknown grade.",
    }
}

/// Get grade emoji/symbol
pub fn grade_symbol(grade: &str) -> &'static str {
    match grade {
        "A+" | "A" | "B+" | "B" => "✓",
        "C+" | "C" | "D" => "⚠",
        "F" => "✗",
        _ => "?",
    }
}

// =============================================================================
// REPORT FORMATTING
// =============================================================================

/// Format purification report as human text
pub fn format_purify_report_human(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("Makefile Purification Report\n");
    output.push_str("============================\n");
    output.push_str(&format!(
        "Transformations Applied: {}\n",
        transformations_applied
    ));
    output.push_str(&format!("Issues Fixed: {}\n", issues_fixed));
    output.push_str(&format!("Manual Fixes Needed: {}\n", manual_fixes_needed));
    output.push('\n');
    for (i, item) in report_items.iter().enumerate() {
        output.push_str(&format!("{}: {}\n", i + 1, item));
    }
    output
}

/// Format purification report as JSON
pub fn format_purify_report_json(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!(
        "  \"transformations_applied\": {},\n",
        transformations_applied
    ));
    output.push_str(&format!("  \"issues_fixed\": {},\n", issues_fixed));
    output.push_str(&format!(
        "  \"manual_fixes_needed\": {},\n",
        manual_fixes_needed
    ));
    output.push_str("  \"report\": [\n");
    for (i, item) in report_items.iter().enumerate() {
        let comma = if i < report_items.len() - 1 { "," } else { "" };
        output.push_str(&format!("    \"{}\"{}\n", item.replace('"', "\\\""), comma));
    }
    output.push_str("  ]\n");
    output.push_str("}\n");
    output
}

/// Format purification report as Markdown
pub fn format_purify_report_markdown(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("# Makefile Purification Report\n\n");
    output.push_str(&format!(
        "**Transformations**: {}\n",
        transformations_applied
    ));
    output.push_str(&format!("**Issues Fixed**: {}\n", issues_fixed));
    output.push_str(&format!(
        "**Manual Fixes Needed**: {}\n\n",
        manual_fixes_needed
    ));
    for (i, item) in report_items.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, item));
    }
    output
}

// =============================================================================
// SCORE FORMATTING
// =============================================================================

/// Format quality score as human text
#[allow(clippy::too_many_arguments)]
pub fn format_score_human(
    grade: &str,
    score: f64,
    complexity: f64,
    safety: f64,
    maintainability: f64,
    testing: f64,
    documentation: f64,
    suggestions: &[String],
    detailed: bool,
) -> String {
    let mut output = String::new();
    output.push('\n');
    output.push_str("Bash Script Quality Score\n");
    output.push_str("=========================\n\n");
    output.push_str(&format!("Overall Grade: {}\n", grade));
    output.push_str(&format!("Overall Score: {:.1}/10.0\n\n", score));

    if detailed {
        output.push_str("Dimension Scores:\n");
        output.push_str("-----------------\n");
        output.push_str(&format!("Complexity:      {:.1}/10.0\n", complexity));
        output.push_str(&format!("Safety:          {:.1}/10.0\n", safety));
        output.push_str(&format!("Maintainability: {:.1}/10.0\n", maintainability));
        output.push_str(&format!("Testing:         {:.1}/10.0\n", testing));
        output.push_str(&format!("Documentation:   {:.1}/10.0\n\n", documentation));
    }

    if !suggestions.is_empty() {
        output.push_str("Improvement Suggestions:\n");
        output.push_str("------------------------\n");
        for (i, suggestion) in suggestions.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, suggestion));
        }
        output.push('\n');
    }

    output.push_str(&format!(
        "{} {}\n",
        grade_symbol(grade),
        grade_interpretation(grade)
    ));
    output
}

/// Classify test result status
pub fn test_result_status(passed: usize, failed: usize, total: usize) -> &'static str {
    if failed > 0 {
        "FAILED"
    } else if passed == total && total > 0 {
        "PASSED"
    } else if total == 0 {
        "NO TESTS"
    } else {
        "PARTIAL"
    }
}

/// Calculate test pass rate
pub fn test_pass_rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (passed as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
#[path = "logic_format_tests_hex_encode.rs"]
mod tests_extracted;
