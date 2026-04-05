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

const VERSION: &str = "1.0.0";
const DEFAULT_WARMUP: usize = 3;
const DEFAULT_ITERATIONS: usize = 10;

/// Benchmark options
#[derive(Debug, Clone)]
pub struct BenchOptions {
    pub scripts: Vec<PathBuf>,
    pub warmup: usize,
    pub iterations: usize,
    pub output: Option<PathBuf>,
    pub strict: bool,
    pub verify_determinism: bool,
    pub show_raw: bool,
    pub quiet: bool,
    pub measure_memory: bool,
    /// Output results in CSV format (Issue #77)
    pub csv: bool,
    /// Disable ANSI colors (Issue #77)
    pub no_color: bool,
}

impl BenchOptions {
    pub fn new(scripts: Vec<PathBuf>) -> Self {
        Self {
            scripts,
            warmup: DEFAULT_WARMUP,
            iterations: DEFAULT_ITERATIONS,
            output: None,
            strict: false,
            verify_determinism: false,
            show_raw: false,
            quiet: false,
            measure_memory: false,
            csv: false,
            no_color: false,
        }
    }
}

/// Environment metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Environment {
    pub cpu: String,
    pub ram: String,
    pub os: String,
    pub hostname: String,
    pub bashrs_version: String,
}

impl Environment {
    pub fn capture() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu = sys
            .cpus()
            .first()
            .map_or_else(|| "unknown".to_string(), |cpu| cpu.brand().to_string());

        let ram = format!("{}GB", sys.total_memory() / 1024 / 1024 / 1024);

        let os = format!(
            "{} {}",
            System::name().unwrap_or_else(|| "unknown".to_string()),
            System::os_version().unwrap_or_else(|| "unknown".to_string())
        );

        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());

        let bashrs_version = env!("CARGO_PKG_VERSION").to_string();

        Self {
            cpu,
            ram,
            os,
            hostname,
            bashrs_version,
        }
    }
}

/// Memory measurement statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryStatistics {
    pub mean_kb: f64,
    pub median_kb: f64,
    pub min_kb: f64,
    pub max_kb: f64,
    pub peak_kb: f64,
}

/// Statistics for benchmark results (Issue #12: Enhanced with MAD, geometric/harmonic means)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Statistics {
    pub mean_ms: f64,
    pub median_ms: f64,
    pub stddev_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub variance_ms: f64,
    /// Median Absolute Deviation (robust to outliers)
    pub mad_ms: f64,
    /// Geometric mean (better for ratios/speedups)
    pub geometric_mean_ms: f64,
    /// Harmonic mean (better for rates/throughput)
    pub harmonic_mean_ms: f64,
    /// Indices of detected outliers
    pub outlier_indices: Vec<usize>,
    pub memory: Option<MemoryStatistics>,
}

impl Statistics {
    pub fn calculate(results: &[f64]) -> Self {
        Self::calculate_with_memory(results, None)
    }

    pub fn calculate_with_memory(results: &[f64], memory_results: Option<&[f64]>) -> Self {
        let mean = calculate_mean(results);
        let median = calculate_median(results);
        let variance = calculate_variance(results, mean);
        let stddev = variance.sqrt();
        let min = results.iter().copied().fold(f64::INFINITY, f64::min);
        let max = results.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Issue #12: Calculate MAD and detect outliers
        let mad = calculate_mad(results);
        let outlier_indices = detect_outliers(results, 3.0); // 3.0 MAD threshold (standard)

        // Issue #12: Calculate geometric and harmonic means
        let geometric_mean = calculate_geometric_mean(results);
        let harmonic_mean = calculate_harmonic_mean(results);

        let memory = memory_results.map(|mem_results| {
            let mean_kb = calculate_mean(mem_results);
            let median_kb = calculate_median(mem_results);
            let min_kb = mem_results.iter().copied().fold(f64::INFINITY, f64::min);
            let max_kb = mem_results
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let peak_kb = max_kb;

            MemoryStatistics {
                mean_kb,
                median_kb,
                min_kb,
                max_kb,
                peak_kb,
            }
        });

        Self {
            mean_ms: mean,
            median_ms: median,
            stddev_ms: stddev,
            min_ms: min,
            max_ms: max,
            variance_ms: variance,
            mad_ms: mad,
            geometric_mean_ms: geometric_mean,
            harmonic_mean_ms: harmonic_mean,
            outlier_indices,
            memory,
        }
    }
}

