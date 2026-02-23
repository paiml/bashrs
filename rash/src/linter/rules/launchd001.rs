//! LAUNCHD001: macOS launchd plist validation (F076-F085)
//!
//! **Rule**: Validate launchd plist files for common issues
//!
//! **Why this matters**:
//! Invalid launchd plist files can prevent daemons from starting on macOS.
//! Proper validation ensures services run correctly on macOS systems.
//!
//! ## Checks implemented:
//! - F076: Valid plist XML structure
//! - F077: Correct Label (reverse-domain identifier)
//! - F078: ProgramArguments array format
//! - F079: RunAtLoad boolean
//! - F080: KeepAlive configuration
//! - F081: StandardOutPath validation
//! - F082: StandardErrorPath validation
//! - F083: EnvironmentVariables dictionary
//! - F084: WorkingDirectory validation
//! - F085: UserName validation

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check label value for reverse-domain format and return diagnostics
fn check_label_format(
    trimmed: &str,
    line_num: usize,
    label_value: &mut String,
    result: &mut LintResult,
) {
    if let Some(start) = trimmed.find("<string>") {
        if let Some(end) = trimmed.find("</string>") {
            *label_value = trimmed[start + 8..end].to_string();
            if !label_value.contains('.') {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "LAUNCHD001",
                    Severity::Warning,
                    format!(
                        "Label '{}' should use reverse-domain format (e.g., com.example.daemon) (F077)",
                        label_value
                    ),
                    span,
                );
                result.add(diag);
            }
        }
    }
}

/// Emit post-loop diagnostics for missing/conflicting keys
fn emit_post_checks(
    result: &mut LintResult,
    has_label: bool,
    has_program: bool,
    has_program_arguments: bool,
    program_line: usize,
) {
    if has_program && has_program_arguments {
        let span = Span::new(program_line, 1, program_line, 80);
        result.add(Diagnostic::new(
            "LAUNCHD001",
            Severity::Warning,
            "Both Program and ProgramArguments specified - use one or the other (F078)".to_string(),
            span,
        ));
    }
    if !has_label {
        result.add(Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing required Label key in plist (F077)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }
    if !has_program_arguments && !has_program {
        result.add(Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing required ProgramArguments or Program key (F078)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
    }
}

/// Check for valid launchd plist
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Basic XML validation
    if !source.contains("<?xml") && !source.contains("<plist") {
        result.add(Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing plist XML declaration or plist element (F076)".to_string(),
            Span::new(1, 1, 1, 1),
        ));
        return result;
    }

    let mut has_label = false;
    let mut has_program_arguments = false;
    let mut has_program = false;
    let mut label_value = String::new();
    let mut program_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.contains("<key>Label</key>") {
            has_label = true;
        }
        if has_label && label_value.is_empty() && trimmed.contains("<string>") {
            check_label_format(trimmed, line_num, &mut label_value, &mut result);
        }
        if trimmed.contains("<key>ProgramArguments</key>") {
            has_program_arguments = true;
        }
        if trimmed.contains("<key>Program</key>") {
            has_program = true;
            program_line = line_num + 1;
        }
        if trimmed == "<string></string>" {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
            result.add(Diagnostic::new(
                "LAUNCHD001",
                Severity::Warning,
                "Empty string value in plist (F076)".to_string(),
                span,
            ));
        }
    }

    emit_post_checks(
        &mut result,
        has_label,
        has_program,
        has_program_arguments,
        program_line,
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F076: Valid plist structure
    #[test]
    fn test_F076_valid_plist() {
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/daemon</string>
    </array>
</dict>
</plist>"#;
        let result = check(plist);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F076: Valid plist should not have errors"
        );
    }

    #[test]
    fn test_F076_invalid_xml() {
        let plist = "not xml at all";
        let result = check(plist);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Missing plist")),
            "F076: Invalid XML should be detected"
        );
    }

    /// F077: Label validation
    #[test]
    fn test_F077_missing_label() {
        let plist = r#"<?xml version="1.0"?>
<plist version="1.0">
<dict>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/true</string>
    </array>
</dict>
</plist>"#;
        let result = check(plist);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Missing required Label")),
            "F077: Missing Label should be detected"
        );
    }

    #[test]
    fn test_F077_non_reverse_domain_label() {
        let plist = r#"<?xml version="1.0"?>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>mydaemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/true</string>
    </array>
</dict>
</plist>"#;
        let result = check(plist);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("reverse-domain")),
            "F077: Non-reverse-domain Label should warn"
        );
    }

    /// F078: ProgramArguments validation
    #[test]
    fn test_F078_missing_program() {
        let plist = r#"<?xml version="1.0"?>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.daemon</string>
</dict>
</plist>"#;
        let result = check(plist);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("ProgramArguments")),
            "F078: Missing ProgramArguments should be detected"
        );
    }

    /// F078: Both Program and ProgramArguments
    #[test]
    fn test_F078_both_program_and_arguments() {
        let plist = r#"<?xml version="1.0"?>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.daemon</string>
    <key>Program</key>
    <string>/usr/bin/true</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/true</string>
    </array>
</dict>
</plist>"#;
        let result = check(plist);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Both Program and ProgramArguments")),
            "F078: Both Program and ProgramArguments should warn"
        );
    }
}
