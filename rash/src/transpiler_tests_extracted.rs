#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;
    use tempfile::TempDir;

    // Test naming convention: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: XTASK_001 (xtask integration - Issue #25)

    #[test]
    fn test_XTASK_001_transpiler_new_has_defaults() {
        let transpiler = Transpiler::new();
        assert!(transpiler.input.is_none());
        assert!(transpiler.output.is_none());
        assert!(transpiler.permissions.is_none());
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_input() {
        let transpiler = Transpiler::new().input("src/main.rs");
        assert_eq!(transpiler.input.unwrap().to_str().unwrap(), "src/main.rs");
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_output() {
        let transpiler = Transpiler::new().output("target/script.sh");
        assert_eq!(
            transpiler.output.unwrap().to_str().unwrap(),
            "target/script.sh"
        );
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_sets_permissions() {
        let transpiler = Transpiler::new().permissions(0o755);
        assert_eq!(transpiler.permissions, Some(0o755));
    }

    #[test]
    fn test_XTASK_001_transpiler_builder_fluent_interface() {
        let transpiler = Transpiler::new()
            .input("src/main.rs")
            .output("target/script.sh")
            .permissions(0o755);

        assert!(transpiler.input.is_some());
        assert!(transpiler.output.is_some());
        assert_eq!(transpiler.permissions, Some(0o755));
    }

    #[test]
    fn test_XTASK_001_transpiler_requires_input() {
        let result = Transpiler::new().output("out.sh").transpile();

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => assert!(msg.contains("Input path not set")),
            _ => panic!("Expected ValidationError for missing input"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_requires_output() {
        let result = Transpiler::new().input("in.rs").transpile();

        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => assert!(msg.contains("Output path not set")),
            _ => panic!("Expected ValidationError for missing output"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_basic_transpilation() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write a simple Rust file
        fs::write(
            &input_path,
            r#"
            fn main() {
                let greeting = "Hello, World!";
                echo(greeting);
            }
            fn echo(msg: &str) {}
            "#,
        )
        .unwrap();

        // Transpile
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        let shell_code = fs::read_to_string(&output_path).unwrap();
        assert!(shell_code.contains("#!/bin/sh"), "Missing shebang");
    }

    #[test]
    fn test_XTASK_001_transpiler_creates_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("nested/deep/output.sh");

        // Write a simple Rust file
        fs::write(
            &input_path,
            r#"
            fn main() {
                let x = 42;
            }
            "#,
        )
        .unwrap();

        // Transpile (should create nested directories)
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");
        assert!(
            output_path.parent().unwrap().exists(),
            "Output directory not created"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_XTASK_001_transpiler_sets_permissions_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write a simple Rust file
        fs::write(&input_path, "fn main() {}").unwrap();

        // Transpile with permissions
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .permissions(0o755)
            .transpile();

        assert!(result.is_ok(), "Transpilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        // Check permissions
        let metadata = fs::metadata(&output_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode() & 0o777;

        assert_eq!(mode, 0o755, "Permissions not set correctly");
    }

    #[test]
    fn test_XTASK_001_transpiler_handles_invalid_rust() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("invalid.rs");
        let output_path = temp_dir.path().join("output.sh");

        // Write invalid Rust code
        fs::write(&input_path, "fn main( { }").unwrap();

        // Transpile should fail
        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_err(), "Expected transpilation to fail");
    }

    #[test]
    fn test_XTASK_001_transpiler_handles_missing_input_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("nonexistent.rs");
        let output_path = temp_dir.path().join("output.sh");

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .transpile();

        assert!(result.is_err(), "Expected error for missing input");
        match result {
            Err(Error::Io(_)) => (), // Expected
            _ => panic!("Expected IO error for missing input file"),
        }
    }

    #[test]
    fn test_XTASK_001_transpiler_with_custom_config() {
        use crate::models::{ShellDialect, VerificationLevel};

        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.rs");
        let output_path = temp_dir.path().join("output.sh");

        fs::write(&input_path, "fn main() { let x = 1; }").unwrap();

        let config = Config {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Strict,
            optimize: true,
            ..Default::default()
        };

        let result = Transpiler::new()
            .input(&input_path)
            .output(&output_path)
            .config(config)
            .transpile();

        assert!(
            result.is_ok(),
            "Transpilation with config failed: {:?}",
            result
        );
        assert!(output_path.exists());
    }
}
