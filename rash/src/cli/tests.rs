use crate::cli::args::{Cli, Commands, InspectionFormat};
use crate::models::{ShellDialect, VerificationLevel};
use clap::Parser;
use std::path::PathBuf;

#[test]
fn test_cli_build_command() {
    let args = vec!["rash", "build", "test.rs"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Build {
            input,
            output,
            emit_proof,
            no_optimize,
        } => {
            assert_eq!(input, PathBuf::from("test.rs"));
            assert_eq!(output, PathBuf::from("install.sh"));
            assert!(!emit_proof);
            assert!(!no_optimize);
        }
        _ => panic!("Expected Build command"),
    }

    assert_eq!(cli.verify, VerificationLevel::Strict);
    assert_eq!(cli.target, ShellDialect::Posix);
    assert!(!cli.verbose);
}

#[test]
fn test_cli_build_with_options() {
    let args = vec![
        "rash",
        "--verify",
        "paranoid",
        "--target",
        "bash",
        "build",
        "test.rs",
        "-o",
        "test.sh",
        "--emit-proof",
    ];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Build {
            input,
            output,
            emit_proof,
            no_optimize,
        } => {
            assert_eq!(input, PathBuf::from("test.rs"));
            assert_eq!(output, PathBuf::from("test.sh"));
            assert!(emit_proof);
            assert!(!no_optimize);
        }
        _ => panic!("Expected Build command"),
    }

    assert_eq!(cli.verify, VerificationLevel::Paranoid);
    assert_eq!(cli.target, ShellDialect::Bash);
}

#[test]
fn test_cli_check_command() {
    let args = vec!["rash", "check", "test.rs"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Check { input } => {
            assert_eq!(input, PathBuf::from("test.rs"));
        }
        _ => panic!("Expected Check command"),
    }
}

#[test]
fn test_cli_init_command() {
    let args = vec!["rash", "init", "--name", "myproject"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Init { path, name } => {
            assert_eq!(path, PathBuf::from("."));
            assert_eq!(name, Some("myproject".to_string()));
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_verify_command() {
    let args = vec!["rash", "verify", "test.rs", "test.sh"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Verify {
            rust_source,
            shell_script,
        } => {
            assert_eq!(rust_source, PathBuf::from("test.rs"));
            assert_eq!(shell_script, PathBuf::from("test.sh"));
        }
        _ => panic!("Expected Verify command"),
    }
}

#[test]
fn test_verification_level_value_enum() {
    use clap::ValueEnum;

    let variants = VerificationLevel::value_variants();
    assert_eq!(variants.len(), 4);

    assert!(VerificationLevel::None.to_possible_value().is_some());
    assert!(VerificationLevel::Basic.to_possible_value().is_some());
    assert!(VerificationLevel::Strict.to_possible_value().is_some());
    assert!(VerificationLevel::Paranoid.to_possible_value().is_some());
}

#[test]
fn test_shell_dialect_value_enum() {
    use clap::ValueEnum;

    let variants = ShellDialect::value_variants();
    assert_eq!(variants.len(), 4);

    assert!(ShellDialect::Posix.to_possible_value().is_some());
    assert!(ShellDialect::Bash.to_possible_value().is_some());
    assert!(ShellDialect::Dash.to_possible_value().is_some());
    assert!(ShellDialect::Ash.to_possible_value().is_some());
}

#[test]
fn test_cli_verbose_flag() {
    let args = vec!["rash", "-v", "check", "test.rs"];
    let cli = Cli::parse_from(args);
    assert!(cli.verbose);

    let args = vec!["rash", "--verbose", "check", "test.rs"];
    let cli = Cli::parse_from(args);
    assert!(cli.verbose);
}

#[test]
fn test_cli_inspect_command() {
    let args = vec!["rash", "inspect", "echo-example"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Inspect {
            input,
            format,
            output,
            detailed,
        } => {
            assert_eq!(input, "echo-example");
            assert!(matches!(format, InspectionFormat::Markdown));
            assert!(output.is_none());
            assert!(!detailed);
        }
        _ => panic!("Expected Inspect command"),
    }
}

#[test]
fn test_cli_inspect_with_options() {
    let args = vec![
        "rash",
        "inspect",
        "bootstrap-example",
        "--format",
        "json",
        "-o",
        "report.json",
        "--detailed",
    ];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Inspect {
            input,
            format,
            output,
            detailed,
        } => {
            assert_eq!(input, "bootstrap-example");
            assert!(matches!(format, InspectionFormat::Json));
            assert_eq!(output, Some(PathBuf::from("report.json")));
            assert!(detailed);
        }
        _ => panic!("Expected Inspect command"),
    }
}

#[test]
fn test_inspection_format_value_enum() {
    use clap::ValueEnum;

    let variants = InspectionFormat::value_variants();
    assert_eq!(variants.len(), 3);

    assert!(InspectionFormat::Markdown.to_possible_value().is_some());
    assert!(InspectionFormat::Json.to_possible_value().is_some());
    assert!(InspectionFormat::Html.to_possible_value().is_some());
}

#[test]
fn test_cli_with_all_options() {
    let args = vec![
        "rash",
        "--verify",
        "basic",
        "--target",
        "dash",
        "--verbose",
        "build",
        "complex.rs",
        "--output",
        "complex.sh",
        "--emit-proof",
        "--no-optimize",
    ];
    let cli = Cli::parse_from(args);

    assert_eq!(cli.verify, VerificationLevel::Basic);
    assert_eq!(cli.target, ShellDialect::Dash);
    assert!(cli.verbose);

    match cli.command {
        Commands::Build {
            input,
            output,
            emit_proof,
            no_optimize,
        } => {
            assert_eq!(input, PathBuf::from("complex.rs"));
            assert_eq!(output, PathBuf::from("complex.sh"));
            assert!(emit_proof);
            assert!(no_optimize);
        }
        _ => panic!("Expected Build command"),
    }
}

#[test]
fn test_init_with_path() {
    let args = vec!["rash", "init", "/path/to/project", "--name", "my-rash-app"];
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Init { path, name } => {
            assert_eq!(path, PathBuf::from("/path/to/project"));
            assert_eq!(name, Some("my-rash-app".to_string()));
        }
        _ => panic!("Expected Init command"),
    }
}
