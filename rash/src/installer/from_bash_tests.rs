//! Coverage tests for installer/from_bash.rs
//!
//! Targets:
//! - generate_installer_toml (all BashPattern variants)
//! - convert_file_to_project (filesystem operations)
//! - generate_warnings (sudo + eval/RANDOM/$$)
//! - extract_patterns edge cases (apt update variant, wget, heredoc, sudo non-apt)
//! - parse_apt_install with "apt install" (not apt-get)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use super::{
    convert_bash_to_installer, convert_file_to_project, BashPattern, ConversionStats,
};

// ---------------------------------------------------------------------------
// generate_installer_toml — all BashPattern variants via convert_bash_to_installer
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_001_root_check_sets_privileges_root() {
    let script = r#"
if [ "$EUID" -ne 0 ]; then echo "run as root"; exit 1; fi
"#;
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("privileges = \"root\""));
    assert_eq!(result.stats.conditionals_converted, 1);
}

#[test]
fn test_FROM_BASH_002_no_root_check_user_privileges() {
    let script = "apt-get update\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("privileges = \"user\""));
}

#[test]
fn test_FROM_BASH_003_apt_update_step_generated() {
    let script = "apt-get update\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("apt-update"));
    assert!(result.installer_toml.contains("apt-get update"));
    assert_eq!(result.stats.steps_generated, 1);
}

#[test]
fn test_FROM_BASH_004_apt_install_step_generated() {
    let script = "apt-get install -y curl wget git\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("install"));
    assert!(result.installer_toml.contains("curl"));
    assert!(result.installer_toml.contains("wget"));
    assert_eq!(result.stats.apt_installs, 1);
    assert_eq!(result.stats.steps_generated, 1);
}

#[test]
fn test_FROM_BASH_005_mkdir_p_step_generated() {
    let script = "mkdir -p /opt/myapp\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("mkdir"));
    assert!(result.installer_toml.contains("/opt/myapp"));
    assert_eq!(result.stats.steps_generated, 1);
}

#[test]
fn test_FROM_BASH_006_download_step_generated() {
    let script = "curl -fsSL https://example.com/tool -o /usr/local/bin/tool\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("download"));
    assert!(result.installer_toml.contains("https://example.com/tool"));
    assert_eq!(result.stats.steps_generated, 1);
}

#[test]
fn test_FROM_BASH_007_download_without_output_file() {
    // curl without -o flag — output is None, uses "downloaded-file" fallback
    let script = "curl -fsSL https://example.com/script.sh\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("download"));
    assert!(result.installer_toml.contains("https://example.com/script.sh"));
    assert!(result.installer_toml.contains("downloaded-file"));
}

#[test]
fn test_FROM_BASH_008_heredoc_step_with_template() {
    let script = "cat << EOF\nhello world\nEOF\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("heredoc"));
    assert_eq!(result.stats.heredocs_converted, 1);
    assert!(!result.templates.is_empty());
    assert!(result.templates[0].content.contains("hello world"));
}

#[test]
fn test_FROM_BASH_009_sudo_command_step_generated() {
    let script = "sudo systemctl restart nginx\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("sudo"));
    assert!(result.installer_toml.contains("systemctl restart nginx"));
    assert_eq!(result.stats.sudo_patterns, 1);
}

#[test]
fn test_FROM_BASH_010_generic_script_step_generated() {
    let script = "chmod +x /usr/local/bin/tool\n";
    let result = convert_bash_to_installer(script, "my-installer").unwrap();
    assert!(result.installer_toml.contains("script"));
    assert!(result.installer_toml.contains("chmod"));
}

#[test]
fn test_FROM_BASH_011_installer_header_contains_name() {
    let script = "echo hello\n";
    let result = convert_bash_to_installer(script, "test-app").unwrap();
    assert!(result.installer_toml.contains("test-app"));
    assert!(result.installer_toml.contains("version = \"1.0.0\""));
}

#[test]
fn test_FROM_BASH_012_empty_script_produces_header_only() {
    let script = "";
    let result = convert_bash_to_installer(script, "empty-app").unwrap();
    assert!(result.installer_toml.contains("[installer]"));
    assert_eq!(result.stats.steps_generated, 0);
    assert!(result.templates.is_empty());
}

#[test]
fn test_FROM_BASH_013_comments_and_blank_lines_skipped() {
    let script = "\n# This is a comment\n\n# Another comment\n\napt-get update\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    // Only the apt-get update should be processed
    assert_eq!(result.stats.steps_generated, 1);
}

