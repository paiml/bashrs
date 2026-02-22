use super::*;

// ===== NASA-QUALITY UNIT TESTS for config_purify_command helpers =====
// Following the pattern established in bash_quality::coverage::tests

#[test]
fn test_should_output_to_stdout_dash() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let stdout_path = Path::new("-");
    assert!(
        should_output_to_stdout(stdout_path),
        "Path '-' should output to stdout"
    );
}

#[test]
fn test_should_output_to_stdout_regular_file() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let file_path = Path::new("/tmp/output.txt");
    assert!(
        !should_output_to_stdout(file_path),
        "Regular file path should NOT output to stdout"
    );
}

#[test]
fn test_should_output_to_stdout_empty_path() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let empty_path = Path::new("");
    assert!(
        !should_output_to_stdout(empty_path),
        "Empty path should NOT output to stdout"
    );
}

#[test]
fn test_generate_diff_lines_no_changes() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3";
    let purified = "line1\nline2\nline3";

    let diffs = generate_diff_lines(original, purified);

    assert!(
        diffs.is_empty(),
        "Identical content should produce no diff lines"
    );
}

#[test]
fn test_generate_diff_lines_single_change() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3";
    let purified = "line1\nMODIFIED\nline3";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 1, "Should have exactly 1 diff");
    let (line_num, orig, pure) = &diffs[0];
    assert_eq!(*line_num, 2, "Diff should be on line 2");
    assert_eq!(orig, "line2", "Original line should be 'line2'");
    assert_eq!(pure, "MODIFIED", "Purified line should be 'MODIFIED'");
}

#[test]
fn test_generate_diff_lines_multiple_changes() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3\nline4";
    let purified = "CHANGED1\nline2\nCHANGED3\nline4";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 2, "Should have exactly 2 diffs");

    let (line_num1, orig1, pure1) = &diffs[0];
    assert_eq!(*line_num1, 1, "First diff on line 1");
    assert_eq!(orig1, "line1");
    assert_eq!(pure1, "CHANGED1");

    let (line_num2, orig2, pure2) = &diffs[1];
    assert_eq!(*line_num2, 3, "Second diff on line 3");
    assert_eq!(orig2, "line3");
    assert_eq!(pure2, "CHANGED3");
}

#[test]
fn test_generate_diff_lines_empty_strings() {
    use super::generate_diff_lines;

    let original = "";
    let purified = "";

    let diffs = generate_diff_lines(original, purified);

    assert!(diffs.is_empty(), "Empty strings should produce no diffs");
}

#[test]
fn test_generate_diff_lines_all_lines_changed() {
    use super::generate_diff_lines;

    let original = "A\nB\nC";
    let purified = "X\nY\nZ";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 3, "All 3 lines should be different");
    assert_eq!(diffs[0].0, 1);
    assert_eq!(diffs[1].0, 2);
    assert_eq!(diffs[2].0, 3);
}

#[test]
fn test_generate_diff_lines_preserves_whitespace() {
    use super::generate_diff_lines;

    let original = "  line1  \nline2";
    let purified = "line1\nline2";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 1, "Should detect whitespace change");
    let (_, orig, pure) = &diffs[0];
    assert_eq!(orig, "  line1  ", "Should preserve original whitespace");
    assert_eq!(pure, "line1", "Should preserve purified whitespace");
}

// =============================================================================
// explain-error command tests (v6.40.0 - Oracle integration)
// =============================================================================

#[cfg(feature = "oracle")]
mod explain_error_tests {
    use super::super::extract_exit_code;

    #[test]
    fn test_extract_exit_code_explicit_patterns() {
        // "exit code X" pattern
        assert_eq!(extract_exit_code("Process exited with exit code 127"), 127);
        assert_eq!(extract_exit_code("Error: exit code 1"), 1);

        // "exited with X" pattern
        assert_eq!(extract_exit_code("Command exited with 126"), 126);

        // "returned X" pattern
        assert_eq!(extract_exit_code("Script returned 2"), 2);

        // "status X" pattern
        assert_eq!(extract_exit_code("Exit status 128"), 128);
    }

