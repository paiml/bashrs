#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generator_creates_posix_shebang() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(tests.starts_with("#!/bin/sh"), "Should have POSIX shebang");
    }

    #[test]
    fn test_generator_includes_determinism_test() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(
            tests.contains("test_determinism"),
            "Should include determinism test"
        );
        assert!(tests.contains("output1"), "Should compare multiple outputs");
        assert!(tests.contains("output2"), "Should compare multiple outputs");
    }

    #[test]
    fn test_generator_includes_idempotency_test() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(
            tests.contains("test_idempotency"),
            "Should include idempotency test"
        );
    }

    #[test]
    fn test_generator_includes_posix_test() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(
            tests.contains("test_posix_compliance"),
            "Should include POSIX test"
        );
        assert!(tests.contains("shellcheck"), "Should use shellcheck");
    }

    #[test]
    fn test_generator_property_tests_optional() {
        // Without property tests
        let options = TestGeneratorOptions {
            property_tests: false,
            property_test_count: 100,
        };
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(
            !tests.contains("test_property_determinism"),
            "Should not include property tests by default"
        );
    }

    #[test]
    fn test_generator_property_tests_enabled() {
        // With property tests
        let options = TestGeneratorOptions {
            property_tests: true,
            property_test_count: 50,
        };
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(
            tests.contains("test_property_determinism"),
            "Should include property tests when enabled"
        );
        assert!(
            tests.contains("50 cases"),
            "Should include configured number of cases"
        );
    }

    #[test]
    fn test_generator_creates_valid_sh_syntax() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        // Check for valid sh constructs
        assert!(tests.contains("if ["), "Should use POSIX test syntax");
        assert!(
            tests.contains("return 0"),
            "Should have proper return codes"
        );
        assert!(tests.contains("exit 0"), "Should have proper exit codes");
    }

    #[test]
    fn test_generator_includes_test_runner() {
        let options = TestGeneratorOptions::default();
        let generator = TestGenerator::new(options);
        let script_path = PathBuf::from("test_script.sh");

        let tests = generator.generate_tests(&script_path, "#!/bin/sh\necho test");

        assert!(tests.contains("Test Runner"), "Should include test runner");
        assert!(tests.contains("All tests passed"), "Should report success");
        assert!(tests.contains("Failed tests"), "Should report failures");
    }
}
