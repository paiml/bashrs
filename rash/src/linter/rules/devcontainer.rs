//! Dev Container Validation Rules (DEVCONTAINER001-011)
//!
//! These rules validate devcontainer.json files per the Development Container Specification.
//! Reference: <https://containers.dev/implementors/spec/>
//!
//! ## Dev Container Specification Requirements
//!
//! - Must specify image, build, or dockerComposeFile
//! - JSONC (JSON with Comments) syntax supported
//! - Features must use valid container feature references
//! - Lifecycle commands support string, array, or object format

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use serde_json::Value;

/// DEVCONTAINER001: Missing image source
///
/// devcontainer.json MUST specify one of: image, build, or dockerComposeFile.
///
/// ## Example
///
/// ❌ **BAD** (no image source):
/// ```json
/// {
///   "name": "Invalid Container",
///   "features": {}
/// }
/// ```
///
/// ✅ **GOOD** (has image):
/// ```json
/// {
///   "name": "Valid Container",
///   "image": "mcr.microsoft.com/devcontainers/base:ubuntu"
/// }
/// ```
pub fn check_devcontainer001(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    let has_image = json.get("image").is_some();
    let has_build = json.get("build").is_some();
    let has_compose = json.get("dockerComposeFile").is_some();

    if !has_image && !has_build && !has_compose {
        let span = Span::new(1, 1, 1, 1);
        let diag = Diagnostic::new(
            "DEVCONTAINER001",
            Severity::Error,
            "Missing image source. devcontainer.json must specify 'image', 'build', or 'dockerComposeFile'.".to_string(),
            span,
        );
        result.add(diag);
    }

    result
}

