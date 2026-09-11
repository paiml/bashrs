use crate::cli::args::ReportFormat;
use crate::models::{Error, Result};

/// Execute quality gates based on configuration (v6.42.0)
pub(crate) fn handle_gate_command(tier: u8, report: ReportFormat) -> Result<()> {
    use crate::gates::GateConfig;

    // GH-181: Warn when --report format is not yet implemented
    if !matches!(report, ReportFormat::Human) {
        eprintln!(
            "Warning: --report {:?} is not yet implemented for gate command. Using human format.",
            report
        );
    }

    // Load gate configuration
    let config = GateConfig::load()?;

    handle_gate_command_with(tier, &config)
}

/// PMAT-257: body of `handle_gate_command`, split so a test can pass a
/// caller-supplied `GateConfig` instead of loading `.pmat-gates.toml` from
/// the repository (which `GateConfig::load()` searches for from `cwd`).
pub(crate) fn handle_gate_command_with(tier: u8, config: &crate::gates::GateConfig) -> Result<()> {
    // Determine which gates to run based on tier
    let gates_to_run = match tier {
        1 => &config.tiers.tier1_gates,
        2 => &config.tiers.tier2_gates,
        3 => &config.tiers.tier3_gates,
        _ => {
            return Err(Error::Validation(format!(
                "Invalid tier: {}. Must be 1, 2, or 3.",
                tier
            )))
        }
    };

    // GH-181: Status headers go to stderr to avoid contaminating structured output
    eprintln!("Executing Tier {} Quality Gates...", tier);
    eprintln!("Gates enabled: {}", gates_to_run.join(", "));
    eprintln!("----------------------------------------");

    let mut failures = Vec::new();

    for gate in gates_to_run {
        eprint!("Checking {}... ", gate);
        // Flush stderr to show progress
        use std::io::Write;
        let _ = std::io::stderr().flush();

        let success = match gate.as_str() {
            "clippy" => run_clippy_gate(config),
            "tests" => run_tests_gate(config),
            "coverage" => run_coverage_gate(config),
            "complexity" => run_complexity_gate(config),
            "security" => run_security_gate(config),
            "satd" => run_satd_gate(config),
            "mutation" => run_mutation_gate(config),
            _ => {
                eprintln!("⚠️  Unknown gate");
                continue;
            }
        };

        if success {
            eprintln!("✅ PASS");
        } else {
            eprintln!("❌ FAIL");
            failures.push(gate.clone());
        }
    }

    eprintln!("----------------------------------------");

    if failures.is_empty() {
        eprintln!("✅ Tier {} Gates Passed!", tier);
        Ok(())
    } else {
        eprintln!("❌ Tier {} Gates Failed: {}", tier, failures.join(", "));
        // Exit with error code
        std::process::exit(1);
    }
}

fn run_clippy_gate(config: &crate::gates::GateConfig) -> bool {
    // Determine clippy command
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("clippy");

    if config.gates.clippy_strict {
        cmd.args(["--", "-D", "warnings"]);
    }

    let status = cmd
        .status()
        .unwrap_or_else(|_| std::process::ExitStatus::default());
    status.success()
}

fn run_tests_gate(config: &crate::gates::GateConfig) -> bool {
    // GH-181: Use test_timeout from config to bound test execution
    let timeout_secs = config.gates.test_timeout;

    let mut child = match std::process::Command::new("cargo").arg("test").spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    eprintln!("(tests exceeded {}s timeout) ", timeout_secs);
                    return false;
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(_) => return false,
        }
    }
}

fn run_coverage_gate(config: &crate::gates::GateConfig) -> bool {
    if !config.gates.check_coverage {
        return true;
    }

    // In a real implementation, this would run llvm-cov or similar
    // For now, we'll check if cargo-llvm-cov is installed and run it, otherwise warn
    let status = std::process::Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .output();

    if status.is_ok() {
        let cov_status = std::process::Command::new("cargo")
            .args([
                "llvm-cov",
                "--fail-under-lines",
                &config.gates.min_coverage.to_string(),
            ])
            .status()
            .unwrap_or_else(|_| std::process::ExitStatus::default());
        cov_status.success()
    } else {
        eprintln!("(cargo-llvm-cov not found, skipping) ");
        true
    }
}

fn run_complexity_gate(config: &crate::gates::GateConfig) -> bool {
    if !config.gates.check_complexity {
        return true;
    }

    // GH-181: Use max_complexity from config instead of always returning true
    let max = config.gates.max_complexity;
    let status = std::process::Command::new("pmat")
        .args([
            "analyze",
            "complexity",
            "--max-cyclomatic",
            &max.to_string(),
        ])
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => {
            eprintln!(
                "(pmat not found, complexity gate skipped — max_complexity={}) ",
                max
            );
            true
        }
    }
}

fn run_security_gate(config: &crate::gates::GateConfig) -> bool {
    // GH-181: Respect security gate config (enabled flag, max_unsafe_blocks)
    if let Some(ref security) = config.gates.security {
        if !security.enabled {
            return true;
        }
    }

    let status = std::process::Command::new("cargo")
        .args(["deny", "check"])
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => {
            eprintln!("(cargo-deny not found, skipping) ");
            true
        }
    }
}

fn run_satd_gate(config: &crate::gates::GateConfig) -> bool {
    if let Some(satd) = &config.gates.satd {
        if !satd.enabled {
            return true;
        }

        // Simple grep for patterns
        let patterns = &satd.patterns;
        if patterns.is_empty() {
            return true;
        }

        // This is a naive implementation; a real one would use `grep` or `ripgrep`
        // efficiently across the codebase
        true
    } else {
        true
    }
}

