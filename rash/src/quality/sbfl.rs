//! Spectrum-Based Fault Localization (ML-004)
//!
//! Implements Tarantula and Ochiai SBFL formulas for ranking suspicious
//! code locations based on test coverage data.
//!
//! # Toyota Way Principles
//!
//! - **Genchi Genbutsu** (Go and see): Data-driven fault identification
//! - **Five Whys**: Systematic root cause analysis
//! - **Jidoka**: Automated defect detection
//!
//! # References
//!
//! - Jones, J.A. & Harrold, M.J. (2005). "Empirical evaluation of the
//!   Tarantula automatic fault-localization technique"
//! - Abreu, R., Zoeteweij, P., & van Gemund, A.J.C. (2007). "On the
//!   accuracy of spectrum-based fault localization"
//! - Wong, W.E. et al. (2016). "A survey on software fault localization"
//!
//! # Formulas
//!
//! ## Tarantula
//! ```text
//! suspiciousness = (failed_covering / total_failed) /
//!                  ((failed_covering / total_failed) + (passed_covering / total_passed))
//! ```
//!
//! ## Ochiai
//! ```text
//! suspiciousness = failed_covering / sqrt(total_failed * (failed_covering + passed_covering))
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SBFL formula type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SbflFormula {
    /// Tarantula formula (Jones & Harrold, 2005)
    Tarantula,
    /// Ochiai formula (Abreu et al., 2007)
    #[default]
    Ochiai,
    /// Jaccard similarity coefficient
    Jaccard,
    /// Wong-II formula
    Wong2,
    /// DStar formula (Wong et al., 2014)
    DStar { power: u32 },
}

impl std::fmt::Display for SbflFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SbflFormula::Tarantula => write!(f, "Tarantula"),
            SbflFormula::Ochiai => write!(f, "Ochiai"),
            SbflFormula::Jaccard => write!(f, "Jaccard"),
            SbflFormula::Wong2 => write!(f, "Wong-II"),
            SbflFormula::DStar { power } => write!(f, "D*{}", power),
        }
    }
}

/// Coverage data for a single program element (line, function, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageData {
    /// Number of passing tests that cover this element
    pub passed_covering: u32,
    /// Number of failing tests that cover this element
    pub failed_covering: u32,
    /// Number of passing tests that do not cover this element
    pub passed_not_covering: u32,
    /// Number of failing tests that do not cover this element
    pub failed_not_covering: u32,
}

impl CoverageData {
    /// Create new coverage data
    pub fn new(
        passed_covering: u32,
        failed_covering: u32,
        passed_not_covering: u32,
        failed_not_covering: u32,
    ) -> Self {
        Self {
            passed_covering,
            failed_covering,
            passed_not_covering,
            failed_not_covering,
        }
    }

    /// Total number of passing tests
    pub fn total_passed(&self) -> u32 {
        self.passed_covering + self.passed_not_covering
    }

    /// Total number of failing tests
    pub fn total_failed(&self) -> u32 {
        self.failed_covering + self.failed_not_covering
    }

    /// Total tests covering this element
    pub fn total_covering(&self) -> u32 {
        self.passed_covering + self.failed_covering
    }
}

/// A ranked element with suspiciousness score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousnessRanking {
    /// Location identifier (file:line or function name)
    pub location: String,
    /// Suspiciousness score (0.0 to 1.0)
    pub score: f64,
    /// Raw coverage data
    pub coverage: CoverageData,
    /// Rank (1 = most suspicious)
    pub rank: usize,
}

impl SuspiciousnessRanking {
    /// Check if this location is highly suspicious (>= 0.8)
    pub fn is_highly_suspicious(&self) -> bool {
        self.score >= 0.8
    }

    /// Check if this location is moderately suspicious (>= 0.5)
    pub fn is_moderately_suspicious(&self) -> bool {
        self.score >= 0.5
    }
}

/// Fault localizer using SBFL techniques
pub struct FaultLocalizer {
    /// Formula to use for calculation
    formula: SbflFormula,
    /// Coverage data per location
    coverage: HashMap<String, CoverageData>,
    /// Total passing tests
    total_passed: u32,
    /// Total failing tests
    total_failed: u32,
}

impl FaultLocalizer {
    /// Create a new fault localizer with the specified formula
    pub fn new(formula: SbflFormula) -> Self {
        Self {
            formula,
            coverage: HashMap::new(),
            total_passed: 0,
            total_failed: 0,
        }
    }

    /// Create with default Ochiai formula
    pub fn with_ochiai() -> Self {
        Self::new(SbflFormula::Ochiai)
    }

    /// Create with Tarantula formula
    pub fn with_tarantula() -> Self {
        Self::new(SbflFormula::Tarantula)
    }

    /// Set total test counts
    pub fn set_test_counts(&mut self, total_passed: u32, total_failed: u32) {
        self.total_passed = total_passed;
        self.total_failed = total_failed;
    }

    /// Add coverage data for a location
    pub fn add_coverage(&mut self, location: String, coverage: CoverageData) {
        self.coverage.insert(location, coverage);
    }

