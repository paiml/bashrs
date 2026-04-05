//! Auto-fix application for linter diagnostics.
//!
//! Automatically applies suggested fixes to source code with safety guarantees:
//! - **Backup creation**: Original file preserved before modification
//! - **Span-based replacement**: Precise, location-aware fixes
//! - **Dry-run mode**: Preview changes without modification
//! - **Safe application**: Reverse-order processing preserves positions
//! - **Priority-based conflict resolution**: High-priority fixes applied first
//! - **Safety levels**: Respects Safe/SafeWithAssumptions/Unsafe guarantees
//!
//! # Examples
//!
//! ## Basic usage with `apply_fixes`
//!
//! ```
//! use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
//!
//! let source = "echo $VAR\n";
//! let mut result = LintResult::new();
//!
//! // Add diagnostic with fix
//! result.add(
//!     Diagnostic::new("SC2086", Severity::Warning, "Quote variable", Span::new(1, 6, 1, 10))
//!         .with_fix(Fix::new("\"$VAR\""))
//! );
//!
//! // Apply fixes with default options
//! let options = autofix::FixOptions::default();
//! let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
//!
//! assert_eq!(fix_result.fixes_applied, 1);
//! assert_eq!(fix_result.modified_source.unwrap(), "echo \"$VAR\"\n");
//! ```
//!
//! ## File-based fixing with backup
//!
//! ```no_run
//! use bashrs::linter::{autofix, LintResult};
//! use std::path::Path;
//!
//! let file_path = Path::new("script.sh");
//! let result = LintResult::new(); // Assume populated with diagnostics
//!
//! let options = autofix::FixOptions {
//!     create_backup: true,
//!     backup_suffix: ".bak".to_string(),
//!     ..Default::default()
//! };
//!
//! let fix_result = autofix::apply_fixes_to_file(file_path, &result, &options).unwrap();
//! println!("Applied {} fixes, backup at {:?}", fix_result.fixes_applied, fix_result.backup_path);
//! ```
//!
//! ## Dry-run mode (preview changes)
//!
//! ```
//! use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
//!
//! let source = "ls $DIR\n";
//! let mut result = LintResult::new();
//! result.add(
//!     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 4, 1, 8))
//!         .with_fix(Fix::new("\"$DIR\""))
//! );
//!
//! let options = autofix::FixOptions {
//!     dry_run: true,
//!     ..Default::default()
//! };
//!
//! let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
//! assert_eq!(fix_result.fixes_applied, 1);
//! assert!(fix_result.modified_source.is_none()); // No source in dry-run
//! ```

use crate::linter::{Diagnostic, LintResult, Span};
use std::fs;
use std::io;
use std::path::Path;

/// Priority for applying fixes when multiple fixes overlap
/// Higher priority fixes are applied first
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FixPriority {
    /// Remove useless constructs (SC2116: useless echo)
    /// Applied FIRST to simplify code before quoting
    RemoveUseless = 3,

    /// Quote command substitutions (SC2046)
    /// Applied SECOND after simplification
    QuoteCommandSub = 2,

    /// Quote variables (SC2086)
    /// Applied LAST (lowest priority)
    QuoteVariable = 1,
}

impl FixPriority {
    /// Get priority for a diagnostic rule code
    fn from_code(code: &str) -> Self {
        match code {
            "SC2116" => FixPriority::RemoveUseless,
            "SC2046" => FixPriority::QuoteCommandSub,
            "SC2086" => FixPriority::QuoteVariable,
            _ => FixPriority::QuoteVariable, // Default to lowest priority
        }
    }
}

/// Check if two spans overlap
fn spans_overlap(a: &Span, b: &Span) -> bool {
    if a.start_line != b.start_line {
        return false; // Different lines, no overlap
    }

    // Check if ranges overlap on same line
    // a.start_col..a.end_col overlaps with b.start_col..b.end_col
    !(a.end_col <= b.start_col || b.end_col <= a.start_col)
}

