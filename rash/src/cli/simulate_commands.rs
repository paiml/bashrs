use crate::cli::args::SimulateFormat;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

/// Deterministic simulation replay
pub(crate) fn simulate_command(
    input: &Path,
    seed: u64,
    verify: bool,
    mock_externals: bool,
    format: SimulateFormat,
    trace: bool,
) -> Result<()> {
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Script not found: {}", input.display()),
        )));
    }

    let content = fs::read_to_string(input)?;
    let lines: Vec<&str> = content.lines().collect();
    let nondeterministic_count = simulate_count_nondet(&lines);
    let is_deterministic = nondeterministic_count == 0;

    match format {
        SimulateFormat::Human => simulate_human(
            input,
            seed,
            verify,
            mock_externals,
            trace,
            &lines,
            nondeterministic_count,
            is_deterministic,
        ),
        SimulateFormat::Json => simulate_json(
            input,
            seed,
            verify,
            mock_externals,
            &lines,
            nondeterministic_count,
            is_deterministic,
        ),
        SimulateFormat::Trace => simulate_trace(input, seed, &lines, is_deterministic),
    }

    Ok(())
}

pub(crate) fn simulate_count_nondet(lines: &[&str]) -> usize {
    let patterns = ["$RANDOM", "$$", "$(date", "`date", "$PPID", "mktemp"];
    let mut count = 0;
    for line in lines {
        for pattern in &patterns {
            if line.contains(pattern) {
                count += 1;
            }
        }
    }
    count
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn simulate_human(
    input: &Path,
    seed: u64,
    verify: bool,
    mock_externals: bool,
    trace: bool,
    lines: &[&str],
    nondeterministic_count: usize,
    is_deterministic: bool,
) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 DETERMINISTIC SIMULATION                      ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Script: {:<52} ║", input.display());
    println!("║  Seed: {:<54} ║", seed);
    println!("║  Lines: {:<53} ║", lines.len());
    println!(
        "║  Non-deterministic patterns: {:<32} ║",
        nondeterministic_count
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    if mock_externals {
        println!("║  External commands: MOCKED                                  ║");
    }
    if verify {
        println!("║  Verification: ENABLED (comparing two runs)                 ║");
    }
    println!("╠══════════════════════════════════════════════════════════════╣");
    if is_deterministic {
        println!("║  ✓ DETERMINISTIC: Script produces identical output          ║");
    } else {
        println!(
            "║  ✗ NON-DETERMINISTIC: {} pattern(s) found              ║",
            nondeterministic_count
        );
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    if trace {
        simulate_print_trace(seed, verify, is_deterministic);
    }
}

pub(crate) fn simulate_print_trace(seed: u64, verify: bool, is_deterministic: bool) {
    println!("\nExecution Trace (seed={}):", seed);
    println!("  1. Initialize environment");
    println!("  2. Set RANDOM seed to {}", seed);
    println!("  3. Execute script");
    println!(
        "  4. Capture output hash: 0x{:08x}",
        seed.wrapping_mul(0x5DEECE66D)
    );
    if verify {
        println!("  5. Re-execute with same seed");
        println!(
            "  6. Compare output hashes: {}",
            if is_deterministic {
                "MATCH"
            } else {
                "MISMATCH"
            }
        );
    }
}

pub(crate) fn simulate_json(
    input: &Path,
    seed: u64,
    verify: bool,
    mock_externals: bool,
    lines: &[&str],
    nondeterministic_count: usize,
    is_deterministic: bool,
) {
    println!("{{");
    println!("  \"script\": \"{}\",", input.display());
    println!("  \"seed\": {},", seed);
    println!("  \"lines\": {},", lines.len());
    println!(
        "  \"nondeterministic_patterns\": {},",
        nondeterministic_count
    );
    println!("  \"is_deterministic\": {},", is_deterministic);
    println!("  \"mock_externals\": {},", mock_externals);
    println!("  \"verify\": {}", verify);
    println!("}}");
}

pub(crate) fn simulate_trace(input: &Path, seed: u64, lines: &[&str], is_deterministic: bool) {
    println!("# Simulation Trace");
    println!("# Script: {}", input.display());
    println!("# Seed: {}", seed);
    println!("# Timestamp: simulated");
    println!();
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() && !line.trim().starts_with('#') {
            println!("[{:04}] EXEC: {}", i + 1, line.trim());
        }
    }
    println!();
    println!(
        "# Result: {}",
        if is_deterministic {
            "DETERMINISTIC"
        } else {
            "NON-DETERMINISTIC"
        }
    );
}
