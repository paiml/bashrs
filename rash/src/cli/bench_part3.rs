// bench.rs - Scientific benchmarking for shell scripts
// EXTREME TDD implementation - GREEN phase (Issue #12 enhancements)

use crate::linter::lint_shell;
use crate::{Error, Result};
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;
use sysinfo::System;


/// Verify that script produces identical output across runs
fn verify_output_determinism(script: &Path) -> Result<bool> {
    const VERIFICATION_RUNS: usize = 3;
    let mut outputs = Vec::new();

    for _ in 0..VERIFICATION_RUNS {
        let output = Command::new("bash")
            .arg(script)
            .output()
            .map_err(Error::Io)?;
        outputs.push(output);
    }

    // Compare all outputs
    let first_output = outputs.first().ok_or_else(|| {
        Error::Internal("No outputs to compare for determinism verification".to_string())
    })?;
    let first_hash = hash_output(first_output);
    for output in outputs.iter().skip(1) {
        if hash_output(output) != first_hash {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Hash command output for comparison
fn hash_output(output: &Output) -> u64 {
    let mut hasher = DefaultHasher::new();
    output.stdout.hash(&mut hasher);
    output.stderr.hash(&mut hasher);
    hasher.finish()
}

/// Display benchmark results to console
fn display_results(
    results: &[BenchmarkResult],
    environment: &Environment,
    options: &BenchOptions,
) -> Result<()> {
    // Single script results
    if results.len() == 1 {
        let result = results.first().ok_or_else(|| {
            Error::Internal("results.len() == 1 but first() returned None".to_string())
        })?;
        println!("\n📈 Results for {}", result.script);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "  Mean:    {:.2}ms ± {:.2}ms",
            result.statistics.mean_ms, result.statistics.stddev_ms
        );
        println!("  Median:  {:.2}ms", result.statistics.median_ms);
        println!("  Min:     {:.2}ms", result.statistics.min_ms);
        println!("  Max:     {:.2}ms", result.statistics.max_ms);
        println!("  StdDev:  {:.2}ms", result.statistics.stddev_ms);
        println!("  Runs:    {}", result.iterations);

        // Display memory statistics if available
        if let Some(mem) = &result.statistics.memory {
            println!("\n💾 Memory Usage");
            println!("  Mean:    {:.2} KB", mem.mean_kb);
            println!("  Median:  {:.2} KB", mem.median_kb);
            println!("  Min:     {:.2} KB", mem.min_kb);
            println!("  Max:     {:.2} KB", mem.max_kb);
            println!("  Peak:    {:.2} KB", mem.peak_kb);
        }

        if options.show_raw {
            println!("\n  Raw results: {:?}", result.raw_results_ms);
        }
    } else {
        // Comparison results
        display_comparison_results(results)?;
    }

    // Environment info
    println!("\n🖥️  Environment");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  CPU:     {}", environment.cpu);
    println!("  RAM:     {}", environment.ram);
    println!("  OS:      {}", environment.os);
    println!("  Date:    {}", Utc::now().to_rfc3339());

    Ok(())
}

/// Display results in CSV format (Issue #77)
fn display_csv_results(results: &[BenchmarkResult]) -> Result<()> {
    // Check if any result has memory statistics
    let has_memory = results.iter().any(|r| r.statistics.memory.is_some());

    // Print CSV header
    if has_memory {
        println!("script,mean_ms,stddev_ms,median_ms,min_ms,max_ms,memory_mean_kb,memory_max_kb,iterations");
    } else {
        println!("script,mean_ms,stddev_ms,median_ms,min_ms,max_ms,iterations");
    }

    // Find slowest for speedup calculation
    let baseline_mean = results
        .iter()
        .map(|r| r.statistics.mean_ms)
        .fold(0.0f64, |a, b| a.max(b));

    // Print each result as CSV row
    for result in results {
        let _speedup = if baseline_mean > 0.0 {
            baseline_mean / result.statistics.mean_ms
        } else {
            1.0
        };

        if has_memory {
            let mem = result.statistics.memory.as_ref();
            println!(
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{}",
                result.script,
                result.statistics.mean_ms,
                result.statistics.stddev_ms,
                result.statistics.median_ms,
                result.statistics.min_ms,
                result.statistics.max_ms,
                mem.map_or(0.0, |m| m.mean_kb),
                mem.map_or(0.0, |m| m.peak_kb),
                result.iterations,
            );
        } else {
            println!(
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{}",
                result.script,
                result.statistics.mean_ms,
                result.statistics.stddev_ms,
                result.statistics.median_ms,
                result.statistics.min_ms,
                result.statistics.max_ms,
                result.iterations,
            );
        }
    }

    Ok(())
}

/// Display comparison results for multiple scripts
fn display_comparison_results(results: &[BenchmarkResult]) -> Result<()> {
    println!("\n📊 Comparison Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Check if any result has memory statistics
    let has_memory = results.iter().any(|r| r.statistics.memory.is_some());

    if has_memory {
        println!(
            "{:<30} {:>12} {:>15} {:>12} {:>10}",
            "Script", "Mean (ms)", "StdDev (ms)", "Memory (KB)", "Speedup"
        );
    } else {
        println!(
            "{:<30} {:>12} {:>15} {:>10}",
            "Script", "Mean (ms)", "StdDev (ms)", "Speedup"
        );
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Find slowest (baseline)
    let baseline = results
        .iter()
        .max_by(|a, b| {
            a.statistics
                .mean_ms
                .partial_cmp(&b.statistics.mean_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or_else(|| Error::Internal("No results to compare".to_string()))?;

    // Sort by speed (fastest first)
    let mut sorted = results.to_vec();
    sorted.sort_by(|a, b| {
        a.statistics
            .mean_ms
            .partial_cmp(&b.statistics.mean_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, result) in sorted.iter().enumerate() {
        let speedup = baseline.statistics.mean_ms / result.statistics.mean_ms;
        let winner = if i == 0 { " 🏆" } else { "" };

        if has_memory {
            let mem_str = result
                .statistics
                .memory
                .as_ref()
                .map_or_else(|| "N/A".to_string(), |m| format!("{:.2}", m.mean_kb));

            println!(
                "{:<30} {:>12.2} {:>15} {:>12} {:>10.2}x{}",
                truncate_path(&result.script, 30),
                result.statistics.mean_ms,
                format!("± {:.2}", result.statistics.stddev_ms),
                mem_str,
                speedup,
                winner
            );
        } else {
            println!(
                "{:<30} {:>12.2} {:>15} {:>10.2}x{}",
                truncate_path(&result.script, 30),
                result.statistics.mean_ms,
                format!("± {:.2}", result.statistics.stddev_ms),
                speedup,
                winner
            );
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let fastest = sorted
        .first()
        .ok_or_else(|| Error::Internal("No sorted results available".to_string()))?;
    let speedup = baseline.statistics.mean_ms / fastest.statistics.mean_ms;
    println!(
        "\n🏆 Winner: {} ({:.2}x faster than baseline)",
        truncate_path(&fastest.script, 50),
        speedup
    );

    Ok(())
}

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


include!("bench_part3_incl2.rs");
