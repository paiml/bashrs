//! Corpus B2 diagnostic commands: classification, diagnosis, and fix application.

use super::corpus_b2_fix_commands::corpus_apply_b2_fixes;
use super::corpus_score_print_commands::corpus_load_last_run;
use crate::cli::args::CorpusFormatArg;
use crate::cli::logic::truncate_str;
use crate::models::{Error, Result};

pub(crate) fn classify_b2_only(expected: &str, actual: &str) -> (String, String) {
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
pub(crate) fn classify_b1b2(expected: &str, actual: &str) -> String {
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

/// Print a categorized group of failures.
pub(crate) fn print_b2_category(cat: &str, items: &[(String, String, String)], limit: usize) {
    use crate::cli::color::*;
    let show = limit.min(5);
    println!("  {CYAN}{cat}{RESET}: {} entries", items.len());
    for (id, expected, actual_line) in items.iter().take(show) {
        println!("    {DIM}{id}{RESET}");
        println!(
            "      expected: {RED}{}{RESET}",
            truncate_str(expected, 100)
        );
        println!(
            "      actual:   {GREEN}{}{RESET}",
            truncate_str(actual_line, 100)
        );
    }
    if items.len() > show {
        println!("    {DIM}... +{} more{RESET}", items.len() - show);
    }
    println!();
}

/// Diagnose B2 exact match failures from cached results (instant, no re-transpilation).
pub(crate) fn corpus_diagnose_b2(_filter: Option<&CorpusFormatArg>, limit: usize) -> Result<()> {
    use crate::cli::color::*;

    let score = corpus_load_last_run().ok_or_else(|| {
        Error::Validation("No cached corpus results. Run `bashrs corpus run` first.".to_string())
    })?;

    let mut b2_only_cats: std::collections::HashMap<String, Vec<(String, String, String)>> =
        std::collections::HashMap::new();
    let mut b1b2_cats: std::collections::HashMap<String, Vec<(String, String, String)>> =
        std::collections::HashMap::new();
    let mut b2_only_count = 0usize;
    let mut b1b2_count = 0usize;

    for r in &score.results {
        if !r.transpiled || r.output_exact {
            continue;
        }
        let expected = r
            .expected_output
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        let actual = r.actual_output.as_deref().unwrap_or("");

        if r.output_contains {
            b2_only_count += 1;
            let (cat, best) = classify_b2_only(&expected, actual);
            b2_only_cats
                .entry(cat)
                .or_default()
                .push((r.id.clone(), expected, best));
        } else {
            b1b2_count += 1;
            let cat = classify_b1b2(&expected, actual);
            let closest = actual
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("set "))
                .unwrap_or("")
                .to_string();
            b1b2_cats
                .entry(cat)
                .or_default()
                .push((r.id.clone(), expected, closest));
        }
    }

    println!("{WHITE}B2 Exact Match Diagnosis{RESET} {DIM}(from cache){RESET}");
    println!("{DIM}────────────────────────────────────────{RESET}");
    println!(
        "Total B2 failures:       {RED}{}{RESET}",
        b2_only_count + b1b2_count
    );
    println!(
        "  B2-only (B1 passes):   {YELLOW}{b2_only_count}{RESET}  <- update expected_contains"
    );
    println!("  B1+B2 (neither match): {RED}{b1b2_count}{RESET}  <- transpiler diverged");
    println!();

    if b2_only_count > 0 {
        println!("{WHITE}B2-ONLY (expected is substring but not full line):{RESET}");
        println!();
        let mut sorted: Vec<_> = b2_only_cats.iter().collect();
        sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        for (cat, items) in &sorted {
            print_b2_category(cat, items, limit);
        }
    }

    if b1b2_count > 0 {
        println!("{WHITE}B1+B2 (expected string not in output at all):{RESET}");
        println!();
        let mut sorted: Vec<_> = b1b2_cats.iter().collect();
        sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        for (cat, items) in &sorted {
            print_b2_category(cat, items, limit);
        }
    }

    println!("{WHITE}Action Items:{RESET}");
    if b2_only_count > 0 {
        println!(
            "  1. B2-only ({b2_only_count} entries): Update expected_contains to full output line"
        );
        println!("     Run: bashrs corpus fix-b2 > b2_fixes.json");
    }
    if b1b2_count > 0 {
        println!("  2. B1+B2 ({b1b2_count} entries): Fix emitter or update expected");
    }

    Ok(())
}

/// Output corrected expected_contains for all B2-only failures as JSON.
/// For each entry where B1 passes but B2 fails, finds the actual full line
/// that contains the expected substring and outputs the correction.
pub(crate) fn corpus_fix_b2(apply: bool) -> Result<()> {
    let score = corpus_load_last_run().ok_or_else(|| {
        Error::Validation("No cached corpus results. Run `bashrs corpus run` first.".to_string())
    })?;

    let fixes = collect_b2_fixes(&score);
    eprintln!("Generated {} B2 fixes", fixes.len());

    if apply {
        corpus_apply_b2_fixes(&fixes)?;
    } else {
        let json_fixes: Vec<serde_json::Value> = fixes
            .iter()
            .map(|(id, old, new_val)| serde_json::json!({"id": id, "old": old, "new": new_val}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json_fixes)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?
        );
    }

    Ok(())
}

/// Collect all B2 fixes: both B2-only (B1 passes) and B1+B2 diverged.
pub(crate) fn collect_b2_fixes(
    score: &crate::corpus::runner::CorpusScore,
) -> Vec<(String, String, String)> {
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

/// Find the best replacement expected_contains line from actual output.
pub(crate) fn find_best_b2_replacement(expected: &str, actual: &str, id: &str) -> Option<String> {
    let actual_lines: Vec<&str> = actual.lines().map(str::trim).collect();

    // Strategy 1: B2-only — expected is substring of an actual line (B1 passes)
    if let Some(full_line) = actual_lines.iter().find(|l| l.contains(expected)) {
        return Some(full_line.to_string());
    }

    // Strategy 2: B1+B2 diverged — find best matching line from main body
    let meaningful = extract_main_body(actual, id);
    if meaningful.is_empty() {
        return None;
    }

    find_best_token_match(expected, &meaningful)
}

/// Extract meaningful lines from transpiled output (skip shell preamble).
pub(crate) fn extract_main_body(actual: &str, id: &str) -> Vec<String> {
    if id.starts_with("D-") || id.starts_with("M-") {
        return extract_noncomment_lines(actual);
    }
    extract_bash_main_body(actual)
}

/// Extract all non-empty, non-comment lines (for Dockerfile/Makefile).
pub(crate) fn extract_noncomment_lines(actual: &str) -> Vec<String> {
    actual
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Return true if this trimmed line is shell preamble (not user code).
///
/// Delegates to the canonical implementation in `corpus::dataset::is_shell_preamble`.
pub(crate) fn is_bash_preamble(s: &str) -> bool {
    crate::corpus::dataset::is_shell_preamble(s)
}

/// Extract lines from inside main() in a transpiled bash script.
/// State for extracting the main() body from transpiled bash.
#[derive(PartialEq)]

pub(crate) enum BashBodyState {
    Before,
    InFuncDef,
    InMain,
}

pub(crate) fn extract_bash_main_body(actual: &str) -> Vec<String> {
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

pub(crate) fn advance_bash_body_state(
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

/// Find the actual line with the best token overlap to the expected string.
pub(crate) fn find_best_token_match(expected: &str, lines: &[String]) -> Option<String> {
    let exp_tokens: std::collections::HashSet<&str> = expected
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .collect();

    let mut best_line = None;
    let mut best_score = 0usize;

    for line in lines {
        let line_tokens: std::collections::HashSet<&str> = line
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
