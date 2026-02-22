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
mod tests {
    use super::*;

    // ===== UTILITY FUNCTION TESTS =====

    #[test]
    fn test_hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_encode_single_byte() {
        assert_eq!(hex_encode(&[0x00]), "00");
        assert_eq!(hex_encode(&[0xff]), "ff");
        assert_eq!(hex_encode(&[0x0a]), "0a");
    }

    #[test]
    fn test_hex_encode_multiple_bytes() {
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
        assert_eq!(hex_encode(&[0x01, 0x23, 0x45, 0x67]), "01234567");
    }

    #[test]
    fn test_format_timestamp_just_now() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 30);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_timestamp_minutes_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 120);
        assert_eq!(result, "2m ago");
    }

    #[test]
    fn test_format_timestamp_hours_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 7200);
        assert_eq!(result, "2h ago");
    }

    #[test]
    fn test_format_timestamp_days_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 172800);
        assert_eq!(result, "2d ago");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_str_empty() {
        assert_eq!(truncate_str("", 10), "");
    }

    #[test]
    fn test_generate_diff_lines_identical() {
        let diff = generate_diff_lines("a\nb\nc", "a\nb\nc");
        assert!(diff.is_empty());
    }

    #[test]
    fn test_generate_diff_lines_one_change() {
        let diff = generate_diff_lines("a\nb\nc", "a\nB\nc");
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], (2, "b".to_string(), "B".to_string()));
    }

    #[test]
    fn test_generate_diff_lines_multiple_changes() {
        let diff = generate_diff_lines("a\nb\nc", "A\nb\nC");
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0], (1, "a".to_string(), "A".to_string()));
        assert_eq!(diff[1], (3, "c".to_string(), "C".to_string()));
    }

    #[test]
    fn test_generate_diff_lines_empty() {
        let diff = generate_diff_lines("", "");
        assert!(diff.is_empty());
    }

    #[test]
    fn test_score_status_excellent() {
        assert_eq!(score_status(10.0), "✅");
        assert_eq!(score_status(8.0), "✅");
    }

    #[test]
    fn test_score_status_warning() {
        assert_eq!(score_status(7.0), "⚠️");
        assert_eq!(score_status(6.0), "⚠️");
    }

    #[test]
    fn test_score_status_poor() {
        assert_eq!(score_status(5.0), "❌");
        assert_eq!(score_status(0.0), "❌");
    }

    #[test]
    fn test_coverage_status_good() {
        assert_eq!(coverage_status(90.0), "✅");
        assert_eq!(coverage_status(80.0), "✅");
    }

    #[test]
    fn test_coverage_status_medium() {
        assert_eq!(coverage_status(75.0), "⚠️");
        assert_eq!(coverage_status(50.0), "⚠️");
    }

    #[test]
    fn test_coverage_status_poor() {
        assert_eq!(coverage_status(49.0), "❌");
        assert_eq!(coverage_status(0.0), "❌");
    }

    #[test]
    fn test_coverage_class_excellent() {
        assert_eq!(coverage_class(100.0), "excellent");
        assert_eq!(coverage_class(95.0), "excellent");
        assert_eq!(coverage_class(90.0), "excellent");
    }

    #[test]
    fn test_coverage_class_good() {
        assert_eq!(coverage_class(89.9), "good");
        assert_eq!(coverage_class(85.0), "good");
        assert_eq!(coverage_class(80.0), "good");
    }

    #[test]
    fn test_coverage_class_fair() {
        assert_eq!(coverage_class(79.9), "fair");
        assert_eq!(coverage_class(75.0), "fair");
        assert_eq!(coverage_class(70.0), "fair");
    }

    #[test]
    fn test_coverage_class_poor() {
        assert_eq!(coverage_class(69.9), "poor");
        assert_eq!(coverage_class(50.0), "poor");
        assert_eq!(coverage_class(0.0), "poor");
    }

    // ===== PERCENTAGE CALCULATION TESTS =====

    #[test]
    fn test_calculate_percentage_normal() {
        assert_eq!(calculate_percentage(50, 100), 50.0);
        assert_eq!(calculate_percentage(75, 100), 75.0);
        assert_eq!(calculate_percentage(1, 4), 25.0);
    }

    #[test]
    fn test_calculate_percentage_zero_total() {
        assert_eq!(calculate_percentage(0, 0), 100.0);
        assert_eq!(calculate_percentage(5, 0), 100.0);
    }

    #[test]
    fn test_calculate_percentage_full() {
        assert_eq!(calculate_percentage(100, 100), 100.0);
    }

    // ===== BYTES FORMATTING TESTS =====

    #[test]
    fn test_format_bytes_human_bytes() {
        assert_eq!(format_bytes_human(0), "0 B");
        assert_eq!(format_bytes_human(512), "512 B");
        assert_eq!(format_bytes_human(999), "999 B");
    }

    #[test]
    fn test_format_bytes_human_kb() {
        assert_eq!(format_bytes_human(1_000), "1.00 KB");
        assert_eq!(format_bytes_human(1_500), "1.50 KB");
        assert_eq!(format_bytes_human(999_999), "1000.00 KB");
    }

    #[test]
    fn test_format_bytes_human_mb() {
        assert_eq!(format_bytes_human(1_000_000), "1.00 MB");
        assert_eq!(format_bytes_human(500_000_000), "500.00 MB");
    }

    #[test]
    fn test_format_bytes_human_gb() {
        assert_eq!(format_bytes_human(1_000_000_000), "1.00 GB");
        assert_eq!(format_bytes_human(2_500_000_000), "2.50 GB");
    }

    // ===== DURATION FORMATTING TESTS =====

    #[test]
    fn test_format_duration_human_seconds() {
        assert_eq!(format_duration_human(0), "0s");
        assert_eq!(format_duration_human(30), "30s");
        assert_eq!(format_duration_human(59), "59s");
    }

    #[test]
    fn test_format_duration_human_minutes() {
        assert_eq!(format_duration_human(60), "1m 0s");
        assert_eq!(format_duration_human(90), "1m 30s");
        assert_eq!(format_duration_human(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_human_hours() {
        assert_eq!(format_duration_human(3600), "1h 0m 0s");
        assert_eq!(format_duration_human(3661), "1h 1m 1s");
        assert_eq!(format_duration_human(7325), "2h 2m 5s");
    }

    // ===== GRADE INTERPRETATION TESTS =====

    #[test]
    fn test_grade_interpretation_excellent() {
        assert!(grade_interpretation("A+").contains("Excellent"));
        assert!(grade_interpretation("A").contains("Great"));
    }

    #[test]
    fn test_grade_interpretation_good() {
        assert!(grade_interpretation("B+").contains("Good"));
        assert!(grade_interpretation("B").contains("Good"));
    }

    #[test]
    fn test_grade_interpretation_average() {
        assert!(grade_interpretation("C+").contains("Average"));
        assert!(grade_interpretation("C").contains("Average"));
    }

    #[test]
    fn test_grade_interpretation_poor() {
        assert!(grade_interpretation("D").contains("Below"));
        assert!(grade_interpretation("F").contains("Poor"));
    }

    #[test]
    fn test_grade_interpretation_unknown() {
        assert!(grade_interpretation("X").contains("Unknown"));
    }

    #[test]
    fn test_grade_symbol() {
        assert_eq!(grade_symbol("A+"), "✓");
        assert_eq!(grade_symbol("A"), "✓");
        assert_eq!(grade_symbol("B+"), "✓");
        assert_eq!(grade_symbol("B"), "✓");
        assert_eq!(grade_symbol("C+"), "⚠");
        assert_eq!(grade_symbol("C"), "⚠");
        assert_eq!(grade_symbol("D"), "⚠");
        assert_eq!(grade_symbol("F"), "✗");
        assert_eq!(grade_symbol("X"), "?");
    }

    // ===== REPORT FORMATTING TESTS =====

    #[test]
    fn test_format_purify_report_human() {
        let items = vec!["Fixed tabs".to_string(), "Added phony".to_string()];
        let report = format_purify_report_human(5, 3, 2, &items);

        assert!(report.contains("Makefile Purification Report"));
        assert!(report.contains("Transformations Applied: 5"));
        assert!(report.contains("Issues Fixed: 3"));
        assert!(report.contains("Manual Fixes Needed: 2"));
        assert!(report.contains("1: Fixed tabs"));
        assert!(report.contains("2: Added phony"));
    }

    #[test]
    fn test_format_purify_report_json() {
        let items = vec!["Fix1".to_string()];
        let report = format_purify_report_json(1, 1, 0, &items);

        assert!(report.contains("\"transformations_applied\": 1"));
        assert!(report.contains("\"issues_fixed\": 1"));
        assert!(report.contains("\"manual_fixes_needed\": 0"));
        assert!(report.contains("\"Fix1\""));
    }

    #[test]
    fn test_format_purify_report_markdown() {
        let items = vec!["Item1".to_string()];
        let report = format_purify_report_markdown(2, 1, 1, &items);

        assert!(report.contains("# Makefile Purification Report"));
        assert!(report.contains("**Transformations**: 2"));
        assert!(report.contains("1. Item1"));
    }

    // ===== SCORE FORMATTING TESTS =====

    #[test]
    fn test_format_score_human_basic() {
        let suggestions = vec!["Add tests".to_string()];
        let report = format_score_human("A", 9.0, 9.0, 9.0, 9.0, 8.0, 8.0, &suggestions, false);

        assert!(report.contains("Overall Grade: A"));
        assert!(report.contains("9.0/10.0"));
        assert!(report.contains("Add tests"));
    }

    #[test]
    fn test_format_score_human_detailed() {
        let report = format_score_human("B", 8.0, 7.0, 8.0, 9.0, 6.0, 7.0, &[], true);

        assert!(report.contains("Dimension Scores:"));
        assert!(report.contains("Complexity:"));
        assert!(report.contains("Safety:"));
        assert!(report.contains("Maintainability:"));
    }

    #[test]
    fn test_format_score_human_no_suggestions() {
        let report = format_score_human("A+", 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, &[], false);

        assert!(!report.contains("Improvement Suggestions:"));
    }

    // ===== TEST RESULT STATUS TESTS =====

    #[test]
    fn test_test_result_status_passed() {
        assert_eq!(test_result_status(10, 0, 10), "PASSED");
    }

    #[test]
    fn test_test_result_status_failed() {
        assert_eq!(test_result_status(8, 2, 10), "FAILED");
    }

    #[test]
    fn test_test_result_status_no_tests() {
        assert_eq!(test_result_status(0, 0, 0), "NO TESTS");
    }

    #[test]
    fn test_test_result_status_partial() {
        assert_eq!(test_result_status(5, 0, 10), "PARTIAL");
    }

    // ===== TEST PASS RATE TESTS =====

    #[test]
    fn test_test_pass_rate_all_passed() {
        assert_eq!(test_pass_rate(10, 10), 100.0);
    }

    #[test]
    fn test_test_pass_rate_half() {
        assert_eq!(test_pass_rate(5, 10), 50.0);
    }

    #[test]
    fn test_test_pass_rate_none() {
        assert_eq!(test_pass_rate(0, 10), 0.0);
    }

    #[test]
    fn test_test_pass_rate_no_tests() {
        assert_eq!(test_pass_rate(0, 0), 100.0);
    }
}