impl MemoryStatistics {
    pub fn calculate(memory_kb: &[f64]) -> Self {
        let mean_kb = calculate_mean(memory_kb);
        let median_kb = calculate_median(memory_kb);
        let min_kb = memory_kb.iter().copied().fold(f64::INFINITY, f64::min);
        let max_kb = memory_kb.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let peak_kb = max_kb;

        Self {
            mean_kb,
            median_kb,
            min_kb,
            max_kb,
            peak_kb,
        }
    }
}

/// Quality check results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Quality {
    pub lint_passed: bool,
    pub determinism_score: f64,
    pub output_identical: bool,
}

/// Single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BenchmarkResult {
    pub script: String,
    pub iterations: usize,
    pub warmup: usize,
    pub statistics: Statistics,
    pub raw_results_ms: Vec<f64>,
    pub quality: Quality,
}

/// Complete benchmark output (with JSON schema support - Issue #12)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BenchmarkOutput {
    pub version: String,
    pub timestamp: String,
    pub environment: Environment,
    pub benchmarks: Vec<BenchmarkResult>,
}

/// Comparison result between two benchmarks (Issue #12 Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComparisonResult {
    /// Speedup factor (baseline_mean / current_mean)
    pub speedup: f64,
    /// Welch's t-statistic
    pub t_statistic: f64,
    /// P-value (probability of observing this difference by chance)
    pub p_value: f64,
    /// Whether the difference is statistically significant
    pub is_significant: bool,
}

impl ComparisonResult {
    /// Create comparison from two Statistics objects
    pub fn from_statistics(baseline: &Statistics, current: &Statistics) -> Self {
        let baseline_samples = vec![baseline.mean_ms; 10]; // Approximate
        let current_samples = vec![current.mean_ms; 10];
        compare_benchmarks(&baseline_samples, &current_samples)
    }
}

/// Regression detection result (Issue #12 Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegressionResult {
    /// Whether a performance regression was detected
    pub is_regression: bool,
    /// Speedup factor (baseline_mean / current_mean)
    pub speedup: f64,
    /// Whether the difference is statistically significant
    pub is_statistically_significant: bool,
    /// Performance change percentage (-20.0 means 20% slower)
    pub change_percent: f64,
}

/// Main benchmark command entry point
pub fn bench_command(options: BenchOptions) -> Result<()> {
    // Validate inputs
    validate_options(&options)?;

    // Capture environment
    let environment = Environment::capture();

    // Run benchmarks for each script
    let mut results = Vec::new();
    for script in &options.scripts {
        let result = benchmark_single_script(script, &options)?;
        results.push(result);
    }

    // Generate output
    let output = BenchmarkOutput {
        version: VERSION.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        environment,
        benchmarks: results.clone(),
    };

    // Display results
    if options.csv {
        // Issue #77: CSV output
        display_csv_results(&results)?;
    } else if !options.quiet {
        display_results(&results, &output.environment, &options)?;
    }

    // Write JSON output if requested
    if let Some(output_path) = &options.output {
        write_json_output(&output, output_path)?;
    }

    Ok(())
}

/// Validate benchmark options
fn validate_options(options: &BenchOptions) -> Result<()> {
    if options.scripts.is_empty() {
        return Err(Error::Validation(
            "No scripts provided for benchmarking".to_string(),
        ));
    }

    if options.iterations == 0 {
        return Err(Error::Validation(
            "Iterations must be at least 1".to_string(),
        ));
    }

    for script in &options.scripts {
        if !script.exists() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Script not found: {}", script.display()),
            )));
        }
    }

    Ok(())
}

/// Run warmup iterations
fn run_warmup(script: &Path, options: &BenchOptions) -> Result<()> {
    if !options.quiet {
        println!("\n🔥 Warmup ({} iterations)...", options.warmup);
    }
    for i in 1..=options.warmup {
        let time_ms = execute_and_time(script)?;
        if !options.quiet {
            println!("  ✓ Iteration {}: {:.2}ms", i, time_ms);
        }
    }
    Ok(())
}