// ---------------------------------------------------------------------------
// generate_warnings
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_014_warning_generated_for_sudo_command() {
    let script = "sudo systemctl stop apache2\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(!result.warnings.is_empty());
    let has_sudo_warning = result
        .warnings
        .iter()
        .any(|w| w.contains("Sudo") || w.contains("sudo"));
    assert!(has_sudo_warning, "Expected sudo warning, got: {:?}", result.warnings);
}

#[test]
fn test_FROM_BASH_015_warning_generated_for_eval_usage() {
    let script = "eval \"$command\"\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    let has_eval_warning = result
        .warnings
        .iter()
        .any(|w| w.contains("eval"));
    assert!(has_eval_warning, "Expected eval warning, got: {:?}", result.warnings);
}

#[test]
fn test_FROM_BASH_016_warning_generated_for_random() {
    let script = "echo $RANDOM\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    let has_nondeterministic_warning = result
        .warnings
        .iter()
        .any(|w| w.contains("Non-deterministic") || w.contains("RANDOM"));
    assert!(
        has_nondeterministic_warning,
        "Expected non-deterministic warning, got: {:?}",
        result.warnings
    );
}

#[test]
fn test_FROM_BASH_017_warning_generated_for_process_id() {
    let script = "echo $$\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    let has_warning = result
        .warnings
        .iter()
        .any(|w| w.contains("Non-deterministic") || w.contains("$$"));
    assert!(has_warning, "Expected non-deterministic warning, got: {:?}", result.warnings);
}

#[test]
fn test_FROM_BASH_018_no_warnings_for_safe_script() {
    let script = "apt-get update\napt-get install -y curl\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(
        result.warnings.is_empty(),
        "Unexpected warnings: {:?}",
        result.warnings
    );
}

// ---------------------------------------------------------------------------
// extract_patterns edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_019_apt_update_without_get() {
    // "apt update" (not "apt-get update") also matches
    let script = "apt update\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.installer_toml.contains("apt-update"));
}

#[test]
fn test_FROM_BASH_020_apt_install_without_get() {
    // "apt install" (not "apt-get install")
    let script = "apt install -y nginx\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.installer_toml.contains("install"));
    assert!(result.installer_toml.contains("nginx"));
    assert_eq!(result.stats.apt_installs, 1);
}

#[test]
fn test_FROM_BASH_021_sudo_apt_install_without_get() {
    let script = "sudo apt install -y curl\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.stats.apt_installs >= 1);
}

#[test]
fn test_FROM_BASH_022_wget_download() {
    let script = "wget https://example.com/file.tar.gz -O /tmp/file.tar.gz\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.installer_toml.contains("download"));
    assert!(result.installer_toml.contains("https://example.com/file.tar.gz"));
}

#[test]
fn test_FROM_BASH_023_wget_download_without_output() {
    let script = "wget https://example.com/script.sh\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.installer_toml.contains("download"));
    assert!(result.installer_toml.contains("downloaded-file"));
}

#[test]
fn test_FROM_BASH_024_sudo_mkdir_is_mkdir_pattern_not_sudo() {
    // "sudo mkdir -p /opt" should be treated as MkdirP, not SudoCommand
    let script = "sudo mkdir -p /opt/app\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.installer_toml.contains("mkdir"));
    assert_eq!(result.stats.sudo_patterns, 0);
}

#[test]
fn test_FROM_BASH_025_heredoc_with_quoted_delimiter() {
    let script = "cat << 'HEREDOC'\nsome content\nHEREDOC\n";
    let result = convert_bash_to_installer(script, "my-app").unwrap();
    assert!(result.stats.heredocs_converted >= 1);
    assert!(!result.templates.is_empty());
}

#[test]
fn test_FROM_BASH_026_multiple_patterns_in_sequence() {
    let script = r#"
if [ "$EUID" -ne 0 ]; then echo "run as root"; exit 1; fi
apt-get update
apt-get install -y curl
mkdir -p /opt/app
"#;
    let result = convert_bash_to_installer(script, "multi-step").unwrap();
    assert!(result.installer_toml.contains("privileges = \"root\""));
    assert_eq!(result.stats.steps_generated, 3);
    assert_eq!(result.stats.conditionals_converted, 1);
}

