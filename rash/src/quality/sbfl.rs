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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "sbfl_tests_ml_004.rs"]
// FIXME(PMAT-238): mod tests_extracted;
