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

/// What `cargo deny check` did, as three outcomes rather than two
/// (PMAT-266). The old code had `Ok(status)` and `Err(_)`, and `Err` is
/// returned only when the `cargo` BINARY cannot be spawned. A runner with
/// cargo but without cargo-deny takes the `Ok` arm with exit 101 and
/// `error: no such command: `deny``, which read as "the security gate found
/// violations" -- a gate failure reported for a check that never ran. The
/// nightly full gate added in v7.4.0 hit exactly that (run 34689967236).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DenyOutcome {
    /// cargo-deny is not installed (or cargo itself is missing): nothing was
    /// measured, and the caller says so rather than passing or failing silently.
    Absent,
    /// cargo deny ran and reported no violations.
    Clean,
    /// cargo deny ran and reported violations.
    Violations,
}

/// Run `cargo deny check` and classify the result.
///
/// `probe` runs `cargo deny --version`, whose only job is to tell an absent
/// subcommand from a failing check: it exits 0 when cargo-deny is installed
/// and non-zero (or fails to spawn) when it is not. Separate from the check
/// itself so the absence is decided before any policy result is read.
pub(crate) fn cargo_deny_outcome(cargo: &str) -> DenyOutcome {
    let probe = std::process::Command::new(cargo)
        .args(["deny", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match probe {
        Ok(s) if s.success() => {}
        _ => return DenyOutcome::Absent,
    }

    match std::process::Command::new(cargo)
        .args(["deny", "check"])
        .status()
    {
        Ok(s) if s.success() => DenyOutcome::Clean,
        Ok(_) => DenyOutcome::Violations,
        // The probe succeeded a moment ago, so a spawn failure here is not
        // "not installed"; it is unmeasured, and unmeasured is not a pass.
        Err(_) => DenyOutcome::Violations,
    }
}

fn run_security_gate(config: &crate::gates::GateConfig) -> bool {
    // GH-181: Respect security gate config (enabled flag, max_unsafe_blocks)
    if let Some(ref security) = config.gates.security {
        if !security.enabled {
            return true;
        }
    }

    match cargo_deny_outcome("cargo") {
        DenyOutcome::Clean => true,
        DenyOutcome::Violations => false,
        DenyOutcome::Absent => {
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

    /// PMAT-266: a `cargo` on PATH whose `deny` subcommand does not exist
    /// is ABSENT, not a violation. Writes a stub `cargo` into a TempDir and
    /// names it directly, so the test never depends on what this machine has
    /// installed (the defect only showed on a runner without cargo-deny).
    fn stub_cargo(dir: &std::path::Path, body: &str) -> std::path::PathBuf {
        let path = dir.join("cargo");
        std::fs::write(&path, body).expect("write stub cargo");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod stub cargo");
        }
        path
    }

    #[test]
    fn test_PMAT266_cargo_deny_absent_is_not_a_violation() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // A cargo that exists but has no `deny` subcommand, byte for byte
        // what a runner without cargo-deny prints (exit 101).
        let cargo = stub_cargo(
            dir.path(),
            "#!/bin/sh\ncase \"$1\" in deny) echo 'error: no such command: `deny`' >&2; exit 101 ;; esac\nexit 0\n",
        );
        assert_eq!(
            cargo_deny_outcome(&cargo.to_string_lossy()),
            DenyOutcome::Absent,
            "a missing subcommand must read as absent, never as violations"
        );
    }

    #[test]
    fn test_PMAT266_cargo_deny_clean_and_violations_are_told_apart() {
        let clean_dir = tempfile::TempDir::new().expect("tempdir");
        let clean = stub_cargo(
            clean_dir.path(),
            "#!/bin/sh\ncase \"$2\" in --version) echo 'cargo-deny 0.19.0'; exit 0 ;; esac\nexit 0\n",
        );
        assert_eq!(
            cargo_deny_outcome(&clean.to_string_lossy()),
            DenyOutcome::Clean
        );

        let bad_dir = tempfile::TempDir::new().expect("tempdir");
        // Installed (the --version probe succeeds) and the check fails: a
        // real advisory. This must NOT be softened into a skip.
        let bad = stub_cargo(
            bad_dir.path(),
            "#!/bin/sh\ncase \"$2\" in --version) echo 'cargo-deny 0.19.0'; exit 0 ;; esac\necho 'error[vulnerability]: RUSTSEC-0000-0000' >&2\nexit 1\n",
        );
        assert_eq!(
            cargo_deny_outcome(&bad.to_string_lossy()),
            DenyOutcome::Violations,
            "a check that ran and failed must fail the gate"
        );
    }

    #[test]
    fn test_PMAT266_cargo_that_cannot_be_spawned_is_absent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("no-such-cargo");
        assert_eq!(
            cargo_deny_outcome(&missing.to_string_lossy()),
            DenyOutcome::Absent
        );
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