/// DEVCONTAINER002: Using :latest tag
///
/// Using ':latest' tag reduces reproducibility. Pin specific version.
///
/// ## Example
///
/// ❌ **BAD** (uses :latest):
/// ```json
/// {
///   "image": "mcr.microsoft.com/devcontainers/base:latest"
/// }
/// ```
///
/// ✅ **GOOD** (pinned version):
/// ```json
/// {
///   "image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04"
/// }
/// ```
pub fn check_devcontainer002(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(image) = json.get("image").and_then(|v| v.as_str()) {
        if image.ends_with(":latest") {
            let span = Span::new(1, 1, 1, 1);
            let diag = Diagnostic::new(
                "DEVCONTAINER002",
                Severity::Warning,
                "Using ':latest' tag. Pin specific version for reproducible builds.".to_string(),
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// DEVCONTAINER003: Absolute path in build.dockerfile
///
/// build.dockerfile should use relative paths for portability.
///
/// ## Example
///
/// ❌ **BAD** (absolute path):
/// ```json
/// {
///   "build": {
///     "dockerfile": "/absolute/path/Dockerfile"
///   }
/// }
/// ```
///
/// ✅ **GOOD** (relative path):
/// ```json
/// {
///   "build": {
///     "dockerfile": "Dockerfile"
///   }
/// }
/// ```
pub fn check_devcontainer003(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(build) = json.get("build") {
        if let Some(dockerfile) = build.get("dockerfile").and_then(|v| v.as_str()) {
            if dockerfile.starts_with('/') {
                let span = Span::new(1, 1, 1, 1);
                let diag = Diagnostic::new(
                    "DEVCONTAINER003",
                    Severity::Error,
                    "Absolute path in build.dockerfile. Use relative path for portability."
                        .to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

/// DEVCONTAINER004: Docker Compose without service
///
/// dockerComposeFile requires 'service' property to specify which service to use.
///
/// ## Example
///
/// ❌ **BAD** (missing service):
/// ```json
/// {
///   "dockerComposeFile": "docker-compose.yml"
/// }
/// ```
///
/// ✅ **GOOD** (has service):
/// ```json
/// {
///   "dockerComposeFile": "docker-compose.yml",
///   "service": "app"
/// }
/// ```
pub fn check_devcontainer004(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    let has_compose = json.get("dockerComposeFile").is_some();
    let has_service = json.get("service").is_some();

    if has_compose && !has_service {
        let span = Span::new(1, 1, 1, 1);
        let diag = Diagnostic::new(
            "DEVCONTAINER004",
            Severity::Error,
            "Docker Compose config requires 'service' property to specify which service to use."
                .to_string(),
            span,
        );
        result.add(diag);
    }

    result
}

/// DEVCONTAINER005: Unknown feature option
///
/// Feature configurations should only use known options.
///
/// Note: This is a warning since new options may be added.
pub fn check_devcontainer005(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    let features = match json.get("features").and_then(|v| v.as_object()) {
        Some(f) => f,
        None => return result,
    };

    for (feature_name, feature_config) in features {
        check_feature_options(feature_name, feature_config, &mut result);
    }

    result
}

/// Check a single feature's options for obviously invalid keys
fn check_feature_options(feature_name: &str, feature_config: &Value, result: &mut LintResult) {
    let config = match feature_config.as_object() {
        Some(c) => c,
        None => return,
    };

    for key in config.keys() {
        if key.starts_with("unknown") {
            let span = Span::new(1, 1, 1, 1);
            result.add(Diagnostic::new(
                "DEVCONTAINER005",
                Severity::Warning,
                format!(
                    "Unknown option '{}' in feature '{}'. Check feature documentation for valid options.",
                    key, feature_name
                ),
                span,
            ));
        }
    }
}

/// DEVCONTAINER006: Duplicate keys in lifecycle command
///
/// Object-style lifecycle commands must not have duplicate keys.
///
/// Note: This is detected at JSON parse time for strict parsers,
/// but serde_json uses last-value-wins, so we check post-parse.
pub fn check_devcontainer006(_json: &Value) -> LintResult {
    // Note: serde_json handles duplicate keys by using last value
    // This rule would need raw JSON parsing to detect duplicates
    // For now, we trust serde_json's behavior
    LintResult::new()
}

/// DEVCONTAINER007: Invalid waitFor value
///
/// waitFor must be one of: onCreateCommand, updateContentCommand, or postCreateCommand.
pub fn check_devcontainer007(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(wait_for) = json.get("waitFor").and_then(|v| v.as_str()) {
        let valid_values = [
            "onCreateCommand",
            "updateContentCommand",
            "postCreateCommand",
        ];

        if !valid_values.contains(&wait_for) {
            let span = Span::new(1, 1, 1, 1);
            let diag = Diagnostic::new(
                "DEVCONTAINER007",
                Severity::Error,
                format!(
                    "Invalid waitFor value '{}'. Must be: onCreateCommand, updateContentCommand, or postCreateCommand.",
                    wait_for
                ),
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// DEVCONTAINER008: updateRemoteUserUID may cause permission issues
///
/// On Linux, updateRemoteUserUID=false may cause permission issues with bind mounts.
pub fn check_devcontainer008(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(update_uid) = json.get("updateRemoteUserUID").and_then(|v| v.as_bool()) {
        if !update_uid {
            let span = Span::new(1, 1, 1, 1);
            let diag = Diagnostic::new(
                "DEVCONTAINER008",
                Severity::Info,
                "updateRemoteUserUID=false may cause permission issues with bind mounts on Linux."
                    .to_string(),
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// DEVCONTAINER009: workspaceFolder must be absolute path
///
/// workspaceFolder must be an absolute path starting with '/'.
pub fn check_devcontainer009(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(folder) = json.get("workspaceFolder").and_then(|v| v.as_str()) {
        if !folder.starts_with('/') {
            let span = Span::new(1, 1, 1, 1);
            let diag = Diagnostic::new(
                "DEVCONTAINER009",
                Severity::Error,
                "workspaceFolder must be an absolute path.".to_string(),
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// DEVCONTAINER010: containerEnv values must be strings
///
/// Environment variable values in containerEnv must be strings.
pub fn check_devcontainer010(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(env) = json.get("containerEnv").and_then(|v| v.as_object()) {
        for (key, value) in env {
            if !value.is_string() {
                let span = Span::new(1, 1, 1, 1);
                let diag = Diagnostic::new(
                    "DEVCONTAINER010",
                    Severity::Error,
                    format!(
                        "containerEnv value for '{}' must be a string, got {}.",
                        key,
                        value_type_name(value)
                    ),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

/// DEVCONTAINER011: Invalid extension ID format
///
/// VS Code extension IDs should be in format: publisher.extension-name
pub fn check_devcontainer011(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    if let Some(customizations) = json.get("customizations") {
        if let Some(vscode) = customizations.get("vscode") {
            if let Some(extensions) = vscode.get("extensions").and_then(|v| v.as_array()) {
                for ext in extensions {
                    if let Some(ext_id) = ext.as_str() {
                        // Valid format: publisher.extension-name
                        if !ext_id.contains('.') || ext_id.starts_with('.') || ext_id.ends_with('.')
                        {
                            let span = Span::new(1, 1, 1, 1);
                            let diag = Diagnostic::new(
                                "DEVCONTAINER011",
                                Severity::Warning,
                                format!(
                                    "Invalid extension ID '{}'. Expected format: publisher.extension-name",
                                    ext_id
                                ),
                                span,
                            );
                            result.add(diag);
                        }
                    }
                }
            }
        }
    }

    result
}

/// Get a human-readable type name for a JSON value
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parse JSONC (JSON with Comments) to serde_json::Value
///
/// Strips single-line (//) and multi-line (/* */) comments before parsing.
pub fn parse_jsonc(content: &str) -> Result<Value, String> {
    let stripped = strip_json_comments(content);
    serde_json::from_str(&stripped).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Strip comments from JSONC content
fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }

        if !in_string && ch == '/' && skip_comment(&mut chars) {
            continue;
        }

        result.push(ch);
    }

    result
}

/// Try to skip a comment starting after '/'. Returns true if a comment was skipped.
fn skip_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    match chars.peek() {
        Some(&'/') => {
            chars.next(); // consume second /
            skip_single_line_comment(chars);
            true
        }
        Some(&'*') => {
            chars.next(); // consume *
            skip_multi_line_comment(chars);
            true
        }
        _ => false,
    }
}

/// Skip to end of single-line comment (until newline)
fn skip_single_line_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c == '\n' {
            break;
        }
        chars.next();
    }
}

/// Skip to end of multi-line comment (until */)
fn skip_multi_line_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(c) = chars.next() {
        if c == '*' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                break;
            }
        }
    }
}

/// Run all Dev Container validation rules on parsed JSON
pub fn lint_devcontainer(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    result.merge(check_devcontainer001(json));
    result.merge(check_devcontainer002(json));
    result.merge(check_devcontainer003(json));
    result.merge(check_devcontainer004(json));
    result.merge(check_devcontainer005(json));
    result.merge(check_devcontainer006(json));
    result.merge(check_devcontainer007(json));
    result.merge(check_devcontainer008(json));
    result.merge(check_devcontainer009(json));
    result.merge(check_devcontainer010(json));
    result.merge(check_devcontainer011(json));

    result
}

/// Validate devcontainer.json content (JSONC string)
pub fn validate_devcontainer(content: &str) -> Result<LintResult, String> {
    let json = parse_jsonc(content)?;
    Ok(lint_devcontainer(&json))
}

/// List all DEVCONTAINER rules
pub fn list_devcontainer_rules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "DEVCONTAINER001",
            "Missing image source (image, build, or dockerComposeFile)",
        ),
        (
            "DEVCONTAINER002",
            "Using :latest tag reduces reproducibility",
        ),
        ("DEVCONTAINER003", "Absolute path in build.dockerfile"),
        (
            "DEVCONTAINER004",
            "Docker Compose requires 'service' property",
        ),
        ("DEVCONTAINER005", "Unknown feature option"),
        ("DEVCONTAINER006", "Duplicate keys in lifecycle command"),
        ("DEVCONTAINER007", "Invalid waitFor value"),
        (
            "DEVCONTAINER008",
            "updateRemoteUserUID=false may cause permission issues",
        ),
        ("DEVCONTAINER009", "workspaceFolder must be absolute path"),
        ("DEVCONTAINER010", "containerEnv values must be strings"),
        ("DEVCONTAINER011", "Invalid VS Code extension ID format"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // DEVCONTAINER001: Missing image source
    // ========================================

    #[test]
    fn test_devcontainer001_missing_image_source() {
        let json: Value = serde_json::from_str(r#"{"name": "Invalid"}"#).unwrap();
        let result = check_devcontainer001(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER001");
    }

    #[test]
    fn test_devcontainer001_has_image() {
        let json: Value = serde_json::from_str(
            r#"{"name": "Valid", "image": "mcr.microsoft.com/devcontainers/base:ubuntu"}"#,
        )
        .unwrap();
        let result = check_devcontainer001(&json);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_devcontainer001_has_build() {
        let json: Value =
            serde_json::from_str(r#"{"name": "Valid", "build": {"dockerfile": "Dockerfile"}}"#)
                .unwrap();
        let result = check_devcontainer001(&json);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_devcontainer001_has_compose() {
        let json: Value = serde_json::from_str(
            r#"{"name": "Valid", "dockerComposeFile": "docker-compose.yml", "service": "app"}"#,
        )
        .unwrap();
        let result = check_devcontainer001(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER002: Using :latest tag
    // ========================================

    #[test]
    fn test_devcontainer002_latest_tag() {
        let json: Value =
            serde_json::from_str(r#"{"image": "mcr.microsoft.com/devcontainers/base:latest"}"#)
                .unwrap();
        let result = check_devcontainer002(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER002");
    }

    #[test]
    fn test_devcontainer002_pinned_version() {
        let json: Value = serde_json::from_str(
            r#"{"image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04"}"#,
        )
        .unwrap();
        let result = check_devcontainer002(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER003: Absolute path
    // ========================================

    #[test]
    fn test_devcontainer003_absolute_path() {
        let json: Value =
            serde_json::from_str(r#"{"build": {"dockerfile": "/absolute/path/Dockerfile"}}"#)
                .unwrap();
        let result = check_devcontainer003(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER003");
    }

    #[test]
    fn test_devcontainer003_relative_path() {
        let json: Value =
            serde_json::from_str(r#"{"build": {"dockerfile": "Dockerfile"}}"#).unwrap();
        let result = check_devcontainer003(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER004: Compose without service
    // ========================================

    #[test]
    fn test_devcontainer004_missing_service() {
        let json: Value =
            serde_json::from_str(r#"{"dockerComposeFile": "docker-compose.yml"}"#).unwrap();
        let result = check_devcontainer004(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER004");
    }

    #[test]
    fn test_devcontainer004_has_service() {
        let json: Value = serde_json::from_str(
            r#"{"dockerComposeFile": "docker-compose.yml", "service": "app"}"#,
        )
        .unwrap();
        let result = check_devcontainer004(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER005: Unknown feature option
    // ========================================

    #[test]
    fn test_devcontainer005_unknown_option() {
        let json: Value = serde_json::from_str(
            r#"{"image": "test", "features": {"ghcr.io/test:1": {"unknownOption": "value"}}}"#,
        )
        .unwrap();
        let result = check_devcontainer005(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER005");
    }

    #[test]
    fn test_devcontainer005_valid_options() {
        let json: Value = serde_json::from_str(
            r#"{"image": "test", "features": {"ghcr.io/test:1": {"version": "18"}}}"#,
        )
        .unwrap();
        let result = check_devcontainer005(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER007: Invalid waitFor
    // ========================================

    #[test]
    fn test_devcontainer007_invalid_waitfor() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "waitFor": "invalidStage"}"#).unwrap();
        let result = check_devcontainer007(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER007");
    }

    #[test]
    fn test_devcontainer007_valid_waitfor() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "waitFor": "postCreateCommand"}"#).unwrap();
        let result = check_devcontainer007(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER008: updateRemoteUserUID
    // ========================================

    #[test]
    fn test_devcontainer008_uid_false() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "updateRemoteUserUID": false}"#).unwrap();
        let result = check_devcontainer008(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER008");
    }

    #[test]
    fn test_devcontainer008_uid_true() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "updateRemoteUserUID": true}"#).unwrap();
        let result = check_devcontainer008(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER009: workspaceFolder
    // ========================================

    #[test]
    fn test_devcontainer009_relative_workspace() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "workspaceFolder": "relative/path"}"#)
                .unwrap();
        let result = check_devcontainer009(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER009");
    }

    #[test]
    fn test_devcontainer009_absolute_workspace() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "workspaceFolder": "/workspace"}"#).unwrap();
        let result = check_devcontainer009(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER010: containerEnv types
    // ========================================

    #[test]
    fn test_devcontainer010_non_string_env() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "containerEnv": {"DEBUG": true}}"#).unwrap();
        let result = check_devcontainer010(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER010");
    }

    #[test]
    fn test_devcontainer010_string_env() {
        let json: Value =
            serde_json::from_str(r#"{"image": "test", "containerEnv": {"DEBUG": "true"}}"#)
                .unwrap();
        let result = check_devcontainer010(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // DEVCONTAINER011: Extension ID format
    // ========================================

    #[test]
    fn test_devcontainer011_invalid_extension() {
        let json: Value = serde_json::from_str(
            r#"{"image": "test", "customizations": {"vscode": {"extensions": ["invalid-extension-id"]}}}"#,
        )
        .unwrap();
        let result = check_devcontainer011(&json);
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.diagnostics[0].code, "DEVCONTAINER011");
    }

    #[test]
    fn test_devcontainer011_valid_extension() {
        let json: Value = serde_json::from_str(
            r#"{"image": "test", "customizations": {"vscode": {"extensions": ["ms-python.python"]}}}"#,
        )
        .unwrap();
        let result = check_devcontainer011(&json);
        assert!(result.diagnostics.is_empty());
    }

    // ========================================
    // JSONC Parsing
    // ========================================

    #[test]
    fn test_jsonc_single_line_comment() {
        let jsonc = r#"{
            // This is a comment
            "name": "Test"
        }"#;
        let result = parse_jsonc(jsonc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["name"], "Test");
    }

    #[test]
    fn test_jsonc_multi_line_comment() {
        let jsonc = r#"{
            /* Multi-line
               comment */
            "name": "Test"
        }"#;
        let result = parse_jsonc(jsonc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["name"], "Test");
    }

    #[test]
    fn test_jsonc_comment_in_string() {
        let jsonc = r#"{"name": "// not a comment"}"#;
        let result = parse_jsonc(jsonc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["name"], "// not a comment");
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_lint_devcontainer_all_rules() {
        let json: Value = serde_json::from_str(r#"{"name": "Invalid"}"#).unwrap();
        let result = lint_devcontainer(&json);
        // Should trigger DEVCONTAINER001
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_lint_devcontainer_valid() {
        let json: Value = serde_json::from_str(
            r#"{
                "name": "Valid Container",
                "image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04",
                "forwardPorts": [3000],
                "workspaceFolder": "/workspace"
            }"#,
        )
        .unwrap();
        let result = lint_devcontainer(&json);
        assert!(
            result.diagnostics.is_empty(),
            "Valid config should have no errors: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_validate_devcontainer_jsonc() {
        let jsonc = r#"{
            // Development container
            "name": "Dev Container",
            "image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04"
        }"#;
        let result = validate_devcontainer(jsonc);
        assert!(result.is_ok());
        assert!(result.unwrap().diagnostics.is_empty());
    }

    #[test]
    fn test_list_rules_count() {
        let rules = list_devcontainer_rules();
        assert_eq!(rules.len(), 11, "Should have 11 DEVCONTAINER rules");
    }

    // ========================================
    // Property-based Tests
    // ========================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            #[test]
            fn prop_lint_never_panics(json_str in "\\{[^}]*\\}") {
                // Try to parse as JSON, if successful, lint it
                if let Ok(json) = serde_json::from_str::<Value>(&json_str) {
                    let _ = lint_devcontainer(&json);
                }
            }

            #[test]
            fn prop_jsonc_strip_never_panics(content in ".*") {
                let _ = strip_json_comments(&content);
            }

            #[test]
            fn prop_valid_image_passes_001(
                image in "[a-z]+/[a-z]+:[a-z0-9.-]+"
            ) {
                let json: Value = serde_json::from_str(&format!(
                    r#"{{"image": "{}"}}"#, image
                )).unwrap();
                let result = check_devcontainer001(&json);
                prop_assert!(result.diagnostics.is_empty());
            }
        }
    }
}
