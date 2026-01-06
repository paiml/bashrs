//! Dev Container Pure Logic - JSON validation helpers
//!
//! Extracted for EXTREME TDD testability.

use serde_json::Value;

/// Check if JSON has image source
pub fn has_image_source(json: &Value) -> bool {
    json.get("image").is_some()
        || json.get("build").is_some()
        || json.get("dockerComposeFile").is_some()
}

/// Check if image uses :latest tag
pub fn uses_latest_tag(image: &str) -> bool {
    image.ends_with(":latest")
}

/// Check if path is absolute (starts with /)
pub fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
}

/// Check if string is empty or whitespace only
pub fn is_empty_string(s: &str) -> bool {
    s.trim().is_empty()
}

/// Check if JSON value is a non-empty string
pub fn is_non_empty_string(value: &Value) -> bool {
    value.as_str().is_some_and(|s| !s.trim().is_empty())
}

/// Check if JSON value is a valid lifecycle command (string, array, or object)
pub fn is_valid_lifecycle_command(value: &Value) -> bool {
    match value {
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(_) => true,
        Value::Object(_) => true,
        _ => false,
    }
}

/// Check if feature reference is valid format
/// Valid: ghcr.io/..., local path starting with ./
pub fn is_valid_feature_reference(feature: &str) -> bool {
    feature.starts_with("ghcr.io/")
        || feature.starts_with("./")
        || feature.starts_with("../")
        || feature.contains('/')
}

/// Check if extensions array has duplicates
pub fn has_duplicate_extensions(extensions: &[String]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for ext in extensions {
        let lower = ext.to_lowercase();
        if !seen.insert(lower) {
            return true;
        }
    }
    false
}

/// Find duplicate extensions
pub fn find_duplicate_extensions(extensions: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut duplicates = Vec::new();
    for ext in extensions {
        let lower = ext.to_lowercase();
        if !seen.insert(lower.clone()) {
            duplicates.push(ext.clone());
        }
    }
    duplicates
}

/// Check if port is in valid range (1-65535)
pub fn is_valid_port(port: u64) -> bool {
    (1..=65535).contains(&port)
}

/// Check if env var name is valid (uppercase with underscores)
pub fn is_valid_env_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
}

/// Check if JSON has remoteUser field
pub fn has_remote_user(json: &Value) -> bool {
    json.get("remoteUser").is_some()
}

/// Check if JSON has name field
pub fn has_name(json: &Value) -> bool {
    json.get("name").is_some()
}

/// Check if mounts array has volumes
pub fn has_volume_mounts(mounts: &[String]) -> bool {
    mounts.iter().any(|m| m.starts_with("type=volume"))
}