/// Options for controlling auto-fix application behavior.
///
/// Configure how fixes are applied to source code, including backup creation,
/// dry-run mode, and safety level filtering.
///
/// # Examples
///
/// ## Default options (safe fixes only, with backup)
///
/// ```
/// use bashrs::linter::autofix::FixOptions;
///
/// let options = FixOptions::default();
/// assert!(options.create_backup);
/// assert!(!options.dry_run);
/// assert!(!options.apply_assumptions); // Safe fixes only
/// assert_eq!(options.backup_suffix, ".bak");
/// ```
///
/// ## Dry-run mode (preview without modification)
///
/// ```
/// use bashrs::linter::autofix::FixOptions;
///
/// let options = FixOptions {
///     dry_run: true,
///     ..Default::default()
/// };
/// assert!(options.dry_run);
/// ```
///
/// ## Apply fixes with assumptions
///
/// ```
/// use bashrs::linter::autofix::FixOptions;
///
/// let options = FixOptions {
///     apply_assumptions: true, // Safe + SafeWithAssumptions
///     ..Default::default()
/// };
/// assert!(options.apply_assumptions);
/// ```
///
/// ## Custom backup suffix
///
/// ```
/// use bashrs::linter::autofix::FixOptions;
///
/// let options = FixOptions {
///     backup_suffix: ".backup".to_string(),
///     ..Default::default()
/// };
/// assert_eq!(options.backup_suffix, ".backup");
/// ```
#[derive(Debug, Clone)]
pub struct FixOptions {
    /// Create backup file before applying fixes (default: `true`).
    ///
    /// Ignored if `dry_run` is `true`.
    pub create_backup: bool,

    /// Dry-run mode - don't modify files, only count fixes (default: `false`).
    ///
    /// When `true`, `modified_source` in `FixResult` will be `None`.
    pub dry_run: bool,

    /// Backup file suffix (default: `".bak"`).
    ///
    /// Backup file will be named `<original><suffix>`.
    pub backup_suffix: String,

    /// Apply fixes with assumptions (default: `false`).
    ///
    /// - `false`: Only **Safe** fixes applied (default, equivalent to `--fix`)
    /// - `true`: **Safe** + **SafeWithAssumptions** fixes applied (equivalent to `--fix --fix-assumptions`)
    ///
    /// **Unsafe** fixes are NEVER auto-applied.
    pub apply_assumptions: bool,

    /// Optional output path (default: `None`).
    ///
    /// - `None`: Modify file in-place
    /// - `Some(path)`: Write modified content to `path`, leave original unchanged
    pub output_path: Option<std::path::PathBuf>,
}

impl Default for FixOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            dry_run: false,
            backup_suffix: ".bak".to_string(),
            apply_assumptions: false, // Default: SAFE fixes only
            output_path: None,
        }
    }
}

/// Result of applying auto-fixes to source code.
///
/// Contains information about what fixes were applied and where backups were created.
///
/// # Examples
///
/// ## Checking fix results
///
/// ```
/// use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
///
/// let source = "echo $VAR\n";
/// let mut result = LintResult::new();
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 6, 1, 10))
///         .with_fix(Fix::new("\"$VAR\""))
/// );
///
/// let options = autofix::FixOptions::default();
/// let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
///
/// assert_eq!(fix_result.fixes_applied, 1);
/// assert!(fix_result.modified_source.is_some());
/// assert_eq!(fix_result.modified_source.unwrap(), "echo \"$VAR\"\n");
/// ```
///
/// ## Dry-run result
///
/// ```
/// use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
///
/// let source = "ls $DIR\n";
/// let mut result = LintResult::new();
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 4, 1, 8))
///         .with_fix(Fix::new("\"$DIR\""))
/// );
///
/// let options = autofix::FixOptions {
///     dry_run: true,
///     ..Default::default()
/// };
///
/// let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
/// assert_eq!(fix_result.fixes_applied, 1);
/// assert!(fix_result.modified_source.is_none()); // No source in dry-run
/// ```
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Number of fixes successfully applied.
    ///
    /// This counts the number of diagnostics with fixes that were actually applied,
    /// excluding any skipped due to conflicts or safety level filtering.
    pub fixes_applied: usize,

    /// Modified source code (only if not dry-run).
    ///
    /// - `Some(source)`: Modified source code with fixes applied
    /// - `None`: Dry-run mode (no modification)
    pub modified_source: Option<String>,

    /// Path to backup file (only if backup was created).
    ///
    /// - `Some(path)`: Backup file created at `path`
    /// - `None`: No backup created (dry-run or `create_backup=false`)
    pub backup_path: Option<String>,
}

