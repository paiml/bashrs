//! Compliance scoring model
//!
//! Per-artifact scoring with weighted rules and Popperian gateway barrier.
//! Citation: Popper (1959) §4, Deming (1986) PDCA cycle.

use super::rules::RuleResult;

/// Grade for compliance scores
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grade {
    APlus,
    A,
    B,
    C,
    F,
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::F => write!(f, "F"),
        }
    }
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        if score >= 95.0 {
            Grade::APlus
        } else if score >= 85.0 {
            Grade::A
        } else if score >= 70.0 {
            Grade::B
        } else if score >= 50.0 {
            Grade::C
        } else {
            Grade::F
        }
    }
}

/// Score for a single artifact
#[derive(Clone, Debug)]
pub struct ArtifactScore {
    pub artifact_name: String,
    pub score: f64,
    pub grade: Grade,
    pub rules_tested: usize,
    pub rules_passed: usize,
    pub violations: usize,
    pub results: Vec<RuleResult>,
}

/// Aggregate project score
#[derive(Clone, Debug)]
pub struct ProjectScore {
    pub total_artifacts: usize,
    pub compliant_artifacts: usize,
    pub score: f64,
    pub grade: Grade,
    pub total_falsification_attempts: usize,
    pub successful_falsifications: usize,
    pub artifact_scores: Vec<ArtifactScore>,
}

/// Compute score for a single artifact from its rule results
pub fn compute_artifact_score(name: &str, results: &[RuleResult]) -> ArtifactScore {
    if results.is_empty() {
        return ArtifactScore {
            artifact_name: name.to_string(),
            score: 100.0,
            grade: Grade::APlus,
            rules_tested: 0,
            rules_passed: 0,
            violations: 0,
            results: vec![],
        };
    }

    let total_weight: u32 = results.iter().map(|r| r.rule.weight()).sum();
    let passed_weight: u32 = results
        .iter()
        .filter(|r| r.passed)
        .map(|r| r.rule.weight())
        .sum();

    let score = if total_weight > 0 {
        (passed_weight as f64 / total_weight as f64) * 100.0
    } else {
        100.0
    };

    // Popperian gateway barrier (C1 §4): below 60% is unfalsifiable
    let adjusted_score = if score < 60.0 {
        score * 0.4 // Cap: too many violations to meaningfully assess
    } else {
        score
    };

    let violations: usize = results.iter().map(|r| r.violations.len()).sum();

    ArtifactScore {
        artifact_name: name.to_string(),
        score: adjusted_score,
        grade: Grade::from_score(adjusted_score),
        rules_tested: results.len(),
        rules_passed: results.iter().filter(|r| r.passed).count(),
        violations,
        results: results.to_vec(),
    }
}

/// Compute aggregate project score
pub fn compute_project_score(artifact_scores: Vec<ArtifactScore>) -> ProjectScore {
    if artifact_scores.is_empty() {
        return ProjectScore {
            total_artifacts: 0,
            compliant_artifacts: 0,
            score: 100.0,
            grade: Grade::APlus,
            total_falsification_attempts: 0,
            successful_falsifications: 0,
            artifact_scores: vec![],
        };
    }

    let total = artifact_scores.len();
    let compliant = artifact_scores
        .iter()
        .filter(|s| s.violations == 0)
        .count();
    let total_tested: usize = artifact_scores.iter().map(|s| s.rules_tested).sum();
    let total_violations: usize = artifact_scores.iter().map(|s| s.violations).sum();

    let avg_score = artifact_scores.iter().map(|s| s.score).sum::<f64>() / total as f64;

    ProjectScore {
        total_artifacts: total,
        compliant_artifacts: compliant,
        score: avg_score,
        grade: Grade::from_score(avg_score),
        total_falsification_attempts: total_tested,
        successful_falsifications: total_violations,
        artifact_scores,
    }
}
