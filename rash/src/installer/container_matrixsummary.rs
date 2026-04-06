/// Summary of matrix test results
#[derive(Debug, Clone)]
pub struct MatrixSummary {
    /// Total platforms tested
    pub total: usize,

    /// Platforms that passed
    pub passed: usize,

    /// Platforms that failed
    pub failed: usize,

    /// Platforms that were skipped
    pub skipped: usize,

    /// Total duration
    pub total_duration: Duration,

    /// Parallelism used
    pub parallelism: usize,
}

impl MatrixSummary {
    /// Format as text
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Summary: {}/{} passed", self.passed, self.total));

        if self.failed > 0 {
            output.push_str(&format!(", {} failed", self.failed));
        }

        if self.skipped > 0 {
            output.push_str(&format!(", {} skipped", self.skipped));
        }

        output.push('\n');

        let duration = if self.total_duration.as_secs() >= 60 {
            format!(
                "{}m {:02}s",
                self.total_duration.as_secs() / 60,
                self.total_duration.as_secs() % 60
            )
        } else {
            format!("{}s", self.total_duration.as_secs())
        };

        output.push_str(&format!(
            "  Total time: {} (parallel execution, {} workers)\n",
            duration, self.parallelism
        ));

        output
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[path = "container_tests_container_00.rs"]
mod tests_extracted;
