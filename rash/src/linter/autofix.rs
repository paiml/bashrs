//! Auto-fix application for linter diagnostics
//!
//! Applies suggested fixes to source code with:
//! - Backup creation before modification
//! - Span-based replacement
//! - Dry-run mode for preview
//! - Safe application (reverse order to preserve positions)
//! - Priority-based conflict resolution

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
///
/// # Conflict Resolution
/// When multiple fixes overlap on the same span, they are applied in priority order:
/// 1. SC2116 (remove useless constructs) - Highest priority
/// 2. SC2046 (quote command substitutions)
/// 3. SC2086 (quote variables) - Lowest priority
///
/// This ensures correct transformation: `$(echo $VAR)` → `$VAR` → `"$VAR"`
pub fn apply_fixes(
    source: &str,
    result: &LintResult,
    options: &FixOptions,
) -> io::Result<FixResult> {
    let mut modified = source.to_string();
    let mut fixes_applied = 0;

    // Get diagnostics with fixes
    let mut diagnostics_with_fixes: Vec<&Diagnostic> = result
        .diagnostics
        .iter()
        .filter(|d| d.fix.is_some())
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

    #[test]
    fn test_conflicting_fixes_priority() {
        // Test the edge case: $(echo $VAR)
        // SC2116 wants to remove useless echo: $VAR
        // SC2046 wants to quote command sub: "$(echo $VAR)"
        // SC2086 wants to quote variable: "$(echo "$VAR")"
        //
        // Priority order should apply SC2116 first (highest priority)
        // Then the result won't have the command sub anymore, so SC2046/SC2086 become moot
        let source = "RELEASE=$(echo $TIMESTAMP)\n";

        let mut result = LintResult::new();

        // Add SC2116 diagnostic (remove useless echo) - Priority 3
        result.add(
            Diagnostic::new(
                "SC2116",
                Severity::Warning,
                "Useless echo".to_string(),
                Span::new(1, 9, 1, 27), // $(echo $TIMESTAMP)
            )
            .with_fix(Fix::new("$TIMESTAMP".to_string())),
        );

        // Add SC2046 diagnostic (quote command sub) - Priority 2
        result.add(
            Diagnostic::new(
                "SC2046",
                Severity::Warning,
                "Unquoted command substitution".to_string(),
                Span::new(1, 9, 1, 27), // $(echo $TIMESTAMP) - OVERLAPS
            )
            .with_fix(Fix::new("\"$(echo $TIMESTAMP)\"".to_string())),
        );

        let options = FixOptions::default();
        let fix_result = apply_fixes(source, &result, &options).unwrap();

        // Should apply SC2116 (highest priority) and skip SC2046 (conflict)
        assert_eq!(fix_result.fixes_applied, 1);
        assert_eq!(fix_result.modified_source.unwrap(), "RELEASE=$TIMESTAMP\n");
    }

    #[test]
    fn test_non_overlapping_fixes() {
        // Test that non-overlapping fixes all get applied
        let source = "cp $FILE1 $FILE2\n";

        let mut result = LintResult::new();

        // Two non-overlapping SC2086 diagnostics
        result.add(
            Diagnostic::new(
                "SC2086",
                Severity::Warning,
                "Unquoted $FILE1".to_string(),
                Span::new(1, 4, 1, 10),
            )
            .with_fix(Fix::new("\"$FILE1\"".to_string())),
        );

        result.add(
            Diagnostic::new(
                "SC2086",
                Severity::Warning,
                "Unquoted $FILE2".to_string(),
                Span::new(1, 11, 1, 17),
            )
            .with_fix(Fix::new("\"$FILE2\"".to_string())),
        );

        let options = FixOptions::default();
        let fix_result = apply_fixes(source, &result, &options).unwrap();

        // Both should be applied (no overlap)
        assert_eq!(fix_result.fixes_applied, 2);
        assert_eq!(
            fix_result.modified_source.unwrap(),
            "cp \"$FILE1\" \"$FILE2\"\n"
        );
    }

    #[test]
    fn test_overlap_detection() {
        // Test spans_overlap function
        let span_a = Span::new(1, 5, 1, 10);
        let span_b = Span::new(1, 8, 1, 12); // Overlaps with A

        assert!(spans_overlap(&span_a, &span_b));
        assert!(spans_overlap(&span_b, &span_a)); // Symmetric

        let span_c = Span::new(1, 11, 1, 15); // No overlap with A
        assert!(!spans_overlap(&span_a, &span_c));

        let span_d = Span::new(2, 5, 2, 10); // Different line
        assert!(!spans_overlap(&span_a, &span_d));
    }
}
