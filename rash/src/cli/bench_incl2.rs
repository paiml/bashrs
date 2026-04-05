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


include!("bench_incl2_incl2.rs");
