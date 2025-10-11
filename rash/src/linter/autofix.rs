//! Auto-fix application for linter diagnostics
//!
//! Applies suggested fixes to source code with:
//! - Backup creation before modification
//! - Span-based replacement
//! - Dry-run mode for preview
//! - Safe application (reverse order to preserve positions)

use crate::linter::{Diagnostic, LintResult, Span};
use std::fs;
use std::io;
use std::path::Path;

/// Options for auto-fix application
#[derive(Debug, Clone)]
pub struct FixOptions {
    /// Create backup file before applying fixes
    pub create_backup: bool,
    /// Dry-run mode (don't actually modify file)
    pub dry_run: bool,
    /// Backup file suffix
    pub backup_suffix: String,
}

impl Default for FixOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            dry_run: false,
            backup_suffix: ".bak".to_string(),
        }
    }
}

/// Result of applying fixes
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Number of fixes applied
    pub fixes_applied: usize,
    /// Modified source code (if not dry-run)
    pub modified_source: Option<String>,
    /// Backup file path (if backup created)
    pub backup_path: Option<String>,
}

/// Apply fixes from lint result to source code
///
/// # Arguments
/// * `source` - Original source code
/// * `result` - Lint result containing diagnostics with fixes
/// * `options` - Fix application options
///
/// # Returns
/// Result containing number of fixes applied and modified source
pub fn apply_fixes(source: &str, result: &LintResult, options: &FixOptions) -> io::Result<FixResult> {
    let mut modified = source.to_string();
    let mut fixes_applied = 0;

    // Get diagnostics with fixes, sorted by position (reverse order to preserve positions)
    let mut diagnostics_with_fixes: Vec<&Diagnostic> = result
        .diagnostics
        .iter()
        .filter(|d| d.fix.is_some())
        .collect();

    // Sort in reverse order (bottom to top, right to left)
    diagnostics_with_fixes.sort_by(|a, b| {
        b.span
            .start_line
            .cmp(&a.span.start_line)
            .then(b.span.start_col.cmp(&a.span.start_col))
    });

    // Apply fixes in reverse order to preserve positions
    for diagnostic in diagnostics_with_fixes {
        if let Some(fix) = &diagnostic.fix {
            modified = apply_single_fix(&modified, &diagnostic.span, &fix.replacement)?;
            fixes_applied += 1;
        }
    }

    Ok(FixResult {
        fixes_applied,
        modified_source: if options.dry_run { None } else { Some(modified) },
        backup_path: None,
    })
}

/// Apply fixes to a file
///
/// # Arguments
/// * `file_path` - Path to file to fix
/// * `result` - Lint result containing diagnostics with fixes
/// * `options` - Fix application options
///
/// # Returns
/// Result containing fix application results
pub fn apply_fixes_to_file(
    file_path: &Path,
    result: &LintResult,
    options: &FixOptions,
) -> io::Result<FixResult> {
    let source = fs::read_to_string(file_path)?;

    let mut fix_result = apply_fixes(&source, result, options)?;

    // Create backup if requested and not dry-run
    if options.create_backup && !options.dry_run {
        let backup_path = format!("{}{}", file_path.display(), options.backup_suffix);
        fs::copy(file_path, &backup_path)?;
        fix_result.backup_path = Some(backup_path);
    }

    // Write modified source if not dry-run
    if !options.dry_run {
        if let Some(ref modified) = fix_result.modified_source {
            fs::write(file_path, modified)?;
        }
    }

    Ok(fix_result)
}

/// Apply a single fix to source code
///
/// # Arguments
/// * `source` - Source code
/// * `span` - Span to replace
/// * `replacement` - Replacement text
///
/// # Returns
/// Modified source code
fn apply_single_fix(source: &str, span: &Span, replacement: &str) -> io::Result<String> {
    let lines: Vec<&str> = source.lines().collect();

    if span.start_line == 0 || span.start_line > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid line number: {}", span.start_line),
        ));
    }

    let line_idx = span.start_line - 1; // Convert to 0-indexed
    let line = lines[line_idx];

    // Check bounds
    if span.start_col == 0 || span.start_col > line.len() + 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid start column: {}", span.start_col),
        ));
    }

    if span.end_col == 0 || span.end_col > line.len() + 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid end column: {}", span.end_col),
        ));
    }

    // Apply fix (columns are 1-indexed)
    let before = &line[..span.start_col - 1];
    let after = &line[span.end_col - 1..];
    let fixed_line = format!("{}{}{}", before, replacement, after);

    // Reconstruct source
    let mut result_lines = lines.clone();
    result_lines[line_idx] = &fixed_line;

    // Note: This is a simplified version that only handles single-line fixes
    // Multi-line fixes would require more complex logic
    let result = result_lines.join("\n");

    // Preserve final newline if original had one
    if source.ends_with('\n') && !result.ends_with('\n') {
        Ok(format!("{}\n", result))
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Diagnostic, Fix, Severity};

    #[test]
    fn test_apply_single_fix_basic() {
        let source = "echo $VAR\n";
        let span = Span::new(1, 6, 1, 10); // $VAR at columns 6-10
        let replacement = "\"$VAR\"";

        let result = apply_single_fix(source, &span, replacement).unwrap();
        assert_eq!(result, "echo \"$VAR\"\n");
    }

    #[test]
    fn test_apply_multiple_fixes_reverse_order() {
        let source = "ls $DIR1 $DIR2\n";

        let mut result = LintResult::new();

        // Add two diagnostics (will be applied in reverse order)
        result.add(
            Diagnostic::new(
                "SC2086",
                Severity::Warning,
                "Unquoted $DIR1".to_string(),
                Span::new(1, 4, 1, 9),
            )
            .with_fix(Fix::new("\"$DIR1\"".to_string())),
        );

        result.add(
            Diagnostic::new(
                "SC2086",
                Severity::Warning,
                "Unquoted $DIR2".to_string(),
                Span::new(1, 10, 1, 15),
            )
            .with_fix(Fix::new("\"$DIR2\"".to_string())),
        );

        let options = FixOptions::default();
        let fix_result = apply_fixes(source, &result, &options).unwrap();

        assert_eq!(fix_result.fixes_applied, 2);
        assert_eq!(
            fix_result.modified_source.unwrap(),
            "ls \"$DIR1\" \"$DIR2\"\n"
        );
    }

    #[test]
    fn test_dry_run_mode() {
        let source = "echo $VAR\n";

        let mut result = LintResult::new();
        result.add(
            Diagnostic::new(
                "SC2086",
                Severity::Warning,
                "Unquoted".to_string(),
                Span::new(1, 6, 1, 10),
            )
            .with_fix(Fix::new("\"$VAR\"".to_string())),
        );

        let options = FixOptions {
            dry_run: true,
            ..Default::default()
        };

        let fix_result = apply_fixes(source, &result, &options).unwrap();

        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.modified_source.is_none()); // No modified source in dry-run
    }

    #[test]
    fn test_no_fixes_to_apply() {
        let source = "echo \"$VAR\"\n";

        let result = LintResult::new(); // No diagnostics

        let options = FixOptions::default();
        let fix_result = apply_fixes(source, &result, &options).unwrap();

        assert_eq!(fix_result.fixes_applied, 0);
        assert_eq!(fix_result.modified_source.unwrap(), source);
    }

    #[test]
    fn test_invalid_span() {
        let source = "echo test\n";
        let span = Span::new(999, 1, 999, 5); // Invalid line

        let result = apply_single_fix(source, &span, "replacement");
        assert!(result.is_err());
    }
}
