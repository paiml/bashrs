//! Pure logic functions for installer CLI commands.
//!
//! This module contains testable, side-effect-free helpers extracted from
//! `commands.rs`. Functions here do NOT perform file I/O, print to stdout,
//! or call external processes. They transform data, validate inputs, and
//! build data structures.

#![allow(dead_code)] // Several helpers are used only in tests or by a subset of commands.

use crate::models::{Error, Result};
use std::path::{Path, PathBuf};

// ============================================================================
// Path Resolution Logic
// ============================================================================

/// Compute the default keyring path following XDG base directory convention.
///
/// Priority:
/// 1. `$XDG_CONFIG_HOME/bashrs/installer/keyring.json`
/// 2. `$HOME/.config/bashrs/installer/keyring.json`
/// 3. `./.config/bashrs/installer/keyring.json` (fallback)
pub(crate) fn keyring_default_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bashrs")
        .join("installer")
        .join("keyring.json")
}

/// Compute the keyring directory path (parent of `keyring.json`).
///
/// Same XDG priority as [`keyring_default_path`].
pub(crate) fn keyring_directory_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bashrs")
        .join("installer")
}

/// Resolve the path to `installer.toml` given a directory or file path.
///
/// If `path` is a directory, appends `installer.toml`.
/// If `path` is already a file path, returns it as-is.
pub(crate) fn resolve_installer_toml_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    }
}

/// Determine the output directory for a bash→installer conversion.
///
/// If `output` is provided, use it directly. Otherwise, derive a default
/// name from the input file stem, e.g. `setup.sh` → `setup-installer/`.
pub(crate) fn determine_output_dir(input: &Path, output: Option<&Path>) -> PathBuf {
    match output {
        Some(path) => path.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("converted-installer");
            PathBuf::from(format!("{}-installer", stem))
        }
    }
}

/// Resolve the checkpoint directory path for an installer run.
///
/// If `checkpoint_dir` is provided, use it; otherwise default to
/// `<installer_path>/.checkpoint`.
pub(crate) fn resolve_checkpoint_path(installer_path: &Path, checkpoint_dir: Option<&Path>) -> PathBuf {
    checkpoint_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| installer_path.join(".checkpoint"))
}

/// Extract the installer name from a path (the final component).
///
/// Falls back to `"installer"` if the path has no file name component.
pub(crate) fn extract_installer_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("installer")
}

// ============================================================================
// Validation Logic
// ============================================================================

/// Validate that a keyring file exists, returning a descriptive error if not.
pub(crate) fn require_keyring_exists(keyring_path: &Path) -> Result<()> {
    if !keyring_path.exists() {
        return Err(Error::Validation(
            "Keyring not initialized. Run 'bashrs installer keyring init' first.".to_string(),
        ));
    }
    Ok(())
}

/// Parse a hex-encoded Ed25519 public key (exactly 64 hex chars = 32 bytes).
///
/// Returns `Error::Validation` for wrong length or invalid hex characters.
pub(crate) fn parse_public_key(hex_str: &str) -> Result<crate::installer::PublicKey> {
    if hex_str.len() != 64 {
        return Err(Error::Validation(format!(
            "Invalid public key length: expected 64 hex chars, got {}",
            hex_str.len()
        )));
    }

    let mut result = [0u8; 32];
    for (dest, chunk) in result.iter_mut().zip(hex_str.as_bytes().chunks(2)) {
        let hex = std::str::from_utf8(chunk)
            .map_err(|_| Error::Validation("Invalid hex string".to_string()))?;
        *dest = u8::from_str_radix(hex, 16)
            .map_err(|_| Error::Validation("Invalid hex character".to_string()))?;
    }

    Ok(result)
}

// ============================================================================
// Severity Parsing
// ============================================================================

