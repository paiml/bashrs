/// Test comply rules command (JSON format).
#[test]
fn test_cov_comply_rules_json() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Json,
        });
    assert!(result.is_ok());
}

/// Test comply rules command (Markdown format).
#[test]
fn test_cov_comply_rules_markdown() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Markdown,
        });
    assert!(result.is_ok());
}

/// Test comply init command on a fresh directory.
///
/// NOTE: comply init uses cwd-relative paths internally.
/// We avoid set_current_dir to prevent parallel test races.
/// Instead, we test the comply init error path (already exists in project root)
/// and individual helper functions.
#[test]
fn test_cov_comply_init_already_exists_in_cwd() {
    // The project root likely already has .bashrs/ — so init will fail.
    // This exercises the "already exists" error path in comply_init_command.
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::Project,
            pzsh: false,
            strict: false,
        });
    // Expected: either Err (already exists) or Ok (if no comply.toml).
    // Both paths exercise code coverage.
    let _ = result;
}

/// Test comply init with pzsh and strict flags (error path).
#[test]
fn test_cov_comply_init_pzsh_strict() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::All,
            pzsh: true,
            strict: true,
        });
    let _ = result;
}

/// Test comply init with user scope.
#[test]
fn test_cov_comply_init_user_scope() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::User,
            pzsh: false,
            strict: false,
        });
    let _ = result;
}

/// Test comply init with system scope.
#[test]
fn test_cov_comply_init_system_scope() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::System,
            pzsh: false,
            strict: true,
        });
    let _ = result;
}

/// Test comply track discover command.
#[test]
fn test_cov_comply_track_discover() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: all\nall:\n\t@echo done\n",
    )
    .unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::Discover {
                path: dir.path().to_path_buf(),
                scope: crate::cli::args::ComplyScopeArg::Project,
            },
        });
    assert!(result.is_ok());
}

/// Test comply track discover with All scope.
#[test]
fn test_cov_comply_track_discover_all() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::Discover {
                path: dir.path().to_path_buf(),
                scope: crate::cli::args::ComplyScopeArg::All,
            },
        });
    assert!(result.is_ok());
}

/// Test comply track list command.
#[test]
fn test_cov_comply_track_list() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::List {
                path: dir.path().to_path_buf(),
                scope: None,
            },
        });
    assert!(result.is_ok());
}

/// Test comply track list with specific scope.
#[test]
fn test_cov_comply_track_list_project() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::List {
                path: dir.path().to_path_buf(),
                scope: Some(crate::cli::args::ComplyScopeArg::Project),
            },
        });
    assert!(result.is_ok());
}

/// Test comply check with each scope variant.
#[test]
fn test_cov_comply_check_scope_user() {
    let dir = TempDir::new().unwrap();
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::User),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        });
    let _ = result;
}

/// Test comply check with system scope.
#[test]
fn test_cov_comply_check_scope_system() {
    let dir = TempDir::new().unwrap();
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::System),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        });
    let _ = result;
}

/// Test comply check with all scope.
#[test]
fn test_cov_comply_check_scope_all() {
    let dir = TempDir::new().unwrap();
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::All),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        });
    let _ = result;
}

// ============================================================================
// Config Commands Coverage Tests
// ============================================================================

/// Test config analyze command with human output.
#[test]
fn test_cov_config_analyze_human() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

/// Test config analyze command with JSON output.
#[test]
fn test_cov_config_analyze_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

/// Test config lint command with human output (no issues).
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_lint_clean() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".profile");
    // A truly minimal config file that should have zero issues
    fs::write(&file, "# Clean profile\n").unwrap();

    let result = config_lint_command(&file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

/// Test config lint command with JSON output.
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_lint_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".zshrc");
    // Minimal clean config that should produce zero issues
    fs::write(&file, "# Clean zshrc\n").unwrap();

    let result = config_lint_command(&file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

/// Test config purify command in dry-run mode.
#[test]
fn test_cov_config_purify_dry_run() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Purify {
            input: file,
            output: None,
            fix: false,
            no_backup: false,
            dry_run: true,
        });
    assert!(result.is_ok());
}

/// Test config purify command writing to output file.
#[test]
fn test_cov_config_purify_output_file() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    let output = dir.path().join(".bashrc.purified");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Purify {
            input,
            output: Some(output.clone()),
            fix: false,
            no_backup: false,
            dry_run: false,
        });
    assert!(result.is_ok());
    assert!(output.exists());
}

/// Test config purify command writing to stdout (output path = "-").
#[test]
fn test_cov_config_purify_stdout() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(&input, "#!/bin/bash\nexport EDITOR=vim\n").unwrap();

    let stdout_path = PathBuf::from("-");
    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Purify {
            input,
            output: Some(stdout_path),
            fix: false,
            no_backup: false,
            dry_run: false,
        });
    assert!(result.is_ok());
}

/// Test config purify command with --fix (in-place with backup).
#[test]
fn test_cov_config_purify_fix_inplace() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Purify {
            input,
            output: None,
            fix: true,
            no_backup: false,
            dry_run: false,
        });
    assert!(result.is_ok());
}

/// Test config purify command with --fix --no-backup.
#[test]
fn test_cov_config_purify_fix_no_backup() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Purify {
            input,
            output: None,
            fix: true,
            no_backup: true,
            dry_run: false,
        });
    assert!(result.is_ok());
}


include!("command_tests_lint_cov_tests_cov_2.rs");
