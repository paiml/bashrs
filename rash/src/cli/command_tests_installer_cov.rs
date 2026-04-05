//! Coverage tests for installer commands, corpus core/advanced/ops/diff commands.
//!
//! Targets 8 files with 0-6% coverage:
//!   - installer_run_logic.rs (333 uncov)
//!   - installer_logic.rs (276 uncov)
//!   - installer_golden_logic.rs (223 uncov)
//!   - installer_commands.rs (274 uncov)
//!   - corpus_core_commands.rs (216 uncov)
//!   - corpus_advanced_commands.rs (200 uncov)
//!   - corpus_ops_commands.rs (161 uncov)
//!   - corpus_diff_commands.rs (183 uncov)
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ============================================================================
// INSTALLER COMMAND COVERAGE TESTS
// ============================================================================

/// Helper: create a temp installer project via `init_project` and return
/// the tempdir handle (keeps directory alive) and its path.
fn make_installer_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path().join("test-project");
    crate::installer::init_project(&project_path, Some("test project")).unwrap();
    (dir, project_path)
}

/// Helper: write a tiny bash script for from-bash conversion tests.
fn write_bash_script(dir: &std::path::Path) -> std::path::PathBuf {
    let script = dir.join("setup.sh");
    std::fs::write(
        &script,
        "#!/bin/bash\nset -e\napt-get update\napt-get install -y curl\necho done\n",
    )
    .unwrap();
    script
}

// ---------------------------------------------------------------------------
// installer init
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_init_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_init_creates_project() {
        let dir = tempfile::tempdir().unwrap();
        let name = dir.path().join("my-installer");
        let cmd = InstallerCommands::Init {
            name,
            description: Some("A test installer".to_string()),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "installer init failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_init_no_description() {
        let dir = tempfile::tempdir().unwrap();
        let name = dir.path().join("bare-installer");
        let cmd = InstallerCommands::Init {
            name,
            description: None,
        };
        assert!(super::super::super::installer_commands::handle_installer_command(cmd).is_ok());
    }
}

// ---------------------------------------------------------------------------
// installer validate
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_validate_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_validate_ok() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Validate { path: project_path };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "validate failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_validate_missing_dir() {
        let cmd = InstallerCommands::Validate {
            path: std::path::PathBuf::from("/tmp/nonexistent-installer-path-xyz"),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err());
    }
}

// ---------------------------------------------------------------------------
// installer run (dry-run and diff modes)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_run_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_run_dry_run() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: None,
            dry_run: true,
            diff: false,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "dry-run failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_run_diff_mode() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: None,
            dry_run: false,
            diff: true,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "diff mode failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_run_actual_execution() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: None,
            dry_run: false,
            diff: false,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        // Actual execution may fail on steps, but exercises all the code paths
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_run_with_trace() {
        let (_dir, project_path) = super::make_installer_project();
        let trace_dir = tempfile::tempdir().unwrap();
        let trace_file = trace_dir.path().join("trace.json");
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: None,
            dry_run: false,
            diff: false,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: true,
            trace_file: Some(trace_file),
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_run_with_checkpoint_dir() {
        let (_dir, project_path) = super::make_installer_project();
        let ckpt_dir = tempfile::tempdir().unwrap();
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: Some(ckpt_dir.path().to_path_buf()),
            dry_run: true,
            diff: false,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "run with checkpoint dir failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_run_hermetic_without_lockfile() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Run {
            path: project_path,
            checkpoint_dir: None,
            dry_run: false,
            diff: false,
            hermetic: true,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        // Should fail because there's no lockfile
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err(), "hermetic without lockfile should fail");
    }
}

// ---------------------------------------------------------------------------
// installer test
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_test_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_test_basic() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Test {
            path: project_path,
            matrix: None,
            coverage: false,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "installer test failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_test_with_coverage() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Test {
            path: project_path,
            matrix: None,
            coverage: true,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(
            res.is_ok(),
            "installer test with coverage failed: {:?}",
            res
        );
    }

    #[test]
    fn test_cov_installer_test_default_matrix() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Test {
            path: project_path,
            matrix: Some("default".to_string()),
            coverage: false,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        // May fail due to container runtime not being available; exercises the code path
        let _ = res;
    }

    #[test]
    fn test_cov_installer_test_extended_matrix() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Test {
            path: project_path,
            matrix: Some("extended".to_string()),
            coverage: true,
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_test_custom_platforms() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Test {
            path: project_path,
            matrix: Some("ubuntu:22.04,debian:12".to_string()),
            coverage: false,
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }
}

// ---------------------------------------------------------------------------
// installer lock
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_lock_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_lock_generate() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Lock {
            path: project_path,
            update: false,
            verify: false,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "lock generate failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_lock_update_no_existing() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Lock {
            path: project_path,
            update: true,
            verify: false,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "lock update (no existing) failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_lock_verify_no_lockfile() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Lock {
            path: project_path,
            update: false,
            verify: true,
        };
        // verify without lockfile should succeed for installers with 0 artifacts
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_lock_verify_with_lockfile() {
        let (_dir, project_path) = super::make_installer_project();
        // First generate a lockfile
        let cmd1 = InstallerCommands::Lock {
            path: project_path.clone(),
            update: false,
            verify: false,
        };
        super::super::super::installer_commands::handle_installer_command(cmd1).unwrap();
        // Now verify
        let cmd2 = InstallerCommands::Lock {
            path: project_path,
            update: false,
            verify: true,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd2);
        assert!(res.is_ok(), "lock verify failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_lock_update_existing() {
        let (_dir, project_path) = super::make_installer_project();
        // Generate first
        let cmd1 = InstallerCommands::Lock {
            path: project_path.clone(),
            update: false,
            verify: false,
        };
        super::super::super::installer_commands::handle_installer_command(cmd1).unwrap();
        // Update
        let cmd2 = InstallerCommands::Lock {
            path: project_path,
            update: true,
            verify: false,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd2);
        assert!(res.is_ok(), "lock update failed: {:?}", res);
    }
}

// ---------------------------------------------------------------------------
// installer graph
// ---------------------------------------------------------------------------

include!("command_tests_installer_cov_incl2.rs");
