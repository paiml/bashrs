//! B2 diagnostic and classification logic for corpus exact-match analysis.
//!
//! This module contains pure logic functions for classifying B2 (exact match)
//! failures, finding replacement strings, extracting meaningful output lines,
//! and formatting Rust string literals for registry edits.

use std::collections::HashSet;

/// Classify a B2-only failure (B1 passes but B2 fails).
///
/// Returns `(category, best_matching_line)` where category is one of:
/// - `"false_positive"` - the expected string matches a full line exactly
/// - `"echo_to_printf"` - echo was converted to printf
/// - `"quoting_added"` - quotes were added around values
/// - `"line_wider"` - the actual line is wider than expected
/// - `"other"` - some other mismatch
/// - `"multiline_mismatch"` - expected string not found in any line
#[must_use]
pub fn classify_b2_only(expected: &str, actual: &str) -> (String, String) {
    let actual_lines: Vec<&str> = actual.lines().map(str::trim).collect();
    let matching: Vec<&&str> = actual_lines
        .iter()
        .filter(|l| l.contains(expected))
        .collect();

    let category = if matching.is_empty() {
        "multiline_mismatch"
    } else {
        let line = matching[0];
        if *line == expected {
            "false_positive"
        } else if line.starts_with("printf ") && expected.starts_with("echo ") {
            "echo_to_printf"
        } else if line.contains('\'') && !expected.contains('\'') {
            "quoting_added"
        } else if line.len() > expected.len() {
            "line_wider"
        } else {
            "other"
        }
    };

    let best = matching.first().map(|l| l.to_string()).unwrap_or_default();
    (category.to_string(), best)
}

/// Classify a B1+B2 failure (neither containment nor exact match).
///
/// Returns a category string:
/// - `"empty_expected"` - the expected string is empty
/// - `"echo_vs_printf"` - echo argument found via different command
/// - `"echo_missing"` - echo argument not found at all
/// - `"partial_match"` - more than half the prefix matches
/// - `"diverged"` - less than half the prefix matches
/// - `"no_output"` - no meaningful output lines
#[must_use]
pub fn classify_b1b2(expected: &str, actual: &str) -> String {
    if expected.is_empty() {
        return "empty_expected".to_string();
    }
    let actual_lines: Vec<&str> = actual.lines().map(str::trim).collect();

    if let Some(arg) = expected.strip_prefix("echo ") {
        if actual_lines.iter().any(|l| l.contains(arg)) {
            return "echo_vs_printf".to_string();
        }
        return "echo_missing".to_string();
    }

    let best = actual_lines
        .iter()
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("set "))
        .min_by_key(|l| {
            let common = expected
                .chars()
                .zip(l.chars())
                .take_while(|(a, b)| a == b)
                .count();
            expected.len().saturating_sub(common)
        });

    match best {
        Some(closest) => {
            let prefix_len = expected
                .chars()
                .zip(closest.chars())
                .take_while(|(a, b)| a == b)
                .count();
            if prefix_len > expected.len() / 2 {
                "partial_match"
            } else {
                "diverged"
            }
        }
        None => "no_output",
    }
    .to_string()
}

/// Find the best replacement expected_contains line from actual output.
///
/// Tries two strategies:
/// 1. B2-only: expected is a substring of an actual line (B1 passes)
/// 2. B1+B2 diverged: find best matching line from main body by token overlap
#[must_use]
pub fn find_best_b2_replacement(expected: &str, actual: &str, id: &str) -> Option<String> {
    let actual_lines: Vec<&str> = actual.lines().map(str::trim).collect();

    // Strategy 1: B2-only -- expected is substring of an actual line (B1 passes)
    if let Some(full_line) = actual_lines.iter().find(|l| l.contains(expected)) {
        return Some(full_line.to_string());
    }

    // Strategy 2: B1+B2 diverged -- find best matching line from main body
    let meaningful = extract_main_body(actual, id);
    if meaningful.is_empty() {
        return None;
    }

    find_best_token_match(expected, &meaningful)
}

/// Find the actual line with the best token overlap to the expected string.
///
/// Tokens are split on non-alphanumeric (except `_`) boundaries.
/// Returns the line with the highest token intersection count.
/// Falls back to the first "distinctive" line (containing `=`, `rash_println`, or starting with `for`),
/// or the very first line if nothing else matches.
#[must_use]
pub fn find_best_token_match(expected: &str, lines: &[String]) -> Option<String> {
    let exp_tokens: HashSet<&str> = expected
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .collect();

    let mut best_line = None;
    let mut best_score = 0usize;

    for line in lines {
        let line_tokens: HashSet<&str> = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|t| !t.is_empty())
            .collect();
        let overlap = exp_tokens.intersection(&line_tokens).count();
        if overlap > best_score {
            best_score = overlap;
            best_line = Some(line.clone());
        }
    }

    // Require at least 1 token overlap, or fall back to first distinctive line
    if best_score >= 1 {
        return best_line;
    }

    lines
        .iter()
        .find(|l| l.contains('=') || l.contains("rash_println") || l.starts_with("for "))
        .or_else(|| lines.first())
        .cloned()
}

