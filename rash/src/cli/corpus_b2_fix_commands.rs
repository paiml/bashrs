//! Corpus B2 fix application: registry scanning, string replacement, and Rust source formatting.

use crate::models::{Error, Result};

pub(crate) fn corpus_apply_b2_fixes(fixes: &[(String, String, String)]) -> Result<()> {
    corpus_apply_b2_fixes_with(std::path::Path::new("rash/src/corpus/registry.rs"), fixes)
}

/// PMAT-257: body of `corpus_apply_b2_fixes`, split so a test can point it at
/// a temporary copy of the registry instead of the real
/// `rash/src/corpus/registry.rs`.
pub(crate) fn corpus_apply_b2_fixes_with(
    registry_path: &std::path::Path,
    fixes: &[(String, String, String)],
) -> Result<()> {
    if !registry_path.exists() {
        return Err(Error::Validation(
            "registry.rs not found (run from project root)".to_string(),
        ));
    }

    let mut content = std::fs::read_to_string(registry_path)
        .map_err(|e| Error::Internal(format!("read registry.rs: {e}")))?;

    let mut applied = 0usize;
    let mut skipped = 0usize;

    // Collect edits as (position, old_len, new_string) and apply in reverse order
    let mut edits: Vec<(usize, usize, String)> = Vec::new();

    for (id, _old_expected, new_expected) in fixes {
        let id_pattern = format!("\"{}\"", id);
        let id_pos = match content.find(&id_pattern) {
            Some(p) => p,
            None => {
                skipped += 1;
                continue;
            }
        };

        match find_last_string_in_entry(&content, id_pos) {
            Some((start, end)) => {
                edits.push((
                    start,
                    end - start,
                    format_rust_string_for_registry(new_expected),
                ));
            }
            None => {
                skipped += 1;
            }
        }
    }

    // Sort edits by position descending to avoid offset shifts
    edits.sort_by(|a, b| b.0.cmp(&a.0));

    for (pos, old_len, new_str) in &edits {
        content.replace_range(*pos..*pos + *old_len, new_str);
        applied += 1;
    }

    std::fs::write(registry_path, content)
        .map_err(|e| Error::Internal(format!("write registry.rs: {e}")))?;

    eprintln!("Applied: {applied}, Skipped: {skipped}");
    Ok(())
}

/// Find the last string literal in a CorpusEntry::new(...) call starting near id_pos.
/// Returns (start_byte, end_byte) of the string literal including delimiters.
pub(crate) fn find_last_string_in_entry(content: &str, id_pos: usize) -> Option<(usize, usize)> {
    let pre_start = id_pos.saturating_sub(200);
    let pre_region = &content[pre_start..id_pos];
    let new_call_rel = pre_region.rfind("CorpusEntry::new(")?;
    let region_start = pre_start + new_call_rel;
    let region_end = std::cmp::min(region_start + 3000, content.len());
    let region = &content[region_start..region_end];
    let paren_start = region.find('(')?;

    let (s, e) = scan_last_string_before_close_paren(region.as_bytes(), paren_start)?;
    Some((region_start + s, region_start + e))
}

/// Scan bytes from `start` to find the last string literal before the balanced `)`.
/// Returns (start, end) offsets within the byte slice, including string delimiters.
/// State for scanning Rust string literals in source code.
#[derive(PartialEq)]

pub(crate) enum RustScanState {
    Normal,
    InStr,
    InRaw,
}

/// Advance result for the Rust source scanner.
pub(crate) enum ScanAdvance {
    Step1,
    Skip(usize),
    Done,
}

pub(crate) fn scan_last_string_before_close_paren(
    bytes: &[u8],
    start: usize,
) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    let mut i = start;
    let mut state = RustScanState::Normal;
    let mut str_start = 0usize;
    let mut last_str: Option<(usize, usize)> = None;

    while i < bytes.len() {
        match advance_scan(bytes, i, &state, &mut depth, &mut str_start, &mut last_str) {
            ScanAdvance::Done => {
                return last_str;
            }
            ScanAdvance::Skip(n) => {
                state = next_scan_state(&state, bytes, i);
                i += n;
            }
            ScanAdvance::Step1 => {
                state = next_scan_state(&state, bytes, i);
                i += 1;
            }
        }
    }
    None
}

