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

include!("autofix_incl2.rs");
