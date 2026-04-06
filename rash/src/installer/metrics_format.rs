/// Kaizen-style improvement report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaizenReport {
    /// Overall health assessment
    pub overall_health: String,
    /// Success rate
    pub success_rate: f64,
    /// Identified bottlenecks
    pub bottlenecks: Vec<String>,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Full metrics summary
    pub metrics_summary: AggregatedMetrics,
}

/// Format metrics as human-readable report
pub fn format_metrics_report(metrics: &InstallerMetrics) -> String {
    let mut report = String::new();

    report.push_str(&format!(
        "Installer Metrics Report: {} v{}\n",
        metrics.installer_name, metrics.installer_version
    ));
    report.push_str(&format!("Run ID: {}\n", metrics.run_id));
    report.push_str(&format!("Started: {}\n", metrics.started_at));
    if let Some(ref ended) = metrics.ended_at {
        report.push_str(&format!("Ended: {}\n", ended));
    }
    report.push_str(&format!(
        "Duration: {:.2}s\n",
        metrics.total_duration_ms as f64 / 1000.0
    ));
    report.push_str(&format!("Outcome: {:?}\n\n", metrics.outcome));

    report.push_str("Steps:\n");
    for step in &metrics.steps {
        let status = match step.outcome {
            StepOutcome::Success => "✓",
            StepOutcome::Failed => "✗",
            StepOutcome::Skipped => "⊘",
            StepOutcome::Timeout => "⏱",
            StepOutcome::Cancelled => "⊗",
        };
        report.push_str(&format!(
            "  {} {} ({:.2}s)",
            status,
            step.step_name,
            step.duration_ms as f64 / 1000.0
        ));
        if step.retry_count > 0 {
            report.push_str(&format!(" [retries: {}]", step.retry_count));
        }
        if let Some(ref err) = step.error_message {
            report.push_str(&format!(" - {}", err));
        }
        report.push('\n');
    }

    report
}

// Helper functions

fn generate_run_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("run-{:x}", timestamp)
}

fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn detect_environment() -> EnvironmentInfo {
    EnvironmentInfo {
        os: std::env::consts::OS.to_string(),
        os_version: String::new(), // Would need platform-specific code
        arch: std::env::consts::ARCH.to_string(),
        hostname_hash: None,
    }
}

#[cfg(test)]
#[path = "metrics_tests_metrics_001.rs"]
mod tests_extracted;
