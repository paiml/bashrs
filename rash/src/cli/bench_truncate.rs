/// Truncate path for display
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    }
}

/// Write JSON output to file
fn write_json_output(output: &BenchmarkOutput, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(output)
        .map_err(|e| Error::Validation(format!("Failed to serialize JSON: {}", e)))?;

    let mut file = fs::File::create(path).map_err(Error::Io)?;
    file.write_all(json.as_bytes()).map_err(Error::Io)?;

    Ok(())
}

// ===== Statistical Helper Functions =====

fn calculate_mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn calculate_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        // Safe: mid > 0 when len is even and > 1
        let lower = sorted.get(mid - 1).copied().unwrap_or(0.0);
        let upper = sorted.get(mid).copied().unwrap_or(0.0);
        f64::midpoint(lower, upper)
    } else {
        sorted.get(mid).copied().unwrap_or(0.0)
    }
}

fn calculate_variance(values: &[f64], mean: f64) -> f64 {
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

// ===== Issue #12 Phase 1: New Statistical Functions =====

/// Calculate Median Absolute Deviation (MAD) - robust to outliers
/// MAD = median(|xi - median(x)|)
fn calculate_mad(values: &[f64]) -> f64 {
    let median = calculate_median(values);
    let absolute_deviations: Vec<f64> = values.iter().map(|v| (v - median).abs()).collect();
    calculate_median(&absolute_deviations)
}

/// Detect outliers using MAD-based method
/// Returns indices of values that are outliers (beyond threshold * MAD from median)
/// Standard threshold is 3.0 (equivalent to ~3 standard deviations)
fn detect_outliers(values: &[f64], threshold: f64) -> Vec<usize> {
    let median = calculate_median(values);
    let mad = calculate_mad(values);

    // Avoid division by zero if MAD is 0 (all values identical)
    if mad == 0.0 {
        return Vec::new();
    }

    values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            let modified_z_score = 0.6745 * (v - median).abs() / mad;
            if modified_z_score > threshold {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Calculate geometric mean (better for ratios and multiplicative relationships)
/// Geometric mean = (x1 * x2 * ... * xn)^(1/n)
fn calculate_geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    // Convert to log space to avoid overflow with large products
    let log_sum: f64 = values.iter().map(|v| v.ln()).sum();
    let log_mean = log_sum / values.len() as f64;
    log_mean.exp()
}

/// Calculate harmonic mean (better for rates and reciprocals)
/// Harmonic mean = n / (1/x1 + 1/x2 + ... + 1/xn)
fn calculate_harmonic_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let reciprocal_sum: f64 = values.iter().map(|v| 1.0 / v).sum();
    values.len() as f64 / reciprocal_sum
}

// ===== Issue #12 Phase 2: Welch's t-test & Regression Detection =====

/// Welch's t-test for comparing two samples with unequal variances
/// Returns the t-statistic
fn welch_t_test(sample1: &[f64], sample2: &[f64]) -> f64 {
    let mean1 = calculate_mean(sample1);
    let mean2 = calculate_mean(sample2);
    let var1 = calculate_variance(sample1, mean1);
    let var2 = calculate_variance(sample2, mean2);
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    // Welch's t-statistic formula
    let numerator = mean1 - mean2;
    let denominator = ((var1 / n1) + (var2 / n2)).sqrt();

    if denominator == 0.0 {
        return 0.0;
    }

    numerator / denominator
}

/// Calculate degrees of freedom for Welch's t-test (Welch-Satterthwaite equation)
fn welch_degrees_of_freedom(sample1: &[f64], sample2: &[f64]) -> f64 {
    let mean1 = calculate_mean(sample1);
    let mean2 = calculate_mean(sample2);
    let var1 = calculate_variance(sample1, mean1);
    let var2 = calculate_variance(sample2, mean2);
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    let numerator = (var1 / n1 + var2 / n2).powi(2);
    let denominator = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);

    if denominator == 0.0 {
        return n1 + n2 - 2.0;
    }

    numerator / denominator
}