/// Applies fixes from a lint result to source code in memory.
///
/// Processes diagnostics with fixes and applies them to the source code string.
/// Fixes are filtered by safety level according to `options.apply_assumptions`.
///
/// # Arguments
///
/// * `source` - Original source code as a string
/// * `result` - Lint result containing diagnostics with suggested fixes
/// * `options` - Configuration for fix application (dry-run, safety levels, etc.)
///
/// # Returns
///
/// * `Ok(FixResult)` - Fix results including count and modified source
/// * `Err(io::Error)` - If fix application fails
///
/// # Conflict Resolution
///
/// When multiple fixes overlap on the same span, they are applied in priority order:
/// 1. **SC2116** (remove useless constructs) - Highest priority
/// 2. **SC2046** (quote command substitutions)
/// 3. **SC2086** (quote variables) - Lowest priority
///
/// This ensures correct transformation: `$(echo $VAR)` → `$VAR` → `"$VAR"`
///
/// # Safety Level Filtering
///
/// * `apply_assumptions = false`: Only **Safe** fixes applied
/// * `apply_assumptions = true`: **Safe** + **SafeWithAssumptions** fixes applied
/// * **Unsafe** fixes are NEVER auto-applied
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
///
/// let source = "echo $VAR\n";
/// let mut result = LintResult::new();
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote variable", Span::new(1, 6, 1, 10))
///         .with_fix(Fix::new("\"$VAR\""))
/// );
///
/// let options = autofix::FixOptions::default();
/// let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
///
/// assert_eq!(fix_result.fixes_applied, 1);
/// assert_eq!(fix_result.modified_source.unwrap(), "echo \"$VAR\"\n");
/// ```
///
/// ## Multiple fixes
///
/// ```
/// use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
///
/// let source = "cp $FILE1 $FILE2\n";
/// let mut result = LintResult::new();
///
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 4, 1, 10))
///         .with_fix(Fix::new("\"$FILE1\""))
/// );
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 11, 1, 17))
///         .with_fix(Fix::new("\"$FILE2\""))
/// );
///
/// let options = autofix::FixOptions::default();
/// let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
///
/// assert_eq!(fix_result.fixes_applied, 2);
/// assert_eq!(fix_result.modified_source.unwrap(), "cp \"$FILE1\" \"$FILE2\"\n");
/// ```
///
/// ## Dry-run mode
///
/// ```
/// use bashrs::linter::{autofix, Diagnostic, Fix, LintResult, Severity, Span};
///
/// let source = "ls $DIR\n";
/// let mut result = LintResult::new();
/// result.add(
///     Diagnostic::new("SC2086", Severity::Warning, "Quote", Span::new(1, 4, 1, 8))
///         .with_fix(Fix::new("\"$DIR\""))
/// );
///
/// let options = autofix::FixOptions {
///     dry_run: true,
///     ..Default::default()
/// };
///
/// let fix_result = autofix::apply_fixes(source, &result, &options).unwrap();
/// assert_eq!(fix_result.fixes_applied, 1);
/// assert!(fix_result.modified_source.is_none()); // No source in dry-run
/// ```
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

#[cfg(test)]
#[path = "autofix_tests_extracted.rs"]
mod tests_extracted;
