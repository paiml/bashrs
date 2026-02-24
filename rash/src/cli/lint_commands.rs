use crate::cli::args::{LintFormat, LintLevel, LintProfileArg};
use crate::cli::logic::convert_lint_profile;
use crate::cli::logic::{is_dockerfile, is_makefile, is_shell_script_file};
use crate::models::{Error, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub(crate) struct LintCommandOptions<'a> {
    pub inputs: &'a [PathBuf],
    pub format: LintFormat,
    pub fix: bool,
    pub fix_assumptions: bool,
    pub output: Option<&'a Path>,
    pub no_ignore: bool,
    pub ignore_file_path: Option<&'a Path>,
    pub quiet: bool,
    pub level: LintLevel,
    pub ignore_rules: Option<&'a str>,
    pub exclude_rules: Option<&'a [String]>,
    pub citl_export_path: Option<&'a Path>,
    pub profile: LintProfileArg,
    pub ci: bool,
    pub fail_on: LintLevel,
}

pub(crate) fn lint_command(opts: LintCommandOptions<'_>) -> Result<()> {
    // Expand inputs: directories are walked for shell/make/docker files
    let files = expand_inputs(opts.inputs)?;

    if files.is_empty() {
        return Err(Error::Validation(
            "No lintable files found in the given inputs".to_string(),
        ));
    }

    // For single file, use the original path for backward compatibility
    if files.len() == 1 {
        return lint_single_file(&files[0], &opts);
    }

    // Multi-file mode: lint each file and aggregate results
    lint_multiple_files(&files, &opts)
}

/// Expand input paths: files pass through, directories are walked for lintable files.
fn expand_inputs(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for input in inputs {
        if input.is_dir() {
            walk_for_lintable_files(input, &mut files)?;
        } else {
            files.push(input.clone());
        }
    }

    Ok(files)
}

/// Recursively find lintable files (shell, Makefile, Dockerfile) in a directory.
fn walk_for_lintable_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
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
            walk_for_lintable_files(&path, out)?;
        } else {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check by filename first (Makefile, Dockerfile)
            if is_makefile(filename) || is_dockerfile(filename) {
                out.push(path);
                continue;
            }

            // Check by extension or shebang for shell scripts
            if let Ok(content) = fs::read_to_string(&path) {
                if is_shell_script_file(&path, &content) {
                    out.push(path);
                }
            }
        }
    }

    Ok(())
}

fn lint_single_file(input: &Path, opts: &LintCommandOptions<'_>) -> Result<()> {
    use crate::linter::ignore_file::IgnoreResult;
    use crate::linter::rules::lint_shell;
    use crate::linter::{
        rules::{lint_dockerfile_with_profile, lint_makefile, LintProfile},
        LintResult,
    };

    info!("Linting {}", input.display());

    // Issue #85: Load .bashrsignore FIRST to get both file patterns and rule codes
    let ignore_file_data = load_ignore_file(input, opts.no_ignore, opts.ignore_file_path);

    // Check if this file should be ignored (file pattern matching)
    if let Some(ref ignore) = ignore_file_data {
        if let IgnoreResult::Ignored(pattern) = ignore.should_ignore(input) {
            info!(
                "Skipped {} (matched .bashrsignore pattern: {})",
                input.display(),
                pattern
            );
            if !opts.ci {
                println!(
                    "Skipped: {} (matched .bashrsignore pattern: '{}')",
                    input.display(),
                    pattern
                );
            }
            return Ok(());
        }
    }

    // Build set of ignored rule codes from --ignore, -e flags, AND .bashrsignore (Issue #82, #85)
    let ignored_rules =
        build_ignored_rules(opts.ignore_rules, opts.exclude_rules, ignore_file_data.as_ref());

    // Determine minimum severity based on --quiet and --level flags (Issue #75)
    let min_severity = determine_min_severity(opts.quiet, opts.level);

    // Helper to filter diagnostics by severity and ignored rules (Issue #75, #82, #85)
    let filter_diagnostics = |result: LintResult| -> LintResult {
        let filtered = result
            .diagnostics
            .into_iter()
            .filter(|d| d.severity >= min_severity)
            .filter(|d| !ignored_rules.contains(&d.code.to_uppercase()))
            .collect();
        LintResult {
            diagnostics: filtered,
        }
    };

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Detect file type and use appropriate linter (using logic module)
    let filename = input.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let file_is_makefile = is_makefile(filename);
    let file_is_dockerfile = is_dockerfile(filename);

    // Convert CLI profile arg to linter profile
    let lint_profile = convert_lint_profile(opts.profile);

    // Run linter based on file type
    let result_raw = if file_is_makefile {
        lint_makefile(&source)
    } else if file_is_dockerfile {
        lint_dockerfile_with_profile(&source, lint_profile)
    } else {
        lint_shell(&source)
    };

    // Display profile info if using non-standard profile
    if file_is_dockerfile && lint_profile != LintProfile::Standard {
        info!("Using lint profile: {}", lint_profile);
    }

    // Apply severity filter (Issue #75: --quiet and --level flags)
    let result = filter_diagnostics(result_raw.clone());

    // Issue #83: Export diagnostics in CITL format if requested
    export_citl_if_requested(input, &result_raw, opts.citl_export_path);

    // Apply fixes if requested (use raw result to find all fixable issues)
    if opts.fix && result_raw.diagnostics.iter().any(|d| d.fix.is_some()) {
        handle_lint_fixes(
            input,
            &result_raw,
            opts.fix_assumptions,
            opts.output,
            file_is_makefile,
            opts.format,
            &filter_diagnostics,
        )
    } else if opts.ci {
        // CI mode: emit GitHub Actions annotations
        emit_ci_annotations(input, &result);
        exit_for_fail_on(&result, opts.fail_on)
    } else {
        output_lint_results(&result, opts.format, input)
    }
}