pub(crate) fn advance_scan(
    bytes: &[u8],
    i: usize,
    state: &RustScanState,
    depth: &mut i32,
    str_start: &mut usize,
    last_str: &mut Option<(usize, usize)>,
) -> ScanAdvance {
    match state {
        RustScanState::InRaw => advance_in_raw(bytes, i, *str_start, last_str),
        RustScanState::InStr => advance_in_str(bytes, i, *str_start, last_str),
        RustScanState::Normal => advance_normal(bytes, i, depth, str_start),
    }
}

pub(crate) fn advance_in_raw(
    bytes: &[u8],
    i: usize,
    str_start: usize,
    last_str: &mut Option<(usize, usize)>,
) -> ScanAdvance {
    if i + 1 < bytes.len() && bytes[i] == b'"' && bytes[i + 1] == b'#' {
        *last_str = Some((str_start, i + 2));
        ScanAdvance::Skip(2)
    } else {
        ScanAdvance::Step1
    }
}

pub(crate) fn advance_in_str(
    bytes: &[u8],
    i: usize,
    str_start: usize,
    last_str: &mut Option<(usize, usize)>,
) -> ScanAdvance {
    if bytes[i] == b'\\' {
        return ScanAdvance::Skip(2);
    }
    if bytes[i] == b'"' {
        *last_str = Some((str_start, i + 1));
    }
    ScanAdvance::Step1
}

pub(crate) fn advance_normal(
    bytes: &[u8],
    i: usize,
    depth: &mut i32,
    str_start: &mut usize,
) -> ScanAdvance {
    if i + 2 < bytes.len() && bytes[i] == b'r' && bytes[i + 1] == b'#' && bytes[i + 2] == b'"' {
        *str_start = i;
        return ScanAdvance::Skip(3);
    }
    match bytes[i] {
        b'"' => {
            *str_start = i;
        }
        b'(' => {
            *depth += 1;
        }
        b')' => {
            *depth -= 1;
            if *depth == 0 {
                return ScanAdvance::Done;
            }
        }
        _ => {}
    }
    ScanAdvance::Step1
}

pub(crate) fn next_scan_state(current: &RustScanState, bytes: &[u8], i: usize) -> RustScanState {
    match current {
        RustScanState::InRaw => {
            if i + 1 < bytes.len() && bytes[i] == b'"' && bytes[i + 1] == b'#' {
                RustScanState::Normal
            } else {
                RustScanState::InRaw
            }
        }
        RustScanState::InStr => {
            if bytes[i] == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                RustScanState::Normal
            } else {
                RustScanState::InStr
            }
        }
        RustScanState::Normal => {
            if i + 2 < bytes.len()
                && bytes[i] == b'r'
                && bytes[i + 1] == b'#'
                && bytes[i + 2] == b'"'
            {
                RustScanState::InRaw
            } else if bytes[i] == b'"' {
                RustScanState::InStr
            } else {
                RustScanState::Normal
            }
        }
    }
}

/// Format a string as a Rust string literal for registry.rs.
/// Uses raw string r#"..."# if the value contains quotes or backslashes,
/// otherwise uses regular "..." with escaping.
pub(crate) fn format_rust_string_for_registry(s: &str) -> String {
    if s.contains('"') || s.contains('\\') {
        // Use raw string — but check it doesn't contain "# which would break r#"..."#
        if s.contains("\"#") {
            // Fall back to regular string with escaping
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        } else {
            format!("r#\"{}\"#", s)
        }
    } else {
        format!("\"{}\"", s)
    }
}

