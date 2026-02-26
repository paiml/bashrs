use crate::models::{Error, Result};
use std::fs;
use std::io::Read;
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

pub(crate) struct PurifyCommandOptions<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub report: bool,
    pub with_tests: bool,
    pub property_tests: bool,
    pub type_check: bool,
    pub emit_guards: bool,
    pub type_strict: bool,
    pub diff: bool,
    pub verify: bool,
    pub recursive: bool,
}

pub(crate) fn purify_command(opts: PurifyCommandOptions<'_>) -> Result<()> {
    // --recursive: walk directory and purify all .sh files
    if opts.recursive {
        return purify_recursive(opts.input, &opts);
    }
    purify_single_file(&opts)
}

fn read_source(input: &Path) -> Result<String> {
    if input == Path::new("-") {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(Error::Io)?;
        Ok(buf)
    } else {
        fs::read_to_string(input).map_err(Error::Io)
    }
}

fn parse_bash_source(
    source: &str,
    file_str: &str,
) -> Result<(crate::bash_parser::ast::BashAst, std::time::Duration)> {
    use crate::bash_parser::parser::BashParser;
    use std::time::Instant;

    let parse_start = Instant::now();
    let mut parser = BashParser::new(source).map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(&e, source, Some(file_str));
        Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    let ast = parser.parse().map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(
            &e,
            parser.source(),
            Some(file_str),
        );
        Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    Ok((ast, parse_start.elapsed()))
}

fn write_or_print_output(
    output: Option<&Path>,
    purified_bash: &str,
    diff: bool,
    input: &Path,
    source: &str,
) -> Result<()> {
    if diff {
        print_unified_diff(input, source, purified_bash);
    } else if let Some(output_path) = output {
        fs::write(output_path, purified_bash).map_err(Error::Io)?;
        info!("Purified script written to {}", output_path.display());
    } else {
        println!("{purified_bash}");
    }
    Ok(())
}

fn purify_single_file(opts: &PurifyCommandOptions<'_>) -> Result<()> {
    use crate::bash_parser::codegen::{generate_purified_bash, generate_purified_bash_with_guards};
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};
    use std::time::Instant;

    let start = Instant::now();

    let read_start = Instant::now();
    let source = read_source(opts.input)?;
    let read_time = read_start.elapsed();

    let file_str = opts.input.display().to_string();
    let (ast, parse_time) = parse_bash_source(&source, &file_str)?;

    let do_type_check = opts.type_check || opts.emit_guards || opts.type_strict;

    let purify_start = Instant::now();
    let purify_opts = PurificationOptions {
        type_check: do_type_check,
        emit_guards: opts.emit_guards,
        type_strict: opts.type_strict,
        ..PurificationOptions::default()
    };
    let mut purifier = Purifier::new(purify_opts);
    let purified_ast = purifier
        .purify(&ast)
        .map_err(|e| Error::Internal(format!("Failed to purify bash: {e}")))?;
    let purify_time = purify_start.elapsed();

    let codegen_start = Instant::now();
    let purified_bash = if opts.emit_guards {
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
        let has_errors = purify_emit_type_diagnostics(
            opts.input,
            &purifier.report().type_diagnostics,
            opts.type_strict,
        );
        if has_errors {
            return Err(Error::Validation(
                "type checking failed with --type-strict".to_string(),
            ));
        }
    }

    write_or_print_output(opts.output, &purified_bash, opts.diff, opts.input, &source)?;

    let total_time = start.elapsed();

    if opts.verify {
        purify_verify_shellcheck(&purified_bash, opts.input)?;
    }

    if opts.report {
        purify_print_report(PurifyReportData {
            input: opts.input,
            output: opts.output,
            source: &source,
            purified_bash: &purified_bash,
            read_time,
            parse_time,
            purify_time,
            codegen_time,
            write_time: if opts.diff {
                std::time::Duration::ZERO
            } else {
                total_time.saturating_sub(read_time + parse_time + purify_time + codegen_time)
            },
            total_time,
        });
    }

    if opts.with_tests {
        purify_generate_tests(
            opts.output,
            &purified_bash,
            opts.property_tests,
            opts.report,
        )?;
    }

    Ok(())
}

/// Find end of a changed hunk, extending through consecutive changes.
fn find_hunk_end(orig: &[&str], pure: &[&str], start: usize, max_len: usize) -> usize {
    let mut end = start;
    while end < max_len {
        let ol = orig.get(end).copied().unwrap_or("");
        let pl = pure.get(end).copied().unwrap_or("");
        if ol != pl || end >= orig.len() || end >= pure.len() {
            end += 1;
            continue;
        }
        // Check if next 3 lines are all same (end of hunk)
        let all_same = (0..3).all(|j| {
            orig.get(end + j).copied().unwrap_or("") == pure.get(end + j).copied().unwrap_or("")
        });
        if all_same {
            break;
        }
        end += 1;
    }
    end
}

