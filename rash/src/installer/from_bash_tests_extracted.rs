#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // RED Phase: Failing Tests First (EXTREME TDD)
    // Test naming: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: INSTALLER_115 (from-bash converter)
    // =========================================================================

    #[test]
    fn test_INSTALLER_115_extract_root_check() {
        let script = r#"
#!/bin/bash
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi
"#;
        let patterns = extract_patterns(script).unwrap();
        assert!(
            patterns.iter().any(|p| matches!(p, BashPattern::RootCheck)),
            "Should detect root check pattern"
        );
    }

    #[test]
    fn test_INSTALLER_115_extract_apt_update() {
        let script = "apt-get update";
        let patterns = extract_patterns(script).unwrap();
        assert!(
            patterns.iter().any(|p| matches!(p, BashPattern::AptUpdate)),
            "Should detect apt-get update"
        );
    }

    #[test]
    fn test_INSTALLER_115_extract_apt_install() {
        let script = "apt-get install -y docker-ce nginx";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::AptInstall { packages } = p {
                Some(packages.clone())
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect apt-get install");
        let packages = found.unwrap();
        assert!(packages.contains(&"docker-ce".to_string()));
        assert!(packages.contains(&"nginx".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_sudo_apt_install() {
        let script = "sudo apt-get install -y curl wget";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::AptInstall { packages } = p {
                Some(packages.clone())
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect sudo apt-get install");
        let packages = found.unwrap();
        assert!(packages.contains(&"curl".to_string()));
        assert!(packages.contains(&"wget".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_mkdir_p() {
        let script = "mkdir -p /opt/myapp/config";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::MkdirP { path } = p {
                Some(path.clone())
            } else {
                None
            }
        });

        assert_eq!(found, Some("/opt/myapp/config".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_curl_download() {
        let script = "curl -fsSL https://example.com/install.sh -o /tmp/install.sh";
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::Download { url, output } = p {
                Some((url.clone(), output.clone()))
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect curl download");
        let (url, output) = found.unwrap();
        assert_eq!(url, "https://example.com/install.sh");
        assert_eq!(output, Some("/tmp/install.sh".to_string()));
    }

    #[test]
    fn test_INSTALLER_115_extract_heredoc() {
        let script = r#"cat << EOF
Hello World
This is content
EOF"#;
        let patterns = extract_patterns(script).unwrap();

        let found = patterns.iter().find_map(|p| {
            if let BashPattern::Heredoc { delimiter, content } = p {
                Some((delimiter.clone(), content.clone()))
            } else {
                None
            }
        });

        assert!(found.is_some(), "Should detect heredoc");
        let (delimiter, content) = found.unwrap();
        assert_eq!(delimiter, "EOF");
        assert!(content.contains("Hello World"));
    }

    #[test]
    fn test_INSTALLER_115_convert_generates_valid_toml() {
        let script = r#"
#!/bin/bash
if [ "$EUID" -ne 0 ]; then exit 1; fi
apt-get update
apt-get install -y docker-ce
"#;
        let result = convert_bash_to_installer(script, "docker-installer").unwrap();

        // Should generate valid TOML
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result
            .installer_toml
            .contains("name = \"docker-installer\""));
        assert!(result.installer_toml.contains("privileges = \"root\""));
        assert!(result.installer_toml.contains("[[step]]"));
    }

    #[test]
    fn test_INSTALLER_115_convert_extracts_templates() {
        let script = r#"
cat << EOF > /etc/config.txt
key=value
setting=123
EOF
"#;
        let result = convert_bash_to_installer(script, "config-installer").unwrap();

        assert!(
            !result.templates.is_empty(),
            "Should extract heredoc as template"
        );
        assert!(result.templates[0].content.contains("key=value"));
    }

    #[test]
    fn test_INSTALLER_115_convert_warns_on_eval() {
        let script = r#"
eval "rm -rf $USER_DIR"
"#;
        let result = convert_bash_to_installer(script, "unsafe-installer").unwrap();

        assert!(
            result.warnings.iter().any(|w| w.contains("eval")),
            "Should warn about eval usage"
        );
    }

    #[test]
    fn test_INSTALLER_115_convert_stats() {
        let script = r#"
apt-get update
apt-get install -y pkg1 pkg2
mkdir -p /opt/app
"#;
        let result = convert_bash_to_installer(script, "test-installer").unwrap();

        assert!(result.stats.steps_generated >= 3);
        assert_eq!(result.stats.apt_installs, 1);
    }

    #[test]
    fn test_INSTALLER_115_full_docker_script() {
        // Realistic Docker installation script
        let script = r#"
#!/bin/bash
set -e

# Check root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Update and install prerequisites
apt-get update
apt-get install -y ca-certificates curl gnupg

# Add Docker's GPG key
mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.gpg

# Install Docker
apt-get install -y docker-ce docker-ce-cli containerd.io
"#;
        let result = convert_bash_to_installer(script, "docker-ce-installer").unwrap();

        // Verify conversion
        assert!(result.installer_toml.contains("privileges = \"root\""));
        assert!(result.stats.apt_installs >= 2);
        assert!(result.stats.steps_generated >= 4);

        // Should be parseable TOML (basic check)
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result.installer_toml.contains("[[step]]"));
    }

    #[test]
    fn test_convert_file_to_project_creates_structure() {
        let dir = std::env::temp_dir().join("bashrs_test_file_to_project");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create test dir");

        let input_file = dir.join("deploy.sh");
        std::fs::write(
            &input_file,
            "#!/bin/bash\napt-get update\napt-get install -y nginx\nmkdir -p /var/www\n",
        )
        .expect("write input");

        let output_dir = dir.join("my-installer");
        let result = convert_file_to_project(&input_file, &output_dir).expect("conversion");

        // Verify directory structure
        assert!(output_dir.exists(), "output dir should exist");
        assert!(output_dir.join("installer.toml").exists());
        assert!(output_dir.join("templates").exists());
        assert!(output_dir.join("tests").exists());

        // Verify content
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result.installer_toml.contains("nginx"));
        assert!(result.stats.apt_installs >= 1);

        // Verify installer.toml was written to disk
        let on_disk = std::fs::read_to_string(output_dir.join("installer.toml")).expect("read");
        assert_eq!(on_disk, result.installer_toml);

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_convert_file_to_project_missing_input() {
        let dir = std::env::temp_dir().join("bashrs_test_missing_input");
        let _ = std::fs::remove_dir_all(&dir);
        let result = convert_file_to_project(Path::new("/nonexistent/file.sh"), &dir);
        assert!(result.is_err(), "should fail on missing input");
    }
}
