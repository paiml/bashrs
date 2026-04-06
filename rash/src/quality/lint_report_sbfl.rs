/// Generate SBFL-only ASCII report (ML-006)
pub fn sbfl_report(
    rankings: &[SuspiciousnessRanking],
    formula: SbflFormula,
    width: usize,
) -> String {
    let mut out = String::new();
    let inner = width - 2;

    // Header
    out.push(box_chars::TOP_LEFT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::TOP_RIGHT);
    out.push('\n');

    let title = format!("FAULT LOCALIZATION REPORT ({})", formula);
    let padding = (inner.saturating_sub(title.len())) / 2;
    out.push(box_chars::VERTICAL);
    for _ in 0..padding {
        out.push(' ');
    }
    out.push_str(&title);
    for _ in 0..(inner - padding - title.len()) {
        out.push(' ');
    }
    out.push(box_chars::VERTICAL);
    out.push('\n');

    // Divider
    out.push(box_chars::T_RIGHT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::T_LEFT);
    out.push('\n');

    // Column headers
    let header = " Rank │ Rule   │ Suspiciousness │ Failed │ Passed │ Explanation";
    out.push(box_chars::VERTICAL);
    out.push_str(header);
    for _ in 0..(inner.saturating_sub(header.len())) {
        out.push(' ');
    }
    out.push(box_chars::VERTICAL);
    out.push('\n');

    // Divider
    out.push(box_chars::T_RIGHT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::T_LEFT);
    out.push('\n');

    // Rows
    for ranking in rankings.iter().take(10) {
        let bar = histogram_bar(ranking.score, 1.0, 10);
        let explanation = if ranking.score > 0.8 {
            "High suspiciousness"
        } else if ranking.score > 0.5 {
            "Moderate suspicion"
        } else {
            "Low suspicion"
        };

        let row = format!(
            "  {:>2}  │ {:>6} │ {} {:>.2}│ {:>6} │ {:>6} │ {}",
            ranking.rank,
            ranking.location,
            bar,
            ranking.score,
            ranking.coverage.failed_covering,
            ranking.coverage.passed_covering,
            explanation
        );

        out.push(box_chars::VERTICAL);
        let truncated = if row.len() > inner {
            &row[..inner]
        } else {
            &row
        };
        out.push_str(truncated);
        for _ in 0..(inner.saturating_sub(row.len())) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    // Footer
    out.push(box_chars::BOTTOM_LEFT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::BOTTOM_RIGHT);
    out.push('\n');

    out
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "lint_report_tests_sample_lint.rs"]
// FIXME(PMAT-238): mod tests_extracted;
