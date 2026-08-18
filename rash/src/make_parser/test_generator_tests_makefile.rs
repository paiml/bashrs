#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_makefile_test_generator_basic() {
        let options = MakefileTestGeneratorOptions::default();
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("test_makefile/Makefile");
        let purified_content = ".PHONY: all\nall:\n\techo test";

        let test_suite = generator.generate_tests(&makefile_path, purified_content);

        // Verify shebang
        assert!(test_suite.starts_with("#!/bin/sh"));

        // Verify test names
        assert!(test_suite.contains("test_determinism"));
        assert!(test_suite.contains("test_idempotency"));
        assert!(test_suite.contains("test_posix_compliance"));

        // Verify test runner
        assert!(test_suite.contains("run_all_tests"));

        // Should not contain property tests (not enabled)
        assert!(!test_suite.contains("test_property_determinism"));
    }

    #[test]
    fn test_makefile_test_generator_with_property_tests() {
        let options = MakefileTestGeneratorOptions {
            property_tests: true,
            property_test_count: 50,
        };
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("Makefile");
        let purified_content = ".PHONY: build\nbuild:\n\techo build";

        let test_suite = generator.generate_tests(&makefile_path, purified_content);

        // Verify property tests included
        assert!(test_suite.contains("test_property_determinism"));
        assert!(test_suite.contains("50 test cases"));
        assert!(test_suite.contains("50 cases"));
    }

    #[test]
    fn test_get_makefile_name() {
        let path = PathBuf::from("/tmp/test/Makefile");
        assert_eq!(MakefileTestGenerator::get_makefile_name(&path), "Makefile");

        let path2 = PathBuf::from("MyMakefile");
        assert_eq!(
            MakefileTestGenerator::get_makefile_name(&path2),
            "MyMakefile"
        );
    }

    #[test]
    fn test_determinism_test_generation() {
        let options = MakefileTestGeneratorOptions::default();
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("Makefile");
        let test = generator.generate_determinism_test(&makefile_path);

        // Verify test structure
        assert!(test.contains("test_determinism()"));
        assert!(test.contains("make -f"));
        assert!(test.contains("/tmp/output1.txt"));
        assert!(test.contains("/tmp/output2.txt"));
        assert!(test.contains("diff"));
    }

    #[test]
    fn test_idempotency_test_generation() {
        let options = MakefileTestGeneratorOptions::default();
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("test.mk");
        let test = generator.generate_idempotency_test(&makefile_path);

        // Verify test structure
        assert!(test.contains("test_idempotency()"));
        assert!(test.contains("make -f \"test.mk\""));
        assert!(test.matches("make -f").count() >= 3); // Should run make 3 times
    }

    #[test]
    fn test_posix_compliance_test_generation() {
        let options = MakefileTestGeneratorOptions::default();
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("Makefile");
        let test = generator.generate_posix_compliance_test(&makefile_path);

        // Verify test structure
        assert!(test.contains("test_posix_compliance()"));
        assert!(test.contains("POSIX"));
    }

    #[test]
    fn test_property_test_generation() {
        let options = MakefileTestGeneratorOptions {
            property_tests: true,
            property_test_count: 100,
        };
        let generator = MakefileTestGenerator::new(options);

        let makefile_path = PathBuf::from("Makefile");
        let test = generator.generate_property_determinism_test(&makefile_path);

        // Verify test structure
        assert!(test.contains("test_property_determinism()"));
        assert!(test.contains("100 test cases"));
        assert!(test.contains("while"));
        assert!(test.contains("100"));
    }

    #[test]
    fn test_runner_without_property_tests() {
        let options = MakefileTestGeneratorOptions::default();
        let generator = MakefileTestGenerator::new(options);

        let runner = generator.generate_test_runner();

        // Verify runner calls all tests
        assert!(runner.contains("test_determinism"));
        assert!(runner.contains("test_idempotency"));
        assert!(runner.contains("test_posix_compliance"));
        assert!(!runner.contains("test_property_determinism"));
        assert!(runner.contains("run_all_tests"));
    }

    #[test]
    fn test_runner_with_property_tests() {
        let options = MakefileTestGeneratorOptions {
            property_tests: true,
            property_test_count: 50,
        };
        let generator = MakefileTestGenerator::new(options);

        let runner = generator.generate_test_runner();

        // Verify runner calls all tests including property tests
        assert!(runner.contains("test_determinism"));
        assert!(runner.contains("test_idempotency"));
        assert!(runner.contains("test_posix_compliance"));
        assert!(runner.contains("test_property_determinism"));
    }

    // ============================================================================
    // Property-Based Tests (EXTREME TDD)
    // ============================================================================

    use proptest::prelude::*;

    proptest! {
        /// Property: Generated test suite always starts with POSIX shebang
        #[test]
        fn prop_test_suite_has_posix_shebang(
            property_tests in proptest::bool::ANY,
            property_test_count in 1usize..200,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("Makefile");
            let purified = ".PHONY: all\nall:\n\techo test";

            let test_suite = generator.generate_tests(&makefile_path, purified);

            prop_assert!(test_suite.starts_with("#!/bin/sh"));
        }

        /// Property: Test suite always contains all three core tests
        #[test]
        fn prop_test_suite_contains_core_tests(
            property_tests in proptest::bool::ANY,
            property_test_count in 1usize..200,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("test.mk");
            let purified = ".PHONY: build\nbuild:\n\techo build";

            let test_suite = generator.generate_tests(&makefile_path, purified);

            prop_assert!(test_suite.contains("test_determinism"));
            prop_assert!(test_suite.contains("test_idempotency"));
            prop_assert!(test_suite.contains("test_posix_compliance"));
        }

        /// Property: Property tests included if and only if enabled
        #[test]
        fn prop_property_tests_conditional(
            property_tests in proptest::bool::ANY,
            property_test_count in 1usize..200,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("Makefile");
            let purified = ".PHONY: clean\nclean:\n\trm -f *.o";

            let test_suite = generator.generate_tests(&makefile_path, purified);

            if property_tests {
                prop_assert!(test_suite.contains("test_property_determinism"));
            } else {
                prop_assert!(!test_suite.contains("test_property_determinism"));
            }
        }

        /// Property: Test suite never panics (generation is always safe)
        #[test]
        fn prop_test_generation_never_panics(
            property_tests in proptest::bool::ANY,
            property_test_count in 1usize..500,
            makefile_name in "[a-zA-Z0-9_-]{1,50}\\.(mk|makefile|Makefile)",
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from(makefile_name);
            let purified = ".PHONY: test\ntest:\n\techo ok";

            // Should never panic
            let _ = generator.generate_tests(&makefile_path, purified);
        }

        /// Property: Generated test count matches configuration
        #[test]
        fn prop_property_test_count_correct(
            property_test_count in 10usize..200,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests: true,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("Makefile");
            let purified = ".PHONY: all\nall:\n\t@echo done";

            let test_suite = generator.generate_tests(&makefile_path, purified);

            // Should contain the count in the test output
            let expected_text = format!("{} test cases", property_test_count);
            prop_assert!(test_suite.contains(&expected_text), "Missing test count: {}", expected_text);
        }

        /// Property: Test suite is valid shell (contains function definitions)
        #[test]
        fn prop_test_suite_is_valid_shell(
            property_tests in proptest::bool::ANY,
            property_test_count in 1usize..200,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("build.mk");
            let purified = "all:\n\techo build";

            let test_suite = generator.generate_tests(&makefile_path, purified);

            // Should contain shell function syntax
            let function_syntax = "() {";
            prop_assert!(test_suite.contains(function_syntax), "Missing function syntax");
            prop_assert!(test_suite.contains("run_all_tests"), "Missing run_all_tests function");
        }

        /// Property: Determinism test always runs make twice
        #[test]
        fn prop_determinism_test_runs_make_twice(
            property_tests in proptest::bool::ANY,
        ) {
            let options = MakefileTestGeneratorOptions {
                property_tests,
                property_test_count: 100,
            };
            let generator = MakefileTestGenerator::new(options);
            let makefile_path = PathBuf::from("Makefile");

            let determinism_test = generator.generate_determinism_test(&makefile_path);

            // Should run make at least twice for comparison
            prop_assert!(determinism_test.matches("make -f").count() >= 2);
        }
    }
}
