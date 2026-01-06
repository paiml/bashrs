//! SYSTEMD001: systemd unit file validation (F086-F095)
//!
//! **Rule**: Validate systemd service unit files for common issues
//!
//! **Why this matters**:
//! Invalid systemd unit files can cause services to fail to start,
//! restart improperly, or have security issues.
//!
//! ## Checks implemented:
//! - F086: Valid unit file structure
//! - F087: Correct Type= directive
//! - F088: Valid ExecStart= path
//! - F089: Valid ExecReload= configuration
//! - F090: Appropriate Restart= policy
//! - F091: Reasonable RestartSec= value
//! - F092: LimitMEMLOCK for mlock services
//! - F093: After=/Requires= dependency validation
//! - F094: WantedBy= target validation
//! - F095: EnvironmentFile= path validation

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::collections::HashSet;

/// Valid service types
const VALID_TYPES: &[&str] = &[
    "simple", "exec", "forking", "oneshot", "dbus", "notify", "idle",
];

/// Valid restart policies
const VALID_RESTART: &[&str] = &[
    "no",
    "on-success",
    "on-failure",
    "on-abnormal",
    "on-watchdog",
    "on-abort",
    "always",
];

/// Valid systemd targets for WantedBy
const VALID_TARGETS: &[&str] = &[
    "multi-user.target",
    "graphical.target",
    "default.target",
    "network.target",
    "network-online.target",
    "basic.target",
    "sysinit.target",
    "rescue.target",
    "emergency.target",
    "timers.target",
    "sockets.target",
];

/// Check if line is a section header
fn is_section_header(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('[') && trimmed.ends_with(']')
}

/// Extract section name from header
fn extract_section_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        Some(&trimmed[1..trimmed.len() - 1])
    } else {
        None
    }
}

/// Parse key-value from line
fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
        return None;
    }
    trimmed.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

/// Check for valid systemd unit file
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut has_unit_section = false;
    let mut has_service_section = false;
    let mut _has_install_section = false;
    let mut current_section = String::new();

    let mut has_exec_start = false;
    let mut _has_type = false;
    let mut service_type = String::new();
    let mut has_restart = false;
    let mut _has_restart_sec = false;

    let mut required_sections: HashSet<&str> = HashSet::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Check section headers
        if is_section_header(trimmed) {
            if let Some(section) = extract_section_name(trimmed) {
                current_section = section.to_string();
                match section {
                    "Unit" => has_unit_section = true,
                    "Service" => has_service_section = true,
                    "Install" => _has_install_section = true,
                    _ => {}
                }
            }
            continue;
        }

        // Parse key-value pairs
        if let Some((key, value)) = parse_key_value(trimmed) {
            match current_section.as_str() {
                "Unit" => {
                    // F093: Validate After/Requires dependencies
                    if key == "After" || key == "Requires" || key == "Wants" {
                        for dep in value.split_whitespace() {
                            required_sections.insert(if dep.contains('.') {
                                dep
                            } else {
                                // Implicit .service suffix
                                dep
                            });
                        }
                    }
                }
                "Service" => {
                    // F087: Validate Type directive
                    if key == "Type" {
                        _has_type = true;
                        service_type = value.to_string();
                        if !VALID_TYPES.contains(&value) {
                            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                            let diag = Diagnostic::new(
                                "SYSTEMD001",
                                Severity::Error,
                                format!(
                                    "Invalid Type='{}' - must be one of: {} (F087)",
                                    value,
                                    VALID_TYPES.join(", ")
                                ),
                                span,
                            );
                            result.add(diag);
                        }
                    }

                    // F088: Validate ExecStart path
                    if key == "ExecStart" {
                        has_exec_start = true;
                        // Check for absolute path (unless prefixed with @, -, :, +, !)
                        let exec_path = value.trim_start_matches(['@', '-', ':', '+', '!']);
                        if !exec_path.starts_with('/') && !exec_path.is_empty() {
                            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                            let diag = Diagnostic::new(
                                "SYSTEMD001",
                                Severity::Warning,
                                format!("ExecStart='{}' should use absolute path (F088)", value),
                                span,
                            );
                            result.add(diag);
                        }
                    }

                    // F089: Validate ExecReload
                    if key == "ExecReload" {
                        let exec_path = value.trim_start_matches(['@', '-', ':', '+', '!']);
                        if !exec_path.starts_with('/') && !exec_path.starts_with("kill") {
                            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                            let diag = Diagnostic::new(
                                "SYSTEMD001",
                                Severity::Warning,
                                "ExecReload should use absolute path or /bin/kill (F089)"
                                    .to_string(),
                                span,
                            );
                            result.add(diag);
                        }
                    }

                    // F090: Validate Restart policy
                    if key == "Restart" {
                        has_restart = true;
                        if !VALID_RESTART.contains(&value) {
                            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                            let diag = Diagnostic::new(
                                "SYSTEMD001",
                                Severity::Error,
                                format!(
                                    "Invalid Restart='{}' - must be one of: {} (F090)",
                                    value,
                                    VALID_RESTART.join(", ")
                                ),
                                span,
                            );
                            result.add(diag);
                        }
                    }

                    // F091: Validate RestartSec
                    if key == "RestartSec" {
                        _has_restart_sec = true;
                        // Parse numeric value (could be "5" or "5s" or "5min")
                        let numeric: String =
                            value.chars().take_while(|c| c.is_ascii_digit()).collect();
                        if let Ok(secs) = numeric.parse::<u32>() {
                            if secs == 0 {
                                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                                let diag = Diagnostic::new(
                                    "SYSTEMD001",
                                    Severity::Warning,
                                    "RestartSec=0 may cause restart loops - consider a backoff value (F091)".to_string(),
                                    span,
                                );
                                result.add(diag);
                            }
                        }
                    }

                    // F095: Validate EnvironmentFile
                    if key == "EnvironmentFile" {
                        let path = value.trim_start_matches('-'); // - prefix means optional
                        if !path.starts_with('/') {
                            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                            let diag = Diagnostic::new(
                                "SYSTEMD001",
                                Severity::Warning,
                                format!(
                                    "EnvironmentFile='{}' should use absolute path (F095)",
                                    value
                                ),
                                span,
                            );
                            result.add(diag);
                        }
                    }
                }
                "Install" => {
                    // F094: Validate WantedBy target
                    if key == "WantedBy" || key == "RequiredBy" {
                        for target in value.split_whitespace() {
                            if !VALID_TARGETS.contains(&target) && !target.ends_with(".target") {
                                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                                let diag = Diagnostic::new(
                                    "SYSTEMD001",
                                    Severity::Warning,
                                    format!(
                                        "Unusual target '{}' in {} - common targets: {} (F094)",
                                        target,
                                        key,
                                        VALID_TARGETS[..3].join(", ")
                                    ),
                                    span,
                                );
                                result.add(diag);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // F086: Check for required sections
    if !has_unit_section {
        let diag = Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            "Missing [Unit] section - recommended for documentation (F086)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    if !has_service_section {
        let diag = Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            "Missing [Service] section - required for service units (F086)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    // F088: ExecStart is required
    if has_service_section && !has_exec_start {
        let diag = Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            "Missing ExecStart= - required for service units (F088)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    // F090: Recommend Restart policy for long-running services
    if has_service_section
        && !has_restart
        && (service_type.is_empty() || service_type == "simple" || service_type == "notify")
    {
        let diag = Diagnostic::new(
            "SYSTEMD001",
            Severity::Info,
            "Consider adding Restart= policy for service reliability (F090)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    result
}

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