/// Parse an audit severity string into the corresponding enum variant.
///
/// Accepted values (case-insensitive): `info`, `suggestion`, `warning`,
/// `error`, `critical`. Returns `Error::Validation` for unknown values.
pub(crate) fn parse_audit_severity(
    sev: &str,
) -> Result<crate::installer::AuditSeverity> {
    use crate::installer::AuditSeverity;
    match sev.to_lowercase().as_str() {
        "info" => Ok(AuditSeverity::Info),
        "suggestion" => Ok(AuditSeverity::Suggestion),
        "warning" => Ok(AuditSeverity::Warning),
        "error" => Ok(AuditSeverity::Error),
        "critical" => Ok(AuditSeverity::Critical),
        _ => Err(Error::Validation(format!(
            "Invalid severity '{}'. Valid values: info, suggestion, warning, error, critical",
            sev
        ))),
    }
}

// ============================================================================
// Resume Logic
// ============================================================================

/// Determine the step ID to resume from.
///
/// If `from` is explicitly specified, validates it exists in `available_step_ids`.
/// Otherwise returns the last successful step ID from `last_successful`.
///
/// Returns `Error::Validation` when:
/// - The explicit `from` step is not found.
/// - No `from` is specified and there are no successful steps.
pub(crate) fn compute_resume_from(
    from: Option<&str>,
    available_step_ids: &[String],
    last_successful: Option<&str>,
) -> Result<String> {
    match from {
        Some(step_id) => {
            if !available_step_ids.iter().any(|id| id == step_id) {
                return Err(Error::Validation(format!(
                    "Step '{}' not found in checkpoint",
                    step_id
                )));
            }
            Ok(step_id.to_string())
        }
        None => last_successful
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Validation("No successful steps to resume from".to_string())),
    }
}

// ============================================================================
// Step Count Helpers
// ============================================================================

/// Compute a summary of step statuses from a slice of step status strings.
///
/// Returns `(completed, failed, pending)` counts where each step's `status`
/// field is compared against the canonical string values.
pub(crate) fn count_step_statuses(statuses: &[&str]) -> (usize, usize, usize) {
    let completed = statuses.iter().filter(|&&s| s == "completed").count();
    let failed = statuses.iter().filter(|&&s| s == "failed").count();
    let pending = statuses.iter().filter(|&&s| s == "pending").count();
    (completed, failed, pending)
}

/// Determine whether the execution mode label indicates hermetic mode.
///
/// Returns `true` when `mode_label` is `"hermetic"` (case-insensitive).
pub(crate) fn is_hermetic_mode(mode_label: &str) -> bool {
    mode_label.eq_ignore_ascii_case("hermetic")
}

// ============================================================================
// Lockfile Helpers
// ============================================================================

/// Check whether a lockfile is needed based on artifact count.
///
/// Returns `true` if `artifact_count > 0`, meaning a lockfile is required.
pub(crate) fn lockfile_required(artifact_count: usize) -> bool {
    artifact_count > 0
}

/// Validate that a lockfile artifact count matches the spec artifact count.
///
/// Returns `Ok(())` if counts match, or a warning message if they differ.
pub(crate) fn validate_lockfile_artifact_count(
    lockfile_count: usize,
    spec_count: usize,
    path_display: &str,
) -> std::result::Result<(), String> {
    if lockfile_count != spec_count {
        Err(format!(
            "Lockfile may be outdated: lockfile has {} artifacts, spec has {}. Run 'bashrs installer lock {} --update' to refresh.",
            lockfile_count, spec_count, path_display
        ))
    } else {
        Ok(())
    }
}

// ============================================================================
// Trace / Golden Logic
// ============================================================================

/// Compute the path for the golden traces directory.
///
/// Places `.golden-traces/` next to the installer directory (in the parent).
pub(crate) fn golden_traces_dir(installer_path: &Path) -> PathBuf {
    installer_path
        .parent()
        .unwrap_or(installer_path)
        .join(".golden-traces")
}

// ============================================================================
// Key ID Derivation
// ============================================================================

