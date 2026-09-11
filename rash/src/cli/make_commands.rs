use crate::cli::args::{LintFormat, MakeCommands, MakeOutputFormat, ReportFormat};
use crate::cli::logic::{
    format_purify_report_human, format_purify_report_json, format_purify_report_markdown,
    parse_rule_filter,
};
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::info;

pub(crate) fn handle_make_command(command: MakeCommands) -> Result<()> {
    match command {
        MakeCommands::Build { input, output } => {
            info!(
                "Building Makefile from {} -> {}",
                input.display(),
                output.display()
            );
            make_build_command(&input, &output)
        }
        MakeCommands::Parse { input, format } => {
            info!("Parsing {}", input.display());
            make_parse_command(&input, format)
        }
        MakeCommands::Purify {
            input,
            output,
            fix,
            report,
            format,
            with_tests,
            property_tests,
            preserve_formatting,
            max_line_length,
            skip_blank_line_removal,
            skip_consolidation,
        } => {
            info!("Purifying {}", input.display());
            make_purify_command(
                &input,
                output.as_deref(),
                fix,
                report,
                format,
                with_tests,
                property_tests,
                preserve_formatting,
                max_line_length,
                skip_blank_line_removal,
                skip_consolidation,
            )
        }
        MakeCommands::Lint {
            input,
            format,
            fix,
            output,
            rules,
        } => {
            info!("Linting {}", input.display());
            make_lint_command(&input, format, fix, output.as_deref(), rules.as_deref())
        }
    }
}

pub(crate) fn make_build_command(input: &Path, output: &Path) -> Result<()> {
    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let config = crate::models::Config::default();

    let makefile_content = crate::transpile_makefile(&source, &config)?;

    fs::write(output, &makefile_content).map_err(Error::Io)?;
    info!("Successfully generated Makefile at {}", output.display());

    // Run lint on generated output
    let lint_result = crate::linter::rules::lint_makefile(&makefile_content);
    if !lint_result.diagnostics.is_empty() {
        use tracing::warn;
        warn!(
            "Generated Makefile has {} lint issues",
            lint_result.diagnostics.len()
        );
    }

    Ok(())
}

pub(crate) fn make_parse_command(input: &Path, format: MakeOutputFormat) -> Result<()> {
    use crate::make_parser::parser::parse_makefile;

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)
        .map_err(|e| Error::Validation(format!("Failed to parse Makefile: {}", e)))?;

    match format {
        MakeOutputFormat::Text => {
            println!("{:#?}", ast);
        }
        MakeOutputFormat::Json => {
            // Note: MakeAst doesn't derive Serialize yet, so we'll use Debug format
            println!("{:#?}", ast);
        }
        MakeOutputFormat::Debug => {
            println!("{:?}", ast);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    report: bool,
    format: ReportFormat,
    with_tests: bool,
    property_tests: bool,
    preserve_formatting: bool,
    max_line_length: Option<usize>,
    skip_blank_line_removal: bool,
    skip_consolidation: bool,
) -> Result<()> {
    use crate::make_parser::{
        generators::{generate_purified_makefile_with_options, MakefileGeneratorOptions},
        parser::parse_makefile,
        purify::purify_makefile,
    };

    if with_tests && output.is_none() {
        return Err(Error::Validation(
            "--with-tests requires -o flag to specify output file".to_string(),
        ));
    }

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)
        .map_err(|e| Error::Validation(format!("Failed to parse Makefile: {}", e)))?;
    let purify_result = purify_makefile(&ast);

    if report {
        print_purify_report(&purify_result, format);
    }

    let generator_options = MakefileGeneratorOptions {
        preserve_formatting,
        max_line_length,
        skip_blank_line_removal,
        skip_consolidation,
    };
    let purified = generate_purified_makefile_with_options(&purify_result.ast, &generator_options);

    make_purify_write_output(input, output, fix, &purified)?;

    if with_tests {
        if let Some(output_path) = output {
            make_purify_generate_tests(output_path, &purified, property_tests)?;
        }
    }

    Ok(())
}

fn make_purify_write_output(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    purified: &str,
) -> Result<()> {
    if let Some(output_path) = output {
        fs::write(output_path, purified).map_err(Error::Io)?;
        info!("Purified Makefile written to {}", output_path.display());
    } else if fix {
        let backup_path = input.with_extension("mk.bak");
        fs::copy(input, &backup_path).map_err(Error::Io)?;
        fs::write(input, purified).map_err(Error::Io)?;
        info!("Purified Makefile written to {}", input.display());
        info!("Backup created at {}", backup_path.display());
    } else {
        println!("{}", purified);
    }
    Ok(())
}

