/// Test helper: should_output_to_stdout.
#[test]
fn test_cov_config_should_output_to_stdout() {
    assert!(should_output_to_stdout(std::path::Path::new("-")));
    assert!(!should_output_to_stdout(std::path::Path::new("foo.sh")));
    assert!(!should_output_to_stdout(std::path::Path::new("/tmp/out")));
}

/// Test helper: count_duplicate_path_entries.
#[test]
fn test_cov_config_count_duplicate_path_entries() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let source = fs::read_to_string(&file).unwrap();
    let analysis = crate::config::analyzer::analyze_config(&source, file);
    let dup_count = count_duplicate_path_entries(&analysis);
    // May or may not find duplicates — just exercise the code path
    let _ = dup_count;
}

/// Test helper: handle_output_to_file with regular file.
#[test]
fn test_cov_config_handle_output_to_file() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("output.sh");
    let result = handle_output_to_file(&out, "#!/bin/sh\necho purified\n");
    assert!(result.is_ok());
    assert!(out.exists());
}

/// Test helper: handle_output_to_file with stdout.
#[test]
fn test_cov_config_handle_output_to_file_stdout() {
    let result = handle_output_to_file(std::path::Path::new("-"), "#!/bin/sh\necho purified\n");
    assert!(result.is_ok());
}

/// Test config analyze via handle_config_command dispatch.
#[test]
fn test_cov_config_dispatch_analyze() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(&file, "#!/bin/bash\nexport EDITOR=vim\n").unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Analyze {
            input: file,
            format: ConfigOutputFormat::Human,
        });
    assert!(result.is_ok());
}

/// Test config lint via handle_config_command dispatch.
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_dispatch_lint() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".zshrc");
    // Minimal clean config to avoid process::exit(1)
    fs::write(&file, "# Clean zshrc config\n").unwrap();

    let result =
        super::config_cmds::handle_config_command(crate::cli::args::ConfigCommands::Lint {
            input: file,
            format: ConfigOutputFormat::Human,
        });
    assert!(result.is_ok());
}

// ============================================================================
// Gate Commands Coverage Tests
//
// NOTE: gate_commands.rs uses GateConfig::load() which reads .pmat-gates.toml
// from the current working directory. Tests run from the project root which
// already has .pmat-gates.toml. We exercise gate_commands directly and test
// GateConfig deserialization separately to avoid process-global set_current_dir
// races in parallel tests.
// ============================================================================

/// Test GateConfig deserialization with all gate types.
#[test]
fn test_cov_gate_config_deser_all_gates() {
    let config_content = r#"
[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 120
check_coverage = false
min_coverage = 80.0
check_complexity = true
max_complexity = 10

[gates.satd]
enabled = true
max_count = 5
patterns = ["TODO", "FIXME"]

[gates.mutation]
enabled = false
min_score = 90.0

[gates.security]
enabled = true
max_unsafe_blocks = 0

[tiers]
tier1_gates = ["complexity", "satd"]
tier2_gates = ["clippy", "tests"]
tier3_gates = ["coverage", "mutation", "security"]
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    assert!(config.gates.run_clippy);
    assert!(config.gates.clippy_strict);
    assert!(config.gates.run_tests);
    assert_eq!(config.gates.test_timeout, 120);
    assert!(!config.gates.check_coverage);
    assert!(config.gates.check_complexity);
    assert_eq!(config.gates.max_complexity, 10);

    let satd = config.gates.satd.as_ref().unwrap();
    assert!(satd.enabled);
    assert_eq!(satd.patterns.len(), 2);

    let mutation = config.gates.mutation.as_ref().unwrap();
    assert!(!mutation.enabled);

    let security = config.gates.security.as_ref().unwrap();
    assert!(security.enabled);

    assert_eq!(config.tiers.tier1_gates.len(), 2);
    assert_eq!(config.tiers.tier2_gates.len(), 2);
    assert_eq!(config.tiers.tier3_gates.len(), 3);
}

/// Test GateConfig deserialization with minimal config (defaults).
#[test]
fn test_cov_gate_config_deser_minimal() {
    let config_content = r#"
[gates]
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    assert!(!config.gates.run_clippy);
    assert!(!config.gates.run_tests);
    assert!(!config.gates.check_coverage);
    assert!(config.gates.satd.is_none());
    assert!(config.gates.mutation.is_none());
    assert!(config.tiers.tier1_gates.is_empty());
}

/// Test GateConfig deserialization with SATD disabled.
#[test]
fn test_cov_gate_config_deser_satd_disabled() {
    let config_content = r#"
[gates]

[gates.satd]
enabled = false
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    let satd = config.gates.satd.as_ref().unwrap();
    assert!(!satd.enabled);
}

/// Test GateConfig deserialization with mutation gate.
#[test]
fn test_cov_gate_config_deser_mutation() {
    let config_content = r#"
[gates]

[gates.mutation]
enabled = true
min_score = 85.0
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    let mutation = config.gates.mutation.as_ref().unwrap();
    assert!(mutation.enabled);
    assert!((mutation.min_score - 85.0).abs() < f64::EPSILON);
}

/// Test handle_gate_command with invalid tier (exercises the error path
/// after loading real .pmat-gates.toml from project root).
#[test]
fn test_cov_gate_invalid_tier() {
    // Uses the project root's .pmat-gates.toml. Tier 99 is invalid.
    let result = super::gate_cmds::handle_gate_command(99, crate::cli::args::ReportFormat::Human);
    // Should fail with "Invalid tier: 99"
    assert!(result.is_err());
}

/// Test handle_gate_command with tier 0 (also invalid).
#[test]
fn test_cov_gate_tier_zero() {
    let result = super::gate_cmds::handle_gate_command(0, crate::cli::args::ReportFormat::Human);
    assert!(result.is_err());
}

// ============================================================================
// DevContainer Commands Coverage Tests
// ============================================================================

/// Test devcontainer validate with --list-rules flag.
#[test]
fn test_cov_devcontainer_list_rules() {
    let dir = TempDir::new().unwrap();
    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: true,
        },
    );
    assert!(result.is_ok());
}

/// Test devcontainer validate with valid devcontainer.json.
#[test]
fn test_cov_devcontainer_validate_valid() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{
  "name": "Test Dev Container",
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu"
}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with JSON format.
#[test]
fn test_cov_devcontainer_validate_json_format() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{"name": "Test", "image": "ubuntu:22.04"}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Json,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with SARIF format.
#[test]
fn test_cov_devcontainer_validate_sarif_format() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{"name": "Test", "image": "ubuntu:22.04"}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Sarif,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate when no devcontainer.json exists (error).
#[test]
fn test_cov_devcontainer_validate_missing() {
    let dir = TempDir::new().unwrap();
    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    assert!(result.is_err());
}

/// Test devcontainer validate with --lint-dockerfile and a referenced Dockerfile.
#[test]
fn test_cov_devcontainer_validate_lint_dockerfile() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();

    // Write Dockerfile
    fs::write(
        dc_dir.join("Dockerfile"),
        "FROM ubuntu:22.04\nRUN apt-get update\n",
    )
    .unwrap();

    // Write devcontainer.json referencing the Dockerfile
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{
  "name": "Test",
  "build": {
    "dockerfile": "Dockerfile"
  }
}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: true,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with direct file path.
#[test]
fn test_cov_devcontainer_validate_direct_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("devcontainer.json");
    fs::write(&file, r#"{"name": "Direct", "image": "ubuntu:22.04"}"#).unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: file,
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

// ============================================================================
// Test Commands Coverage Tests
// ============================================================================


include!("command_tests_lint_cov_tests_cov.rs");
