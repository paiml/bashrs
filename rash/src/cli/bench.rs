// bench.rs - Scientific benchmarking for shell scripts
// EXTREME TDD implementation - GREEN phase

use crate::linter::lint_shell;
use crate::{Error, Result};
use chrono::Utc;
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
        }
    }
}

/// Environment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "unknown".to_string());

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

/// Statistics for benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub mean_ms: f64,
    pub median_ms: f64,
    pub stddev_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub variance_ms: f64,
}

impl Statistics {
    pub fn calculate(results: &[f64]) -> Self {
        let mean = calculate_mean(results);
        let median = calculate_median(results);
        let variance = calculate_variance(results, mean);
        let stddev = variance.sqrt();
        let min = results.iter().copied().fold(f64::INFINITY, f64::min);
        let max = results.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        Self {
            mean_ms: mean,
            median_ms: median,
            stddev_ms: stddev,
            min_ms: min,
            max_ms: max,
            variance_ms: variance,
        }
    }
}

/// Quality check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quality {
    pub lint_passed: bool,
    pub determinism_score: f64,
    pub output_identical: bool,
}

/// Single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub script: String,
    pub iterations: usize,
    pub warmup: usize,
    pub statistics: Statistics,
    pub raw_results_ms: Vec<f64>,
    pub quality: Quality,
}

/// Complete benchmark output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkOutput {
    pub version: String,
    pub timestamp: String,
    pub environment: Environment,
    pub benchmarks: Vec<BenchmarkResult>,
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
    if !options.quiet {
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

/// Benchmark a single script
fn benchmark_single_script(script: &Path, options: &BenchOptions) -> Result<BenchmarkResult> {
    if !options.quiet {
        println!("📊 Benchmarking: {}", script.display());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    // Quality gates (if strict mode)
    let quality = if options.strict || options.verify_determinism {
        run_quality_gates(script, options)?
    } else {
        Quality {
            lint_passed: true,
            determinism_score: 1.0,
            output_identical: true,
        }
    };

    // Warmup runs
    if !options.quiet {
        println!("\n🔥 Warmup ({} iterations)...", options.warmup);
    }
    for i in 1..=options.warmup {
        let time_ms = execute_and_time(script)?;
        if !options.quiet {
            println!("  ✓ Iteration {}: {:.2}ms", i, time_ms);
        }
    }

    // Measured runs
    if !options.quiet {
        println!("\n⏱️  Measuring ({} iterations)...", options.iterations);
    }
    let mut results = Vec::new();
    for i in 1..=options.iterations {
        let time_ms = execute_and_time(script)?;
        results.push(time_ms);
        if !options.quiet {
            println!("  ✓ Iteration {}: {:.2}ms", i, time_ms);
        }
    }

    // Calculate statistics
    let statistics = Statistics::calculate(&results);

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
            quality.lint_passed = false;
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
    let first_hash = hash_output(&outputs[0]);
    for output in &outputs[1..] {
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
        let result = &results[0];
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

/// Display comparison results for multiple scripts
fn display_comparison_results(results: &[BenchmarkResult]) -> Result<()> {
    println!("\n📊 Comparison Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!(
        "{:<30} {:>12} {:>15} {:>10}",
        "Script", "Mean (ms)", "StdDev (ms)", "Speedup"
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Find slowest (baseline)
    let baseline = results
        .iter()
        .max_by(|a, b| {
            a.statistics
                .mean_ms
                .partial_cmp(&b.statistics.mean_ms)
                .unwrap()
        })
        .unwrap();

    // Sort by speed (fastest first)
    let mut sorted = results.to_vec();
    sorted.sort_by(|a, b| {
        a.statistics
            .mean_ms
            .partial_cmp(&b.statistics.mean_ms)
            .unwrap()
    });

    for (i, result) in sorted.iter().enumerate() {
        let speedup = baseline.statistics.mean_ms / result.statistics.mean_ms;
        let winner = if i == 0 { " 🏆" } else { "" };

        println!(
            "{:<30} {:>12.2} {:>15} {:>10.2}x{}",
            truncate_path(&result.script, 30),
            result.statistics.mean_ms,
            format!("± {:.2}", result.statistics.stddev_ms),
            speedup,
            winner
        );
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let fastest = &sorted[0];
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
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn calculate_variance(values: &[f64], mean: f64) -> f64 {
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_mean(&values), 3.0);
    }

    #[test]
    fn test_calculate_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_median(&values), 3.0);
    }

    #[test]
    fn test_calculate_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_median(&values), 2.5);
    }

    #[test]
    fn test_calculate_variance() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = 3.0;
        assert_eq!(calculate_variance(&values, mean), 2.0);
    }

    #[test]
    fn test_statistics_calculate() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::calculate(&values);
        assert_eq!(stats.mean_ms, 3.0);
        assert_eq!(stats.median_ms, 3.0);
        assert_eq!(stats.min_ms, 1.0);
        assert_eq!(stats.max_ms, 5.0);
    }

    #[test]
    fn test_truncate_path() {
        let path = "/very/long/path/to/some/script.sh";
        // max_len=20 -> "..." (3) + last 17 chars = "...to/some/script.sh" (20 total)
        assert_eq!(truncate_path(path, 20), "...to/some/script.sh");
        assert_eq!(truncate_path("short.sh", 20), "short.sh");
    }
}
