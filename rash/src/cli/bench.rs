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

include!("bench_incl2.rs");
