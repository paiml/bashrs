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

    // ============================================================================
    // Mutation Testing Coverage Tests
    // ============================================================================
    // These tests target specific mutants that survived initial testing
    // to achieve ≥90% mutation score

    #[test]
    fn test_backup_created_only_when_both_flags_true() {
        // MUTANT: Line 208 - replace && with || in apply_fixes_to_file
        // This test ensures backup is created ONLY when create_backup=true AND dry_run=false
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "echo $VAR").unwrap();
        let temp_path = temp_file.path();

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

        // Test 1: create_backup=true AND dry_run=false → backup SHOULD be created
        let options = FixOptions {
            create_backup: true,
            dry_run: false,
            backup_suffix: ".bak".to_string(),
            apply_assumptions: false,
            output_path: None,
        };

        let fix_result = apply_fixes_to_file(temp_path, &result, &options).unwrap();
        assert!(
            fix_result.backup_path.is_some(),
            "Backup should be created when create_backup=true AND dry_run=false"
        );

        // Cleanup backup
        if let Some(backup) = fix_result.backup_path {
            let _ = std::fs::remove_file(backup);
        }

        // Test 2: create_backup=false OR dry_run=true → backup should NOT be created
        let options_no_backup = FixOptions {
            create_backup: false, // FALSE
            dry_run: false,
            backup_suffix: ".bak".to_string(),
            apply_assumptions: false,
            output_path: None,
        };

        // Need to recreate temp file since it was modified
        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file2, "echo $VAR").unwrap();
        let temp_path2 = temp_file2.path();

        let fix_result2 = apply_fixes_to_file(temp_path2, &result, &options_no_backup).unwrap();
        assert!(
            fix_result2.backup_path.is_none(),
            "Backup should NOT be created when create_backup=false"
        );

        // Test 3: create_backup=true BUT dry_run=true → backup should NOT be created
        let options_dry_run = FixOptions {
            create_backup: true,
            dry_run: true, // TRUE
            backup_suffix: ".bak".to_string(),
            apply_assumptions: false,
            output_path: None,
        };

        let mut temp_file3 = NamedTempFile::new().unwrap();
        writeln!(temp_file3, "echo $VAR").unwrap();
        let temp_path3 = temp_file3.path();

        let fix_result3 = apply_fixes_to_file(temp_path3, &result, &options_dry_run).unwrap();
        assert!(
            fix_result3.backup_path.is_none(),
            "Backup should NOT be created when dry_run=true"
        );
    }

    #[test]
    fn test_fix_priority_sc2046_coverage() {
        // MUTANT: Line 37 - delete match arm "SC2046" in FixPriority::from_code
        // This test ensures SC2046 has correct priority assignment
        let source = "cp $(cat file.txt) /dest\n";

        let mut result = LintResult::new();
        result.add(
            Diagnostic::new(
                "SC2046",
                Severity::Warning,
                "Unquoted command substitution".to_string(),
                Span::new(1, 4, 1, 22),
            )
            .with_fix(Fix::new("\"$(cat file.txt)\"".to_string())),
        );

        let options = FixOptions::default();
        let fix_result = apply_fixes(source, &result, &options).unwrap();

        // Verify SC2046 fix is applied
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.modified_source.is_some());
        assert!(fix_result
            .modified_source
            .unwrap()
            .contains("\"$(cat file.txt)\""));
    }

    #[test]
    fn test_span_boundary_conditions() {
        // MUTANTS: Lines 253, 260 - various operators in apply_single_fix
        // Test boundary conditions for span calculations

        // Test 1: Fix at start of line (col 1)
        let source = "$VAR rest\n";
        let span = Span::new(1, 1, 1, 5); // Entire $VAR
        let result = apply_single_fix(source, &span, "\"$VAR\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"$VAR\" rest\n");

        // Test 2: Fix at end of line
        let source = "start $VAR\n";
        let span = Span::new(1, 7, 1, 11); // $VAR at end
        let result = apply_single_fix(source, &span, "\"$VAR\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "start \"$VAR\"\n");

        // Test 3: Multi-line source with fix on second line
        let source = "line1\necho $VAR\nline3\n";
        let span = Span::new(2, 6, 2, 10); // $VAR on line 2
        let result = apply_single_fix(source, &span, "\"$VAR\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "line1\necho \"$VAR\"\nline3\n");
    }

    #[test]
    fn test_logical_operators_in_conditions() {
        // MUTANT: Line 281 - replace && with || in apply_single_fix
        // This tests the condition: start_idx <= source.len() && end_idx <= source.len()

        // Both conditions must be true for fix to succeed
        let source = "echo test\n";
        let valid_span = Span::new(1, 6, 1, 10); // "test"
        let result = apply_single_fix(source, &valid_span, "replacement");
        assert!(result.is_ok(), "Both conditions true should succeed");

        // If EITHER condition is false, fix should fail
        let invalid_span = Span::new(1, 6, 1, 999); // end_idx > source.len()
        let result = apply_single_fix(source, &invalid_span, "replacement");
        assert!(result.is_err(), "Invalid end_idx should fail");

        let invalid_span2 = Span::new(999, 1, 999, 5); // start line invalid
        let result = apply_single_fix(source, &invalid_span2, "replacement");
        assert!(result.is_err(), "Invalid start line should fail");
    }
}