fn make_purify_generate_tests(
    output_path: &Path,
    purified: &str,
    property_tests: bool,
) -> Result<()> {
    use crate::make_parser::{MakefileTestGenerator, MakefileTestGeneratorOptions};

    let test_options = MakefileTestGeneratorOptions {
        property_tests,
        property_test_count: 100,
    };
    let test_generator = MakefileTestGenerator::new(test_options);
    let test_suite = test_generator.generate_tests(output_path, purified);

    let file_name = output_path
        .file_name()
        .ok_or_else(|| Error::Internal("Invalid output path".to_string()))?
        .to_str()
        .ok_or_else(|| Error::Internal("Invalid UTF-8 in filename".to_string()))?;
    let test_file = output_path.with_file_name(format!("{}.test.sh", file_name));

    fs::write(&test_file, test_suite).map_err(Error::Io)?;
    info!("Test suite written to {}", test_file.display());
    Ok(())
}

/// Thin shim - delegates formatting to pure logic functions
fn print_purify_report(
    result: &crate::make_parser::purify::PurificationResult,
    format: ReportFormat,
) {
    let output = match format {
        ReportFormat::Human => format_purify_report_human(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
        ReportFormat::Json => format_purify_report_json(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
        ReportFormat::Markdown => format_purify_report_markdown(
            result.transformations_applied,
            result.issues_fixed,
            result.manual_fixes_needed,
            &result.report,
        ),
    };
    print!("{}", output);
}

/// Convert LintFormat to OutputFormat
pub(crate) fn convert_lint_format(format: LintFormat) -> crate::linter::output::OutputFormat {
    use crate::linter::output::OutputFormat;
    match format {
        LintFormat::Human => OutputFormat::Human,
        LintFormat::Json => OutputFormat::Json,
        LintFormat::Sarif => OutputFormat::Sarif,
    }
}

/// Run linter and optionally filter results by specific rules (thin shim)
pub(crate) fn run_filtered_lint(source: &str, rules: Option<&str>) -> crate::linter::LintResult {
    use crate::linter::rules::lint_makefile;

    let mut result = lint_makefile(source);

    // Filter by specific rules if requested - uses logic::parse_rule_filter
    if let Some(rule_filter) = rules {
        let allowed_rules = parse_rule_filter(rule_filter);
        result
            .diagnostics
            .retain(|d| allowed_rules.iter().any(|rule| d.code.contains(rule)));
    }

    result
}

/// Apply fixes and write to separate output file (not in-place)
fn apply_fixes_to_output(
    source: &str,
    result: &crate::linter::LintResult,
    output_path: &Path,
    format: LintFormat,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes, FixOptions},
        output::write_results,
        rules::lint_makefile,
    };

    let fix_options = FixOptions {
        create_backup: false, // Don't create backup for output file
        dry_run: false,
        backup_suffix: String::new(),
        apply_assumptions: false,
        output_path: None,
    };

    let fix_result = apply_fixes(source, result, &fix_options)
        .map_err(|e| Error::Internal(format!("Failed to apply fixes: {e}")))?;

    if let Some(fixed_source) = fix_result.modified_source {
        fs::write(output_path, &fixed_source).map_err(Error::Io)?;
        info!("Fixed Makefile written to {}", output_path.display());

        // Re-lint the fixed content
        let result_after = lint_makefile(&fixed_source);
        if result_after.diagnostics.is_empty() {
            info!("✓ All issues fixed!");
        } else {
            info!("Remaining issues after auto-fix:");
            let output_format = convert_lint_format(format);
            let file_path = output_path.to_str().unwrap_or("unknown");
            write_results(
                &mut std::io::stdout(),
                &result_after,
                output_format,
                file_path,
            )
            .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;
        }
    }

    Ok(())
}