/// Extract meaningful lines from transpiled output (skip shell preamble).
///
/// For Dockerfile (`D-`) and Makefile (`M-`) entries, extracts non-comment lines.
/// For Bash entries, extracts lines from inside the `main()` function body.
#[must_use]
pub fn extract_main_body(actual: &str, id: &str) -> Vec<String> {
    if id.starts_with("D-") || id.starts_with("M-") {
        return extract_noncomment_lines(actual);
    }
    extract_bash_main_body(actual)
}

/// Extract all non-empty, non-comment lines (for Dockerfile/Makefile).
#[must_use]
pub fn extract_noncomment_lines(actual: &str) -> Vec<String> {
    actual
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Return true if this trimmed line is shell preamble (not user code).
#[must_use]
pub fn is_bash_preamble(s: &str) -> bool {
    s.is_empty()
        || s.starts_with('#')
        || s.starts_with("set ")
        || s.starts_with("IFS=")
        || s.starts_with("export ")
        || s.starts_with("trap ")
        || s == "main \"$@\""
}

/// State for extracting the main() body from transpiled bash.
#[derive(Debug, PartialEq)]
pub enum BashBodyState {
    /// Before entering any function definition.
    Before,
    /// Inside a helper function definition (rash_println, rash_eprintln).
    InFuncDef,
    /// Inside the main() function body.
    InMain,
}

/// Extract lines from inside main() in a transpiled bash script.
#[must_use]
pub fn extract_bash_main_body(actual: &str) -> Vec<String> {
    let mut meaningful = Vec::new();
    let mut state = BashBodyState::Before;
    for line in actual.lines() {
        let s = line.trim();
        if is_bash_preamble(s) {
            continue;
        }
        state = advance_bash_body_state(s, state, &mut meaningful);
    }
    meaningful
}

/// Advance the state machine for extracting main() body lines.
///
/// Transitions:
/// - `InFuncDef` + `}` -> `Before`
/// - `Before` + `rash_println()`/`rash_eprintln()` -> `InFuncDef`
/// - `Before` + `main()` -> `InMain`
/// - `InMain` + `}` -> `Before`
/// - `InMain` + other -> collect line
pub fn advance_bash_body_state(
    s: &str,
    state: BashBodyState,
    out: &mut Vec<String>,
) -> BashBodyState {
    match state {
        BashBodyState::InFuncDef => {
            if s == "}" {
                BashBodyState::Before
            } else {
                BashBodyState::InFuncDef
            }
        }
        BashBodyState::Before => {
            if s.starts_with("rash_println()") || s.starts_with("rash_eprintln()") {
                BashBodyState::InFuncDef
            } else if s.starts_with("main()") {
                BashBodyState::InMain
            } else {
                BashBodyState::Before
            }
        }
        BashBodyState::InMain => {
            if s == "}" {
                BashBodyState::Before
            } else {
                out.push(s.to_string());
                BashBodyState::InMain
            }
        }
    }
}

/// Format a string as a Rust string literal for registry.rs.
///
/// Uses raw string `r#"..."#` if the value contains quotes or backslashes,
/// otherwise uses regular `"..."` with escaping.
/// Falls back to escaped regular string if the value contains `"#`.
#[must_use]
pub fn format_rust_string_for_registry(s: &str) -> String {
    if s.contains('"') || s.contains('\\') {
        // Use raw string -- but check it doesn't contain "# which would break r#"..."#
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

/// State for scanning Rust string literals in source code.
#[derive(PartialEq)]
pub enum RustScanState {
    /// Normal code (not inside a string).
    Normal,
    /// Inside a regular string literal.
    InStr,
    /// Inside a raw string literal.
    InRaw,
}

/// Advance result for the Rust source scanner.
pub enum ScanAdvance {
    /// Advance by 1 byte.
    Step1,
    /// Skip forward by N bytes.
    Skip(usize),
    /// Scanning is complete (balanced paren found).
    Done,
}

/// Find the last string literal in a `CorpusEntry::new(...)` call starting near `id_pos`.
///
/// Returns `(start_byte, end_byte)` of the string literal including delimiters.
#[must_use]
pub fn find_last_string_in_entry(content: &str, id_pos: usize) -> Option<(usize, usize)> {
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
///
/// Returns `(start, end)` offsets within the byte slice, including string delimiters.
#[must_use]
pub fn scan_last_string_before_close_paren(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
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

/// Advance the scanner by one step, returning the action to take.
pub fn advance_scan(
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

/// Advance scanner when inside a raw string literal.
pub fn advance_in_raw(
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

/// Advance scanner when inside a regular string literal.
pub fn advance_in_str(
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

/// Advance scanner when in normal (non-string) code.
pub fn advance_normal(bytes: &[u8], i: usize, depth: &mut i32, str_start: &mut usize) -> ScanAdvance {
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

/// Compute the next scan state based on the current byte.
#[must_use]
pub fn next_scan_state(current: &RustScanState, bytes: &[u8], i: usize) -> RustScanState {
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

/// Collect all B2 fixes: both B2-only (B1 passes) and B1+B2 diverged.
///
/// Returns a vector of `(id, old_expected, new_expected)` tuples.
#[must_use]
pub fn collect_b2_fixes(score: &crate::corpus::runner::CorpusScore) -> Vec<(String, String, String)> {
    let mut fixes = Vec::new();

    for r in &score.results {
        if !r.transpiled || r.output_exact {
            continue;
        }
        let expected = r.expected_output.as_deref().unwrap_or("").trim();
        if expected.is_empty() {
            continue;
        }
        let actual = r.actual_output.as_deref().unwrap_or("");
        if actual.trim().is_empty() {
            continue;
        }

        if let Some(new_expected) = find_best_b2_replacement(expected, actual, &r.id) {
            if new_expected != expected {
                fixes.push((r.id.clone(), expected.to_string(), new_expected));
            }
        }
    }

    fixes
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn test_b2_classify_b2_only_categories() {
        let (cat, best) = classify_b2_only("echo hello", "  echo hello  \n  other line  ");
        assert_eq!(cat, "false_positive"); assert_eq!(best, "echo hello");
        let (cat, _) = classify_b2_only("abcdef", "abcde\nabcdef\n"); assert_eq!(cat, "false_positive");
        let (cat, _) = classify_b2_only("echo x", "printf '%s\\n' echo x\nother"); assert_eq!(cat, "echo_to_printf");
        let (cat, _) = classify_b2_only("x=5", "x=5 # with 'quotes'\nother"); assert_eq!(cat, "quoting_added");
        let (cat, best) = classify_b2_only("echo", "echo hello world\nother");
        assert_eq!(cat, "line_wider"); assert_eq!(best, "echo hello world");
        let (cat, _) = classify_b2_only("echo hello world", "echo hel\necho hello world here"); assert_eq!(cat, "line_wider");
        let (cat, best) = classify_b2_only("not_here", "some other\noutput lines");
        assert_eq!(cat, "multiline_mismatch"); assert_eq!(best, "");
    }
    #[test]
    fn test_b2_classify_b1b2_categories() {
        assert_eq!(classify_b1b2("", "some output"), "empty_expected");
        assert_eq!(classify_b1b2("echo hello", "printf '%s\\n' hello\n"), "echo_vs_printf");
        assert_eq!(classify_b1b2("echo hello", "totally different output\n"), "echo_missing");
        assert_eq!(classify_b1b2("variable_name=value", "variable_name=other_value\n"), "partial_match");
        assert_eq!(classify_b1b2("completely_different", "zzz_no_match\n"), "diverged");
        assert_eq!(classify_b1b2("something", "# comment\nset -e\n"), "no_output");
    }
    #[test]
    fn test_b2_find_best_token_match() {
        let s = |x: &str| x.to_string();
        let lines = vec![s("foo bar baz"), s("qux quux")];
        assert_eq!(find_best_token_match("foo bar", &lines), Some(s("foo bar baz")));
        let lines = vec![s("x=42"), s("other_line")];
        assert_eq!(find_best_token_match("zzz_none", &lines), Some(s("x=42")));
        let lines = vec![s("rash_println hello"), s("some_other")];
        assert_eq!(find_best_token_match("zzz_none", &lines), Some(s("rash_println hello")));
        let lines = vec![s("first line"), s("second line")];
        assert_eq!(find_best_token_match("zzz_nothing", &lines), Some(s("first line")));
        assert_eq!(find_best_token_match("anything", &[]), None);
    }
    #[test]
    fn test_b2_extract_main_body_and_helpers() {
        assert_eq!(extract_main_body("# comment\nFROM ubuntu\nRUN apt-get update\n", "D-001"), vec!["FROM ubuntu", "RUN apt-get update"]);
        assert_eq!(extract_main_body("# Makefile comment\nall: build\n\tbuild step\n", "M-001"), vec!["all: build", "build step"]);
        assert_eq!(extract_main_body("#!/bin/sh\nset -e\nmain() {\n  echo hello\n  echo world\n}\nmain \"$@\"\n", "B-001"), vec!["echo hello", "echo world"]);
        assert_eq!(extract_bash_main_body("#!/bin/sh\nrash_println() {\n  printf '%s\\n' \"$1\"\n}\nmain() {\n  x=5\n  rash_println \"$x\"\n}\nmain \"$@\"\n"), vec!["x=5", "rash_println \"$x\""]);
        assert!(extract_bash_main_body("#!/bin/sh\nset -e\necho orphan\n").is_empty());
        assert_eq!(extract_noncomment_lines("# comment\n\nFROM ubuntu\n  RUN echo hi  \n# another comment\n"), vec!["FROM ubuntu", "RUN echo hi"]);
        assert!(extract_noncomment_lines("# only comments\n# here\n\n\n").is_empty());
    }
    #[test]
    fn test_b2_is_bash_preamble() {
        for s in &["", "# this is a comment", "#!/bin/sh", "set -euo pipefail", "IFS=$'\\n\\t'", "export PATH=/usr/bin", "trap 'cleanup' EXIT", "main \"$@\""] {
            assert!(is_bash_preamble(s), "{s:?} should be preamble");
        }
        for s in &["echo hello", "x=5", "for i in 1 2 3; do"] { assert!(!is_bash_preamble(s), "{s:?} should not be preamble"); }
    }
    #[test]
    fn test_b2_format_rust_string_for_registry() {
        assert_eq!(format_rust_string_for_registry("hello"), "\"hello\"");
        assert_eq!(format_rust_string_for_registry(""), "\"\"");
        assert_eq!(format_rust_string_for_registry("say \"hi\""), "r#\"say \"hi\"\"#");
        assert_eq!(format_rust_string_for_registry("path\\to\\file"), "r#\"path\\to\\file\"#");
        assert_eq!(format_rust_string_for_registry("value\"#end"), "\"value\\\"#end\"");
    }
    #[test]
    fn test_b2_advance_bash_body_state_transitions() {
        let mut out = Vec::new();
        let state = advance_bash_body_state("rash_println() {", BashBodyState::Before, &mut out);
        assert_eq!(state, BashBodyState::InFuncDef); assert!(out.is_empty());
        let state = advance_bash_body_state("printf '%s\\n' \"$1\"", state, &mut out); assert_eq!(state, BashBodyState::InFuncDef);
        let state = advance_bash_body_state("}", state, &mut out); assert_eq!(state, BashBodyState::Before);
        let state = advance_bash_body_state("main() {", state, &mut out); assert_eq!(state, BashBodyState::InMain);
        let state = advance_bash_body_state("echo hello", state, &mut out);
        assert_eq!(state, BashBodyState::InMain); assert_eq!(out, vec!["echo hello"]);
        let state = advance_bash_body_state("}", state, &mut out);
        assert_eq!(state, BashBodyState::Before); assert_eq!(out.len(), 1);
    }
    #[test]
    fn test_b2_find_last_string_in_entry_and_scan() {
        let content = r#"CorpusEntry::new("B-001", "name", "desc", CorpusFormat::Bash, CorpusTier::Adversarial, "input", "expected")"#;
        let (start, end) = find_last_string_in_entry(content, content.find("\"B-001\"").unwrap()).unwrap();
        assert_eq!(&content[start..end], "\"expected\"");
        let content = r##"CorpusEntry::new("B-002", "name", "desc", CorpusFormat::Bash, CorpusTier::Adversarial, "input", r#"raw expected"#)"##;
        let (start, end) = find_last_string_in_entry(content, content.find("\"B-002\"").unwrap()).unwrap();
        assert!((&content[start..end]).contains("raw expected"));
        let code = b"(\"first\", \"second\", \"third\")";
        let (s, e) = scan_last_string_before_close_paren(code, 0).unwrap(); assert_eq!(&code[s..e], b"\"third\"");
        let code = b"(\"first\", r#\"raw value\"#)";
        let (s, e) = scan_last_string_before_close_paren(code, 0).unwrap();
        assert_eq!(std::str::from_utf8(&code[s..e]).unwrap(), "r#\"raw value\"#");
        assert!(scan_last_string_before_close_paren(b"(\"first\", \"second\"", 0).is_none());
    }
}
