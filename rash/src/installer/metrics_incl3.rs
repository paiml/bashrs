impl MetricsAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a run to the aggregator
    pub fn add_run(&mut self, metrics: InstallerMetrics) {
        self.runs.push(metrics);
    }

    /// Load runs from a directory
    pub fn load_from_dir(&mut self, dir: &std::path::Path) -> std::io::Result<usize> {
        let mut count = 0;
        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(metrics) = serde_json::from_str::<InstallerMetrics>(&content) {
                            self.runs.push(metrics);
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    /// Generate aggregated metrics
    pub fn aggregate(&self) -> AggregatedMetrics {
        if self.runs.is_empty() {
            return AggregatedMetrics::default();
        }

        let total_runs = self.runs.len() as u64;
        let successful_runs = self
            .runs
            .iter()
            .filter(|r| r.outcome == StepOutcome::Success)
            .count() as u64;
        let failed_runs = total_runs - successful_runs;

        let durations: Vec<f64> = self
            .runs
            .iter()
            .map(|r| r.total_duration_ms as f64)
            .collect();
        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;

        let mut sorted_durations = durations.clone();
        sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        #[allow(clippy::manual_is_multiple_of)]
        let median_duration = if sorted_durations.len() % 2 == 0 {
            let mid = sorted_durations.len() / 2;
            let a = sorted_durations.get(mid - 1).copied().unwrap_or(0.0);
            let b = sorted_durations.get(mid).copied().unwrap_or(0.0);
            f64::midpoint(a, b)
        } else {
            sorted_durations
                .get(sorted_durations.len() / 2)
                .copied()
                .unwrap_or(0.0)
        };

        let p95_idx = (sorted_durations.len() as f64 * 0.95) as usize;
        let p95_duration = sorted_durations.get(p95_idx).copied().unwrap_or(0.0);

        // Aggregate per-step metrics
        let mut step_data: HashMap<String, Vec<&StepMetrics>> = HashMap::new();
        for run in &self.runs {
            for step in &run.steps {
                step_data
                    .entry(step.step_id.clone())
                    .or_default()
                    .push(step);
            }
        }

        let step_aggregates: HashMap<String, StepAggregate> = step_data
            .into_iter()
            .map(|(step_id, metrics)| {
                let total = metrics.len() as u64;
                let successful = metrics
                    .iter()
                    .filter(|m| m.outcome == StepOutcome::Success)
                    .count() as u64;
                let failed = total - successful;

                let durations: Vec<f64> = metrics.iter().map(|m| m.duration_ms as f64).collect();
                let avg_dur = durations.iter().sum::<f64>() / durations.len() as f64;
                let min_dur = durations.iter().copied().fold(f64::INFINITY, f64::min);
                let max_dur = durations.iter().copied().fold(0.0_f64, f64::max);

                let avg_retries =
                    metrics.iter().map(|m| m.retry_count as f64).sum::<f64>() / total as f64;

                (
                    step_id.clone(),
                    StepAggregate {
                        step_id,
                        total_executions: total,
                        successful_executions: successful,
                        failed_executions: failed,
                        success_rate: if total > 0 {
                            successful as f64 / total as f64
                        } else {
                            0.0
                        },
                        avg_duration_ms: avg_dur,
                        min_duration_ms: if min_dur.is_infinite() { 0.0 } else { min_dur },
                        max_duration_ms: max_dur,
                        avg_retries,
                    },
                )
            })
            .collect();

        AggregatedMetrics {
            total_runs,
            successful_runs,
            failed_runs,
            success_rate: if total_runs > 0 {
                successful_runs as f64 / total_runs as f64
            } else {
                0.0
            },
            avg_duration_ms: avg_duration,
            median_duration_ms: median_duration,
            p95_duration_ms: p95_duration,
            step_aggregates,
        }
    }

    /// Get runs count
    pub fn runs_count(&self) -> usize {
        self.runs.len()
    }

    /// Generate Kaizen improvement report
    pub fn kaizen_report(&self) -> KaizenReport {
        let aggregates = self.aggregate();
        let mut improvements = Vec::new();
        let mut bottlenecks = Vec::new();

        // Identify bottlenecks (steps with high failure rate or long duration)
        for (step_id, agg) in &aggregates.step_aggregates {
            if agg.success_rate < 0.95 {
                bottlenecks.push(format!(
                    "Step '{}' has {:.1}% success rate (target: 95%)",
                    step_id,
                    agg.success_rate * 100.0
                ));
                improvements.push(format!(
                    "Investigate failures in step '{}' - {} failures out of {}",
                    step_id, agg.failed_executions, agg.total_executions
                ));
            }

            if agg.avg_duration_ms > 60000.0 {
                // > 1 minute
                bottlenecks.push(format!(
                    "Step '{}' takes {:.1}s on average",
                    step_id,
                    agg.avg_duration_ms / 1000.0
                ));
                improvements.push(format!(
                    "Consider optimizing step '{}' or adding parallelization",
                    step_id
                ));
            }

            if agg.avg_retries > 0.5 {
                improvements.push(format!(
                    "Step '{}' has high retry rate ({:.1}) - check preconditions",
                    step_id, agg.avg_retries
                ));
            }
        }

        // Overall health check
        if aggregates.success_rate < 0.9 {
            improvements.push(format!(
                "Overall success rate is {:.1}% - needs improvement",
                aggregates.success_rate * 100.0
            ));
        }

        KaizenReport {
            overall_health: if aggregates.success_rate >= 0.95 {
                "Excellent"
            } else if aggregates.success_rate >= 0.9 {
                "Good"
            } else if aggregates.success_rate >= 0.8 {
                "Needs Improvement"
            } else {
                "Critical"
            }
            .to_string(),
            success_rate: aggregates.success_rate,
            bottlenecks,
            improvements,
            metrics_summary: aggregates,
        }
    }
}


include!("metrics_incl2.rs");