/// Derive a key ID from a key file path by taking the file stem.
///
/// Falls back to `"imported-key"` when the stem cannot be determined.
pub(crate) fn key_id_from_path(key_path: &Path) -> String {
    key_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported-key")
        .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_INSTALLER_LOGIC_001_keyring_default_path_ends_with_keyring_json() {
        let path = keyring_default_path();
        assert!(path.to_string_lossy().ends_with("keyring.json"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_002_keyring_default_path_contains_bashrs_installer() {
        let s = keyring_default_path().to_string_lossy().into_owned();
        assert!(s.contains("bashrs") && s.contains("installer"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_003_keyring_directory_path_ends_with_installer() {
        assert!(keyring_directory_path().ends_with("installer"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_004_resolve_installer_toml_from_file_path() {
        let p = PathBuf::from("/some/installer.toml");
        assert_eq!(resolve_installer_toml_path(&p), p);
    }

    #[test]
    fn test_INSTALLER_LOGIC_005_resolve_installer_toml_from_dir_path() {
        let dir = std::env::temp_dir();
        assert_eq!(resolve_installer_toml_path(&dir), dir.join("installer.toml"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_006_determine_output_dir_uses_explicit_output() {
        let input = PathBuf::from("setup.sh");
        let output = PathBuf::from("/custom/output");
        assert_eq!(determine_output_dir(&input, Some(&output)), output);
    }

    #[test]
    fn test_INSTALLER_LOGIC_007_determine_output_dir_derives_from_stem() {
        let input = PathBuf::from("setup.sh");
        assert_eq!(determine_output_dir(&input, None), PathBuf::from("setup-installer"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_008_determine_output_dir_fallback_when_no_stem() {
        // Empty path has no file_stem → fallback "converted-installer" is used as
        // the stem, then "-installer" appended → "converted-installer-installer".
        assert_eq!(
            determine_output_dir(&PathBuf::from(""), None),
            PathBuf::from("converted-installer-installer")
        );
    }

    #[test]
    fn test_INSTALLER_LOGIC_009_resolve_checkpoint_path_uses_explicit_dir() {
        let installer = PathBuf::from("/my/installer");
        let cp = PathBuf::from("/explicit/checkpoint");
        assert_eq!(resolve_checkpoint_path(&installer, Some(&cp)), cp);
    }

    #[test]
    fn test_INSTALLER_LOGIC_010_resolve_checkpoint_path_defaults_to_dot_checkpoint() {
        let installer = PathBuf::from("/my/installer");
        assert_eq!(
            resolve_checkpoint_path(&installer, None),
            PathBuf::from("/my/installer/.checkpoint")
        );
    }

    #[test]
    fn test_INSTALLER_LOGIC_011_extract_installer_name_returns_last_component() {
        assert_eq!(
            extract_installer_name(&PathBuf::from("/home/user/my-installer")),
            "my-installer"
        );
    }

    #[test]
    fn test_INSTALLER_LOGIC_012_extract_installer_name_fallback_for_root() {
        assert_eq!(extract_installer_name(&PathBuf::from("/")), "installer");
    }

    #[test]
    fn test_INSTALLER_LOGIC_013_parse_public_key_valid_64_hex_chars() {
        let key = parse_public_key(&"a".repeat(64)).unwrap();
        assert_eq!(key.len(), 32);
        assert!(key.iter().all(|&b| b == 0xaa));
    }

    #[test]
    fn test_INSTALLER_LOGIC_014_parse_public_key_rejects_wrong_length() {
        assert!(parse_public_key(&"a".repeat(32)).is_err());
        assert!(parse_public_key(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_INSTALLER_LOGIC_015_parse_public_key_rejects_invalid_hex_char() {
        let hex = format!("g{}", "a".repeat(63));
        assert!(parse_public_key(&hex).is_err());
    }

    #[test]
    fn test_INSTALLER_LOGIC_016_parse_public_key_all_zeros() {
        let key = parse_public_key(&"0".repeat(64)).unwrap();
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_INSTALLER_LOGIC_017_parse_audit_severity_all_valid_values() {
        use crate::installer::AuditSeverity;
        assert!(matches!(parse_audit_severity("info").unwrap(), AuditSeverity::Info));
        assert!(matches!(parse_audit_severity("suggestion").unwrap(), AuditSeverity::Suggestion));
        assert!(matches!(parse_audit_severity("WARNING").unwrap(), AuditSeverity::Warning));
        assert!(matches!(parse_audit_severity("Error").unwrap(), AuditSeverity::Error));
        assert!(matches!(parse_audit_severity("CRITICAL").unwrap(), AuditSeverity::Critical));
    }

    #[test]
    fn test_INSTALLER_LOGIC_018_parse_audit_severity_invalid_returns_error() {
        let err = parse_audit_severity("blocker").unwrap_err().to_string();
        assert!(err.contains("blocker") && err.contains("info"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_019_require_keyring_exists_missing_path() {
        let err = require_keyring_exists(&PathBuf::from("/nonexistent/keyring.json"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("Keyring not initialized"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_020_require_keyring_exists_present_path() {
        assert!(require_keyring_exists(&std::env::temp_dir()).is_ok());
    }

    #[test]
    fn test_INSTALLER_LOGIC_021_compute_resume_from_explicit_valid_step() {
        let steps = vec!["step-1".to_string(), "step-2".to_string()];
        assert_eq!(compute_resume_from(Some("step-1"), &steps, None).unwrap(), "step-1");
    }

    #[test]
    fn test_INSTALLER_LOGIC_022_compute_resume_from_explicit_invalid_step() {
        let steps = vec!["step-1".to_string()];
        let err = compute_resume_from(Some("step-99"), &steps, None).unwrap_err().to_string();
        assert!(err.contains("step-99"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_023_compute_resume_from_uses_last_successful() {
        let steps = vec!["step-1".to_string(), "step-2".to_string()];
        assert_eq!(compute_resume_from(None, &steps, Some("step-2")).unwrap(), "step-2");
    }

    #[test]
    fn test_INSTALLER_LOGIC_024_compute_resume_from_error_no_successful_steps() {
        let err = compute_resume_from(None, &[], None).unwrap_err().to_string();
        assert!(err.contains("No successful steps"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_025_count_step_statuses_mixed() {
        let (c, f, p) = count_step_statuses(&["completed", "completed", "failed", "pending"]);
        assert_eq!((c, f, p), (2, 1, 1));
    }

    #[test]
    fn test_INSTALLER_LOGIC_026_count_step_statuses_empty() {
        assert_eq!(count_step_statuses(&[]), (0, 0, 0));
    }

    #[test]
    fn test_INSTALLER_LOGIC_027_is_hermetic_mode_case_insensitive() {
        assert!(is_hermetic_mode("hermetic"));
        assert!(is_hermetic_mode("HERMETIC"));
        assert!(!is_hermetic_mode("normal"));
        assert!(!is_hermetic_mode(""));
    }

    #[test]
    fn test_INSTALLER_LOGIC_028_lockfile_required_based_on_artifact_count() {
        assert!(lockfile_required(1));
        assert!(!lockfile_required(0));
    }

    #[test]
    fn test_INSTALLER_LOGIC_029_validate_lockfile_count_match() {
        assert!(validate_lockfile_artifact_count(3, 3, "/installer").is_ok());
    }

    #[test]
    fn test_INSTALLER_LOGIC_030_validate_lockfile_count_mismatch() {
        let msg = validate_lockfile_artifact_count(2, 5, "/installer").unwrap_err();
        assert!(msg.contains("2") && msg.contains("5"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_031_golden_traces_dir_next_to_installer() {
        let dir = golden_traces_dir(&PathBuf::from("/projects/my-installer"));
        assert_eq!(dir, PathBuf::from("/projects/.golden-traces"));
    }

    #[test]
    fn test_INSTALLER_LOGIC_032_key_id_from_path_uses_file_stem() {
        assert_eq!(key_id_from_path(&PathBuf::from("/keys/alice.pub")), "alice");
    }

    #[test]
    fn test_INSTALLER_LOGIC_033_key_id_from_path_fallback_when_no_stem() {
        assert_eq!(key_id_from_path(&PathBuf::from("/")), "imported-key");
    }
}
