//! ANSI color utilities for CLI output (pmat-style palette)
//!
//! Provides consistent colorized terminal output matching the pmat query visual style.

// ANSI escape codes
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[1;37m";
pub const BRIGHT_GREEN: &str = "\x1b[1;32m";
pub const BRIGHT_RED: &str = "\x1b[1;31m";
pub const BRIGHT_YELLOW: &str = "\x1b[1;33m";
pub const BRIGHT_CYAN: &str = "\x1b[1;36m";

/// Map a letter grade to an ANSI color code.
pub fn grade_color(grade: &str) -> &'static str {
    match grade {
        "A+" | "A" => BRIGHT_GREEN,
        "B+" | "B" => YELLOW,
        "C+" | "C" => YELLOW,
        "D" => RED,
        "F" => BRIGHT_RED,
        _ => WHITE,
    }
}

/// Map a percentage (0.0–100.0) to an ANSI color code.
pub fn pct_color(pct: f64) -> &'static str {
    if pct >= 99.0 {
        GREEN
    } else if pct >= 95.0 {
        YELLOW
    } else {
        RED
    }
}

/// Map a score dimension percentage to a color (more lenient thresholds for scores).
pub fn score_color(pct: f64) -> &'static str {
    if pct >= 80.0 {
        GREEN
    } else if pct >= 50.0 {
        YELLOW
    } else {
        RED
    }
}

/// Render a progress bar with colored fill.
///
/// Returns a string like `████████░░░░░░░░` where filled = green, empty = dim.
pub fn progress_bar(pass: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("{DIM}{}{RESET}", "░".repeat(width));
    }
    let filled = (pass as f64 / total as f64 * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    let fill_color = if pass == total {
        GREEN
    } else if pass as f64 / total as f64 >= 0.95 {
        YELLOW
    } else {
        RED
    };
    format!(
        "{fill_color}{}{RESET}{DIM}{}{RESET}",
        "█".repeat(filled),
        "░".repeat(empty),
    )
}

/// Colorize a pass/fail indicator.
pub fn pass_fail(passed: bool) -> String {
    if passed {
        format!("{GREEN}PASS{RESET}")
    } else {
        format!("{BRIGHT_RED}FAIL{RESET}")
    }
}

/// Colorize a pass/fail count (e.g., "500/500 passed").
pub fn pass_count(pass: usize, total: usize) -> String {
    let color = pct_color(pass as f64 / total.max(1) as f64 * 100.0);
    format!("{color}{pass}{RESET}/{total} passed")
}

/// Format a delta value with color: green if positive, red if negative, dim if zero.
pub fn delta_color(delta: f64) -> String {
    if delta > 0.0 {
        format!("{GREEN}+{delta:.4}{RESET}")
    } else if delta < 0.0 {
        format!("{RED}{delta:.4}{RESET}")
    } else {
        format!("{DIM}{delta:.4}{RESET}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_color_a_plus() {
        assert_eq!(grade_color("A+"), BRIGHT_GREEN);
    }

    #[test]
    fn test_grade_color_a() {
        assert_eq!(grade_color("A"), BRIGHT_GREEN);
    }

    #[test]
    fn test_grade_color_b() {
        assert_eq!(grade_color("B"), YELLOW);
    }

    #[test]
    fn test_grade_color_d() {
        assert_eq!(grade_color("D"), RED);
    }

    #[test]
    fn test_grade_color_f() {
        assert_eq!(grade_color("F"), BRIGHT_RED);
    }

    #[test]
    fn test_grade_color_unknown() {
        assert_eq!(grade_color("X"), WHITE);
    }

    #[test]
    fn test_pct_color_high() {
        assert_eq!(pct_color(100.0), GREEN);
        assert_eq!(pct_color(99.0), GREEN);
    }

    #[test]
    fn test_pct_color_medium() {
        assert_eq!(pct_color(95.0), YELLOW);
        assert_eq!(pct_color(97.5), YELLOW);
    }

    #[test]
    fn test_pct_color_low() {
        assert_eq!(pct_color(90.0), RED);
        assert_eq!(pct_color(50.0), RED);
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(10, 10, 16);
        assert!(bar.contains("████████████████"));
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 10, 16);
        assert!(bar.contains("░░░░░░░░░░░░░░░░"));
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let bar = progress_bar(0, 0, 8);
        assert!(bar.contains("░░░░░░░░"));
    }

    #[test]
    fn test_pass_fail_true() {
        let s = pass_fail(true);
        assert!(s.contains("PASS"));
        assert!(s.contains(GREEN));
    }

    #[test]
    fn test_pass_fail_false() {
        let s = pass_fail(false);
        assert!(s.contains("FAIL"));
        assert!(s.contains(BRIGHT_RED));
    }

    #[test]
    fn test_pass_count_perfect() {
        let s = pass_count(500, 500);
        assert!(s.contains("500"));
        assert!(s.contains("/500 passed"));
    }

    #[test]
    fn test_delta_color_positive() {
        let s = delta_color(0.0123);
        assert!(s.contains("+"));
        assert!(s.contains(GREEN));
    }

    #[test]
    fn test_delta_color_negative() {
        let s = delta_color(-0.05);
        assert!(s.contains(RED));
    }

    #[test]
    fn test_delta_color_zero() {
        let s = delta_color(0.0);
        assert!(s.contains(DIM));
    }

    #[test]
    fn test_score_color_high() {
        assert_eq!(score_color(90.0), GREEN);
    }

    #[test]
    fn test_score_color_mid() {
        assert_eq!(score_color(60.0), YELLOW);
    }

    #[test]
    fn test_score_color_low() {
        assert_eq!(score_color(30.0), RED);
    }
}
