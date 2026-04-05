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