// ---------------------------------------------------------------------------
// convert_file_to_project
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_027_convert_file_creates_output_structure() {
    let tmp = TempDir::new().unwrap();
    let input_path = tmp.path().join("install.sh");
    let output_dir = tmp.path().join("my-project");

    fs::write(
        &input_path,
        "apt-get update\napt-get install -y curl\n",
    )
    .unwrap();

    let result = convert_file_to_project(&input_path, &output_dir).unwrap();

    // Check output directory was created
    assert!(output_dir.exists());
    assert!(output_dir.join("installer.toml").exists());
    assert!(output_dir.join("templates").exists());
    assert!(output_dir.join("tests").exists());

    // Check installer.toml has correct content
    let toml_content = fs::read_to_string(output_dir.join("installer.toml")).unwrap();
    assert!(toml_content.contains("my-project"));
    assert_eq!(result.stats.steps_generated, 2);
}

#[test]
fn test_FROM_BASH_028_convert_file_writes_templates() {
    let tmp = TempDir::new().unwrap();
    let input_path = tmp.path().join("install.sh");
    let output_dir = tmp.path().join("template-project");

    fs::write(&input_path, "cat << EOF\nhello template\nEOF\n").unwrap();

    let result = convert_file_to_project(&input_path, &output_dir).unwrap();

    assert!(!result.templates.is_empty());
    let template_file = output_dir.join("templates").join(&result.templates[0].name);
    assert!(template_file.exists());
    let content = fs::read_to_string(template_file).unwrap();
    assert!(content.contains("hello template"));
}

#[test]
fn test_FROM_BASH_029_convert_file_nonexistent_input_returns_error() {
    let tmp = TempDir::new().unwrap();
    let input_path = PathBuf::from("/nonexistent/path/install.sh");
    let output_dir = tmp.path().join("output");

    let err = convert_file_to_project(&input_path, &output_dir).unwrap_err();
    assert!(
        format!("{err}").contains("Failed to read") || format!("{err}").contains("not found"),
        "Expected read error, got: {err}"
    );
}

#[test]
fn test_FROM_BASH_030_convert_file_uses_dir_name_as_project_name() {
    let tmp = TempDir::new().unwrap();
    let input_path = tmp.path().join("install.sh");
    let output_dir = tmp.path().join("awesome-app");

    fs::write(&input_path, "echo hello\n").unwrap();

    let result = convert_file_to_project(&input_path, &output_dir).unwrap();

    let toml_content = fs::read_to_string(output_dir.join("installer.toml")).unwrap();
    assert!(toml_content.contains("awesome-app"));
    drop(result);
}

#[test]
fn test_FROM_BASH_031_convert_file_with_all_patterns() {
    let tmp = TempDir::new().unwrap();
    let input_path = tmp.path().join("complex.sh");
    let output_dir = tmp.path().join("complex-project");

    let script = r#"if [ "$EUID" -ne 0 ]; then echo "need root"; exit 1; fi
apt-get update
apt-get install -y docker.io
mkdir -p /etc/docker
curl -fsSL https://example.com/config -o /etc/docker/config.json
sudo systemctl enable docker
cat << EOF
[Service]
Type=notify
EOF
"#;

    fs::write(&input_path, script).unwrap();

    let result = convert_file_to_project(&input_path, &output_dir).unwrap();

    assert!(result.stats.steps_generated >= 5);
    assert_eq!(result.stats.conditionals_converted, 1);
    assert_eq!(result.stats.apt_installs, 1);
}

// ---------------------------------------------------------------------------
// ConversionStats default
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_032_conversion_stats_default_is_zero() {
    let stats = ConversionStats::default();
    assert_eq!(stats.steps_generated, 0);
    assert_eq!(stats.apt_installs, 0);
    assert_eq!(stats.heredocs_converted, 0);
    assert_eq!(stats.sudo_patterns, 0);
    assert_eq!(stats.conditionals_converted, 0);
}

// ---------------------------------------------------------------------------
// BashPattern PartialEq
// ---------------------------------------------------------------------------

#[test]
fn test_FROM_BASH_033_bash_pattern_partial_eq() {
    assert_eq!(BashPattern::RootCheck, BashPattern::RootCheck);
    assert_eq!(BashPattern::AptUpdate, BashPattern::AptUpdate);
    assert_ne!(BashPattern::RootCheck, BashPattern::AptUpdate);
    assert_eq!(
        BashPattern::Script {
            content: "echo hi".to_string()
        },
        BashPattern::Script {
            content: "echo hi".to_string()
        }
    );
    assert_ne!(
        BashPattern::Script {
            content: "echo hi".to_string()
        },
        BashPattern::Script {
            content: "echo bye".to_string()
        }
    );
}
