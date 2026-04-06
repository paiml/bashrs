use super::*;

#[test]
fn test_ml_004_tarantula_basic() {
    let mut localizer = FaultLocalizer::with_tarantula();
    localizer.set_test_counts(10, 2);

    // Location covered by 2 failing tests and 2 passing tests
    let coverage = CoverageData::new(2, 2, 8, 0);
    let score = localizer.calculate_suspiciousness(&coverage);

    // Should be highly suspicious since all failures cover it
    assert!(score > 0.5);
}

#[test]
fn test_ml_004_ochiai_basic() {
    let mut localizer = FaultLocalizer::with_ochiai();
    localizer.set_test_counts(10, 2);

    // Location covered by all failing tests, no passing tests
    let coverage = CoverageData::new(0, 2, 10, 0);
    let score = localizer.calculate_suspiciousness(&coverage);

    // Perfect suspicious score
    assert_eq!(score, 1.0);
}

#[test]
fn test_ml_004_ochiai_no_fails() {
    let mut localizer = FaultLocalizer::with_ochiai();
    localizer.set_test_counts(10, 0);

    let coverage = CoverageData::new(5, 0, 5, 0);
    let score = localizer.calculate_suspiciousness(&coverage);

    // No failures means no suspiciousness
    assert_eq!(score, 0.0);
}

#[test]
fn test_ml_004_jaccard() {
    let localizer = FaultLocalizer::new(SbflFormula::Jaccard);

    // ef=2, ep=0, nf=2 -> 2/(2+0+0) = 1.0
    let _coverage = CoverageData::new(0, 2, 0, 0);
    let score = localizer.jaccard(2.0, 0.0, 2.0);

    assert_eq!(score, 1.0);
}

#[test]
fn test_ml_004_ranking() {
    let mut localizer = FaultLocalizer::with_ochiai();
    localizer.set_test_counts(8, 2);

    // Most suspicious - covered by both failures
    localizer.add_coverage("file.rs:10".to_string(), CoverageData::new(0, 2, 8, 0));

    // Less suspicious - covered by one failure
    localizer.add_coverage("file.rs:20".to_string(), CoverageData::new(4, 1, 4, 1));

    // Not suspicious - no failures
    localizer.add_coverage("file.rs:30".to_string(), CoverageData::new(5, 0, 3, 2));

    let rankings = localizer.rank();

    assert_eq!(rankings.len(), 3);
    assert_eq!(rankings[0].location, "file.rs:10");
    assert_eq!(rankings[0].rank, 1);
    assert!(rankings[0].score > rankings[1].score);
    assert!(rankings[1].score > rankings[2].score);
}

#[test]
fn test_ml_004_top_n() {
    let mut localizer = FaultLocalizer::with_ochiai();
    localizer.set_test_counts(10, 2);

    for i in 0..10 {
        localizer.add_coverage(
            format!("file.rs:{}", i * 10),
            CoverageData::new(i as u32, (10 - i) as u32, 0, 0),
        );
    }

    let top_3 = localizer.top_n(3);
    assert_eq!(top_3.len(), 3);

    // Higher failed_covering should rank higher
    assert!(top_3[0].coverage.failed_covering >= top_3[1].coverage.failed_covering);
}

#[test]
fn test_ml_004_localize_faults() {
    let coverage_data = vec![
        (
            "test1".to_string(),
            true,
            vec!["a.rs:1".to_string(), "a.rs:2".to_string()],
        ),
        (
            "test2".to_string(),
            true,
            vec!["a.rs:1".to_string(), "a.rs:3".to_string()],
        ),
        (
            "test3".to_string(),
            false,
            vec!["a.rs:2".to_string(), "a.rs:3".to_string()],
        ),
    ];

    let rankings = localize_faults(&coverage_data, SbflFormula::Ochiai);

    assert!(!rankings.is_empty());

    // a.rs:2 and a.rs:3 are covered by the failing test
    // a.rs:1 is only covered by passing tests
    let suspicious: Vec<_> = rankings.iter().filter(|r| r.score > 0.0).collect();
    assert!(!suspicious.is_empty());
}

