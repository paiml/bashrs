#[cfg(test)]
mod tests {
    use super::*;

    /// F086: Valid unit file structure
    #[test]
    fn test_F086_valid_unit_structure() {
        let unit = r#"[Unit]
Description=Test Service

[Service]
ExecStart=/usr/bin/test

[Install]
WantedBy=multi-user.target"#;
        let result = check(unit);

        // Should not have critical errors
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F086: Valid unit should not have errors. Got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_F086_missing_service_section() {
        let unit = r#"[Unit]
Description=Test Service"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("[Service]")),
            "F086: Should detect missing [Service] section"
        );
    }

    /// F087: Correct Type directive
    #[test]
    fn test_F087_valid_type() {
        let unit = r#"[Service]
Type=notify
ExecStart=/usr/bin/test"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Invalid Type")),
            "F087: Should accept valid Type=notify"
        );
    }

    #[test]
    fn test_F087_invalid_type() {
        let unit = r#"[Service]
Type=invalid
ExecStart=/usr/bin/test"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Invalid Type")),
            "F087: Should reject invalid Type"
        );
    }

    /// F088: Valid ExecStart path
    #[test]
    fn test_F088_absolute_exec_start() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("absolute path") && d.message.contains("ExecStart")),
            "F088: Should accept absolute path"
        );
    }

    #[test]
    fn test_F088_relative_exec_start() {
        let unit = r#"[Service]
ExecStart=test"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("absolute path")),
            "F088: Should warn about relative path"
        );
    }

    /// F090: Restart policy
    #[test]
    fn test_F090_valid_restart() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test
Restart=on-failure"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Invalid Restart")),
            "F090: Should accept valid Restart policy"
        );
    }

    #[test]
    fn test_F090_invalid_restart() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test
Restart=invalid"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Invalid Restart")),
            "F090: Should reject invalid Restart policy"
        );
    }

    /// F091: RestartSec validation
    #[test]
    fn test_F091_restart_sec_zero() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test
Restart=always
RestartSec=0"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("RestartSec=0")),
            "F091: Should warn about RestartSec=0"
        );
    }

    #[test]
    fn test_F091_restart_sec_valid() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test
Restart=always
RestartSec=5"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("RestartSec=0")),
            "F091: Should accept non-zero RestartSec"
        );
    }

    /// F094: WantedBy target validation
    #[test]
    fn test_F094_valid_wantedby() {
        let unit = r#"[Service]
ExecStart=/usr/bin/test

[Install]
WantedBy=multi-user.target"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Unusual target")),
            "F094: Should accept common target"
        );
    }

    /// F095: EnvironmentFile validation
    #[test]
    fn test_F095_environment_file_absolute() {
        let unit = r#"[Service]
EnvironmentFile=/etc/default/myservice
ExecStart=/usr/bin/test"#;
        let result = check(unit);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("EnvironmentFile") && d.message.contains("absolute")),
            "F095: Should accept absolute EnvironmentFile path"
        );
    }

    #[test]
    fn test_F095_environment_file_relative() {
        let unit = r#"[Service]
EnvironmentFile=myservice.env
ExecStart=/usr/bin/test"#;
        let result = check(unit);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("EnvironmentFile") && d.message.contains("absolute")),
            "F095: Should warn about relative EnvironmentFile path"
        );
    }
}