/// Run measured iterations, returning time and optional memory results
fn run_measured_iterations(script: &Path, options: &BenchOptions) -> Result<(Vec<f64>, Vec<f64>)> {
    if !options.quiet {
        let mem_str = if options.measure_memory {
            " + memory"
        } else {
            ""
        };
        println!(
            "\n⏱️  Measuring ({} iterations{})...",
            options.iterations, mem_str
        );
    }
    let mut results = Vec::new();
    let mut memory_results = Vec::new();
    for i in 1..=options.iterations {
        let (time_ms, memory_kb) = if options.measure_memory {
            execute_and_time_with_memory(script)?
        } else {
            (execute_and_time(script)?, 0.0)
        };
        results.push(time_ms);
        if options.measure_memory {
            memory_results.push(memory_kb);
        }
        if !options.quiet {
            if options.measure_memory {
                println!("  ✓ Iteration {}: {:.2}ms, {:.2} KB", i, time_ms, memory_kb);
            } else {
                println!("  ✓ Iteration {}: {:.2}ms", i, time_ms);
            }
        }
    }
    Ok((results, memory_results))
}

/// Benchmark a single script
fn benchmark_single_script(script: &Path, options: &BenchOptions) -> Result<BenchmarkResult> {
    if !options.quiet {
        println!("📊 Benchmarking: {}", script.display());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    let quality = if options.strict || options.verify_determinism {
        run_quality_gates(script, options)?
    } else {
        Quality {
            lint_passed: true,
            determinism_score: 1.0,
            output_identical: true,
        }
    };

    run_warmup(script, options)?;
    let (results, memory_results) = run_measured_iterations(script, options)?;

    let statistics = if options.measure_memory {
        Statistics::calculate_with_memory(&results, Some(&memory_results))
    } else {
        Statistics::calculate(&results)
    };

    Ok(BenchmarkResult {
        script: script.to_string_lossy().to_string(),
        iterations: options.iterations,
        warmup: options.warmup,
        statistics,
        raw_results_ms: results,
        quality,
    })
}

/// Execute script and measure time in milliseconds
fn execute_and_time(script: &Path) -> Result<f64> {
    let start = Instant::now();

    Command::new("bash")
        .arg(script)
        .output()
        .map_err(Error::Io)?;

    let elapsed = start.elapsed();
    Ok(elapsed.as_secs_f64() * 1000.0)
}

/// Execute script and measure both time and memory usage
/// Returns (time_ms, memory_kb)
fn execute_and_time_with_memory(script: &Path) -> Result<(f64, f64)> {
    let start = Instant::now();

    // Use /usr/bin/time to measure memory
    // -f "%M" outputs maximum resident set size in KB
    let output = Command::new("/usr/bin/time")
        .arg("-f")
        .arg("%M")
        .arg("bash")
        .arg(script)
        .output()
        .map_err(Error::Io)?;

    let elapsed = start.elapsed();
    let time_ms = elapsed.as_secs_f64() * 1000.0;

    // Parse memory usage from stderr (time outputs to stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let memory_kb = stderr
        .lines()
        .last()
        .and_then(|line| line.trim().parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok((time_ms, memory_kb))
}

/// Run quality gates on script
fn run_quality_gates(script: &Path, options: &BenchOptions) -> Result<Quality> {
    let mut quality = Quality {
        lint_passed: true,
        determinism_score: 1.0,
        output_identical: true,
    };

    // Strict mode: Run linter
    if options.strict {
        let source = fs::read_to_string(script).map_err(Error::Io)?;
        let lint_result = lint_shell(&source);

        if !lint_result.diagnostics.is_empty() {
            eprintln!("\n❌ Quality gate failed: Lint check");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("\nFound {} issue(s):", lint_result.diagnostics.len());
            for diag in &lint_result.diagnostics {
                eprintln!("  {} [{}]", diag.message, diag.code);
            }
            eprintln!("\nRun 'bashrs lint {}' for details.", script.display());
            // Note: Don't need to set quality.lint_passed = false here since we return early
            return Err(Error::Validation("Linting failed".to_string()));
        }
        quality.lint_passed = true;
    }

    // Verify determinism
    if options.verify_determinism {
        quality.output_identical = verify_output_determinism(script)?;
        if !quality.output_identical {
            eprintln!("\n❌ Determinism verification failed");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("\nOutput differs between runs.");
            eprintln!("This indicates non-deterministic behavior in the script.");
            eprintln!("Common causes:");
            eprintln!("  - $RANDOM usage");
            eprintln!("  - Timestamp generation");
            eprintln!("  - Uninitialized variables");
            eprintln!("  - Race conditions");
            return Err(Error::Validation(
                "Non-deterministic output detected".to_string(),
            ));
        } else if !options.quiet {
            println!("\n✓ Determinism verified");
        }
    }

    Ok(quality)
}

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

#[cfg(test)]
#[path = "bench_tests_extracted.rs"]
mod tests_extracted;
