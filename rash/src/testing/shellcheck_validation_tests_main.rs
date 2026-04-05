fn test_shellcheck_long_variable_names() {
    let source = r#"
        fn main() {
            let very_long_variable_name_that_is_descriptive = "test";
            let another_extremely_long_name_for_testing = "value";

            echo(very_long_variable_name_that_is_descriptive);
            echo(another_extremely_long_name_for_testing);
        }
    "#;

    transpile_and_validate(source).expect("Long variable names should pass shellcheck");
}

// ============================================================================
// PROPERTY 8: Boolean values pass ShellCheck
// ============================================================================

#[test]
fn test_shellcheck_boolean_true() {
    let source = r#"
        fn main() {
            let flag = true;
            if flag {
                echo("true");
            }
        }
    "#;

    transpile_and_validate(source).expect("Boolean true should pass shellcheck");
}

#[test]
fn test_shellcheck_boolean_false() {
    let source = r#"
        fn main() {
            let flag = false;
            if flag {
                echo("won't run");
            } else {
                echo("will run");
            }
        }
    "#;

    transpile_and_validate(source).expect("Boolean false should pass shellcheck");
}

// ============================================================================
// PROPERTY 9: Complex real-world scenarios
// ============================================================================

#[test]
fn test_shellcheck_installer_pattern() {
    let source = r#"
        fn main() {
            let package_name = "my-app";
            let version = "1.0.0";
            let install_path = "/usr/local/bin";

            echo("Installing...");
            check_requirements();
            download_package(package_name, version);
            install_binary(package_name, install_path);
            echo("Installation complete!");
        }

        fn check_requirements() {
            echo("Checking requirements...");
        }

        fn download_package(name: &str, ver: &str) {
            echo(name);
            echo(ver);
        }

        fn install_binary(name: &str, path: &str) {
            echo(name);
            echo(path);
        }
    "#;

    transpile_and_validate(source).expect("Installer pattern should pass shellcheck");
}

#[test]
fn test_shellcheck_error_handling_pattern() {
    let source = r#"
        fn main() {
            let success = true;

            if success {
                echo("Success!");
            } else {
                handle_error("Operation failed");
            }
        }

        fn handle_error(message: &str) {
            echo(message);
        }
    "#;

    transpile_and_validate(source).expect("Error handling pattern should pass shellcheck");
}

// ============================================================================
// PROPERTY 10: Determinism - byte-identical output
// ============================================================================

#[test]
fn test_deterministic_output() {
    let source = r#"
        fn main() {
            let x = "test";
            if true {
                echo(x);
            }
        }
    "#;

    let config = Config::default();

    // Transpile 10 times
    let results: Vec<String> = (0..10)
        .map(|_| transpile(source, &config).unwrap())
        .collect();

    // All results should be byte-identical
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(
            &results[0], result,
            "Transpilation {} differs from first result",
            i
        );
    }

    // And should pass shellcheck
    shellcheck_validate(&results[0]).expect("Deterministic output should pass shellcheck");
}
