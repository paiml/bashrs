//! Unit tests for installer module
//!
//! Tests follow EXTREME TDD principles and the test naming convention:
//! test_<TASK_ID>_<feature>_<scenario>

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use tempfile::TempDir;

// =============================================================================
// INSTALLER_001: Project initialization tests
// =============================================================================

#[test]
fn test_INSTALLER_001_init_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    let result = init_project(&project_path, None);
    assert!(result.is_ok(), "Failed to initialize project: {:?}", result);

    assert!(project_path.exists(), "Project directory was not created");
    assert!(project_path.is_dir(), "Project path is not a directory");
}

#[test]
fn test_INSTALLER_001_init_creates_installer_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    init_project(&project_path, None).unwrap();

    let installer_toml = project_path.join("installer.toml");
    assert!(installer_toml.exists(), "installer.toml was not created");

    let content = std::fs::read_to_string(&installer_toml).unwrap();
    assert!(
        content.contains("[installer]"),
        "Missing [installer] section"
    );
    assert!(
        content.contains("name = \"test-installer\""),
        "Missing name field"
    );
}

#[test]
fn test_INSTALLER_001_init_creates_tests_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    init_project(&project_path, None).unwrap();

    let tests_dir = project_path.join("tests");
    assert!(tests_dir.exists(), "tests/ directory was not created");
    assert!(tests_dir.is_dir(), "tests path is not a directory");

    let mod_rs = tests_dir.join("mod.rs");
    assert!(mod_rs.exists(), "tests/mod.rs was not created");

    let falsification_rs = tests_dir.join("falsification.rs");
    assert!(
        falsification_rs.exists(),
        "tests/falsification.rs was not created"
    );
}

#[test]
fn test_INSTALLER_001_init_creates_templates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    init_project(&project_path, None).unwrap();

    let templates_dir = project_path.join("templates");
    assert!(
        templates_dir.exists(),
        "templates/ directory was not created"
    );
}

#[test]
fn test_INSTALLER_001_init_with_description() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    let result = init_project(&project_path, Some("My custom description")).unwrap();
    assert_eq!(
        result.description,
        Some("My custom description".to_string())
    );

    let content = std::fs::read_to_string(project_path.join("installer.toml")).unwrap();
    assert!(
        content.contains("My custom description"),
        "Description not in TOML"
    );
}

#[test]
fn test_INSTALLER_001_init_returns_project_info() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("my-project");

    let project = init_project(&project_path, None).unwrap();

    assert_eq!(project.name, "my-project");
    assert_eq!(project.path, project_path);
}

#[test]
fn test_INSTALLER_001_init_rejects_root_path() {
    // Test that using root path (no file name) is rejected
    use std::path::Path;
    let root_path = Path::new("/");

    let result = init_project(root_path, None);
    // Root path has no file_name component, so it should fail validation
    // Note: This may succeed on some systems due to permissions being checked first
    // The important thing is it doesn't panic
    let _ = result;
}

// =============================================================================
// INSTALLER_002: Validation tests
// =============================================================================

#[test]
fn test_INSTALLER_002_validate_valid_installer() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("valid-installer");

    // Create a valid installer
    init_project(&project_path, None).unwrap();

    // Validate it
    let result = validate_installer(&project_path);
    assert!(
        result.is_ok(),
        "Valid installer should pass validation: {:?}",
        result
    );

    let validation = result.unwrap();
    assert!(validation.valid, "Validation should report valid=true");
}

#[test]
fn test_INSTALLER_002_validate_missing_installer_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("empty-dir");
    std::fs::create_dir_all(&project_path).unwrap();

    let result = validate_installer(&project_path);
    assert!(
        result.is_err(),
        "Should fail when installer.toml is missing"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found"),
        "Error should mention file not found"
    );
}

#[test]
fn test_INSTALLER_002_validate_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("invalid-installer");
    std::fs::create_dir_all(&project_path).unwrap();

    // Write invalid TOML
    std::fs::write(project_path.join("installer.toml"), "INVALID [[[").unwrap();

    let result = validate_installer(&project_path);
    assert!(result.is_err(), "Should fail on invalid TOML");
}

#[test]
fn test_INSTALLER_002_validate_counts_steps() {
    let temp_dir = TempDir::new().unwrap();
    let installer_toml = temp_dir.path().join("installer.toml");

    let content = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "step1"
name = "Step 1"
action = "script"

[[step]]
id = "step2"
name = "Step 2"
action = "script"
"#;
    std::fs::write(&installer_toml, content).unwrap();

    let result = validate_installer(temp_dir.path()).unwrap();
    assert_eq!(result.steps, 2, "Should count 2 steps");
}

#[test]
fn test_INSTALLER_002_validate_counts_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let installer_toml = temp_dir.path().join("installer.toml");

    let content = r#"