/// Approximate p-value from t-statistic and degrees of freedom
/// Uses a simplified approximation suitable for benchmarking
fn approximate_p_value(t_statistic: f64, df: f64) -> f64 {
    let abs_t = t_statistic.abs();

    // For large df (>30), use normal approximation
    if df > 30.0 {
        // Two-tailed test
        let z = abs_t;
        // Approximation of normal CDF
        let p = 1.0 / (1.0 + 0.2316419 * z);
        let d = 0.3989423 * (-z * z / 2.0).exp();
        let prob = d
            * p
            * (0.319381530
                + p * (-0.356563782 + p * (1.781477937 + p * (-1.821255978 + p * 1.330274429))));
        return 2.0 * prob; // Two-tailed
    }

    // For smaller df, use lookup table approximation
    // Critical values for two-tailed test at α=0.05
    let critical_value_05 = if df < 5.0 {
        2.776 // Conservative estimate
    } else if df < 10.0 {
        2.262
    } else if df < 20.0 {
        2.093
    } else {
        2.042
    };

    // Simple approximation: if |t| > critical value, p < 0.05
    if abs_t > critical_value_05 {
        0.01 // Significant
    } else if abs_t > critical_value_05 * 0.7 {
        0.10 // Borderline
    } else {
        0.50 // Not significant
    }
}

/// Check if two samples are statistically significantly different
#[cfg(test)]
fn is_statistically_significant(sample1: &[f64], sample2: &[f64], alpha: f64) -> bool {
    let t_stat = welch_t_test(sample1, sample2);
    let df = welch_degrees_of_freedom(sample1, sample2);
    let p_value = approximate_p_value(t_stat, df);
    p_value < alpha
}

/// Compare two benchmark samples and return comparison results
fn compare_benchmarks(baseline: &[f64], current: &[f64]) -> ComparisonResult {
    let baseline_mean = calculate_mean(baseline);
    let current_mean = calculate_mean(current);
    let speedup = baseline_mean / current_mean;
    let t_statistic = welch_t_test(baseline, current);
    let df = welch_degrees_of_freedom(baseline, current);
    let p_value = approximate_p_value(t_statistic, df);
    let is_significant = p_value < 0.05;

    ComparisonResult {
        speedup,
        t_statistic,
        p_value,
        is_significant,
    }
}

/// Detect performance regression with default 5% threshold
#[cfg(test)]
fn detect_regression(baseline: &[f64], current: &[f64], alpha: f64) -> RegressionResult {
    detect_regression_with_threshold(baseline, current, alpha, 0.05)
}

/// Detect performance regression with custom threshold
/// threshold: Minimum performance degradation to consider (e.g., 0.05 = 5%)
#[cfg(test)]
fn detect_regression_with_threshold(
    baseline: &[f64],
    current: &[f64],
    alpha: f64,
    threshold: f64,
) -> RegressionResult {
    let baseline_mean = calculate_mean(baseline);
    let current_mean = calculate_mean(current);
    let speedup = baseline_mean / current_mean;
    let change_percent = (1.0 - speedup) * 100.0;

    // Check for zero-variance samples (all values identical)
    let baseline_var = calculate_variance(baseline, baseline_mean);
    let current_var = calculate_variance(current, current_mean);

    let is_significant = if baseline_var == 0.0 && current_var == 0.0 {
        // Both samples have no variance - consider significant if means differ
        baseline_mean != current_mean
    } else {
        // Use statistical test for samples with variance
        is_statistically_significant(baseline, current, alpha)
    };

    // Regression criteria:
    // 1. Current is slower (speedup < 1.0)
    // 2. Difference is statistically significant (or deterministic)
    // 3. Performance degradation exceeds threshold
    let is_regression =
        speedup < 1.0 && is_significant && change_percent.abs() > (threshold * 100.0);

    RegressionResult {
        is_regression,
        speedup,
        is_statistically_significant: is_significant,
        change_percent,
    }
}

#[cfg(test)]
#[path = "bench_coverage_tests.rs"]
mod bench_coverage_tests;

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "bench_tests_calculate_me.rs"]
// FIXME(PMAT-238): mod tests_extracted;