/// Apply fixes in-place to the original file with backup
fn apply_fixes_inplace(
    input: &Path,
    result: &crate::linter::LintResult,
    format: LintFormat,
) -> Result<()> {
    use crate::linter::{
        autofix::{apply_fixes_to_file, FixOptions},
        output::write_results,
        rules::lint_makefile,
    };

    let options = FixOptions {
        create_backup: true,
        dry_run: false,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: false,
        output_path: None,
    };

    match apply_fixes_to_file(input, result, &options) {
        Ok(fix_result) => {
            info!(
                "Applied {} fix(es) to {}",
                fix_result.fixes_applied,
                input.display()
            );
            if let Some(backup_path) = &fix_result.backup_path {
                info!("Backup created at {}", backup_path);
            }

            // Re-lint to show remaining issues
            let source_after = fs::read_to_string(input).map_err(Error::Io)?;
            let result_after = lint_makefile(&source_after);

            if result_after.diagnostics.is_empty() {
                info!("✓ All issues fixed!");
            } else {
                info!("Remaining issues after auto-fix:");
                let output_format = convert_lint_format(format);
                let file_path = input.to_str().unwrap_or("unknown");
                write_results(
                    &mut std::io::stdout(),
                    &result_after,
                    output_format,
                    file_path,
                )
                .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;
            }
        }
        Err(e) => {
            return Err(Error::Internal(format!("Failed to apply fixes: {e}")));
        }
    }

    Ok(())
}

/// Show lint results without applying fixes
pub(crate) fn show_lint_results(
    result: &crate::linter::LintResult,
    format: LintFormat,
    input: &Path,
) -> Result<()> {
    use crate::linter::output::write_results;

    let output_format = convert_lint_format(format);
    let file_path = input.to_str().unwrap_or("unknown");
    write_results(&mut std::io::stdout(), result, output_format, file_path)
        .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

    // Exit with appropriate code
    if result.has_errors() {
        std::process::exit(2);
    } else if result.has_warnings() {
        std::process::exit(1);
    }

    Ok(())
}

pub(crate) fn make_lint_command(
    input: &Path,
    format: LintFormat,
    fix: bool,
    output: Option<&Path>,
    rules: Option<&str>,
) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Run linter and filter by rules if requested
    let result = run_filtered_lint(&source, rules);

    // Apply fixes if requested
    if fix && result.diagnostics.iter().any(|d| d.fix.is_some()) {
        if let Some(output_path) = output {
            // Output to separate file: don't modify original
            apply_fixes_to_output(&source, &result, output_path, format)?;
        } else {
            // In-place fixing: modify original file
            apply_fixes_inplace(input, &result, format)?;
        }
    } else {
        // Just show lint results
        show_lint_results(&result, format, input)?;
    }

    Ok(())
}

// PMAT-257: unit coverage for the `bashrs make` CLI handlers. These tests only
// ever operate on files written under `tempfile::TempDir`, never on files in
// the repository, and never touch `CorpusRegistry` / the corpus runner.
//
// `show_lint_results` calls `std::process::exit(1)`/`exit(2)` once the final
// `LintResult` has warnings/errors -- calling it (directly or via
// `make_lint_command`) with a source that actually produces diagnostics would
// kill the test process. Every test that reaches `show_lint_results` below
// therefore uses either a rule filter that matches nothing, or a Makefile
// with zero lint findings.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::cli::args::MakeCommands;
    use tempfile::TempDir;

    const VALID_MAKEFILE: &str = ".PHONY: all\nall:\n\t@echo test\n";
    const INVALID_MAKEFILE: &str = "this is not : : a valid makefile ][{{{";
    const MKDIR_MAKEFILE: &str = ".PHONY: build\nbuild:\n\tmkdir build\n\t@echo done\n";
    const DSL_SOURCE: &str = r#"
