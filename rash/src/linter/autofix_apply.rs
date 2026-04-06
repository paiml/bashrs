pub fn apply_fixes(
    source: &str,
    result: &LintResult,
    options: &FixOptions,
) -> io::Result<FixResult> {
    let mut modified = source.to_string();
    let mut fixes_applied = 0;

    // Get diagnostics with fixes, filtered by safety level
    let mut diagnostics_with_fixes: Vec<&Diagnostic> = result
        .diagnostics
        .iter()
        .filter(|d| {
            if let Some(fix) = &d.fix {
                // Apply based on safety level and options
                if options.apply_assumptions {
                    // With --fix-assumptions: SAFE + SAFE-WITH-ASSUMPTIONS
                    fix.is_safe_with_assumptions()
                } else {
                    // Default (--fix only): SAFE only
                    fix.is_safe()
                }
            } else {
                false
            }
        })
        .collect();

    // Sort by priority (high to low), then by position (reverse order)
    // This ensures:
    // 1. High-priority fixes are applied first (SC2116 before SC2086)
    // 2. Within same priority, fixes are applied bottom-to-top, right-to-left
    diagnostics_with_fixes.sort_by(|a, b| {
        let priority_a = FixPriority::from_code(&a.code);
        let priority_b = FixPriority::from_code(&b.code);

        // Higher priority first (reverse order)
        priority_b
            .cmp(&priority_a)
            // Then by position (reverse order)
            .then(b.span.start_line.cmp(&a.span.start_line))
            .then(b.span.start_col.cmp(&a.span.start_col))
    });

    // Track which spans we've already fixed to skip conflicts
    let mut applied_spans: Vec<Span> = Vec::new();

    // Apply fixes with conflict detection
    for diagnostic in diagnostics_with_fixes {
        if let Some(fix) = &diagnostic.fix {
            // Check if this span overlaps with any already-applied fix
            let has_conflict = applied_spans
                .iter()
                .any(|s| spans_overlap(s, &diagnostic.span));

            if has_conflict {
                // Skip this fix - higher priority fix already applied
                continue;
            }

            // Apply the fix
            modified = apply_single_fix(&modified, &diagnostic.span, &fix.replacement)?;
            fixes_applied += 1;
            applied_spans.push(diagnostic.span);
        }
    }

    Ok(FixResult {
        fixes_applied,
        modified_source: if options.dry_run {
            None
        } else {
            Some(modified)
        },
        backup_path: None,
    })
}

/// Applies fixes from a lint result to a file on disk.
///
/// Reads the file, applies fixes, and writes the result back. Optionally creates
/// a backup before modification.
///
/// # Arguments
///
/// * `file_path` - Path to the file to fix
/// * `result` - Lint result containing diagnostics with suggested fixes
/// * `options` - Configuration for fix application
///
/// # Returns
///
/// * `Ok(FixResult)` - Fix results including count, modified source, and backup path
/// * `Err(io::Error)` - If file I/O or fix application fails
///
/// # Backup Behavior
///
/// If `options.create_backup = true` and `options.dry_run = false`:
/// - Creates backup at `<file_path><backup_suffix>`
/// - Backup path returned in `FixResult.backup_path`
///
/// # Output Behavior
///
/// * `options.output_path = None`: Modify file in-place (default)
/// * `options.output_path = Some(path)`: Write to `path`, leave original unchanged
///
/// # Examples
///
/// ## Apply fixes with backup
///
/// ```no_run
/// use bashrs::linter::{autofix, LintResult};
/// use std::path::Path;
///
/// let file_path = Path::new("script.sh");
/// let result = LintResult::new(); // Assume populated with diagnostics
///
/// let options = autofix::FixOptions {
///     create_backup: true,
///     backup_suffix: ".bak".to_string(),
///     ..Default::default()
/// };
///
/// let fix_result = autofix::apply_fixes_to_file(file_path, &result, &options).unwrap();
/// println!("Applied {} fixes", fix_result.fixes_applied);
/// if let Some(backup) = fix_result.backup_path {
///     println!("Backup created at: {}", backup);
/// }
/// ```
///
/// ## Dry-run on file
///
/// ```no_run
/// use bashrs::linter::{autofix, LintResult};
/// use std::path::Path;
///
/// let file_path = Path::new("script.sh");
/// let result = LintResult::new(); // Assume populated
///
/// let options = autofix::FixOptions {
///     dry_run: true,
///     ..Default::default()
/// };
///
/// let fix_result = autofix::apply_fixes_to_file(file_path, &result, &options).unwrap();
/// println!("Would apply {} fixes (dry-run)", fix_result.fixes_applied);
/// assert!(fix_result.backup_path.is_none()); // No backup in dry-run
/// ```
///
/// ## Write to different file
///
/// ```no_run
/// use bashrs::linter::{autofix, LintResult};
/// use std::path::{Path, PathBuf};
///
/// let input_path = Path::new("original.sh");
/// let output_path = PathBuf::from("fixed.sh");
/// let result = LintResult::new(); // Assume populated
///
/// let options = autofix::FixOptions {
///     output_path: Some(output_path),
///     ..Default::default()
/// };
///
/// let fix_result = autofix::apply_fixes_to_file(input_path, &result, &options).unwrap();
/// println!("Wrote fixed version to fixed.sh");
/// ```
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
            // Write to output_path if specified, otherwise in-place
            let output = if let Some(ref out_path) = options.output_path {
                out_path
            } else {
                file_path
            };
            fs::write(output, modified)?;
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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "autofix_tests_apply_single.rs"]
// FIXME(PMAT-238): mod tests_extracted;