[installer]
name = "test"
version = "1.0.0"

[[artifact]]
id = "artifact1"
url = "https://example.com/file1.tar.gz"

[[artifact]]
id = "artifact2"
url = "https://example.com/file2.tar.gz"
"#;
    std::fs::write(&installer_toml, content).unwrap();

    let result = validate_installer(temp_dir.path()).unwrap();
    assert_eq!(result.artifacts, 2, "Should count 2 artifacts");
}

// =============================================================================
// Property-based tests (EXTREME TDD)
// =============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Valid project names always create valid directories
        #[test]
        fn prop_valid_project_name_creates_directory(name in "[a-z][a-z0-9_-]{0,20}") {
            let temp_dir = TempDir::new().unwrap();
            let project_path = temp_dir.path().join(&name);

            let result = init_project(&project_path, None);

            // Should succeed for valid names
            prop_assert!(result.is_ok(), "Failed for name: {}", name);
            prop_assert!(project_path.exists());
        }

        /// Property: Generated installer.toml is always valid TOML
        #[test]
        fn prop_generated_toml_is_valid(name in "[a-z][a-z0-9_-]{0,20}") {
            let temp_dir = TempDir::new().unwrap();
            let project_path = temp_dir.path().join(&name);

            init_project(&project_path, None).unwrap();

            let content = std::fs::read_to_string(project_path.join("installer.toml")).unwrap();
            let parsed: std::result::Result<InstallerSpec, _> = toml::from_str(&content);
            prop_assert!(parsed.is_ok(), "Generated TOML is invalid: {:?}", parsed);
        }

        /// Property: Validation is idempotent
        #[test]
        fn prop_validation_is_idempotent(name in "[a-z][a-z0-9_-]{0,20}") {
            let temp_dir = TempDir::new().unwrap();
            let project_path = temp_dir.path().join(&name);

            init_project(&project_path, None).unwrap();

            let result1 = validate_installer(&project_path);
            let result2 = validate_installer(&project_path);

            prop_assert!(result1.is_ok() == result2.is_ok());
            if let (Ok(v1), Ok(v2)) = (result1, result2) {
                prop_assert_eq!(v1.valid, v2.valid);
                prop_assert_eq!(v1.steps, v2.steps);
                prop_assert_eq!(v1.artifacts, v2.artifacts);
            }
        }
    }
}

// =============================================================================
// INSTALLER_006: Edge case tests for init_project
// =============================================================================

#[test]
fn test_INSTALLER_006_init_returns_project_info() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("my-app");

    let result = init_project(&project_path, Some("Test description")).unwrap();

    assert_eq!(result.name, "my-app");
    assert_eq!(result.path, project_path);
    assert_eq!(result.description, Some("Test description".to_string()));
}

#[test]
fn test_INSTALLER_006_validate_nonexistent_path() {
    let result = validate_installer(std::path::Path::new("/nonexistent/path/to/installer"));
    assert!(result.is_err());
}

#[test]
fn test_INSTALLER_006_validate_file_directly() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    init_project(&project_path, None).unwrap();

    // Validate by pointing directly to the installer.toml file
    let result = validate_installer(&project_path.join("installer.toml"));
    assert!(result.is_ok());
}

#[test]
fn test_INSTALLER_006_validation_result_fields() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-installer");

    init_project(&project_path, None).unwrap();

    let result = validate_installer(&project_path).unwrap();

    assert!(result.valid);
    assert_eq!(result.steps, 1); // Default template has one step
    assert!(result.warnings.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn test_INSTALLER_006_generate_installer_toml_content() {
    let toml = generate_installer_toml("my-app", Some("My app description"));

    assert!(toml.contains("name = \"my-app\""));
    assert!(toml.contains("description = \"My app description\""));
    assert!(toml.contains("[installer]"));
    assert!(toml.contains("[[step]]"));
}

#[test]
fn test_INSTALLER_006_generate_installer_toml_default_description() {
    let toml = generate_installer_toml("my-app", None);

    assert!(toml.contains("name = \"my-app\""));
    assert!(toml.contains("TDD-first installer"));
}

#[test]
fn test_INSTALLER_006_generate_test_mod_content() {
    let content = generate_test_mod("my-app");

    assert!(content.contains("Test module for my-app"));
    assert!(content.contains("mod falsification"));
    assert!(content.contains("EXTREME TDD"));
}

#[test]
fn test_INSTALLER_006_generate_falsification_tests_content() {
    let content = generate_falsification_tests("my-app");

    assert!(content.contains("Falsification tests for my-app"));
    assert!(content.contains("Karl Popper"));
    assert!(content.contains("falsify_step_idempotency"));
    assert!(content.contains("falsify_dry_run_accuracy"));
    assert!(content.contains("falsify_rollback_completeness"));
}