#[test]
fn test_ml_004_suspiciousness_ranking_traits() {
    let ranking = SuspiciousnessRanking {
        location: "test.rs:10".to_string(),
        score: 0.85,
        coverage: CoverageData::default(),
        rank: 1,
    };

    assert!(ranking.is_highly_suspicious());
    assert!(ranking.is_moderately_suspicious());

    let low_ranking = SuspiciousnessRanking {
        location: "test.rs:20".to_string(),
        score: 0.3,
        coverage: CoverageData::default(),
        rank: 5,
    };

    assert!(!low_ranking.is_highly_suspicious());
    assert!(!low_ranking.is_moderately_suspicious());
}

#[test]
fn test_ml_004_dstar_formula() {
    let localizer = FaultLocalizer::new(SbflFormula::DStar { power: 2 });

    // D* with power 2: ef^2 / (nf - ef + ep)
    // ef=2, ep=0, nf=2 -> 4 / (2-2+0) = infinity
    let score = localizer.dstar(2.0, 0.0, 2.0, 2);
    assert!(score.is_infinite());

    // ef=2, ep=2, nf=4 -> 4 / (4-2+2) = 1.0
    let score2 = localizer.dstar(2.0, 2.0, 4.0, 2);
    assert_eq!(score2, 1.0);
}

#[test]
fn test_ml_004_formula_display() {
    assert_eq!(format!("{}", SbflFormula::Tarantula), "Tarantula");
    assert_eq!(format!("{}", SbflFormula::Ochiai), "Ochiai");
    assert_eq!(format!("{}", SbflFormula::DStar { power: 2 }), "D*2");
}

// ===== Additional tests for coverage =====

#[test]
fn test_formula_display_jaccard() {
    assert_eq!(format!("{}", SbflFormula::Jaccard), "Jaccard");
}

#[test]
fn test_formula_display_wong2() {
    assert_eq!(format!("{}", SbflFormula::Wong2), "Wong-II");
}

#[test]
fn test_formula_default() {
    let formula = SbflFormula::default();
    assert_eq!(formula, SbflFormula::Ochiai);
}

#[test]
fn test_coverage_data_default() {
    let coverage = CoverageData::default();
    assert_eq!(coverage.passed_covering, 0);
    assert_eq!(coverage.failed_covering, 0);
    assert_eq!(coverage.passed_not_covering, 0);
    assert_eq!(coverage.failed_not_covering, 0);
}

#[test]
fn test_coverage_data_total_passed() {
    let coverage = CoverageData::new(3, 2, 7, 1);
    assert_eq!(coverage.total_passed(), 10);
}

#[test]
fn test_coverage_data_total_failed() {
    let coverage = CoverageData::new(3, 2, 7, 1);
    assert_eq!(coverage.total_failed(), 3);
}

#[test]
fn test_coverage_data_total_covering() {
    let coverage = CoverageData::new(3, 2, 7, 1);
    assert_eq!(coverage.total_covering(), 5);
}

#[test]
fn test_wong2_formula() {
    let localizer = FaultLocalizer::new(SbflFormula::Wong2);

    // Wong-II: ef - ep
    let score = localizer.wong2(5.0, 3.0);
    assert_eq!(score, 2.0);

    let score_negative = localizer.wong2(2.0, 5.0);
    assert_eq!(score_negative, -3.0);
}

#[test]
fn test_tarantula_zero_passed() {
    let mut localizer = FaultLocalizer::with_tarantula();
    localizer.set_test_counts(0, 5);

    let coverage = CoverageData::new(0, 3, 0, 2);
    let score = localizer.calculate_suspiciousness(&coverage);

    // With zero passed tests, formula should still work
    assert!(score >= 0.0);
}

#[test]
fn test_tarantula_zero_denom() {
    let localizer = FaultLocalizer::with_tarantula();

    // ef=0, ep=0 -> denominator = 0
    let score = localizer.tarantula(0.0, 0.0, 2.0, 2.0);
    assert_eq!(score, 0.0);
}

#[test]
fn test_jaccard_zero_denom() {
    let localizer = FaultLocalizer::new(SbflFormula::Jaccard);

    // ef=0, nf-ef=0, ep=0 -> denominator = 0
    let score = localizer.jaccard(0.0, 0.0, 0.0);
    assert_eq!(score, 0.0);
}

