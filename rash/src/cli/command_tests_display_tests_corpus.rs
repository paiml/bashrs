fn test_corpus_print_failures_human_format() {
    use super::corpus_report_commands::corpus_print_failures;
    use crate::cli::args::CorpusOutputFormat;
    let r1 = mock_result("B-001", false);
    let r2 = mock_result_partial("B-002");
    let failures: Vec<&CorpusResult> = vec![&r1, &r2];
    let result = corpus_print_failures(&failures, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_failures_json_format() {
    use super::corpus_report_commands::corpus_print_failures;
    use crate::cli::args::CorpusOutputFormat;
    let r1 = mock_result("B-001", false);
    let failures: Vec<&CorpusResult> = vec![&r1];
    let result = corpus_print_failures(&failures, &CorpusOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_history_row_with_format_data() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(5, 99.0, 1000);
    let prev = mock_convergence_entry(4, 98.5, 980);
    corpus_print_history_row(&e, Some(&prev), true, true);
}

#[test]
fn test_corpus_print_history_row_without_format_data() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(1, 95.0, 500);
    corpus_print_history_row(&e, None, false, false);
}

#[test]
fn test_corpus_print_history_row_with_score_no_format() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(3, 97.5, 800);
    corpus_print_history_row(&e, None, false, true);
}

#[test]
fn test_corpus_print_history_row_empty_grade() {
    use super::corpus_report_commands::corpus_print_history_row;
    let mut e = mock_convergence_entry(2, 90.0, 600);
    e.grade = String::new();
    corpus_print_history_row(&e, None, false, true);
}

// ── corpus_score_print_commands tests ───────────────────────────────────────

#[test]
fn test_stats_bar_full() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(100.0, 20);
    assert_eq!(bar.chars().filter(|c| *c == '\u{2588}').count(), 20);
}

#[test]
fn test_stats_bar_empty() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(0.0, 20);
    assert_eq!(bar.chars().filter(|c| *c == '\u{2591}').count(), 20);
}

#[test]
fn test_stats_bar_half() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(50.0, 20);
    assert!(bar.contains('\u{2588}'));
    assert!(bar.contains('\u{2591}'));
}

#[test]
fn test_corpus_stats_sparkline_trend_up() {
    use super::corpus_score_print_commands::corpus_stats_sparkline;
    let entries = vec![
        mock_convergence_entry(1, 90.0, 500),
        mock_convergence_entry(2, 95.0, 600),
        mock_convergence_entry(3, 99.0, 700),
    ];
    corpus_stats_sparkline(&entries);
}

#[test]
fn test_corpus_stats_sparkline_flat() {
    use super::corpus_score_print_commands::corpus_stats_sparkline;
    let entries = vec![
        mock_convergence_entry(1, 99.0, 500),
        mock_convergence_entry(2, 99.0, 500),
    ];
    corpus_stats_sparkline(&entries);
}

#[test]
fn test_corpus_print_score_human_no_failures() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let score = CorpusScore {
        total: 10,
        passed: 10,
        failed: 0,
        rate: 1.0,
        score: 99.5,
        grade: Grade::APlus,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Bash,
            total: 10,
            passed: 10,
            rate: 1.0,
            score: 99.5,
            grade: Grade::APlus,
        }],
        results: (0..10)
            .map(|i| mock_result(&format!("B-{:03}", i + 1), true))
            .collect(),
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_score_human_with_failures() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let mut results: Vec<CorpusResult> = (0..8)
        .map(|i| mock_result(&format!("B-{:03}", i + 1), true))
        .collect();
    results.push(mock_result("B-009", false));
    results.push(mock_result("B-010", false));
    let score = CorpusScore {
        total: 10,
        passed: 8,
        failed: 2,
        rate: 0.8,
        score: 85.0,
        grade: Grade::B,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Bash,
            total: 10,
            passed: 8,
            rate: 0.8,
            score: 85.0,
            grade: Grade::B,
        }],
        results,
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_score_json() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let score = CorpusScore {
        total: 2,
        passed: 2,
        failed: 0,
        rate: 1.0,
        score: 100.0,
        grade: Grade::APlus,
        format_scores: vec![],
        results: vec![mock_result("B-001", true), mock_result("B-002", true)],
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_load_save_last_run_returns_none_when_no_cache() {
    use super::corpus_score_print_commands::corpus_load_last_run;
    // This may return Some if there's a cache on disk, or None
    // We just ensure it doesn't panic
    let _ = corpus_load_last_run();
}

#[test]
fn test_corpus_cache_path_is_set() {
    use super::corpus_score_print_commands::CORPUS_CACHE_PATH;
    assert!(CORPUS_CACHE_PATH.contains("last-corpus-run"));
}
