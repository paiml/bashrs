//! `bashrs fix` command (SSC v11 Section 8.1, Linter Spec S9).
//!
//! Standalone auto-fix command that applies linter fixes to shell scripts.
//! Thin wrapper over the autofix infrastructure in `linter::autofix`.
//!
//! ```text
//! bashrs fix script.sh           # Apply SAFE fixes in-place
//! bashrs fix --dry-run script.sh # Preview what would change
//! bashrs fix --assumptions ...   # Include SAFE-WITH-ASSUMPTIONS fixes
//! ```

use crate::linter::autofix::{apply_fixes_to_file, FixOptions};
use crate::linter::{lint_shell, Diagnostic};
use crate::models::{Error, Result};
use std::path::Path;

/// Run `bashrs fix` on one or more files.
pub(crate) fn fix_command(
    inputs: &[std::path::PathBuf],
    dry_run: bool,
    assumptions: bool,
    output: Option<&Path>,
) -> Result<()> {
    if inputs.is_empty() {
        return Err(Error::Validation("No input files specified".to_string()));
    }

    let mut total_fixed = 0usize;
    let mut total_files = 0usize;

    for input in inputs {
        let result = fix_single_file(input, dry_run, assumptions, output)?;
        if result > 0 {
            total_files += 1;
        }
        total_fixed += result;
    }

    print_summary(total_fixed, total_files, inputs.len(), dry_run);
    Ok(())
}

/// Fix a single file and return the number of fixes applied.
fn fix_single_file(
    input: &Path,
    dry_run: bool,
    assumptions: bool,
    output: Option<&Path>,
) -> Result<usize> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", input.display())))?;

    let lint_result = lint_shell(&source);

    let fixable = count_fixable(&lint_result.diagnostics, assumptions);
    if fixable == 0 {
        if !dry_run {
            println!("  {}: no fixable issues", input.display());
        }
        return Ok(0);
    }

    let options = FixOptions {
        create_backup: !dry_run,
        dry_run,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: assumptions,
        output_path: output.map(|p| p.to_path_buf()),
    };

    let fix_result = apply_fixes_to_file(input, &lint_result, &options)
        .map_err(|e| Error::Io(e))?;

    print_file_result(input, &fix_result, dry_run);

    Ok(fix_result.fixes_applied)
}

/// Count how many diagnostics have applicable fixes.
fn count_fixable(diagnostics: &[Diagnostic], assumptions: bool) -> usize {
    diagnostics
        .iter()
        .filter(|d| {
            d.fix.as_ref().is_some_and(|f| {
                use crate::linter::FixSafetyLevel;
                matches!(f.safety_level, FixSafetyLevel::Safe)
                    || (assumptions
                        && matches!(f.safety_level, FixSafetyLevel::SafeWithAssumptions))
            })
        })
        .count()
}

/// Print results for a single file.
fn print_file_result(
    input: &Path,
    result: &crate::linter::autofix::FixResult,
    dry_run: bool,
) {
    let action = if dry_run { "would fix" } else { "fixed" };
    println!(
        "  {}: {action} {} issue{}",
        input.display(),
        result.fixes_applied,
        if result.fixes_applied == 1 { "" } else { "s" }
    );

    if let Some(ref backup) = result.backup_path {
        println!("    backup: {backup}");
    }
}

/// Print overall summary.
fn print_summary(total_fixed: usize, files_changed: usize, total_files: usize, dry_run: bool) {
    if dry_run {
        println!(
            "\nDry run: {total_fixed} fix{} would be applied across {files_changed}/{total_files} file{}.",
            if total_fixed == 1 { "" } else { "es" },
            if total_files == 1 { "" } else { "s" }
        );
    } else if total_fixed > 0 {
        println!(
            "\nApplied {total_fixed} fix{} across {files_changed} file{}.",
            if total_fixed == 1 { "" } else { "es" },
            if files_changed == 1 { "" } else { "s" }
        );
    } else {
        println!("\nNo fixable issues found.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_script(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create temp file");
        f.write_all(content.as_bytes())
            .expect("write temp file");
        f.flush().expect("flush temp file");
        f
    }

    #[test]
    fn test_fix_no_issues() {
        let f = write_temp_script("#!/bin/sh\necho \"hello\"\n");
        let result = fix_single_file(f.path(), true, false, None);
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), 0);
    }

    #[test]
    fn test_fix_dry_run_does_not_modify() {
        let f = write_temp_script("#!/bin/sh\necho $VAR\n");
        let original = std::fs::read_to_string(f.path()).expect("read");
        let _ = fix_single_file(f.path(), true, false, None);
        let after = std::fs::read_to_string(f.path()).expect("read after");
        assert_eq!(original, after, "dry run should not modify file");
    }

    #[test]
    fn test_fix_applies_safe_fixes() {
        let f = write_temp_script("#!/bin/sh\nmkdir /tmp/testdir\n");
        let result = fix_single_file(f.path(), false, true, None);
        assert!(result.is_ok());
        let fixed = std::fs::read_to_string(f.path()).expect("read fixed");
        // IDEM001 fix adds -p flag
        if result.expect("should succeed") > 0 {
            assert!(fixed.contains("-p"), "should contain -p flag after fix");
        }
    }

    #[test]
    fn test_fix_command_empty_inputs() {
        let result = fix_command(&[], false, false, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_count_fixable_no_fixes() {
        let diagnostics = vec![crate::linter::Diagnostic::new(
            "SEC001",
            crate::linter::Severity::Warning,
            "test",
            crate::linter::Span::new(1, 1, 1, 5),
        )];
        assert_eq!(count_fixable(&diagnostics, false), 0);
    }

    #[test]
    fn test_count_fixable_with_safe_fix() {
        let diag = crate::linter::Diagnostic::new(
            "IDEM001",
            crate::linter::Severity::Warning,
            "test",
            crate::linter::Span::new(1, 1, 1, 5),
        )
        .with_fix(crate::linter::Fix::new_with_assumptions(
            "mkdir -p /tmp/test",
            vec!["Directory does not require special permissions".to_string()],
        ));
        let diagnostics = vec![diag];
        // Without assumptions: 0, with assumptions: 1
        assert_eq!(count_fixable(&diagnostics, false), 0);
        assert_eq!(count_fixable(&diagnostics, true), 1);
    }
}
