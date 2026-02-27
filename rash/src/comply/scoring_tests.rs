//! Tests for comply/scoring.rs — coverage of scoring functions and Grade enum
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::rules::{RuleId, RuleResult, Violation};
use super::scoring::*;

fn make_result(rule: RuleId, passed: bool) -> RuleResult {
    RuleResult {
        rule,
        passed,
        violations: if passed {
            vec![]
        } else {
            vec![Violation {
                rule,
                line: Some(1),
                message: format!("{:?} violation", rule),
            }]
        },
    }
}

// ===== Grade ===== //

#[test]
fn test_grade_display_all_variants() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::F), "F");
}

#[test]
fn test_grade_from_score_boundaries() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(95.0), Grade::APlus);
    assert_eq!(Grade::from_score(94.9), Grade::A);
    assert_eq!(Grade::from_score(85.0), Grade::A);
    assert_eq!(Grade::from_score(84.9), Grade::B);
    assert_eq!(Grade::from_score(70.0), Grade::B);
    assert_eq!(Grade::from_score(69.9), Grade::C);
    assert_eq!(Grade::from_score(50.0), Grade::C);
    assert_eq!(Grade::from_score(49.9), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

// ===== compute_artifact_score ===== //

#[test]
fn test_artifact_score_empty_results() {
    let score = compute_artifact_score("test.sh", &[]);
    assert_eq!(score.artifact_name, "test.sh");
    assert_eq!(score.score, 100.0);
    assert_eq!(score.grade, Grade::APlus);
    assert_eq!(score.rules_tested, 0);
    assert_eq!(score.rules_passed, 0);
    assert_eq!(score.violations, 0);
}

#[test]
fn test_artifact_score_all_passed() {
    let results = vec![
        make_result(RuleId::Posix, true),
        make_result(RuleId::Determinism, true),
        make_result(RuleId::Security, true),
    ];
    let score = compute_artifact_score("clean.sh", &results);
    assert_eq!(score.score, 100.0);
    assert_eq!(score.grade, Grade::APlus);
    assert_eq!(score.rules_tested, 3);
    assert_eq!(score.rules_passed, 3);
    assert_eq!(score.violations, 0);
}

#[test]
fn test_artifact_score_some_failed() {
    let results = vec![
        make_result(RuleId::Posix, true),     // weight 20
        make_result(RuleId::Determinism, false), // weight 15
        make_result(RuleId::Security, true),   // weight 20
    ];
    let score = compute_artifact_score("partial.sh", &results);
    // passed_weight = 20 + 20 = 40, total_weight = 20 + 15 + 20 = 55
    // score = 40/55 * 100 = 72.7...
    assert!(score.score > 72.0 && score.score < 73.0);
    assert_eq!(score.grade, Grade::B);
    assert_eq!(score.rules_tested, 3);
    assert_eq!(score.rules_passed, 2);
    assert_eq!(score.violations, 1);
}

#[test]
fn test_artifact_score_all_failed_popperian_barrier() {
    let results = vec![
        make_result(RuleId::Posix, false),
        make_result(RuleId::Determinism, false),
        make_result(RuleId::Security, false),
        make_result(RuleId::Quoting, false),
        make_result(RuleId::Idempotency, false),
    ];
    let score = compute_artifact_score("terrible.sh", &results);
    // All failed: score = 0.0, below 60% → multiplied by 0.4
    assert_eq!(score.score, 0.0);
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.violations, 5);
}

#[test]
fn test_artifact_score_below_60_gateway() {
    // Just security (20) passed out of posix(20)+det(15)+security(20)+quoting(10)+idem(15) = 80
    // score = 20/80 * 100 = 25%, which is < 60%, gets multiplied by 0.4 → 10%
    let results = vec![
        make_result(RuleId::Posix, false),
        make_result(RuleId::Determinism, false),
        make_result(RuleId::Security, true),
        make_result(RuleId::Quoting, false),
        make_result(RuleId::Idempotency, false),
    ];
    let score = compute_artifact_score("bad.sh", &results);
    assert!(score.score < 20.0, "Score {} should be capped below 20", score.score);
    assert_eq!(score.grade, Grade::F);
}

#[test]
fn test_artifact_score_above_60_no_gateway() {
    // posix(20) + security(20) passed out of posix(20)+security(20)+quoting(10) = 50
    // score = 40/50 * 100 = 80% ≥ 60%, no gateway penalty
    let results = vec![
        make_result(RuleId::Posix, true),
        make_result(RuleId::Security, true),
        make_result(RuleId::Quoting, false),
    ];
    let score = compute_artifact_score("decent.sh", &results);
    assert_eq!(score.score, 80.0);
    assert_eq!(score.grade, Grade::B);
}

// ===== compute_project_score ===== //

#[test]
fn test_project_score_empty() {
    let score = compute_project_score(vec![]);
    assert_eq!(score.total_artifacts, 0);
    assert_eq!(score.score, 100.0);
    assert_eq!(score.grade, Grade::APlus);
}

#[test]
fn test_project_score_single_artifact() {
    let results = vec![make_result(RuleId::Posix, true)];
    let artifact = compute_artifact_score("test.sh", &results);
    let project = compute_project_score(vec![artifact]);
    assert_eq!(project.total_artifacts, 1);
    assert_eq!(project.compliant_artifacts, 1);
    assert_eq!(project.score, 100.0);
    assert_eq!(project.grade, Grade::APlus);
}

#[test]
fn test_project_score_multiple_mixed() {
    let good = compute_artifact_score("good.sh", &[make_result(RuleId::Posix, true)]);
    let bad = compute_artifact_score("bad.sh", &[
        make_result(RuleId::Posix, false),
        make_result(RuleId::Security, false),
    ]);
    let project = compute_project_score(vec![good, bad]);
    assert_eq!(project.total_artifacts, 2);
    assert_eq!(project.compliant_artifacts, 1);
    // avg_score = (100.0 + 0.0) / 2 = 50.0 (but bad has gateway penalty)
    assert!(project.score < 60.0);
}

#[test]
fn test_project_score_all_compliant() {
    let a1 = compute_artifact_score("a.sh", &[make_result(RuleId::Posix, true)]);
    let a2 = compute_artifact_score("b.sh", &[make_result(RuleId::Security, true)]);
    let a3 = compute_artifact_score("Makefile", &[make_result(RuleId::MakefileSafety, true)]);
    let project = compute_project_score(vec![a1, a2, a3]);
    assert_eq!(project.total_artifacts, 3);
    assert_eq!(project.compliant_artifacts, 3);
    assert_eq!(project.score, 100.0);
    assert_eq!(project.grade, Grade::APlus);
    assert_eq!(project.successful_falsifications, 0);
}

#[test]
fn test_project_score_falsification_counts() {
    let r = vec![
        make_result(RuleId::Posix, true),
        make_result(RuleId::Determinism, false),
    ];
    let artifact = compute_artifact_score("test.sh", &r);
    let project = compute_project_score(vec![artifact]);
    assert_eq!(project.total_falsification_attempts, 2);
    assert_eq!(project.successful_falsifications, 1);
}
