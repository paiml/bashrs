//! Corpus display logic: pure computation extracted from CLI display functions.
//! All functions are stateless and free of I/O side effects.

use crate::corpus::registry::Grade;
use crate::corpus::runner::{ConvergenceEntry, CorpusResult, FormatScore};

/// Aggregated V2 component breakdown counts.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct V2Breakdown {
    pub n: usize,
    pub a_pass: usize, pub b1_pass: usize, pub b2_pass: usize, pub b3_pass: usize,
    pub d_pass: usize, pub e_pass: usize, pub f_pass: usize, pub g_pass: usize,
    pub c_avg: f64,
}

/// A single row in the V2 component table.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct V2ComponentRow {
    pub code: &'static str, pub label: &'static str,
    pub pass: usize, pub total: usize, pub pct: f64, pub pts: f64, pub max_pts: f64,
}

/// Compute V2 component breakdown from corpus results (spec section 11.4, 11.12).
#[must_use]
pub(crate) fn compute_v2_breakdown(results: &[CorpusResult]) -> V2Breakdown {
    let n = results.len();
    if n == 0 {
        return V2Breakdown {
            n: 0, a_pass: 0, b1_pass: 0, b2_pass: 0, b3_pass: 0,
            d_pass: 0, e_pass: 0, f_pass: 0, g_pass: 0, c_avg: 0.0,
        };
    }
    V2Breakdown {
        n,
        a_pass: results.iter().filter(|r| r.transpiled).count(),
        b1_pass: results.iter().filter(|r| r.output_contains).count(),
        b2_pass: results.iter().filter(|r| r.output_exact).count(),
        b3_pass: results.iter().filter(|r| r.output_behavioral).count(),
        d_pass: results.iter().filter(|r| r.lint_clean).count(),
        e_pass: results.iter().filter(|r| r.deterministic).count(),
        f_pass: results.iter().filter(|r| r.metamorphic_consistent).count(),
        g_pass: results.iter().filter(|r| r.cross_shell_agree).count(),
        c_avg: results.iter().map(|r| r.coverage_ratio).sum::<f64>() / n as f64,
    }
}

/// Convert a V2Breakdown into component rows (A, B1, B2, B3, C, D, E, F, G).
#[must_use]
pub(crate) fn v2_breakdown_rows(bd: &V2Breakdown) -> Vec<V2ComponentRow> {
    let n = bd.n;
    if n == 0 { return Vec::new(); }
    let pct = |pass: usize| -> f64 { pass as f64 / n as f64 * 100.0 };
    let pts = |pass: usize, max: f64| -> f64 { pass as f64 / n as f64 * max };
    let row = |code, label, pass, max_pts| V2ComponentRow {
        code, label, pass, total: n, pct: pct(pass), pts: pts(pass, max_pts), max_pts,
    };
    vec![
        row("A", "Transpilation", bd.a_pass, 30.0),
        row("B1", "Containment", bd.b1_pass, 10.0),
        row("B2", "Exact match", bd.b2_pass, 8.0),
        row("B3", "Behavioral", bd.b3_pass, 7.0),
        V2ComponentRow {
            code: "C", label: "Coverage",
            pass: (bd.c_avg * n as f64) as usize, total: n,
            pct: bd.c_avg * 100.0, pts: bd.c_avg * 15.0, max_pts: 15.0,
        },
        row("D", "Lint clean", bd.d_pass, 10.0),
        row("E", "Deterministic", bd.e_pass, 10.0),
        row("F", "Metamorphic", bd.f_pass, 5.0),
        row("G", "Cross-shell", bd.g_pass, 5.0),
    ]
}

/// Collect failure entries: returns `(id, error_message)` tuples.
#[must_use]
pub(crate) fn collect_failures(results: &[CorpusResult]) -> Vec<(&str, &str)> {
    results.iter().filter(|r| !r.transpiled)
        .map(|r| (r.id.as_str(), r.error.as_deref().unwrap_or("unknown error")))
        .collect()
}

