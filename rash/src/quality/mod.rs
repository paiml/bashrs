//! Quality Gates Module (ML-001 through ML-017)
//!
//! Implements tiered quality gate enforcement following Toyota Production System
//! principles. Gates are configured via `.pmat-gates.toml` and enforced at three tiers:
//!
//! - **Tier 1 (ON-SAVE)**: Sub-second checks (clippy, complexity)
//! - **Tier 2 (ON-COMMIT)**: 1-5 minute checks (tests, coverage, SATD)
//! - **Tier 3 (NIGHTLY)**: Hours (mutation testing, security audit)
//!
//! # Features
//!
//! - **Quality Gates** (ML-001 to ML-003): Tiered enforcement with configurable thresholds
//! - **SBFL Fault Localization** (ML-004 to ML-006): Tarantula, Ochiai formulas
//! - **Oracle ML Classification** (ML-007 to ML-010): k-NN classifier, pattern library, drift detection
//! - **Rich ASCII Reporting** (ML-011 to ML-014): Box drawing, sparklines, histograms
//! - **Control Flow Analysis** (ML-015 to ML-017): CFG generation, complexity metrics
//!
//! # Toyota Way Principles
//!
//! - **Jidoka** (Automation with human touch): ML classifies, human approves
//! - **Kaizen** (Continuous improvement): Learn from fix acceptance
//! - **Visual Management** (Mieruka): Rich ASCII dashboards
//! - **Genchi Genbutsu** (Go and see): SBFL locates actual faults
//! - **Poka-yoke** (Error-proofing): Confidence scores prevent bad fixes
//!
//! # References
//!
//! - BASHRS-SPEC-ML-001: ML-Powered Linting specification
//! - Jones & Harrold (2005): Tarantula fault localization
//! - Abreu et al. (2007): Ochiai formula
//! - McCabe (1976): Cyclomatic complexity
//! - Kim et al. (2013): Bug localization with learning-to-rank

// Core quality gates
pub mod gates;

// Rich ASCII reporting
pub mod report;

// SBFL fault localization
pub mod sbfl;

// Oracle ML classification
pub mod oracle;

// Rich lint report with clustering
pub mod lint_report;

// Control flow graph and complexity
pub mod cfg;

// Re-exports for convenience
pub use cfg::{
    render_cfg_ascii, CfgBuilder, CfgEdge, CfgNode, ComplexityGrade, ComplexityMetrics,
    ControlFlowGraph,
};
pub use gates::{GateConfig, GateResult, GateSummary, QualityGate, Tier};
pub use lint_report::{histogram_bar, sbfl_report, ErrorCluster, RichLintReport};
pub use oracle::{
    bootstrap_patterns, ClassificationResult, DriftDetector, DriftStatus, FeatureVector,
    FixPattern, KnnClassifier, Oracle, ShellErrorCategory,
};
pub use report::{gate_report, sparkline, Grade, ReportBuilder, RichReport};
pub use sbfl::{CoverageData, FaultLocalizer, SbflFormula, SuspiciousnessRanking};