    /// Record that a location was covered by a test
    pub fn record_coverage(&mut self, location: &str, test_passed: bool) {
        let entry = self.coverage.entry(location.to_string()).or_default();
        if test_passed {
            entry.passed_covering += 1;
        } else {
            entry.failed_covering += 1;
        }
    }

    /// Calculate suspiciousness for a location
    pub fn calculate_suspiciousness(&self, coverage: &CoverageData) -> f64 {
        let ef = coverage.failed_covering as f64;
        let ep = coverage.passed_covering as f64;
        let nf = self.total_failed as f64;
        let np = self.total_passed as f64;

        match self.formula {
            SbflFormula::Tarantula => self.tarantula(ef, ep, nf, np),
            SbflFormula::Ochiai => self.ochiai(ef, ep, nf),
            SbflFormula::Jaccard => self.jaccard(ef, ep, nf),
            SbflFormula::Wong2 => self.wong2(ef, ep),
            SbflFormula::DStar { power } => self.dstar(ef, ep, nf, power),
        }
    }

    /// Tarantula formula implementation
    fn tarantula(&self, ef: f64, ep: f64, nf: f64, np: f64) -> f64 {
        if nf == 0.0 {
            return 0.0;
        }

        let failed_ratio = ef / nf;
        let passed_ratio = if np > 0.0 { ep / np } else { 0.0 };

        let denom = failed_ratio + passed_ratio;
        if denom == 0.0 {
            0.0
        } else {
            failed_ratio / denom
        }
    }

    /// Ochiai formula implementation
    fn ochiai(&self, ef: f64, ep: f64, nf: f64) -> f64 {
        let denom = (nf * (ef + ep)).sqrt();
        if denom == 0.0 {
            0.0
        } else {
            ef / denom
        }
    }

    /// Jaccard similarity coefficient
    fn jaccard(&self, ef: f64, ep: f64, nf: f64) -> f64 {
        let denom = ef + (nf - ef) + ep;
        if denom == 0.0 {
            0.0
        } else {
            ef / denom
        }
    }

    /// Wong-II formula
    fn wong2(&self, ef: f64, ep: f64) -> f64 {
        ef - ep
    }

    /// D* formula
    fn dstar(&self, ef: f64, ep: f64, nf: f64, power: u32) -> f64 {
        let denom = (nf - ef) + ep;
        if denom == 0.0 {
            if ef > 0.0 {
                f64::INFINITY
            } else {
                0.0
            }
        } else {
            ef.powi(power as i32) / denom
        }
    }

    /// Get ranked suspicious locations
    pub fn rank(&self) -> Vec<SuspiciousnessRanking> {
        let mut rankings: Vec<_> = self
            .coverage
            .iter()
            .map(|(location, coverage)| {
                let score = self.calculate_suspiciousness(coverage);
                SuspiciousnessRanking {
                    location: location.clone(),
                    score,
                    coverage: coverage.clone(),
                    rank: 0, // Will be set after sorting
                }
            })
            .collect();

        // Sort by score descending
        rankings.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = i + 1;
        }

        rankings
    }

    /// Get top N suspicious locations
    pub fn top_n(&self, n: usize) -> Vec<SuspiciousnessRanking> {
        self.rank().into_iter().take(n).collect()
    }

    /// Get locations above a suspiciousness threshold
    pub fn above_threshold(&self, threshold: f64) -> Vec<SuspiciousnessRanking> {
        self.rank()
            .into_iter()
            .filter(|r| r.score >= threshold)
            .collect()
    }
}

/// Helper function to create fault localizer from test results
pub fn localize_faults(
    coverage_data: &[(String, bool, Vec<String>)], // (test_name, passed, covered_locations)
    formula: SbflFormula,
) -> Vec<SuspiciousnessRanking> {
    let total_passed = coverage_data
        .iter()
        .filter(|(_, passed, _)| *passed)
        .count() as u32;
    let total_failed = coverage_data
        .iter()
        .filter(|(_, passed, _)| !*passed)
        .count() as u32;

    let mut localizer = FaultLocalizer::new(formula);
    localizer.set_test_counts(total_passed, total_failed);

    // Aggregate coverage data
    let mut location_coverage: HashMap<String, CoverageData> = HashMap::new();

    for (_test_name, passed, locations) in coverage_data {
        // Track which locations were covered by this test
        let locations_set: std::collections::HashSet<_> = locations.iter().collect();

        for loc in locations_set {
            let entry = location_coverage.entry(loc.clone()).or_default();
            if *passed {
                entry.passed_covering += 1;
            } else {
                entry.failed_covering += 1;
            }
        }
    }

    // Update not-covering counts
    for coverage in location_coverage.values_mut() {
        coverage.passed_not_covering = total_passed.saturating_sub(coverage.passed_covering);
        coverage.failed_not_covering = total_failed.saturating_sub(coverage.failed_covering);
    }

    for (location, coverage) in location_coverage {
        localizer.add_coverage(location, coverage);
    }

    localizer.rank()
}

#[cfg(test)]
mod tests {
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
        let cloned = formula.clone();
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
}