/// A computed row for the per-format statistics table.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormatStatRow {
    pub format: String, pub total: usize, pub pass_pct: f64, pub grade: String,
    pub passed: usize, pub rate: f64, pub score: f64,
}

/// Compute per-format statistics table rows from format scores.
#[must_use]
pub(crate) fn compute_format_stats(format_scores: &[FormatScore]) -> Vec<FormatStatRow> {
    format_scores.iter().map(|fs| FormatStatRow {
        format: fs.format.to_string(), total: fs.total, pass_pct: fs.rate * 100.0,
        grade: fs.grade.to_string(), passed: fs.passed, rate: fs.rate, score: fs.score,
    }).collect()
}

/// Overall (totals) row for the stats table.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OverallStatRow {
    pub total: usize, pub pass_pct: f64, pub grade: String, pub score: f64,
}

/// Compute the totals row from overall score fields.
#[must_use]
pub(crate) fn compute_overall_stat(
    total: usize, rate: f64, grade: &Grade, score: f64,
) -> OverallStatRow {
    OverallStatRow { total, pass_pct: rate * 100.0, grade: grade.to_string(), score }
}

/// A single metamorphic relation check result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MrCheckResult {
    pub name: &'static str, pub passed: bool, pub description: &'static str,
}

/// Aggregated metamorphic check results for a corpus entry.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MrResults {
    pub checks: Vec<MrCheckResult>, pub total: u32, pub passed: u32, pub pass_pct: f64,
}

/// Compute MR check results from two runs of the same entry (spec section 11.2).
#[must_use]
pub(crate) fn compute_mr_results(result: &CorpusResult, result2: &CorpusResult) -> MrResults {
    let mr1 = result.actual_output == result2.actual_output;
    let checks = vec![
        MrCheckResult { name: "MR-1 Determinism", passed: mr1, description: "transpile(X) == transpile(X)" },
        MrCheckResult { name: "MR-2 Transpilation", passed: result.transpiled, description: "transpile(X) succeeds" },
        MrCheckResult { name: "MR-3 Containment", passed: result.output_contains, description: "output contains expected" },
        MrCheckResult { name: "MR-4 Exact match", passed: result.output_exact, description: "output == expected" },
        MrCheckResult { name: "MR-5 Behavioral", passed: result.output_behavioral, description: "sh -c terminates" },
        MrCheckResult { name: "MR-6 Lint clean", passed: result.lint_clean, description: "linter passes" },
        MrCheckResult { name: "MR-7 Cross-shell", passed: result.cross_shell_agree, description: "sh + dash agree" },
    ];
    let passed = checks.iter().filter(|c| c.passed).count() as u32;
    MrResults { checks, total: 7, passed, pass_pct: passed as f64 / 7.0 * 100.0 }
}

/// Tier distribution for corpus entries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TierDistribution {
    pub total: usize, pub tier_counts: [u32; 6],
    pub format_tiers: std::collections::HashMap<String, [u32; 6]>,
}

/// A single tier row for display.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TierRow { pub tier: u8, pub label: &'static str, pub count: u32, pub pct: f64 }

/// Per-format tier breakdown row.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormatTierRow { pub key: String, pub label: &'static str, pub parts: Vec<String> }

/// Compute tier distribution from `(format_prefix, tier)` pairs.
#[must_use]
pub(crate) fn compute_tier_distribution(entries: &[(String, u8)]) -> TierDistribution {
    let mut tier_counts = [0u32; 6];
    let mut format_tiers: std::collections::HashMap<String, [u32; 6]> =
        std::collections::HashMap::new();
    for (id_prefix, tier) in entries {
        let idx = (*tier as usize).min(5);
        tier_counts[idx] += 1;
        format_tiers.entry(id_prefix.clone()).or_insert([0u32; 6])[idx] += 1;
    }
    TierDistribution { total: entries.len(), tier_counts, format_tiers }
}

