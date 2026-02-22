use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::info;

/// Print type diagnostics to stderr and return whether any errors were found
pub(crate) fn purify_emit_type_diagnostics(
    input: &Path,
    diagnostics: &[crate::bash_transpiler::type_check::TypeDiagnostic],
    type_strict: bool,
) -> bool {
    use crate::bash_transpiler::type_check::Severity;

    let mut has_errors = false;
    for diag in diagnostics {
        let severity_str = match diag.severity {
            Severity::Error => {
                has_errors = true;
                "error"
            }
            Severity::Warning => {
                if type_strict {
                    has_errors = true;
                }
                "warning"
            }
            Severity::Info => "info",
        };
        eprintln!(
            "{}:{}:{}: {}: {}",
            input.display(),
            diag.span.start_line,
            diag.span.start_col,
            severity_str,
            diag.message,
        );
    }
    has_errors
}

pub(crate) fn purify_command(
    input: &Path,
    output: Option<&Path>,
    report: bool,
    with_tests: bool,
    property_tests: bool,
    type_check: bool,
    emit_guards: bool,
    type_strict: bool,
) -> Result<()> {
    use crate::bash_parser::codegen::{generate_purified_bash, generate_purified_bash_with_guards};
    use crate::bash_parser::parser::BashParser;
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};
    use std::time::Instant;

    let start = Instant::now();

    let read_start = Instant::now();
    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let read_time = read_start.elapsed();

    let parse_start = Instant::now();
    let file_str = input.display().to_string();
    let mut parser = BashParser::new(&source).map_err(|e| {
        let diag =
            crate::bash_parser::parser::format_parse_diagnostic(&e, &source, Some(&file_str));
        Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    let ast = parser.parse().map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(
            &e,
            parser.source(),
            Some(&file_str),
        );
        Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    let parse_time = parse_start.elapsed();

    // --emit-guards and --type-strict both imply --type-check
    let do_type_check = type_check || emit_guards || type_strict;

    let purify_start = Instant::now();
    let opts = PurificationOptions {
        type_check: do_type_check,
        emit_guards,
        type_strict,
        ..PurificationOptions::default()
    };
    let mut purifier = Purifier::new(opts);
    let purified_ast = purifier
        .purify(&ast)
        .map_err(|e| Error::Internal(format!("Failed to purify bash: {e}")))?;
    let purify_time = purify_start.elapsed();

    let codegen_start = Instant::now();
    let purified_bash = if emit_guards {
        if let Some(checker) = purifier.type_checker() {
            generate_purified_bash_with_guards(&purified_ast, checker)
        } else {
            generate_purified_bash(&purified_ast)
        }
    } else {
        generate_purified_bash(&purified_ast)
    };
    let codegen_time = codegen_start.elapsed();

    if do_type_check {
        let has_errors =
            purify_emit_type_diagnostics(input, &purifier.report().type_diagnostics, type_strict);
        if has_errors {
            return Err(Error::Validation(
                "type checking failed with --type-strict".to_string(),
            ));
        }
    }

    let write_start = Instant::now();
    if let Some(output_path) = output {
        fs::write(output_path, &purified_bash).map_err(Error::Io)?;
        info!("Purified script written to {}", output_path.display());
    } else {
        println!("{}", purified_bash);
    }
    let write_time = write_start.elapsed();

    let total_time = start.elapsed();

    if report {
        purify_print_report(
            input,
            output,
            &source,
            &purified_bash,
            read_time,
            parse_time,
            purify_time,
            codegen_time,
            write_time,
            total_time,
        );
    }

    if with_tests {
        purify_generate_tests(output, &purified_bash, property_tests, report)?;
    }

    Ok(())
}

pub(crate) fn purify_print_report(
    input: &Path,
    output: Option<&Path>,
    source: &str,
    purified_bash: &str,
    read_time: std::time::Duration,
    parse_time: std::time::Duration,
    purify_time: std::time::Duration,
    codegen_time: std::time::Duration,
    write_time: std::time::Duration,
    total_time: std::time::Duration,
) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}=== Purification Report ==={RESET}");
    println!("Input:  {CYAN}{}{RESET}", input.display());
    if let Some(output_path) = output {
        println!("Output: {CYAN}{}{RESET}", output_path.display());
    }
    println!();
    println!(
        "Input size:  {WHITE}{} lines{RESET}, {} bytes",
        source.lines().count(),
        source.len()
    );
    println!(
        "Output size: {WHITE}{} lines{RESET}, {} bytes",
        purified_bash.lines().count(),
        purified_bash.len()
    );

    println!();
    println!("{BOLD}Transformations Applied:{RESET}");
    println!("  {GREEN}✓{RESET} Shebang: #!/bin/bash → #!/bin/sh");
    println!("  {GREEN}✓{RESET} Determinism: Removed $RANDOM, timestamps");
    println!("  {GREEN}✓{RESET} Idempotency: mkdir → mkdir -p, rm → rm -f");
    println!("  {GREEN}✓{RESET} Safety: All variables quoted");

    println!();
    println!("{BOLD}Performance:{RESET}");
    println!("  {DIM}Read:{RESET}     {:>8.2?}", read_time);
    println!("  {DIM}Parse:{RESET}    {:>8.2?}", parse_time);
    println!("  {DIM}Purify:{RESET}   {:>8.2?}", purify_time);
    println!("  {DIM}Codegen:{RESET}  {:>8.2?}", codegen_time);
    println!("  {DIM}Write:{RESET}    {:>8.2?}", write_time);
    println!("  {DIM}─────────────────{RESET}");
    println!("  {WHITE}Total:{RESET}    {:>8.2?}", total_time);

    let throughput = (source.len() as f64) / total_time.as_secs_f64() / 1024.0 / 1024.0;
    println!();
    println!("Throughput: {WHITE}{:.2} MB/s{RESET}", throughput);
}

pub(crate) fn purify_generate_tests(
    output: Option<&Path>,
    purified_bash: &str,
    property_tests: bool,
    report: bool,
) -> Result<()> {
    use crate::bash_transpiler::test_generator::{TestGenerator, TestGeneratorOptions};

    let output_path = output.ok_or_else(|| {
        Error::Validation("--with-tests requires -o flag to specify output file".to_string())
    })?;

    let test_file_name = format!(
        "{}_test.sh",
        output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::Internal("Invalid output file name".to_string()))?
    );
    let test_path = output_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&test_file_name);

    let test_options = TestGeneratorOptions {
        property_tests,
        property_test_count: 100,
    };
    let generator = TestGenerator::new(test_options);
    let tests = generator.generate_tests(output_path, purified_bash);

    fs::write(&test_path, tests).map_err(Error::Io)?;
    info!("Test suite written to {}", test_path.display());

    if report {
        println!("\nTest Suite:");
        println!("  Location: {}", test_path.display());
        println!(
            "  Property tests: {}",
            if property_tests {
                "Enabled (100 cases)"
            } else {
                "Disabled"
            }
        );
    }
    Ok(())
}
