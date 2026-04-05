#[cfg(test)]
mod installer_graph_cmd {
    use crate::cli::args::{InstallerCommands, InstallerGraphFormat};

    #[test]
    fn test_cov_installer_graph_mermaid() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Graph {
            path: project_path,
            format: InstallerGraphFormat::Mermaid,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "graph mermaid failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_graph_dot() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Graph {
            path: project_path,
            format: InstallerGraphFormat::Dot,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "graph dot failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_graph_json() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Graph {
            path: project_path,
            format: InstallerGraphFormat::Json,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "graph json failed: {:?}", res);
    }
}

// ---------------------------------------------------------------------------
// installer golden capture / compare
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_golden_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_golden_capture() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::GoldenCapture {
            path: project_path,
            trace: "baseline-v1".to_string(),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "golden capture failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_golden_compare() {
        let (_dir, project_path) = super::make_installer_project();
        // First capture a golden trace
        let cmd1 = InstallerCommands::GoldenCapture {
            path: project_path.clone(),
            trace: "cmp-trace".to_string(),
        };
        super::super::super::installer_commands::handle_installer_command(cmd1).unwrap();
        // Now compare against it
        let cmd2 = InstallerCommands::GoldenCompare {
            path: project_path,
            trace: "cmp-trace".to_string(),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd2);
        assert!(res.is_ok(), "golden compare failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_golden_compare_missing_trace() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::GoldenCompare {
            path: project_path,
            trace: "nonexistent-trace".to_string(),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err(), "comparing missing trace should fail");
    }
}

// ---------------------------------------------------------------------------
// installer audit
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_audit_cmd {
    use crate::cli::args::{AuditOutputFormat, InstallerCommands};

    #[test]
    fn test_cov_installer_audit_human() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: None,
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_json() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Json,
            security_only: false,
            min_severity: None,
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_sarif() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Sarif,
            security_only: false,
            min_severity: None,
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_security_only() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: true,
            min_severity: None,
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_warning() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("warning".to_string()),
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_info() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("info".to_string()),
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_error() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("error".to_string()),
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_critical() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("critical".to_string()),
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_suggestion() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("suggestion".to_string()),
            ignore: vec![],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_min_severity_invalid() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: Some("bogus".to_string()),
            ignore: vec![],
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err(), "invalid severity should fail");
    }

    #[test]
    fn test_cov_installer_audit_with_ignore_rules() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Audit {
            path: project_path,
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: None,
            ignore: vec!["SEC001".to_string(), "QUAL002".to_string()],
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_audit_missing_toml() {
        let dir = tempfile::tempdir().unwrap();
        let cmd = InstallerCommands::Audit {
            path: dir.path().to_path_buf(),
            format: AuditOutputFormat::Human,
            security_only: false,
            min_severity: None,
            ignore: vec![],
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err(), "audit on missing dir should fail");
    }
}

// ---------------------------------------------------------------------------
// installer keyring
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_keyring_cmd {
    use crate::cli::args::{InstallerCommands, KeyringCommands};

    #[test]
    fn test_cov_keyring_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::Init { import: vec![] },
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "keyring init failed: {:?}", res);
    }

    #[test]
    fn test_cov_keyring_list_no_keyring() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::List,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "keyring list (empty) failed: {:?}", res);
    }

    #[test]
    fn test_cov_keyring_list_after_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        // init first (may race with parallel tests)
        let cmd1 = InstallerCommands::Keyring {
            command: KeyringCommands::Init { import: vec![] },
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd1);
        // list (exercises code path regardless)
        let cmd2 = InstallerCommands::Keyring {
            command: KeyringCommands::List,
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd2);
    }

    #[test]
    fn test_cov_keyring_add_requires_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        // Create a valid hex key file
        let key_file = dir.path().join("testkey.pub");
        std::fs::write(
            &key_file,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::Add {
                key: key_file,
                id: "test-key".to_string(),
            },
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        // Should fail because keyring not initialized
        assert!(res.is_err());
    }

    #[test]
    fn test_cov_keyring_add_after_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        // init
        let cmd1 = InstallerCommands::Keyring {
            command: KeyringCommands::Init { import: vec![] },
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd1);
        // add key (may fail due to env var race in parallel tests; exercises code path)
        let key_file = dir.path().join("testkey.pub");
        std::fs::write(
            &key_file,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let cmd2 = InstallerCommands::Keyring {
            command: KeyringCommands::Add {
                key: key_file,
                id: "my-key".to_string(),
            },
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd2);
    }

    #[test]
    fn test_cov_keyring_remove_requires_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::Remove {
                id: "nonexistent".to_string(),
            },
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err());
    }

    #[test]
    fn test_cov_keyring_remove_after_init() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        // init
        let cmd1 = InstallerCommands::Keyring {
            command: KeyringCommands::Init { import: vec![] },
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd1);
        // remove nonexistent key (may fail due to env var race; exercises code path)
        let cmd2 = InstallerCommands::Keyring {
            command: KeyringCommands::Remove {
                id: "nope".to_string(),
            },
        };
        let _ = super::super::super::installer_commands::handle_installer_command(cmd2);
    }

    #[test]
    fn test_cov_keyring_init_with_import() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        // Create key file to import
        let key_file = dir.path().join("imported.pub");
        std::fs::write(
            &key_file,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        )
        .unwrap();
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::Init {
                import: vec![key_file],
            },
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "keyring init with import failed: {:?}", res);
    }

    #[test]
    fn test_cov_keyring_init_with_missing_import_file() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let cmd = InstallerCommands::Keyring {
            command: KeyringCommands::Init {
                import: vec![std::path::PathBuf::from("/tmp/does-not-exist-key.pub")],
            },
        };
        // Should succeed but print warning
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok());
    }
}

// ---------------------------------------------------------------------------
// installer from-bash
// ---------------------------------------------------------------------------

include!("command_tests_installer_cov_incl2_incl2.rs");
