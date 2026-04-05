/// Build a set of ignored rule codes from `--ignore`, `-e` flags, and `.bashrsignore` rule codes.
pub(crate) fn build_ignored_rules(
    ignore_rules: Option<&str>,
    exclude_rules: Option<&[String]>,
    ignore_file_data: Option<&crate::linter::ignore_file::IgnoreFile>,
) -> HashSet<String> {
    let mut rules = HashSet::new();
    // Add from --ignore (comma-separated)
    if let Some(ignore_str) = ignore_rules {
        for code in ignore_str.split(',') {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }
    // Add from -e (can be repeated)
    if let Some(excludes) = exclude_rules {
        for code in excludes {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }
    // Issue #85: Add rule codes from .bashrsignore file
    if let Some(ignore) = ignore_file_data {
        for code in ignore.ignored_rules() {
            rules.insert(code);
        }
    }
    rules
}

/// Determine the minimum severity level based on `--quiet` and `--level` flags.
pub(crate) fn determine_min_severity(quiet: bool, level: LintLevel) -> crate::linter::Severity {
    use crate::linter::Severity;

    if quiet {
        Severity::Warning // --quiet suppresses info
    } else {
        match level {
            LintLevel::Info => Severity::Info,
            LintLevel::Warning => Severity::Warning,
            LintLevel::Error => Severity::Error,
        }
    }
}

/// Export diagnostics in CITL format if a path was provided.
pub(crate) fn export_citl_if_requested(
    input: &Path,
    result_raw: &crate::linter::LintResult,
    citl_export_path: Option<&Path>,
) {
    use crate::linter::citl::CitlExport;

    let Some(citl_path) = citl_export_path else {
        return;
    };

    let export = CitlExport::from_lint_result(
        input.to_str().unwrap_or("unknown"),
        result_raw, // Export raw results (unfiltered) for complete data
    );
    if let Err(e) = export.write_to_file(citl_path) {
        warn!(
            "Failed to write CITL export to {}: {}",
            citl_path.display(),
            e
        );
    } else {
        info!(
            "CITL export written to {} ({} diagnostics)",
            citl_path.display(),
            export.summary.total
        );
    }
}

/// Apply auto-fixes to the file, re-lint, and display remaining issues.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_lint_fixes(
    input: &Path,
    result_raw: &crate::linter::LintResult,
    fix_assumptions: bool,
    output: Option<&Path>,
    file_is_makefile: bool,
    format: LintFormat,
    filter_diagnostics: &dyn Fn(crate::linter::LintResult) -> crate::linter::LintResult,
) -> Result<()> {
    use crate::linter::autofix::{apply_fixes_to_file, FixOptions};
    use crate::linter::output::write_results;
    use crate::linter::rules::{lint_makefile, lint_shell};

    let options = FixOptions {
        create_backup: true,
        dry_run: false,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: fix_assumptions,
        output_path: output.map(|p| p.to_path_buf()),
    };

    match apply_fixes_to_file(input, result_raw, &options) {
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
            let result_after_raw = if file_is_makefile {
                lint_makefile(&source_after)
            } else {
                lint_shell(&source_after)
            };
            let result_after = filter_diagnostics(result_after_raw);

            if result_after.diagnostics.is_empty() {
                info!("All issues fixed!");
                return Ok(());
            }

            info!("Remaining issues after auto-fix:");
            let output_format = super::convert_lint_format(format);
            let file_path = input.to_str().unwrap_or("unknown");
            write_results(
                &mut std::io::stdout(),
                &result_after,
                output_format,
                file_path,
            )
            .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

            Ok(())
        }
        Err(e) => Err(Error::Internal(format!("Failed to apply fixes: {e}"))),
    }
}

/// Display lint results and exit with the appropriate code.
pub(crate) fn output_lint_results(
    result: &crate::linter::LintResult,
    format: LintFormat,
    input: &Path,
) -> Result<()> {
    use crate::linter::output::write_results;

    let output_format = super::convert_lint_format(format);
    let file_path = input.to_str().unwrap_or("unknown");
    write_results(&mut std::io::stdout(), result, output_format, file_path)
        .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

    // Exit with appropriate code (Issue #6)
    // Exit 0: No issues
    // Exit 1: Warnings found
    // Exit 2: Errors found
    if result.has_errors() {
        std::process::exit(2);
    } else if result.has_warnings() {
        std::process::exit(1);
    }

    Ok(())
}

/// Display lint results without calling process::exit (for multi-file aggregation).
fn output_lint_results_no_exit(
    result: &crate::linter::LintResult,
    format: LintFormat,
    input: &Path,
) -> Result<()> {
    use crate::linter::output::write_results;

    let output_format = super::convert_lint_format(format);
    let file_path = input.to_str().unwrap_or("unknown");
    write_results(&mut std::io::stdout(), result, output_format, file_path)
        .map_err(|e| Error::Internal(format!("Failed to write lint results: {e}")))?;

    Ok(())
}