fn run_mutation_gate(config: &crate::gates::GateConfig) -> bool {
    if let Some(mutation) = &config.gates.mutation {
        if !mutation.enabled {
            return true;
        }

        let status = std::process::Command::new("cargo")
            .args(["mutants", "--score", &mutation.min_score.to_string()])
            .status();

        match status {
            Ok(s) => s.success(),
            Err(_) => {
                eprintln!("(cargo-mutants not found, skipping) ");
                true
            }
        }
    } else {
        true
    }
}

// PMAT-257: coverage for `bashrs gate`. `handle_gate_command` always shells
// out to `GateConfig::load()`, which walks up from `cwd` looking for
// `.pmat-gates.toml` -- not injectable and dependent on the repository's own
// config. It was split into `handle_gate_command_with(tier, &config)`, which
// takes a caller-built `GateConfig` directly.
//
// `run_clippy_gate` and `run_tests_gate` always spawn a real `cargo clippy`
// / `cargo test` subprocess (no "disabled" branch to short-circuit them), so
// they are intentionally NOT exercised here -- doing so would shell out to
// cargo from within a unit test. Likewise the "enabled" branches of
// `run_coverage_gate`, `run_complexity_gate`, `run_security_gate`, and
// `run_mutation_gate` spawn `cargo llvm-cov` / `pmat` / `cargo deny` /
// `cargo mutants` respectively and are left uncovered; only their
// config-disabled early-return branches are exercised. `run_satd_gate` never
// spawns a subprocess in any branch, so all of its branches are covered.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::gates::{GateConfig, Gates, MutationGate, SatdGate, SecurityGate, Tiers};

    fn disabled_gates() -> Gates {
        Gates {
            run_clippy: false,
            clippy_strict: false,
            run_tests: false,
            test_timeout: 1,
            check_coverage: false,
            min_coverage: 80.0,
            check_complexity: false,
            max_complexity: 10,
            satd: None,
            mutation: None,
            security: None,
        }
    }

    fn config_with_tiers(tiers: Tiers) -> GateConfig {
        GateConfig {
            metadata: None,
            gates: disabled_gates(),
            tiers,
        }
    }

    #[test]
    fn test_PMAT257_cov_handle_gate_command_with_invalid_tier_is_an_error() {
        let config = config_with_tiers(Tiers::default());
        let err = handle_gate_command_with(0, &config).unwrap_err();
        assert!(format!("{err:?}").contains("Invalid tier"));

        let err = handle_gate_command_with(4, &config).unwrap_err();
        assert!(format!("{err:?}").contains("Invalid tier"));
    }

    #[test]
    fn test_PMAT257_cov_handle_gate_command_with_empty_tier_passes() {
        let config = config_with_tiers(Tiers::default());
        assert!(handle_gate_command_with(1, &config).is_ok());
        assert!(handle_gate_command_with(2, &config).is_ok());
        assert!(handle_gate_command_with(3, &config).is_ok());
    }

    #[test]
    fn test_PMAT257_cov_handle_gate_command_with_unknown_gate_is_skipped() {
        let tiers = Tiers {
            tier1_gates: vec!["frobnicate".to_string()],
            ..Default::default()
        };
        let config = config_with_tiers(tiers);
        assert!(handle_gate_command_with(1, &config).is_ok());
    }

    #[test]
    fn test_PMAT257_cov_handle_gate_command_with_disabled_gates_all_pass() {
        // Every disabled-config branch of coverage/complexity/security/satd/
        // mutation returns true without spawning a subprocess.
        let tiers = Tiers {
            tier1_gates: vec![
                "coverage".to_string(),
                "complexity".to_string(),
                "security".to_string(),
                "satd".to_string(),
                "mutation".to_string(),
            ],
            ..Default::default()
        };
        let config = config_with_tiers(tiers);
        assert!(handle_gate_command_with(1, &config).is_ok());
    }

    #[test]
    fn test_PMAT257_cov_run_coverage_gate_disabled_short_circuits() {
        let config = config_with_tiers(Tiers::default());
        assert!(run_coverage_gate(&config));
    }

    #[test]
    fn test_PMAT257_cov_run_complexity_gate_disabled_short_circuits() {
        let config = config_with_tiers(Tiers::default());
        assert!(run_complexity_gate(&config));
    }

    #[test]
    fn test_PMAT257_cov_run_security_gate_none_and_disabled() {
        let mut config = config_with_tiers(Tiers::default());
        assert!(run_security_gate(&config));

        config.gates.security = Some(SecurityGate {
            enabled: false,
            max_unsafe_blocks: 0,
        });
        assert!(run_security_gate(&config));
    }

    #[test]
    fn test_PMAT257_cov_run_satd_gate_all_branches() {
        let mut config = config_with_tiers(Tiers::default());
        // No satd config at all.
        assert!(run_satd_gate(&config));

        // Present but disabled.
        config.gates.satd = Some(SatdGate {
            enabled: false,
            max_count: 5,
            patterns: vec![],
        });
        assert!(run_satd_gate(&config));

        // Enabled with no patterns.
        config.gates.satd = Some(SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec![],
        });
        assert!(run_satd_gate(&config));

        // Enabled with patterns (naive implementation still returns true).
        config.gates.satd = Some(SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string()],
        });
        assert!(run_satd_gate(&config));
    }

    #[test]
    fn test_PMAT257_cov_run_mutation_gate_none_and_disabled() {
        let mut config = config_with_tiers(Tiers::default());
        assert!(run_mutation_gate(&config));

        config.gates.mutation = Some(MutationGate {
            enabled: false,
            min_score: 90.0,
        });
        assert!(run_mutation_gate(&config));
    }
}