    #[test]
    fn test_extract_exit_code_wellknown_messages() {
        // Command not found -> 127
        assert_eq!(extract_exit_code("bash: foo: command not found"), 127);

        // Permission denied -> 126
        assert_eq!(extract_exit_code("/bin/script.sh: Permission denied"), 126);
        assert_eq!(
            extract_exit_code("Error: permission denied for file.txt"),
            126
        );
    }

    #[test]
    fn test_extract_exit_code_default() {
        // Unknown error -> 1 (default)
        assert_eq!(extract_exit_code("Some random error message"), 1);
        assert_eq!(extract_exit_code(""), 1);
    }

    #[test]
    fn test_extract_exit_code_case_insensitive() {
        // Should match case-insensitively
        assert_eq!(extract_exit_code("EXIT CODE 42"), 42);
        assert_eq!(extract_exit_code("Exit Code 5"), 5);
    }
}

// =============================================================================
// --ignore and -e flag tests (Issue #82)
// =============================================================================

mod ignore_flag_tests {
    use std::collections::HashSet;

    /// Helper to build ignored rules set (mirrors lint_command logic)
    fn build_ignored_rules(
        ignore_rules: Option<&str>,
        exclude_rules: Option<&[String]>,
    ) -> HashSet<String> {
        let mut rules = HashSet::new();
        if let Some(ignore_str) = ignore_rules {
            for code in ignore_str.split(',') {
                let code = code.trim().to_uppercase();
                if !code.is_empty() {
                    rules.insert(code);
                }
            }
        }
        if let Some(excludes) = exclude_rules {
            for code in excludes {
                let code = code.trim().to_uppercase();
                if !code.is_empty() {
                    rules.insert(code);
                }
            }
        }
        rules
    }

    #[test]
    fn test_ignore_flag_single_rule() {
        let ignored = build_ignored_rules(Some("SEC010"), None);
        assert!(ignored.contains("SEC010"));
        assert_eq!(ignored.len(), 1);
    }