#[cfg(test)]
mod config_purify_tests {
    use crate::cli::commands::should_output_to_stdout;
    use crate::cli::logic::generate_diff_lines;
    use std::path::Path;

    // ===== NASA-QUALITY UNIT TESTS for config_purify_command helpers =====

    #[test]
    fn test_should_output_to_stdout_dash() {
        let stdout_path = Path::new("-");
        assert!(
            should_output_to_stdout(stdout_path),
            "Path '-' should output to stdout"
        );
    }

    #[test]
    fn test_should_output_to_stdout_regular_file() {
        let file_path = Path::new("/tmp/output.txt");
        assert!(
            !should_output_to_stdout(file_path),
            "Regular file path should NOT output to stdout"
        );
    }

    #[test]
    fn test_should_output_to_stdout_empty_path() {
        let empty_path = Path::new("");
        assert!(
            !should_output_to_stdout(empty_path),
            "Empty path should NOT output to stdout"
        );
    }

    #[test]
    fn test_generate_diff_lines_no_changes() {
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
        let original = "";
        let purified = "";

        let diffs = generate_diff_lines(original, purified);

        assert!(diffs.is_empty(), "Empty strings should produce no diffs");
    }

    #[test]
    fn test_generate_diff_lines_all_lines_changed() {
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
        let original = "  line1  \nline2";
        let purified = "line1\nline2";

        let diffs = generate_diff_lines(original, purified);

        assert_eq!(diffs.len(), 1, "Should detect whitespace change");
        let (_, orig, pure) = &diffs[0];
        assert_eq!(orig, "  line1  ", "Should preserve original whitespace");
        assert_eq!(pure, "line1", "Should preserve purified whitespace");
    }
}

