//! Corpus registry types for transpilation quality measurement.
//!
//! Defines `CorpusEntry` and `CorpusRegistry` following the depyler corpus
//! pattern (Gift, 2025) with metadata for quality tracking, tier assignment,
//! and falsification protocol support.

use serde::{Deserialize, Serialize};

/// Target transpilation format for a corpus entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorpusFormat {
    /// POSIX shell (purified bash)
    Bash,
    /// GNU Makefile
    Makefile,
    /// Dockerfile
    Dockerfile,
}

impl std::fmt::Display for CorpusFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Makefile => write!(f, "makefile"),
            Self::Dockerfile => write!(f, "dockerfile"),
        }
    }
}

/// Difficulty tier for a corpus entry (progressive difficulty, Vygotsky 1978).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CorpusTier {
    /// Tier 1: Single constructs (10-20 LOC), 100% expected pass rate
    Trivial = 1,
    /// Tier 2: Common patterns (20-100 LOC), 99% expected pass rate
    Standard = 2,
    /// Tier 3: Real-world programs (100-500 LOC), 98% expected pass rate
    Complex = 3,
    /// Tier 4: Edge cases, injection attempts, Unicode, 95% expected pass rate
    Adversarial = 4,
    /// Tier 5: Full production scripts, 95% expected pass rate
    Production = 5,
}

impl CorpusTier {
    /// Scoring weight for aggregate calculations (Pareto principle, Juran 1951).
    /// Higher tiers contribute more to overall score.
    pub fn weight(&self) -> f64 {
        match self {
            Self::Trivial => 1.0,
            Self::Standard => 1.5,
            Self::Complex => 2.0,
            Self::Adversarial => 2.5,
            Self::Production => 3.0,
        }
    }

    /// Expected minimum pass rate for this tier.
    pub fn target_rate(&self) -> f64 {
        match self {
            Self::Trivial => 1.0,
            Self::Standard => 0.99,
            Self::Complex => 0.98,
            Self::Adversarial => 0.95,
            Self::Production => 0.95,
        }
    }
}

/// Quality grade derived from 100-point score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    /// 97-100: Production-ready, fully validated
    APlus,
    /// 90-96: Near-production, minor gaps
    A,
    /// 80-89: Good quality, known limitations
    B,
    /// 70-79: Functional, significant gaps
    C,
    /// 60-69: Partially functional
    D,
    /// <60: Not yet viable
    F,
}

impl Grade {
    /// Derive grade from a 100-point score.
    pub fn from_score(score: f64) -> Self {
        if score >= 97.0 {
            Self::APlus
        } else if score >= 90.0 {
            Self::A
        } else if score >= 80.0 {
            Self::B
        } else if score >= 70.0 {
            Self::C
        } else if score >= 60.0 {
            Self::D
        } else {
            Self::F
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::APlus => write!(f, "A+"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::F => write!(f, "F"),
        }
    }
}

/// A single corpus entry: an input-output pair that serves as a potential falsifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// Unique identifier (e.g., "B-001", "M-042", "D-015")
    pub id: String,
    /// Human-readable name (e.g., "hello-world")
    pub name: String,
    /// Description of what this entry tests
    pub description: String,
    /// Target transpilation format
    pub format: CorpusFormat,
    /// Difficulty tier
    pub tier: CorpusTier,
    /// Rust DSL source code (the input)
    pub input: String,
    /// Expected transpiled output (the prediction)
    pub expected_output: String,
    /// Whether this entry's output must pass shellcheck (Bash only)
    pub shellcheck: bool,
    /// Whether this entry's output must be deterministic
    pub deterministic: bool,
    /// Whether this entry's output must be idempotent
    pub idempotent: bool,
}

include!("mod_incl2.rs");
