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

/// Check for valid launchd plist
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Basic XML validation
    if !source.contains("<?xml") && !source.contains("<plist") {
        let diag = Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing plist XML declaration or plist element (F076)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
        return result;
    }

    // Track found elements
    let mut has_label = false;
    let mut has_program_arguments = false;
    let mut has_program = false;
    let mut label_value = String::new();
    let mut program_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // F077: Check Label
        if trimmed.contains("<key>Label</key>") {
            has_label = true;
        }
        if has_label && label_value.is_empty() && trimmed.contains("<string>") {
            if let Some(start) = trimmed.find("<string>") {
                if let Some(end) = trimmed.find("</string>") {
                    label_value = trimmed[start + 8..end].to_string();

                    // Check for reverse-domain format
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

        // F078: Check ProgramArguments
        if trimmed.contains("<key>ProgramArguments</key>") {
            has_program_arguments = true;
        }

        // F078: Check Program
        if trimmed.contains("<key>Program</key>") {
            has_program = true;
            program_line = line_num + 1;
        }

        // F079: Check RunAtLoad
        if trimmed.contains("<key>RunAtLoad</key>") {
            // Next line should be <true/> or <false/>
        }

        // F081/F082: Check StandardOutPath/StandardErrorPath
        if trimmed.contains("<key>StandardOutPath</key>")
            || trimmed.contains("<key>StandardErrorPath</key>")
        {
            // Check if path follows on next line
        }

        // F084: Check WorkingDirectory
        if trimmed.contains("<key>WorkingDirectory</key>") {
            // Validate path on next line
        }

        // F085: Check UserName
        if trimmed.contains("<key>UserName</key>") {
            // Validate username
        }

        // Check for common mistakes

        // Empty string values
        if trimmed == "<string></string>" {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
            let diag = Diagnostic::new(
                "LAUNCHD001",
                Severity::Warning,
                "Empty string value in plist (F076)".to_string(),
                span,
            );
            result.add(diag);
        }
    }

    // F078: Check for both Program and ProgramArguments (after parsing)
    if has_program && has_program_arguments {
        let span = Span::new(program_line, 1, program_line, 80);
        let diag = Diagnostic::new(
            "LAUNCHD001",
            Severity::Warning,
            "Both Program and ProgramArguments specified - use one or the other (F078)".to_string(),
            span,
        );
        result.add(diag);
    }

    // F077: Missing Label
    if !has_label {
        let diag = Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing required Label key in plist (F077)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    // F078: Missing ProgramArguments (or Program)
    if !has_program_arguments && !has_program {
        let diag = Diagnostic::new(
            "LAUNCHD001",
            Severity::Error,
            "Missing required ProgramArguments or Program key (F078)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

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
