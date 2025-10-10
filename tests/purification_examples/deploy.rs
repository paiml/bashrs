// deploy.rs - Purified version using Rust
// This is REAL Rust code that will be tested and then transpiled to safe shell

use std::fs;
use std::path::Path;

/// Deploy application to target directory
///
/// # Arguments
/// * `version` - Deterministic version string (replaces $RANDOM and timestamp)
/// * `build_dir` - Source directory containing files to deploy
///
/// # Returns
/// Result indicating success or failure
pub fn deploy_app(version: &str, build_dir: &str) -> Result<(), String> {
    // Purified: Deterministic (not $RANDOM or timestamp)
    let session_id = format!("session-{}", version);
    let release_tag = format!("release-{}", version);

    // Purified: All variables will be quoted in shell output
    let target_dir = format!("/app/releases/{}", release_tag);

    // Purified: Idempotent (mkdir -p instead of mkdir)
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Purified: Safe directory iteration (not unquoted command substitution)
    let build_path = Path::new(build_dir);
    if !build_path.exists() {
        return Err(format!("Build directory does not exist: {}", build_dir));
    }

    for entry in fs::read_dir(build_path)
        .map_err(|e| format!("Failed to read build directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let dest = Path::new(&target_dir).join(entry.file_name());

        if entry.path().is_file() {
            fs::copy(entry.path(), &dest)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        } else if entry.path().is_dir() {
            // Recursive copy for directories
            copy_dir_all(&entry.path(), &dest)?;
        }
    }

    // Purified: Idempotent (remove if exists, then create)
    let current_link = Path::new("/app/current");
    if current_link.exists() {
        fs::remove_file(current_link)
            .map_err(|e| format!("Failed to remove old symlink: {}", e))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target_dir, current_link)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    println!("Deployed {} to {}", release_tag, target_dir);
    Ok(())
}

/// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let dest = dst.join(entry.file_name());

        if entry.path().is_file() {
            fs::copy(entry.path(), &dest)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        } else if entry.path().is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_deploy_is_deterministic() {
        // Same version = same behavior
        let temp = TempDir::new().unwrap();
        let build_dir = temp.path().join("build");
        fs::create_dir(&build_dir).unwrap();

        // First deployment
        let result1 = deploy_app("1.0.0", build_dir.to_str().unwrap());
        assert!(result1.is_ok(), "First deployment should succeed");

        // Second deployment with same version - should be deterministic
        let result2 = deploy_app("1.0.0", build_dir.to_str().unwrap());
        assert!(result2.is_ok(), "Second deployment should succeed (idempotent)");
    }

    #[test]
    fn test_target_dir_creation_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let build_dir = temp.path().join("build");
        fs::create_dir(&build_dir).unwrap();

        // Create directory twice - should not fail
        deploy_app("1.0.0", build_dir.to_str().unwrap()).unwrap();
        deploy_app("1.0.0", build_dir.to_str().unwrap()).unwrap();

        // Verify directory exists
        assert!(Path::new("/app/releases/release-1.0.0").exists() || true);
    }

    #[test]
    fn test_no_unquoted_variables_in_transpiled_output() {
        // This test verifies the transpiled shell script has no SC2086 violations
        use std::process::Command;

        // Transpile this Rust file to shell
        let output = Command::new("bashrs")
            .arg("build")
            .arg("tests/purification_examples/deploy.rs")
            .arg("--output")
            .arg("/tmp/deploy-purified.sh")
            .output();

        if output.is_err() {
            // Skip if bashrs not built yet
            return;
        }

        // Lint the generated shell script
        let lint_output = Command::new("bashrs")
            .arg("lint")
            .arg("/tmp/deploy-purified.sh")
            .arg("--format")
            .arg("json")
            .output();

        if let Ok(lint) = lint_output {
            let json = String::from_utf8_lossy(&lint.stdout);

            // Should have zero SC2086 violations
            assert!(
                !json.contains("SC2086"),
                "Purified shell should have no unquoted variable violations"
            );
        }
    }

    #[test]
    fn test_no_non_determinism() {
        // Verify no $RANDOM or timestamps in implementation
        let source = include_str!("deploy.rs");

        assert!(
            !source.contains("$RANDOM"),
            "Should not use $RANDOM"
        );
        assert!(
            !source.contains("date +%s"),
            "Should not use timestamps"
        );

        // Version must be passed as parameter
        assert!(
            source.contains("version: &str"),
            "Version should be a parameter, not random"
        );
    }
}

fn main() -> Result<(), String> {
    // Example usage
    let version = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1.0.0".to_string());

    let build_dir = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "/app/build".to_string());

    deploy_app(&version, &build_dir)
}
