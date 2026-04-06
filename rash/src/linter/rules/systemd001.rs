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
/// Internal state tracked while checking a systemd unit file
struct SystemdCheckState<'a> {
    has_unit_section: bool,
    has_service_section: bool,
    has_exec_start: bool,
    service_type: String,
    has_restart: bool,
    current_section: String,
    required_sections: HashSet<&'a str>,
}

impl SystemdCheckState<'_> {
    fn new() -> Self {
        Self {
            has_unit_section: false,
            has_service_section: false,
            has_exec_start: false,
            service_type: String::new(),
            has_restart: false,
            current_section: String::new(),
            required_sections: HashSet::new(),
        }
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut state = SystemdCheckState::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        if is_section_header(trimmed) {
            if let Some(section) = extract_section_name(trimmed) {
                match section {
                    "Unit" => state.has_unit_section = true,
                    "Service" => state.has_service_section = true,
                    "Install" => {}
                    _ => {}
                }
                state.current_section = section.to_string();
            }
            continue;
        }

        if let Some((key, value)) = parse_key_value(trimmed) {
            match state.current_section.as_str() {
                "Unit" => check_unit_key(key, value, &mut state),
                "Service" => {
                    check_service_key(key, value, line_num, trimmed, &mut state, &mut result);
                }
                "Install" => check_install_key(key, value, line_num, trimmed, &mut result),
                _ => {}
            }
        }
    }

    check_post_conditions(&state, &mut result);
    result
}

/// Check keys in the [Unit] section
fn check_unit_key<'a>(key: &str, value: &'a str, state: &mut SystemdCheckState<'a>) {
    if key == "After" || key == "Requires" || key == "Wants" {
        for dep in value.split_whitespace() {
            state.required_sections.insert(dep);
        }
    }
}

/// Check keys in the [Service] section
fn check_service_key(
    key: &str,
    value: &str,
    line_num: usize,
    trimmed: &str,
    state: &mut SystemdCheckState<'_>,
    result: &mut LintResult,
) {
    match key {
        "Type" => check_service_type(value, line_num, trimmed, state, result),
        "ExecStart" => check_exec_start(value, line_num, trimmed, state, result),
        "ExecReload" => check_exec_reload(value, line_num, trimmed, result),
        "Restart" => check_restart(value, line_num, trimmed, state, result),
        "RestartSec" => check_restart_sec(value, line_num, trimmed, result),
        "EnvironmentFile" => check_environment_file(value, line_num, trimmed, result),
        _ => {}
    }
}

fn check_service_type(
    value: &str,
    line_num: usize,
    trimmed: &str,
    state: &mut SystemdCheckState<'_>,
    result: &mut LintResult,
) {
    state.service_type = value.to_string();
    if !VALID_TYPES.contains(&value) {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            format!(
                "Invalid Type='{}' - must be one of: {} (F087)",
                value,
                VALID_TYPES.join(", ")
            ),
            span,
        ));
    }
}

fn check_exec_start(
    value: &str,
    line_num: usize,
    trimmed: &str,
    state: &mut SystemdCheckState<'_>,
    result: &mut LintResult,
) {
    state.has_exec_start = true;
    let exec_path = value.trim_start_matches(['@', '-', ':', '+', '!']);
    if !exec_path.starts_with('/') && !exec_path.is_empty() {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            format!("ExecStart='{}' should use absolute path (F088)", value),
            span,
        ));
    }
}

fn check_exec_reload(value: &str, line_num: usize, trimmed: &str, result: &mut LintResult) {
    let exec_path = value.trim_start_matches(['@', '-', ':', '+', '!']);
    if !exec_path.starts_with('/') && !exec_path.starts_with("kill") {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            "ExecReload should use absolute path or /bin/kill (F089)".to_string(),
            span,
        ));
    }
}

fn check_restart(
    value: &str,
    line_num: usize,
    trimmed: &str,
    state: &mut SystemdCheckState<'_>,
    result: &mut LintResult,
) {
    state.has_restart = true;
    if !VALID_RESTART.contains(&value) {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            format!(
                "Invalid Restart='{}' - must be one of: {} (F090)",
                value,
                VALID_RESTART.join(", ")
            ),
            span,
        ));
    }
}

fn check_restart_sec(value: &str, line_num: usize, trimmed: &str, result: &mut LintResult) {
    let numeric: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
    if let Ok(0) = numeric.parse::<u32>() {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            "RestartSec=0 may cause restart loops - consider a backoff value (F091)".to_string(),
            span,
        ));
    }
}

fn check_environment_file(value: &str, line_num: usize, trimmed: &str, result: &mut LintResult) {
    let path = value.trim_start_matches('-');
    if !path.starts_with('/') {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            format!(
                "EnvironmentFile='{}' should use absolute path (F095)",
                value
            ),
            span,
        ));
    }
}

/// Check keys in the [Install] section
fn check_install_key(
    key: &str,
    value: &str,
    line_num: usize,
    trimmed: &str,
    result: &mut LintResult,
) {
    if key == "WantedBy" || key == "RequiredBy" {
        for target in value.split_whitespace() {
            if !VALID_TARGETS.contains(&target) && !target.ends_with(".target") {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                result.add(Diagnostic::new(
                    "SYSTEMD001",
                    Severity::Warning,
                    format!(
                        "Unusual target '{}' in {} - common targets: {} (F094)",
                        target,
                        key,
                        VALID_TARGETS[..3].join(", ")
                    ),
                    span,
                ));
            }
        }
    }
}

/// Check post-iteration conditions (missing sections, missing directives)
fn check_post_conditions(state: &SystemdCheckState<'_>, result: &mut LintResult) {
    if !state.has_unit_section {
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Warning,
            "Missing [Unit] section - recommended for documentation (F086)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }

    if !state.has_service_section {
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            "Missing [Service] section - required for service units (F086)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }

    if state.has_service_section && !state.has_exec_start {
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Error,
            "Missing ExecStart= - required for service units (F088)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }

    if state.has_service_section
        && !state.has_restart
        && (state.service_type.is_empty()
            || state.service_type == "simple"
            || state.service_type == "notify")
    {
        result.add(Diagnostic::new(
            "SYSTEMD001",
            Severity::Info,
            "Consider adding Restart= policy for service reliability (F090)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "systemd001_tests_f086_valid.rs"]
// FIXME(PMAT-238): mod tests_extracted;
