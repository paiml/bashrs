//! Corpus B2 fix application: registry scanning, string replacement, and Rust source formatting.

use crate::models::{Error, Result};

pub(crate) fn corpus_apply_b2_fixes(fixes: &[(String, String, String)]) -> Result<()> {
    let registry_path = std::path::Path::new("rash/src/corpus/registry.rs");
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


pub(crate) fn scan_last_string_before_close_paren(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
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


pub(crate) fn advance_normal(bytes: &[u8], i: usize, depth: &mut i32, str_start: &mut usize) -> ScanAdvance {
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
    use std::path::Path;
    use crate::cli::commands::should_output_to_stdout;
    use crate::cli::logic::generate_diff_lines;

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