/// Check if privileged mode is enabled
pub fn is_privileged(json: &Value) -> bool {
    json.get("privileged")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ===== HAS IMAGE SOURCE =====

    #[test]
    fn test_has_image_source_image() {
        let json = json!({"image": "ubuntu"});
        assert!(has_image_source(&json));
    }

    #[test]
    fn test_has_image_source_build() {
        let json = json!({"build": {"dockerfile": "Dockerfile"}});
        assert!(has_image_source(&json));
    }

    #[test]
    fn test_has_image_source_compose() {
        let json = json!({"dockerComposeFile": "docker-compose.yml"});
        assert!(has_image_source(&json));
    }

    #[test]
    fn test_has_image_source_none() {
        let json = json!({"name": "test"});
        assert!(!has_image_source(&json));
    }

    // ===== USES LATEST TAG =====

    #[test]
    fn test_uses_latest_tag_true() {
        assert!(uses_latest_tag("ubuntu:latest"));
        assert!(uses_latest_tag(
            "mcr.microsoft.com/devcontainers/base:latest"
        ));
    }

    #[test]
    fn test_uses_latest_tag_false() {
        assert!(!uses_latest_tag("ubuntu:22.04"));
        assert!(!uses_latest_tag("ubuntu"));
    }

    // ===== IS ABSOLUTE PATH =====

    #[test]
    fn test_is_absolute_path_true() {
        assert!(is_absolute_path("/home/user/Dockerfile"));
        assert!(is_absolute_path("/Dockerfile"));
    }

    #[test]
    fn test_is_absolute_path_false() {
        assert!(!is_absolute_path("./Dockerfile"));
        assert!(!is_absolute_path("Dockerfile"));
    }

    // ===== IS EMPTY STRING =====

    #[test]
    fn test_is_empty_string_true() {
        assert!(is_empty_string(""));
        assert!(is_empty_string("   "));
        assert!(is_empty_string("\t\n"));
    }

    #[test]
    fn test_is_empty_string_false() {
        assert!(!is_empty_string("hello"));
        assert!(!is_empty_string("  x  "));
    }

    // ===== IS NON EMPTY STRING =====

    #[test]
    fn test_is_non_empty_string_true() {
        assert!(is_non_empty_string(&json!("hello")));
    }

    #[test]
    fn test_is_non_empty_string_false() {
        assert!(!is_non_empty_string(&json!("")));
        assert!(!is_non_empty_string(&json!("   ")));
        assert!(!is_non_empty_string(&json!(123)));
    }

    // ===== IS VALID LIFECYCLE COMMAND =====

    #[test]
    fn test_is_valid_lifecycle_command_string() {
        assert!(is_valid_lifecycle_command(&json!("npm install")));
        assert!(!is_valid_lifecycle_command(&json!("")));
    }

    #[test]
    fn test_is_valid_lifecycle_command_array() {
        assert!(is_valid_lifecycle_command(&json!(["npm", "install"])));
    }

    #[test]
    fn test_is_valid_lifecycle_command_object() {
        assert!(is_valid_lifecycle_command(
            &json!({"install": "npm install"})
        ));
    }

    #[test]
    fn test_is_valid_lifecycle_command_invalid() {
        assert!(!is_valid_lifecycle_command(&json!(123)));
        assert!(!is_valid_lifecycle_command(&json!(null)));
    }

    // ===== IS VALID FEATURE REFERENCE =====

    #[test]
    fn test_is_valid_feature_reference_ghcr() {
        assert!(is_valid_feature_reference(
            "ghcr.io/devcontainers/features/node:1"
        ));
    }

    #[test]
    fn test_is_valid_feature_reference_local() {
        assert!(is_valid_feature_reference("./local-feature"));
        assert!(is_valid_feature_reference("../shared-feature"));
    }

    #[test]
    fn test_is_valid_feature_reference_with_slash() {
        assert!(is_valid_feature_reference("owner/repo/feature"));
    }

    // ===== HAS DUPLICATE EXTENSIONS =====

    #[test]
    fn test_has_duplicate_extensions_true() {
        let exts = vec!["ms-python.python".into(), "MS-Python.Python".into()];
        assert!(has_duplicate_extensions(&exts));
    }

    #[test]
    fn test_has_duplicate_extensions_false() {
        let exts = vec!["ms-python.python".into(), "rust-lang.rust-analyzer".into()];
        assert!(!has_duplicate_extensions(&exts));
    }

    // ===== FIND DUPLICATE EXTENSIONS =====

    #[test]
    fn test_find_duplicate_extensions() {
        let exts = vec!["a".into(), "b".into(), "A".into()];
        let dups = find_duplicate_extensions(&exts);
        assert_eq!(dups, vec!["A"]);
    }

    // ===== IS VALID PORT =====

    #[test]
    fn test_is_valid_port_true() {
        assert!(is_valid_port(80));
        assert!(is_valid_port(443));
        assert!(is_valid_port(1));
        assert!(is_valid_port(65535));
    }

    #[test]
    fn test_is_valid_port_false() {
        assert!(!is_valid_port(0));
        assert!(!is_valid_port(65536));
    }

    // ===== IS VALID ENV NAME =====

    #[test]
    fn test_is_valid_env_name_true() {
        assert!(is_valid_env_name("HOME"));
        assert!(is_valid_env_name("MY_VAR"));
        assert!(is_valid_env_name("_PRIVATE"));
    }

    #[test]
    fn test_is_valid_env_name_false() {
        assert!(!is_valid_env_name(""));
        assert!(!is_valid_env_name("myVar")); // lowercase
        assert!(!is_valid_env_name("123VAR")); // starts with number
    }

    // ===== HAS REMOTE USER =====

    #[test]
    fn test_has_remote_user_true() {
        let json = json!({"remoteUser": "vscode"});
        assert!(has_remote_user(&json));
    }

    #[test]
    fn test_has_remote_user_false() {
        let json = json!({"name": "test"});
        assert!(!has_remote_user(&json));
    }

    // ===== HAS NAME =====

    #[test]
    fn test_has_name_true() {
        let json = json!({"name": "My Container"});
        assert!(has_name(&json));
    }

    #[test]
    fn test_has_name_false() {
        let json = json!({"image": "ubuntu"});
        assert!(!has_name(&json));
    }

    // ===== HAS VOLUME MOUNTS =====

    #[test]
    fn test_has_volume_mounts_true() {
        let mounts = vec!["type=volume,source=mydata,target=/data".into()];
        assert!(has_volume_mounts(&mounts));
    }

    #[test]
    fn test_has_volume_mounts_false() {
        let mounts = vec!["type=bind,source=./,target=/workspace".into()];
        assert!(!has_volume_mounts(&mounts));
        let empty: Vec<String> = vec![];
        assert!(!has_volume_mounts(&empty));
    }

    // ===== IS PRIVILEGED =====

    #[test]
    fn test_is_privileged_true() {
        let json = json!({"privileged": true});
        assert!(is_privileged(&json));
    }

    #[test]
    fn test_is_privileged_false() {
        let json = json!({"privileged": false});
        assert!(!is_privileged(&json));
        let json2 = json!({"name": "test"});
        assert!(!is_privileged(&json2));
    }
}
