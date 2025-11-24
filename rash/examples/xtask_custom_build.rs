#![allow(clippy::unwrap_used)] // Examples can use unwrap() for simplicity
                               // Example: Custom build script with full control
                               //
                               // This demonstrates the programmatic API for complex build scenarios.
                               // Run with: cargo run --example custom_build

use bashrs::models::{ShellDialect, VerificationLevel};
use bashrs::{Config, Transpiler};

fn main() -> bashrs::Result<()> {
    println!("Custom Build Script - bashrs xtask integration");
    println!("================================================\n");

    // Example 1: Basic transpilation
    println!("1. Basic hook transpilation:");
    basic_transpilation()?;

    // Example 2: Custom configuration
    println!("\n2. Transpilation with custom config:");
    custom_config_transpilation()?;

    // Example 3: Batch processing
    println!("\n3. Batch transpilation:");
    batch_transpilation()?;

    println!("\n✓ All examples completed successfully!");
    Ok(())
}

/// Example 1: Basic transpilation with default settings
fn basic_transpilation() -> bashrs::Result<()> {
    // Use a temporary file for the example
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut input_file = NamedTempFile::new().unwrap();
    write!(
        input_file,
        "fn main() {{ let x = 42; echo(\"Hello\"); }} fn echo(s: &str) {{}}"
    )
    .unwrap();

    Transpiler::new()
        .input(input_file.path())
        .output("target/examples/pre-commit")
        .permissions(0o755)
        .transpile()?;

    println!("  ✓ Transpiled pre-commit hook");
    Ok(())
}

/// Example 2: Transpilation with custom configuration
fn custom_config_transpilation() -> bashrs::Result<()> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Strict,
        optimize: true,
        emit_proof: false,
        validation_level: None,
        strict_mode: true,
    };

    let mut input_file = NamedTempFile::new().unwrap();
    write!(
        input_file,
        "fn main() {{ let x = 42; echo(\"Hello\"); }} fn echo(s: &str) {{}}"
    )
    .unwrap();

    Transpiler::new()
        .input(input_file.path())
        .output("target/examples/pre-commit-strict")
        .permissions(0o755)
        .config(config)
        .transpile()?;

    println!("  ✓ Transpiled with strict verification");
    Ok(())
}

/// Example 3: Batch transpilation of multiple hooks
fn batch_transpilation() -> bashrs::Result<()> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create multiple temporary files
    let mut files = Vec::new();
    for i in 1..=3 {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "fn main() {{ let x = {}; }} ", i).unwrap();
        files.push(file);
    }

    let outputs = vec![
        "target/examples/batch/hook1",
        "target/examples/batch/hook2",
        "target/examples/batch/hook3",
    ];

    for (i, output) in outputs.iter().enumerate() {
        print!("  Transpiling hook{} ... ", i + 1);
        Transpiler::new()
            .input(files[i].path())
            .output(output)
            .permissions(0o755)
            .transpile()?;
        println!("✓");
    }

    println!("  ✓ Batch transpilation complete");
    Ok(())
}