fn main() {
    let cc = "gcc";
    target("build", &["main.c"], &["gcc -o build main.c"]);
}
"#;

    // ---------------------------------------------------------------
    // handle_make_command dispatch
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_handle_make_command_build_dispatch() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("build.rs");
        let output = dir.path().join("Makefile");
        fs::write(&input, DSL_SOURCE).unwrap();

        let result = handle_make_command(MakeCommands::Build {
            input,
            output: output.clone(),
        });
        assert!(result.is_ok(), "build dispatch failed: {result:?}");
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("CC := gcc"), "got: {content}");
    }

    #[test]
    fn test_PMAT257_cov_handle_make_command_parse_dispatch() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = handle_make_command(MakeCommands::Parse {
            input,
            format: MakeOutputFormat::Text,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_handle_make_command_purify_dispatch() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = handle_make_command(MakeCommands::Purify {
            input,
            output: None,
            fix: false,
            report: false,
            format: ReportFormat::Human,
            with_tests: false,
            property_tests: false,
            preserve_formatting: false,
            max_line_length: None,
            skip_blank_line_removal: false,
            skip_consolidation: false,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_handle_make_command_lint_dispatch() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = handle_make_command(MakeCommands::Lint {
            input,
            format: LintFormat::Human,
            fix: false,
            output: None,
            rules: Some("NONEXISTENT".to_string()),
        });
        assert!(result.is_ok());
    }

    // ---------------------------------------------------------------
    // make_build_command
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_make_build_command_success() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("build.rs");
        let output = dir.path().join("Makefile");
        fs::write(&input, DSL_SOURCE).unwrap();

        let result = make_build_command(&input, &output);
        assert!(result.is_ok(), "build failed: {result:?}");
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("CC := gcc"));
        assert!(content.contains("build"));
    }

    #[test]
    fn test_PMAT257_cov_make_build_command_missing_input() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("does-not-exist.rs");
        let output = dir.path().join("Makefile");

        let result = make_build_command(&input, &output);
        assert!(result.is_err());
    }

    #[test]
    fn test_PMAT257_cov_make_build_command_invalid_dsl() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("bad.rs");
        let output = dir.path().join("Makefile");
        fs::write(&input, "not valid rust syntax {{{").unwrap();

        let result = make_build_command(&input, &output);
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // make_parse_command
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_make_parse_command_text_format() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = make_parse_command(&input, MakeOutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_make_parse_command_json_format() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = make_parse_command(&input, MakeOutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_make_parse_command_debug_format() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = make_parse_command(&input, MakeOutputFormat::Debug);
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_make_parse_command_missing_input() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("does-not-exist");

        let result = make_parse_command(&input, MakeOutputFormat::Text);
        assert!(result.is_err());
    }

    #[test]
    fn test_PMAT257_cov_make_parse_command_lenient_makefile_still_parses() {
        // The Makefile parser is lenient: a malformed recipe line is kept as
        // text rather than refused, so this exercises the recovery path. The
        // missing-file case above is the real error path.
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, INVALID_MAKEFILE).unwrap();

        let result = make_parse_command(&input, MakeOutputFormat::Text);
        assert!(result.is_ok(), "lenient parse must not fail: {result:?}");
    }

    // ---------------------------------------------------------------
    // make_purify_command
    // ---------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn purify_opts(
        input: &Path,
        output: Option<&Path>,
        fix: bool,
        report: bool,
        format: ReportFormat,
        with_tests: bool,
        property_tests: bool,
    ) -> Result<()> {
        make_purify_command(
            input,
            output,
            fix,
            report,
            format,
            with_tests,
            property_tests,
            false,
            None,
            false,
            false,
        )
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_with_tests_requires_output() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(&input, None, false, false, ReportFormat::Human, true, false);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("--with-tests"));
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_prints_to_stdout() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(
            &input,
            None,
            false,
            false,
            ReportFormat::Human,
            false,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_writes_output_file() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        let output = dir.path().join("Makefile.purified");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(
            &input,
            Some(&output),
            false,
            false,
            ReportFormat::Human,
            false,
            false,
        );
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_fix_creates_backup() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(&input, None, true, false, ReportFormat::Human, false, false);
        assert!(result.is_ok());
        let backup = input.with_extension("mk.bak");
        assert!(backup.exists(), "expected backup at {backup:?}");
        assert_eq!(fs::read_to_string(&backup).unwrap(), VALID_MAKEFILE);
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_report_all_formats() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        for format in [
            ReportFormat::Human,
            ReportFormat::Json,
            ReportFormat::Markdown,
        ] {
            let label = format!("{format:?}");
            let result = purify_opts(&input, None, false, true, format, false, false);
            assert!(result.is_ok(), "report format {label} failed");
        }
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_generates_test_suite() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        let output = dir.path().join("Makefile.purified");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(
            &input,
            Some(&output),
            false,
            false,
            ReportFormat::Human,
            true,
            false,
        );
        assert!(result.is_ok(), "{result:?}");
        let test_file = dir.path().join("Makefile.purified.test.sh");
        assert!(test_file.exists());
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_generates_property_test_suite() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        let output = dir.path().join("Makefile.purified");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let result = purify_opts(
            &input,
            Some(&output),
            false,
            false,
            ReportFormat::Human,
            true,
            true,
        );
        assert!(result.is_ok(), "{result:?}");
        let test_file = dir.path().join("Makefile.purified.test.sh");
        assert!(test_file.exists());
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_missing_input() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("does-not-exist");

        let result = purify_opts(
            &input,
            None,
            false,
            false,
            ReportFormat::Human,
            false,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_PMAT257_cov_make_purify_command_lenient_makefile_still_purifies() {
        // Same leniency as the parser: purification of a malformed recipe
        // line succeeds and reports what it kept.
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, INVALID_MAKEFILE).unwrap();

        let result = purify_opts(
            &input,
            None,
            false,
            false,
            ReportFormat::Human,
            false,
            false,
        );
        assert!(result.is_ok(), "lenient purify must not fail: {result:?}");
    }

    // ---------------------------------------------------------------
    // convert_lint_format / run_filtered_lint
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_convert_lint_format_all_branches() {
        use crate::linter::output::OutputFormat;
        assert!(matches!(
            convert_lint_format(LintFormat::Human),
            OutputFormat::Human
        ));
        assert!(matches!(
            convert_lint_format(LintFormat::Json),
            OutputFormat::Json
        ));
        assert!(matches!(
            convert_lint_format(LintFormat::Sarif),
            OutputFormat::Sarif
        ));
    }

    #[test]
    fn test_PMAT257_cov_run_filtered_lint_no_filter_keeps_all() {
        let result = run_filtered_lint(MKDIR_MAKEFILE, None);
        assert!(
            result.diagnostics.iter().any(|d| d.code == "MAKE002"),
            "expected MAKE002 in {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_PMAT257_cov_run_filtered_lint_matching_filter_keeps_rule() {
        let result = run_filtered_lint(MKDIR_MAKEFILE, Some("MAKE002"));
        assert!(result.diagnostics.iter().any(|d| d.code == "MAKE002"));
    }

    #[test]
    fn test_PMAT257_cov_run_filtered_lint_nonmatching_filter_drops_all() {
        let result = run_filtered_lint(MKDIR_MAKEFILE, Some("ZZZDOESNOTEXIST"));
        assert!(result.diagnostics.is_empty());
    }

    // ---------------------------------------------------------------
    // show_lint_results (never with real diagnostics -- see module doc)
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_show_lint_results_no_diagnostics_all_formats() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, VALID_MAKEFILE).unwrap();

        let empty = crate::linter::LintResult::new();
        for format in [LintFormat::Human, LintFormat::Json, LintFormat::Sarif] {
            let result = show_lint_results(&empty, format, &input);
            assert!(result.is_ok());
        }
    }

    // ---------------------------------------------------------------
    // make_lint_command
    // ---------------------------------------------------------------

    #[test]
    fn test_PMAT257_cov_make_lint_command_show_results_branch() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, MKDIR_MAKEFILE).unwrap();

        // Rule filter matches nothing -> empty diagnostics -> show_lint_results
        // takes the Ok() path without hitting warning/error exit codes.
        let result = make_lint_command(
            &input,
            LintFormat::Human,
            false,
            None,
            Some("ZZZDOESNOTEXIST"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_PMAT257_cov_make_lint_command_missing_input() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("does-not-exist");

        let result = make_lint_command(&input, LintFormat::Human, false, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_PMAT257_cov_make_lint_command_fix_to_output_file() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        let output = dir.path().join("Makefile.fixed");
        fs::write(&input, MKDIR_MAKEFILE).unwrap();

        let result = make_lint_command(&input, LintFormat::Human, true, Some(&output), None);
        assert!(result.is_ok(), "{result:?}");
        let fixed = fs::read_to_string(&output).unwrap();
        assert!(fixed.contains("mkdir -p"), "got: {fixed}");
        // Original is untouched when fixing to a separate output file.
        assert_eq!(fs::read_to_string(&input).unwrap(), MKDIR_MAKEFILE);
    }

    #[test]
    fn test_PMAT257_cov_make_lint_command_fix_inplace_with_backup() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, MKDIR_MAKEFILE).unwrap();

        let result = make_lint_command(&input, LintFormat::Human, true, None, None);
        assert!(result.is_ok(), "{result:?}");
        let fixed = fs::read_to_string(&input).unwrap();
        assert!(fixed.contains("mkdir -p"), "got: {fixed}");
        let backup = dir.path().join("Makefile.bak");
        assert!(backup.exists());
        assert_eq!(fs::read_to_string(&backup).unwrap(), MKDIR_MAKEFILE);
    }

    #[test]
    fn test_PMAT257_cov_make_lint_command_fix_but_nothing_fixable_falls_through() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("Makefile");
        fs::write(&input, MKDIR_MAKEFILE).unwrap();

        // fix=true but the rule filter drops the only (fixable) diagnostic,
        // so `make_lint_command` must fall through to `show_lint_results`.
        let result = make_lint_command(
            &input,
            LintFormat::Human,
            true,
            None,
            Some("ZZZDOESNOTEXIST"),
        );
        assert!(result.is_ok());
    }
}