fn lint_multiple_files(files: &[PathBuf], opts: &LintCommandOptions<'_>) -> Result<()> {
    use crate::linter::rules::lint_shell;
    use crate::linter::{
        rules::{lint_dockerfile_with_profile, lint_makefile},
        LintResult,
    };

    let ignored_rules =
        build_ignored_rules(opts.ignore_rules, opts.exclude_rules, None);
    let min_severity = determine_min_severity(opts.quiet, opts.level);
    let lint_profile = convert_lint_profile(opts.profile);

    let mut all_results: Vec<(PathBuf, LintResult)> = Vec::new();
    let mut total_errors = 0u32;
    let mut total_warnings = 0u32;

    for file in files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                warn!("Could not read {}: {}", file.display(), e);
                continue;
            }
        };

        let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_is_makefile = is_makefile(filename);
        let file_is_dockerfile = is_dockerfile(filename);

        let result_raw = if file_is_makefile {
            lint_makefile(&source)
        } else if file_is_dockerfile {
            lint_dockerfile_with_profile(&source, lint_profile)
        } else {
            lint_shell(&source)
        };

        let result = LintResult {
            diagnostics: result_raw
                .diagnostics
                .into_iter()
                .filter(|d| d.severity >= min_severity)
                .filter(|d| !ignored_rules.contains(&d.code.to_uppercase()))
                .collect(),
        };

        if result.has_errors() {
            total_errors += 1;
        }
        if result.has_warnings() {
            total_warnings += 1;
        }

        if opts.ci {
            emit_ci_annotations(file, &result);
        }

        if !result.diagnostics.is_empty() {
            all_results.push((file.clone(), result));
        }
    }

    // Output results
    if !opts.ci {
        for (file, result) in &all_results {
            output_lint_results_no_exit(result, opts.format, file)?;
        }
        eprintln!(
            "\nLinted {} file(s): {} with errors, {} with warnings",
            files.len(),
            total_errors,
            total_warnings,
        );
    }

    // Exit based on --fail-on threshold
    let has_failing = match opts.fail_on {
        LintLevel::Error => total_errors > 0,
        LintLevel::Warning => total_errors > 0 || total_warnings > 0,
        LintLevel::Info => all_results.iter().any(|(_, r)| !r.diagnostics.is_empty()),
    };

    if has_failing {
        if total_errors > 0 {
            std::process::exit(2);
        } else {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Emit GitHub Actions workflow annotations for each diagnostic.
fn emit_ci_annotations(input: &Path, result: &crate::linter::LintResult) {
    use crate::linter::Severity;

    for diag in &result.diagnostics {
        let level = match diag.severity {
            Severity::Error => "error",
            Severity::Warning | Severity::Risk => "warning",
            Severity::Info | Severity::Note | Severity::Perf => "notice",
        };
        println!(
            "::{level} file={},line={},col={},title={}::{}",
            input.display(),
            diag.span.start_line,
            diag.span.start_col,
            diag.code,
            diag.message,
        );
    }
}

/// Check result against --fail-on threshold and exit appropriately.
fn exit_for_fail_on(
    result: &crate::linter::LintResult,
    fail_on: LintLevel,
) -> Result<()> {
    let should_fail = match fail_on {
        LintLevel::Error => result.has_errors(),
        LintLevel::Warning => result.has_errors() || result.has_warnings(),
        LintLevel::Info => !result.diagnostics.is_empty(),
    };

    if should_fail {
        if result.has_errors() {
            std::process::exit(2);
        } else {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Load `.bashrsignore` file and return it if found.
///
/// Returns `None` when `no_ignore` is set, no ignore file exists, or the file
/// cannot be loaded. The caller is responsible for checking `should_ignore`.
pub(crate) fn load_ignore_file(
    input: &Path,
    no_ignore: bool,
    ignore_file_path: Option<&Path>,
) -> Option<crate::linter::ignore_file::IgnoreFile> {
    use crate::linter::ignore_file::IgnoreFile;

    if no_ignore {
        return None;
    }

    // Determine ignore file path
    let ignore_path = ignore_file_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            // Look for .bashrsignore in current directory or parent directories
            let mut current = input
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            loop {
                let candidate = current.join(".bashrsignore");
                if candidate.exists() {
                    return candidate;
                }
                if !current.pop() {
                    break;
                }
            }
            // Default to current directory
            PathBuf::from(".bashrsignore")
        });

    // Load ignore file if it exists
    match IgnoreFile::load(&ignore_path) {
        Ok(Some(ignore)) => Some(ignore),
        Ok(None) => None,
        Err(e) => {
            warn!("Failed to load .bashrsignore: {}", e);
            None
        }
    }
}

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