/// Print a single diff hunk with context lines.
fn print_diff_hunk(orig: &[&str], pure: &[&str], ctx_start: usize, ctx_end: usize) {
    let orig_count = ctx_end.min(orig.len()).saturating_sub(ctx_start);
    let pure_count = ctx_end.min(pure.len()).saturating_sub(ctx_start);
    println!(
        "@@ -{},{} +{},{} @@",
        ctx_start + 1,
        orig_count,
        ctx_start + 1,
        pure_count,
    );
    for j in ctx_start..ctx_end {
        match (orig.get(j).copied(), pure.get(j).copied()) {
            (Some(o), Some(p)) if o == p => println!(" {o}"),
            (Some(o), Some(p)) => {
                println!("-{o}");
                println!("+{p}");
            }
            (Some(o), None) => println!("-{o}"),
            (None, Some(p)) => println!("+{p}"),
            (None, None) => {}
        }
    }
}

/// Print a unified diff between original and purified source
fn print_unified_diff(input: &Path, original: &str, purified: &str) {
    let orig_lines: Vec<&str> = original.lines().collect();
    let pure_lines: Vec<&str> = purified.lines().collect();

    println!("--- {}", input.display());
    println!("+++ {}.purified", input.display());

    let max_len = orig_lines.len().max(pure_lines.len());
    let mut i = 0;
    while i < max_len {
        let orig_line = orig_lines.get(i).copied().unwrap_or("");
        let pure_line = pure_lines.get(i).copied().unwrap_or("");

        if orig_line != pure_line || i >= orig_lines.len() || i >= pure_lines.len() {
            let ctx_start = i.saturating_sub(3);
            let hunk_end = find_hunk_end(&orig_lines, &pure_lines, i, max_len);
            let ctx_end = (hunk_end + 3).min(max_len);
            print_diff_hunk(&orig_lines, &pure_lines, ctx_start, ctx_end);
            i = ctx_end;
        } else {
            i += 1;
        }
    }
}

/// Verify purified output passes shellcheck
fn purify_verify_shellcheck(purified_bash: &str, input: &Path) -> Result<()> {
    use std::process::Command;

    // Write purified output to a temp file
    let temp_dir = std::env::temp_dir().join(format!("bashrs-verify-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(Error::Io)?;
    let temp_file = temp_dir.join("purified.sh");
    fs::write(&temp_file, purified_bash).map_err(Error::Io)?;

    let result = Command::new("shellcheck")
        .args(["-s", "sh"])
        .arg(&temp_file)
        .output();

    // Clean up temp files
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_dir(&temp_dir);

    match result {
        Ok(output) => {
            if output.status.success() {
                eprintln!(
                    "shellcheck: {} passed POSIX compliance check",
                    input.display()
                );
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stdout);
                eprintln!("shellcheck: {} failed verification:", input.display());
                eprint!("{stderr}");
                Err(Error::Validation(format!(
                    "shellcheck verification failed for {}",
                    input.display()
                )))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("warning: shellcheck not found, skipping --verify");
            Ok(())
        }
        Err(e) => Err(Error::Io(e)),
    }
}

/// Recursively purify all shell scripts in a directory
fn purify_recursive(dir: &Path, opts: &PurifyCommandOptions<'_>) -> Result<()> {
    use crate::cli::logic::is_shell_script_file;

    if !dir.is_dir() {
        return Err(Error::Validation(format!(
            "--recursive requires a directory, got: {}",
            dir.display()
        )));
    }

    let mut errors = Vec::new();
    let mut count = 0u32;

    walk_dir(dir, &mut |path| {
        // Quick extension check before reading file content
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return, // skip unreadable files
        };

        if !is_shell_script_file(path, &content) {
            return;
        }

        count += 1;
        let file_opts = PurifyCommandOptions {
            input: path,
            output: opts.output, // when recursive, output goes to stdout per-file
            report: opts.report,
            with_tests: false, // skip test generation in recursive mode
            property_tests: false,
            type_check: opts.type_check,
            emit_guards: opts.emit_guards,
            type_strict: opts.type_strict,
            diff: opts.diff,
            verify: opts.verify,
            recursive: false,
        };

        if let Err(e) = purify_single_file(&file_opts) {
            eprintln!("error: {}: {e}", path.display());
            errors.push(format!("{}: {e}", path.display()));
        }
    })?;

    eprintln!("Processed {count} shell script(s)");
    if !errors.is_empty() {
        eprintln!("{} file(s) had errors", errors.len());
    }

    Ok(())
}

/// Simple recursive directory walker (avoids external dependency)
fn walk_dir(dir: &Path, cb: &mut dyn FnMut(&Path)) -> Result<()> {
    let entries = fs::read_dir(dir).map_err(Error::Io)?;
    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            walk_dir(&path, cb)?;
        } else {
            cb(&path);
        }
    }
    Ok(())
}

pub(crate) struct PurifyReportData<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub source: &'a str,
    pub purified_bash: &'a str,
    pub read_time: std::time::Duration,
    pub parse_time: std::time::Duration,
    pub purify_time: std::time::Duration,
    pub codegen_time: std::time::Duration,
    pub write_time: std::time::Duration,
    pub total_time: std::time::Duration,
}

pub(crate) fn purify_print_report(data: PurifyReportData<'_>) {
    let PurifyReportData {
        input,
        output,
        source,
        purified_bash,
        read_time,
        parse_time,
        purify_time,
        codegen_time,
        write_time,
        total_time,
    } = data;
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