#[test]
fn test_dstar_zero_ef() {
    let localizer = FaultLocalizer::new(SbflFormula::DStar { power: 2 });

    // ef=0 -> result = 0
    let score = localizer.dstar(0.0, 5.0, 5.0, 2);
    assert_eq!(score, 0.0);
}

#[test]
fn test_dstar_power_3() {
    let localizer = FaultLocalizer::new(SbflFormula::DStar { power: 3 });

    // ef=2, power=3 -> 8 / denom
    let score = localizer.dstar(2.0, 2.0, 4.0, 3);
    assert_eq!(score, 2.0); // 8 / (4-2+2) = 8/4 = 2
}

#[test]
fn test_record_coverage() {
    let mut localizer = FaultLocalizer::with_ochiai();

    localizer.record_coverage("file.rs:10", true);
    localizer.record_coverage("file.rs:10", true);
    localizer.record_coverage("file.rs:10", false);

    let rankings = localizer.rank();
    assert_eq!(rankings.len(), 1);
    assert_eq!(rankings[0].coverage.passed_covering, 2);
    assert_eq!(rankings[0].coverage.failed_covering, 1);
}

#[test]
fn test_above_threshold() {
    let mut localizer = FaultLocalizer::with_ochiai();
    localizer.set_test_counts(10, 2);

    localizer.add_coverage("high.rs:1".to_string(), CoverageData::new(0, 2, 10, 0));
    localizer.add_coverage("medium.rs:1".to_string(), CoverageData::new(5, 1, 5, 1));
    localizer.add_coverage("low.rs:1".to_string(), CoverageData::new(8, 0, 2, 2));

    let above_half = localizer.above_threshold(0.5);
    assert!(!above_half.is_empty());
    assert!(above_half.iter().all(|r| r.score >= 0.5));
}

#[test]
fn test_suspiciousness_ranking_clone() {
    let ranking = SuspiciousnessRanking {
        location: "test.rs:5".to_string(),
        score: 0.75,
        coverage: CoverageData::new(1, 2, 3, 4),
        rank: 2,
    };

    let cloned = ranking.clone();
    assert_eq!(cloned.location, "test.rs:5");
    assert_eq!(cloned.score, 0.75);
    assert_eq!(cloned.rank, 2);
}

#[test]
fn test_fault_localizer_empty() {
    let localizer = FaultLocalizer::with_ochiai();
    let rankings = localizer.rank();
    assert!(rankings.is_empty());
}

#[test]
fn test_localize_faults_empty() {
    let coverage_data: Vec<(String, bool, Vec<String>)> = vec![];
    let rankings = localize_faults(&coverage_data, SbflFormula::Ochiai);
    assert!(rankings.is_empty());
}

#[test]
fn test_localize_faults_all_passing() {
    let coverage_data = vec![
        ("test1".to_string(), true, vec!["a.rs:1".to_string()]),
        ("test2".to_string(), true, vec!["a.rs:1".to_string()]),
    ];

    let rankings = localize_faults(&coverage_data, SbflFormula::Ochiai);

    // With all passing tests, suspiciousness should be 0
    assert!(rankings.iter().all(|r| r.score == 0.0));
}

#[test]
fn test_formula_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SbflFormula::Tarantula);
    set.insert(SbflFormula::Ochiai);
    set.insert(SbflFormula::Jaccard);
    set.insert(SbflFormula::Wong2);
    set.insert(SbflFormula::DStar { power: 2 });
    assert_eq!(set.len(), 5);
}

#[test]
fn test_formula_clone() {
    let formula = SbflFormula::DStar { power: 3 };
    let cloned = formula;
    assert_eq!(cloned, SbflFormula::DStar { power: 3 });
}

#[test]
fn test_coverage_data_clone() {
    let coverage = CoverageData::new(1, 2, 3, 4);
    let cloned = coverage.clone();
    assert_eq!(cloned.passed_covering, 1);
    assert_eq!(cloned.failed_covering, 2);
    assert_eq!(cloned.passed_not_covering, 3);
    assert_eq!(cloned.failed_not_covering, 4);
}