/// Convert a TierDistribution into display rows for tiers 1-5.
#[must_use]
pub(crate) fn tier_distribution_rows(dist: &TierDistribution) -> Vec<TierRow> {
    use crate::cli::corpus_score_logic::tier_label;
    (1..=5u8).map(|t| {
        let count = dist.tier_counts[t as usize];
        let pct = if dist.total == 0 { 0.0 } else { count as f64 / dist.total as f64 * 100.0 };
        TierRow { tier: t, label: tier_label(t), count, pct }
    }).collect()
}

/// Build per-format tier breakdown rows for display.
#[must_use]
pub(crate) fn format_tier_rows(dist: &TierDistribution) -> Vec<FormatTierRow> {
    [("B", "Bash"), ("M", "Makefile"), ("D", "Dockerfile")].iter().filter_map(|&(key, label)| {
        dist.format_tiers.get(key).and_then(|ft| {
            let parts: Vec<String> = (1..=5u8)
                .filter(|&t| ft[t as usize] > 0)
                .map(|t| format!("T{t}:{}", ft[t as usize])).collect();
            if parts.is_empty() { None }
            else { Some(FormatTierRow { key: key.to_string(), label, parts }) }
        })
    }).collect()
}

/// Formatted parts of a convergence log entry for display.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConvergenceParts {
    pub format_parts: Vec<String>, pub lint_pct: Option<f64>, pub lint_gap: Option<f64>,
}