    #[test]
    fn test_ignore_flag_multiple_rules() {
        let ignored = build_ignored_rules(Some("SEC010,DET002,SC2086"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert!(ignored.contains("SC2086"));
        assert_eq!(ignored.len(), 3);
    }

    #[test]
    fn test_ignore_flag_case_insensitive() {
        let ignored = build_ignored_rules(Some("sec010,Det002"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_ignore_flag_with_whitespace() {
        let ignored = build_ignored_rules(Some(" SEC010 , DET002 "), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_exclude_flag_single() {
        let excludes = vec!["SEC010".to_string()];
        let ignored = build_ignored_rules(None, Some(&excludes));
        assert!(ignored.contains("SEC010"));
    }

    #[test]
    fn test_exclude_flag_multiple() {
        let excludes = vec!["SEC010".to_string(), "DET002".to_string()];
        let ignored = build_ignored_rules(None, Some(&excludes));
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_combined_ignore_and_exclude() {
        let excludes = vec!["SEC008".to_string()];
        let ignored = build_ignored_rules(Some("SEC010,DET002"), Some(&excludes));
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert!(ignored.contains("SEC008"));
        assert_eq!(ignored.len(), 3);
    }

    #[test]
    fn test_empty_ignore() {
        let ignored = build_ignored_rules(None, None);
        assert!(ignored.is_empty());
    }

    #[test]
    fn test_ignore_flag_empty_entries() {
        let ignored = build_ignored_rules(Some("SEC010,,DET002,"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert_eq!(ignored.len(), 2);
    }
}

// ============================================================================
// Helper Function Tests - Boost coverage for small utility functions
// ============================================================================

#[test]
fn test_hex_encode_empty() {
    assert_eq!(hex_encode(&[]), "");
}

#[test]
fn test_hex_encode_single_byte() {
    assert_eq!(hex_encode(&[0x00]), "00");
    assert_eq!(hex_encode(&[0xff]), "ff");
    assert_eq!(hex_encode(&[0x42]), "42");
}

#[test]
fn test_hex_encode_multiple_bytes() {
    assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    assert_eq!(hex_encode(&[0x01, 0x23, 0x45, 0x67]), "01234567");
}

#[test]
fn test_truncate_str_short() {
    assert_eq!(truncate_str("hello", 10), "hello");
    assert_eq!(truncate_str("hi", 5), "hi");
}

#[test]
fn test_truncate_str_exact() {
    assert_eq!(truncate_str("hello", 5), "hello");
}

#[test]
fn test_truncate_str_long() {
    assert_eq!(truncate_str("hello world", 8), "hello...");
    assert_eq!(truncate_str("abcdefghij", 6), "abc...");
}

#[test]
fn test_truncate_str_edge_cases() {
    assert_eq!(truncate_str("abc", 3), "abc");
    assert_eq!(truncate_str("abcd", 3), "...");
    assert_eq!(truncate_str("", 5), "");
}

#[test]
fn test_should_output_to_stdout() {
    use std::path::Path;
    assert!(should_output_to_stdout(Path::new("-")));
    assert!(!should_output_to_stdout(Path::new("output.sh")));
    assert!(!should_output_to_stdout(Path::new("/tmp/file.txt")));
    assert!(!should_output_to_stdout(Path::new("--")));
}

#[test]
fn test_format_timestamp_just_now() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Test a timestamp from a few seconds ago
    let result = format_timestamp(now - 30);
    assert_eq!(result, "just now");
}

#[test]
fn test_format_timestamp_minutes_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 120); // 2 minutes ago
    assert_eq!(result, "2m ago");
}

#[test]
fn test_format_timestamp_hours_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 7200); // 2 hours ago
    assert_eq!(result, "2h ago");
}

#[test]
fn test_format_timestamp_days_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 172800); // 2 days ago
    assert_eq!(result, "2d ago");
}

#[cfg(feature = "oracle")]
#[test]
fn test_extract_exit_code_patterns() {
    assert_eq!(extract_exit_code("exit code 127"), 127);
    assert_eq!(extract_exit_code("exited with 1"), 1);
    assert_eq!(extract_exit_code("returned 255"), 255);
    assert_eq!(extract_exit_code("status 42"), 42);
}

#[cfg(feature = "oracle")]
#[test]
fn test_extract_exit_code_special_cases() {
    assert_eq!(extract_exit_code("command not found"), 127);
    assert_eq!(extract_exit_code("Permission denied"), 126);
    assert_eq!(extract_exit_code("permission denied"), 126);
    assert_eq!(extract_exit_code("unknown error"), 1);
}

// ============================================================================
// Config Analysis Helper Tests
// ============================================================================

#[test]
fn test_count_duplicate_path_entries_empty() {
    let analysis = crate::config::ConfigAnalysis {
        file_path: PathBuf::from("/tmp/test"),
        config_type: crate::config::ConfigType::Bashrc,
        line_count: 0,
        complexity_score: 0,
        issues: vec![],
        path_entries: vec![],
        performance_issues: vec![],
    };
    assert_eq!(count_duplicate_path_entries(&analysis), 0);
}

#[test]
fn test_count_duplicate_path_entries_with_duplicates() {
    let analysis = crate::config::ConfigAnalysis {
        file_path: PathBuf::from("/tmp/test"),
        config_type: crate::config::ConfigType::Bashrc,
        line_count: 3,
        complexity_score: 1,
        issues: vec![],
        path_entries: vec![
            crate::config::PathEntry {
                line: 1,
                path: "/usr/bin".to_string(),
                is_duplicate: false,
            },
            crate::config::PathEntry {
                line: 2,
                path: "/usr/bin".to_string(),
                is_duplicate: true,
            },
            crate::config::PathEntry {
                line: 3,
                path: "/usr/local/bin".to_string(),
                is_duplicate: false,
            },
        ],
        performance_issues: vec![],
    };
    assert_eq!(count_duplicate_path_entries(&analysis), 1);
}

// ============================================================================
// Handle Output Tests
// ============================================================================

#[test]
fn test_handle_output_to_file_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.txt");

    let result = handle_output_to_file(&output_path, "test content");
    assert!(result.is_ok());
    assert!(output_path.exists());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "test content");
}

// ============================================================================
// Parse Public Key Test
// ============================================================================

#[test]
fn test_parse_public_key_valid() {
    // 32 bytes = 64 hex chars
    let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = parse_public_key(hex);
    assert!(result.is_ok());
}

#[test]
fn test_parse_public_key_invalid_length() {
    let result = parse_public_key("0123456789abcdef");
    assert!(result.is_err());
}

#[test]
fn test_parse_public_key_invalid_hex() {
    let result =
        parse_public_key("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    assert!(result.is_err());
}