// PMAT-257: coverage for the B2-fix registry-rewriting helpers. These are
// exercised against a small in-tempdir stand-in for registry.rs — never the
// real `rash/src/corpus/registry.rs` — and against the byte-level Rust source
// scanner directly, so no `CorpusRegistry::load_full()` or `CorpusRunner` is
// ever reached from this module.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;

    fn sample_entry(id: &str, expected: &str) -> String {
        format!(
            "CorpusEntry::new(\n    \"{id}\",\n    \"a-name\",\n    \"a description\",\n    CorpusFormat::Bash,\n    CorpusTier::Trivial,\n    \"fn main() {{ let greeting = \\\"hello\\\"; }}\",\n    \"{expected}\",\n)"
        )
    }

    #[test]
    fn test_PMAT257_cov_apply_b2_fixes_with_replaces_the_expected_field() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry_path = dir.path().join("registry.rs");
        let content = format!(
            "// fixture registry\n{}\n{}\n",
            sample_entry("B-001", "old_expected"),
            sample_entry("B-002", "other_expected"),
        );
        std::fs::write(&registry_path, &content).expect("write fixture");

        corpus_apply_b2_fixes_with(
            &registry_path,
            &[(
                "B-001".to_string(),
                "old_expected".to_string(),
                "new_expected".to_string(),
            )],
        )
        .expect("applying a fix against a tempdir registry must succeed");

        let updated = std::fs::read_to_string(&registry_path).expect("read back");
        assert!(
            updated.contains("new_expected"),
            "the new expected value must be written: {updated}"
        );
        assert!(
            !updated.contains("old_expected"),
            "the old expected value must be gone: {updated}"
        );
        assert!(
            updated.contains("other_expected"),
            "the untouched entry must be left alone: {updated}"
        );
    }

    #[test]
    fn test_PMAT257_cov_apply_b2_fixes_with_missing_registry_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("does-not-exist.rs");
        let err = corpus_apply_b2_fixes_with(
            &missing,
            &[("B-001".to_string(), "old".to_string(), "new".to_string())],
        )
        .expect_err("a missing registry file must be a validation error");
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn test_PMAT257_cov_apply_b2_fixes_with_skips_an_unknown_id() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry_path = dir.path().join("registry.rs");
        let content = sample_entry("B-001", "old_expected");
        std::fs::write(&registry_path, &content).expect("write fixture");

        corpus_apply_b2_fixes_with(
            &registry_path,
            &[(
                "B-999".to_string(),
                "old_expected".to_string(),
                "new_expected".to_string(),
            )],
        )
        .expect("an id absent from the registry is skipped, not an error");

        let unchanged = std::fs::read_to_string(&registry_path).expect("read back");
        assert_eq!(
            unchanged, content,
            "no matching id means the file is rewritten unchanged"
        );
    }

    #[test]
    fn test_PMAT257_cov_find_last_string_in_entry_finds_the_final_field() {
        let content = sample_entry("B-001", "old_expected");
        let id_pos = content.find("\"B-001\"").expect("id present");
        let (start, end) =
            find_last_string_in_entry(&content, id_pos).expect("a closed call has a last string");
        assert_eq!(&content[start..end], "\"old_expected\"");
    }

    #[test]
    fn test_PMAT257_cov_find_last_string_in_entry_none_without_a_call() {
        let content = "no corpus entry call here at all";
        assert!(find_last_string_in_entry(content, 5).is_none());
    }

    #[test]
    fn test_PMAT257_cov_scan_last_string_before_close_paren_handles_raw_strings() {
        let region = "(\"id\", r#\"has \"quotes\" inside\"#)";
        let paren_start = region.find('(').expect("open paren");
        let (start, end) = scan_last_string_before_close_paren(region.as_bytes(), paren_start)
            .expect("raw string is the last literal before the close paren");
        assert_eq!(&region[start..end], "r#\"has \"quotes\" inside\"#");
    }

    #[test]
    fn test_PMAT257_cov_scan_last_string_before_close_paren_handles_escaped_quotes() {
        let region = "(\"first\", \"it's a \\\"quoted\\\" word\")";
        let paren_start = region.find('(').expect("open paren");
        let (start, end) = scan_last_string_before_close_paren(region.as_bytes(), paren_start)
            .expect("escaped-quote string is the last literal before the close paren");
        assert_eq!(&region[start..end], "\"it's a \\\"quoted\\\" word\"");
    }

    #[test]
    fn test_PMAT257_cov_scan_last_string_before_close_paren_unbalanced_is_none() {
        let region = "(\"first\", \"second\"";
        let paren_start = region.find('(').expect("open paren");
        assert!(scan_last_string_before_close_paren(region.as_bytes(), paren_start).is_none());
    }

    #[test]
    fn test_PMAT257_cov_scan_last_string_before_close_paren_nested_parens() {
        // A nested call after the last string must not confuse the depth
        // counter into stopping early.
        let region = "(\"only-string\", inner(1, 2))";
        let paren_start = region.find('(').expect("open paren");
        let (start, end) = scan_last_string_before_close_paren(region.as_bytes(), paren_start)
            .expect("string before a nested call is still found");
        assert_eq!(&region[start..end], "\"only-string\"");
    }

    #[test]
    fn test_PMAT257_cov_advance_in_raw_steps_until_the_closing_delimiter() {
        let bytes = b"xy\"#";
        let mut last_str = None;
        // Not yet at the closing `"#` — must just step forward.
        assert!(matches!(
            advance_in_raw(bytes, 0, 0, &mut last_str),
            ScanAdvance::Step1
        ));
        assert!(last_str.is_none());
        // At the closing `"#` — must record the string and skip both bytes.
        assert!(matches!(
            advance_in_raw(bytes, 2, 0, &mut last_str),
            ScanAdvance::Skip(2)
        ));
        assert_eq!(last_str, Some((0, 4)));
    }

    #[test]
    fn test_PMAT257_cov_advance_in_str_skips_escaped_characters() {
        let bytes = b"a\\\"b\"";
        let mut last_str = None;
        // A backslash must skip both itself and the escaped character
        // without closing the string.
        assert!(matches!(
            advance_in_str(bytes, 1, 0, &mut last_str),
            ScanAdvance::Skip(2)
        ));
        assert!(last_str.is_none());
        // The unescaped quote at the end closes the string.
        assert!(matches!(
            advance_in_str(bytes, 4, 0, &mut last_str),
            ScanAdvance::Step1
        ));
        assert_eq!(last_str, Some((0, 5)));
    }

    #[test]
    fn test_PMAT257_cov_advance_normal_tracks_depth_and_string_starts() {
        let bytes = b"(\"a\")";
        let mut depth = 0;
        let mut str_start = 0usize;
        // '(' opens a level of nesting.
        assert!(matches!(
            advance_normal(bytes, 0, &mut depth, &mut str_start),
            ScanAdvance::Step1
        ));
        assert_eq!(depth, 1);
        // '"' records the start of a string literal.
        assert!(matches!(
            advance_normal(bytes, 1, &mut depth, &mut str_start),
            ScanAdvance::Step1
        ));
        assert_eq!(str_start, 1);
        // The balancing ')' at depth 1 signals Done.
        assert!(matches!(
            advance_normal(bytes, 4, &mut depth, &mut str_start),
            ScanAdvance::Done
        ));
        assert_eq!(depth, 0);
    }

    #[test]
    fn test_PMAT257_cov_advance_normal_enters_raw_string() {
        let bytes = b"r#\"x\"#";
        let mut depth = 0;
        let mut str_start = 0usize;
        assert!(matches!(
            advance_normal(bytes, 0, &mut depth, &mut str_start),
            ScanAdvance::Skip(3)
        ));
        assert_eq!(str_start, 0);
    }

    #[test]
    fn test_PMAT257_cov_next_scan_state_transitions() {
        // Normal -> InRaw on `r#"`.
        assert!(matches!(
            next_scan_state(&RustScanState::Normal, b"r#\"x", 0),
            RustScanState::InRaw
        ));
        // Normal -> InStr on a plain `"`.
        assert!(matches!(
            next_scan_state(&RustScanState::Normal, b"\"x", 0),
            RustScanState::InStr
        ));
        // Normal -> Normal otherwise.
        assert!(matches!(
            next_scan_state(&RustScanState::Normal, b"x", 0),
            RustScanState::Normal
        ));
        // InStr -> Normal on an unescaped closing quote.
        assert!(matches!(
            next_scan_state(&RustScanState::InStr, b"a\"", 1),
            RustScanState::Normal
        ));
        // InStr -> InStr while not yet closed.
        assert!(matches!(
            next_scan_state(&RustScanState::InStr, b"ab", 0),
            RustScanState::InStr
        ));
        // InRaw -> Normal on the raw-string terminator.
        assert!(matches!(
            next_scan_state(&RustScanState::InRaw, b"x\"#", 1),
            RustScanState::Normal
        ));
        // InRaw -> InRaw while not yet closed.
        assert!(matches!(
            next_scan_state(&RustScanState::InRaw, b"xy", 0),
            RustScanState::InRaw
        ));
    }

    #[test]
    fn test_PMAT257_cov_format_rust_string_for_registry_plain() {
        assert_eq!(format_rust_string_for_registry("plain"), "\"plain\"");
    }

    #[test]
    fn test_PMAT257_cov_format_rust_string_for_registry_uses_raw_string_for_quotes() {
        let out = format_rust_string_for_registry("has \"quotes\" inside");
        assert_eq!(out, "r#\"has \"quotes\" inside\"#");
    }

    #[test]
    fn test_PMAT257_cov_format_rust_string_for_registry_falls_back_when_raw_would_break() {
        // A value containing `"#` cannot safely use `r#"..."#` (it would
        // terminate the raw string early), so it must fall back to an
        // escaped regular string.
        let out = format_rust_string_for_registry("ends with \"#");
        assert_eq!(out, "\"ends with \\\"#\"");
        assert!(!out.starts_with("r#"));
    }
}