/// Build formatted convergence parts from a convergence entry.
#[must_use]
pub(crate) fn format_convergence_parts(entry: &ConvergenceEntry) -> ConvergenceParts {
    let format_parts = if entry.bash_total > 0 || entry.makefile_total > 0 || entry.dockerfile_total > 0 {
        let fmt = |name: &str, p: usize, t: usize| if t > 0 { format!("{name} {p}/{t}") } else { String::new() };
        [fmt("Bash", entry.bash_passed, entry.bash_total),
         fmt("Make", entry.makefile_passed, entry.makefile_total),
         fmt("Docker", entry.dockerfile_passed, entry.dockerfile_total)]
            .into_iter().filter(|s| !s.is_empty()).collect()
    } else { Vec::new() };
    let lint_pct = if entry.lint_passed > 0 { Some(entry.lint_rate * 100.0) } else { None };
    let lint_gap = if entry.lint_passed > 0 {
        let gap = ((entry.rate - entry.lint_rate) * 100.0).abs();
        if gap > 0.1 { Some(gap) } else { None }
    } else { None };
    ConvergenceParts { format_parts, lint_pct, lint_gap }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn mk(id: &str, t: bool, b1: bool, b2: bool, b3: bool, d: bool, e: bool, f: bool, g: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(), transpiled: t, output_contains: b1, output_exact: b2,
            output_behavioral: b3, has_test: false, coverage_ratio: 0.0, schema_valid: true,
            lint_clean: d, deterministic: e, metamorphic_consistent: f, cross_shell_agree: g,
            expected_output: None, actual_output: None, error: None,
            error_category: None, error_confidence: None, decision_trace: None,
        }
    }
    fn pass(id: &str) -> CorpusResult { mk(id, true, true, true, true, true, true, true, true) }
    fn fail(id: &str) -> CorpusResult {
        let mut r = mk(id, false, false, false, false, false, false, false, false);
        r.error = Some("transpilation failed".to_string()); r
    }
    fn mfs(fmt: crate::corpus::registry::CorpusFormat, t: usize, p: usize, s: f64) -> FormatScore {
        let rate = if t > 0 { p as f64 / t as f64 } else { 0.0 };
        FormatScore { format: fmt, total: t, passed: p, rate, score: s, grade: Grade::from_score(s) }
    }
    fn mce(bp: usize, bt: usize, mp: usize, mt: usize, dp: usize, dt: usize,
           lp: usize, lr: f64, rate: f64) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration: 1, date: "2025-01-01".into(), total: bt+mt+dt, passed: bp+mp+dp,
            failed: (bt-bp)+(mt-mp)+(dt-dp), rate, delta: 0.0, notes: "test".into(),
            bash_passed: bp, bash_total: bt, makefile_passed: mp, makefile_total: mt,
            dockerfile_passed: dp, dockerfile_total: dt, score: 95.0, grade: "A".into(),
            bash_score: 0.0, makefile_score: 0.0, dockerfile_score: 0.0, lint_passed: lp, lint_rate: lr,
        }
    }

    // V2 breakdown
    #[test] fn test_CORPUS_DISPLAY_001_v2_breakdown_empty() {
        let bd = compute_v2_breakdown(&[]);
        assert_eq!(bd.n, 0); assert_eq!(bd.a_pass, 0); assert_eq!(bd.c_avg, 0.0);
    }
    #[test] fn test_CORPUS_DISPLAY_002_v2_breakdown_all_pass() {
        let bd = compute_v2_breakdown(&[pass("B-1"), pass("B-2"), pass("B-3")]);
        assert_eq!(bd.n, 3);
        assert_eq!([bd.a_pass, bd.b1_pass, bd.b2_pass, bd.b3_pass, bd.d_pass, bd.e_pass, bd.f_pass, bd.g_pass], [3; 8]);
    }
    #[test] fn test_CORPUS_DISPLAY_003_v2_breakdown_all_fail() {
        let bd = compute_v2_breakdown(&[fail("B-1"), fail("B-2")]);
        assert_eq!(bd.n, 2); assert_eq!(bd.a_pass, 0); assert_eq!(bd.b1_pass, 0);
    }
    #[test] fn test_CORPUS_DISPLAY_004_v2_breakdown_mixed() {
        let mut r2 = pass("B-2"); r2.transpiled = false; r2.lint_clean = false;
        let bd = compute_v2_breakdown(&[pass("B-1"), r2]);
        assert_eq!((bd.a_pass, bd.d_pass, bd.b1_pass), (1, 1, 2));
    }
    #[test] fn test_CORPUS_DISPLAY_005_v2_breakdown_coverage_avg() {
        let mut r1 = pass("B-1"); r1.coverage_ratio = 0.8;
        let mut r2 = pass("B-2"); r2.coverage_ratio = 0.4;
        assert!((compute_v2_breakdown(&[r1, r2]).c_avg - 0.6).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_006_v2_breakdown_single() {
        let bd = compute_v2_breakdown(&[pass("B-1")]);
        assert_eq!((bd.n, bd.a_pass, bd.g_pass), (1, 1, 1));
    }
    // V2 rows
    #[test] fn test_CORPUS_DISPLAY_007_rows_empty() {
        assert!(v2_breakdown_rows(&compute_v2_breakdown(&[])).is_empty());
    }
    #[test] fn test_CORPUS_DISPLAY_008_rows_count() {
        assert_eq!(v2_breakdown_rows(&compute_v2_breakdown(&[pass("B-1")])).len(), 9);
    }
    #[test] fn test_CORPUS_DISPLAY_009_rows_all_pass_pct() {
        let bd = compute_v2_breakdown(&[pass("B-1"), pass("B-2")]);
        for r in &v2_breakdown_rows(&bd) {
            if r.code != "C" { assert!((r.pct - 100.0).abs() < 1e-9, "{}", r.code); }
        }
    }
    #[test] fn test_CORPUS_DISPLAY_010_rows_max_pts() {
        let mp: Vec<f64> = v2_breakdown_rows(&compute_v2_breakdown(&[pass("B-1")])).iter().map(|r| r.max_pts).collect();
        assert_eq!(mp, vec![30.0, 10.0, 8.0, 7.0, 15.0, 10.0, 10.0, 5.0, 5.0]);
    }
    #[test] fn test_CORPUS_DISPLAY_011_rows_codes() {
        let c: Vec<&str> = v2_breakdown_rows(&compute_v2_breakdown(&[pass("B-1")])).iter().map(|r| r.code).collect();
        assert_eq!(c, ["A","B1","B2","B3","C","D","E","F","G"]);
    }
    #[test] fn test_CORPUS_DISPLAY_012_rows_half_pass() {
        let rows = v2_breakdown_rows(&compute_v2_breakdown(&[pass("B-1"), fail("B-2")]));
        assert!((rows[0].pts - 15.0).abs() < 1e-9); assert!((rows[0].pct - 50.0).abs() < 1e-9);
    }
    // Failures
    #[test] fn test_CORPUS_DISPLAY_013_failures_none() {
        assert!(collect_failures(&[pass("B-1"), pass("B-2")]).is_empty());
    }
    #[test] fn test_CORPUS_DISPLAY_014_failures_some() {
        let b = [pass("B-1"), fail("B-2")];
        let f = collect_failures(&b);
        assert_eq!((f.len(), f[0].0, f[0].1), (1, "B-2", "transpilation failed"));
    }
    #[test] fn test_CORPUS_DISPLAY_015_failures_unknown_error() {
        let mut r = fail("B-3"); r.error = None;
        assert_eq!(collect_failures(&[r])[0].1, "unknown error");
    }
    // Format stats
    #[test] fn test_CORPUS_DISPLAY_016_format_stats_empty() { assert!(compute_format_stats(&[]).is_empty()); }
    #[test] fn test_CORPUS_DISPLAY_017_format_stats_single() {
        use crate::corpus::registry::CorpusFormat;
        let rows = compute_format_stats(&[mfs(CorpusFormat::Bash, 100, 95, 92.0)]);
        assert_eq!((rows[0].format.as_str(), rows[0].total, rows[0].passed), ("bash", 100, 95));
        assert!((rows[0].pass_pct - 95.0).abs() < 1e-9); assert_eq!(rows[0].grade, "A");
    }
    #[test] fn test_CORPUS_DISPLAY_018_format_stats_multi() {
        use crate::corpus::registry::CorpusFormat;
        let rows = compute_format_stats(&[
            mfs(CorpusFormat::Bash, 100, 95, 92.0),
            mfs(CorpusFormat::Makefile, 50, 48, 88.0),
            mfs(CorpusFormat::Dockerfile, 30, 30, 99.0),
        ]);
        assert_eq!([rows[0].format.as_str(), rows[1].format.as_str(), rows[2].format.as_str()], ["bash","makefile","dockerfile"]);
    }
    #[test] fn test_CORPUS_DISPLAY_019_format_stats_score() {
        use crate::corpus::registry::CorpusFormat;
        assert!((compute_format_stats(&[mfs(CorpusFormat::Bash, 10, 8, 75.5)])[0].score - 75.5).abs() < 1e-9);
    }
    // Overall stat
    #[test] fn test_CORPUS_DISPLAY_020_overall_basic() {
        let r = compute_overall_stat(100, 0.95, &Grade::A, 92.0);
        assert_eq!(r.total, 100); assert!((r.pass_pct - 95.0).abs() < 1e-9);
        assert_eq!(r.grade, "A"); assert!((r.score - 92.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_021_overall_aplus() {
        assert_eq!(compute_overall_stat(200, 0.99, &Grade::APlus, 99.0).grade, "A+");
    }
    #[test] fn test_CORPUS_DISPLAY_022_overall_failing() {
        let r = compute_overall_stat(50, 0.5, &Grade::F, 45.0);
        assert_eq!(r.grade, "F"); assert!((r.pass_pct - 50.0).abs() < 1e-9);
    }
    // MR results
    #[test] fn test_CORPUS_DISPLAY_023_mr_all_pass() {
        let r = pass("B-1");
        let mr = compute_mr_results(&r, &r);
        assert_eq!((mr.total, mr.passed), (7, 7)); assert!((mr.pass_pct - 100.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_024_mr_all_fail() {
        let r = fail("B-1");
        assert_eq!(compute_mr_results(&r, &r).passed, 1); // only MR-1
    }
    #[test] fn test_CORPUS_DISPLAY_025_mr_nondeterministic() {
        let mut r1 = pass("B-1"); r1.actual_output = Some("v1".into());
        let mut r2 = pass("B-1"); r2.actual_output = Some("v2".into());
        let mr = compute_mr_results(&r1, &r2);
        assert!(!mr.checks[0].passed); assert_eq!(mr.passed, 6);
    }
    #[test] fn test_CORPUS_DISPLAY_026_mr_check_names() {
        let r = pass("B-1");
        let n: Vec<&str> = compute_mr_results(&r, &r).checks.iter().map(|c| c.name).collect();
        assert_eq!(n, ["MR-1 Determinism","MR-2 Transpilation","MR-3 Containment",
            "MR-4 Exact match","MR-5 Behavioral","MR-6 Lint clean","MR-7 Cross-shell"]);
    }
    #[test] fn test_CORPUS_DISPLAY_027_mr_partial() {
        let mut r = pass("B-1"); r.lint_clean = false; r.cross_shell_agree = false;
        let mr = compute_mr_results(&r, &r.clone());
        assert!(!mr.checks[5].passed); assert!(!mr.checks[6].passed); assert_eq!(mr.passed, 5);
    }
    #[test] fn test_CORPUS_DISPLAY_028_mr_pct() {
        let mut r = pass("B-1"); r.output_exact = false;
        assert!((compute_mr_results(&r, &r.clone()).pass_pct - 6.0/7.0*100.0).abs() < 1e-9);
    }
    // Tier distribution
    #[test] fn test_CORPUS_DISPLAY_029_tier_empty() {
        let d = compute_tier_distribution(&[]);
        assert_eq!(d.total, 0); assert_eq!(d.tier_counts, [0; 6]);
    }
    #[test] fn test_CORPUS_DISPLAY_030_tier_single() {
        let d = compute_tier_distribution(&[("B".into(),1u8),("B".into(),1),("B".into(),1)]);
        assert_eq!((d.total, d.tier_counts[1], d.tier_counts[2]), (3, 3, 0));
    }
    #[test] fn test_CORPUS_DISPLAY_031_tier_mixed() {
        let d = compute_tier_distribution(&[("B".into(),1u8),("B".into(),2),("M".into(),3),("D".into(),5)]);
        assert_eq!([d.tier_counts[1], d.tier_counts[2], d.tier_counts[3], d.tier_counts[5]], [1,1,1,1]);
    }
    #[test] fn test_CORPUS_DISPLAY_032_tier_format_breakdown() {
        let d = compute_tier_distribution(&[("B".into(),1u8),("B".into(),2),("M".into(),1)]);
        assert_eq!((d.format_tiers["B"][1], d.format_tiers["B"][2], d.format_tiers["M"][1]), (1,1,1));
    }
    #[test] fn test_CORPUS_DISPLAY_033_tier_clamps() {
        assert_eq!(compute_tier_distribution(&[("B".into(),255u8)]).tier_counts[5], 1);
    }
    #[test] fn test_CORPUS_DISPLAY_034_tier_rows_labels() {
        let rows = tier_distribution_rows(&compute_tier_distribution(&[("B".into(),1u8)]));
        assert_eq!(rows.iter().map(|r| r.label).collect::<Vec<_>>(),
            ["Trivial","Standard","Complex","Adversarial","Production"]);
    }
    #[test] fn test_CORPUS_DISPLAY_035_tier_rows_pct() {
        let rows = tier_distribution_rows(&compute_tier_distribution(
            &[("B".into(),1u8),("B".into(),1),("B".into(),2),("B".into(),2)]));
        assert!((rows[0].pct - 50.0).abs() < 1e-9); assert!((rows[1].pct - 50.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_036_tier_rows_empty() {
        for r in &tier_distribution_rows(&compute_tier_distribution(&[])) {
            assert_eq!(r.count, 0); assert!(r.pct.abs() < 1e-9);
        }
    }
    #[test] fn test_CORPUS_DISPLAY_037_format_tier_basic() {
        let rows = format_tier_rows(&compute_tier_distribution(&[("B".into(),1u8),("B".into(),3),("M".into(),2)]));
        assert_eq!(rows.len(), 2);
        let b = rows.iter().find(|r| r.key == "B").unwrap();
        assert!(b.parts.contains(&"T1:1".into()) && b.parts.contains(&"T3:1".into()));
    }
    #[test] fn test_CORPUS_DISPLAY_038_format_tier_skips_empty() {
        let rows = format_tier_rows(&compute_tier_distribution(&[("B".into(),1u8)]));
        assert_eq!((rows.len(), rows[0].key.as_str()), (1, "B"));
    }
    // Convergence parts
    #[test] fn test_CORPUS_DISPLAY_039_conv_all_formats() {
        let p = format_convergence_parts(&mce(90,100,45,50,28,30,0,0.0,0.9));
        assert_eq!(p.format_parts.len(), 3);
        assert!(p.format_parts.contains(&"Bash 90/100".into()));
        assert!(p.format_parts.contains(&"Make 45/50".into()));
        assert!(p.format_parts.contains(&"Docker 28/30".into()));
    }
    #[test] fn test_CORPUS_DISPLAY_040_conv_single() {
        let p = format_convergence_parts(&mce(90,100,0,0,0,0,0,0.0,0.9));
        assert_eq!((p.format_parts.len(), p.format_parts[0].as_str()), (1, "Bash 90/100"));
    }
    #[test] fn test_CORPUS_DISPLAY_041_conv_no_formats() {
        assert!(format_convergence_parts(&mce(0,0,0,0,0,0,0,0.0,0.0)).format_parts.is_empty());
    }
    #[test] fn test_CORPUS_DISPLAY_042_conv_lint_present() {
        assert!((format_convergence_parts(&mce(90,100,0,0,0,0,85,0.85,0.9)).lint_pct.unwrap() - 85.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_043_conv_lint_absent() {
        let p = format_convergence_parts(&mce(90,100,0,0,0,0,0,0.0,0.9));
        assert!(p.lint_pct.is_none() && p.lint_gap.is_none());
    }
    #[test] fn test_CORPUS_DISPLAY_044_conv_lint_gap_significant() {
        assert!((format_convergence_parts(&mce(90,100,0,0,0,0,80,0.80,0.90)).lint_gap.unwrap() - 10.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_045_conv_lint_gap_negligible() {
        assert!(format_convergence_parts(&mce(90,100,0,0,0,0,90,0.9,0.9)).lint_gap.is_none());
    }
    #[test] fn test_CORPUS_DISPLAY_046_conv_lint_gap_threshold() {
        assert!(format_convergence_parts(&mce(90,100,0,0,0,0,90,0.9,0.9)).lint_gap.is_none());
        if let Some(gap) = format_convergence_parts(&mce(90,100,0,0,0,0,89,0.899,0.9)).lint_gap {
            assert!(gap < 0.2, "near threshold: {gap}");
        }
    }
    // Edge cases
    #[test] fn test_CORPUS_DISPLAY_047_coverage_zero_one() {
        let mut r1 = pass("B-1"); r1.coverage_ratio = 0.0;
        let mut r2 = pass("B-2"); r2.coverage_ratio = 1.0;
        assert!((compute_v2_breakdown(&[r1, r2]).c_avg - 0.5).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_DISPLAY_048_mr_both_none() {
        let mut r = pass("B-1"); r.actual_output = None;
        assert!(compute_mr_results(&r, &r.clone()).checks[0].passed);
    }
    #[test] fn test_CORPUS_DISPLAY_049_mr_none_vs_some() {
        let mut r1 = pass("B-1"); r1.actual_output = None;
        let mut r2 = pass("B-1"); r2.actual_output = Some("hello".into());
        assert!(!compute_mr_results(&r1, &r2).checks[0].passed);
    }
    #[test] fn test_CORPUS_DISPLAY_050_format_stats_zero_total() {
        use crate::corpus::registry::CorpusFormat;
        let r = compute_format_stats(&[mfs(CorpusFormat::Bash, 0, 0, 0.0)]);
        assert_eq!(r[0].total, 0); assert!(r[0].pass_pct.abs() < 1e-9);
    }
}
