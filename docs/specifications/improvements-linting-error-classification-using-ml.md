# Specification: ML-Powered Linting, Error Classification, and Rich Reporting

**Document ID:** BASHRS-SPEC-ML-001
**Version:** 1.0.0
**Status:** IMPLEMENTED
**Created:** 2025-12-07
**Implemented:** 2025-12-07
**Author:** Claude Code + Noah Gift

## Executive Summary

This specification defines enhancements to bashrs for ML-powered error classification, spectrum-based fault localization, and rich ASCII reporting. Following Toyota Production System (TPS) principles, we implement **Jidoka** (automation with human touch) for intelligent error handling and **Visual Management** for immediate feedback through rich terminal output.

## Table of Contents

1. [Motivation](#1-motivation)
2. [Toyota Way Alignment](#2-toyota-way-alignment)
3. [Feature Specifications](#3-feature-specifications)
4. [Implementation Roadmap](#4-implementation-roadmap)
5. [Quality Gates](#5-quality-gates)
6. [References](#6-references)

---

## 1. Motivation

Current bashrs linting provides diagnostic output but lacks:

1. **Intelligent Classification**: Errors are reported individually without clustering or pattern recognition
2. **Root Cause Analysis**: No automated fault localization when multiple issues exist
3. **Learning from Feedback**: No mechanism to improve fix suggestions based on user acceptance
4. **Visual Feedback**: Plain text output without progress visualization or statistical summaries

### 1.1 Problem Statement

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT STATE (Muda - Waste)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  User runs: bashrs lint script.sh                                           │
│                                                                             │
│  Output: 47 individual diagnostics with no clustering                       │
│          No indication which issues block the most progress                 │
│          No learning from which fixes users accept                          │
│          Plain text without visual hierarchy                                │
│                                                                             │
│  Result: User overwhelmed, doesn't know where to start                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Target State

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TARGET STATE (Kaizen)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  User runs: bashrs lint script.sh --rich                                    │
│                                                                             │
│  Output: ╔═══════════════════════════════════════════════════════════════╗  │
│          ║ BASHRS LINT REPORT - script.sh                               ║  │
│          ╠═══════════════════════════════════════════════════════════════╣  │
│          ║ Issues: 47 │ Clusters: 3 │ Top Blocker: SC2086 (31 issues)   ║  │
│          ║ Fix Confidence: 94% │ Auto-fixable: 38/47                    ║  │
│          ╠═══════════════════════════════════════════════════════════════╣  │
│          ║ Cluster Analysis:                                            ║  │
│          ║   ████████████████████░░░░░ SC2086 Quoting (31) - 94% conf   ║  │
│          ║   ██████░░░░░░░░░░░░░░░░░░░ DET001 Random (12) - 87% conf    ║  │
│          ║   ██░░░░░░░░░░░░░░░░░░░░░░░ SEC010 Paths (4)  - 91% conf     ║  │
│          ╚═══════════════════════════════════════════════════════════════╝  │
│                                                                             │
│  Result: User knows exactly where to focus effort (Pareto principle)        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Toyota Way Alignment

This specification follows the 14 principles of the Toyota Way [1]:

| Principle | Application in bashrs |
|-----------|----------------------|
| **Jidoka** (Automation with human touch) | ML classifies errors but human approves fixes |
| **Genchi Genbutsu** (Go and see) | SBFL locates actual fault locations in code |
| **Kaizen** (Continuous improvement) | Oracle learns from user fix acceptance |
| **Heijunka** (Level the workload) | Cluster errors to batch similar fixes |
| **Visual Management** | Rich ASCII dashboards and sparklines |
| **Andon** (Signal problems) | Color-coded severity with visual hierarchy |
| **Poka-yoke** (Error-proofing) | Confidence scores prevent bad auto-fixes |
| **Nemawashi** (Consensus building) | CITL export enables team review |

---

## 3. Feature Specifications

### 3.1 Quality Gate Configuration (`.pmat-gates.toml`)

**Source:** Adapted from ruchy and depyler projects

#### 3.1.1 Specification

```toml
# .pmat-gates.toml - bashrs Quality Gate Configuration
# Toyota Way: Standardized work enables continuous improvement

[metadata]
version = "1.0.0" # Kaizen: Version should support SemVer parsing for future migrations.
tool = "bashrs"

[gates]
# Core quality gates
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 300 # Heijunka: For Tier 1 gates, this is too long. Consider a shorter default (e.g., 60s) for fail-fast.
check_coverage = true
min_coverage = 85.0 # Poka-Yoke: This value needs runtime validation (0.0-100.0) upon loading.
check_complexity = true
max_complexity = 10  # Toyota standard: TEN, not 15, not 20

[gates.satd]
# Self-Admitted Technical Debt (Zero tolerance - Jidoka)
enabled = true
max_count = 0
patterns = ["TODO", "FIXME", "HACK", "XXX"]
require_issue_links = true
fail_on_violation = true

[gates.mutation]
# Mutation Testing (Tier 3 - expensive operations)
enabled = false  # Manual via `make tier3-nightly`
min_score = 85.0
tool = "cargo-mutants"
strategy = "incremental"

[gates.security]
# Security Audits (Poka-yoke)
enabled = true
audit_vulnerabilities = "deny"
audit_unmaintained = "warn"
max_unsafe_blocks = 0

[tiers]
# Tiered enforcement (Heijunka - level the workload)
tier1_gates = ["clippy", "complexity"]           # ON-SAVE (<1s)
tier2_gates = ["clippy", "tests", "coverage"]    # ON-COMMIT (1-5min)
tier3_gates = ["mutation", "security", "satd"]   # NIGHTLY (hours)
# Visual Management: These stringly-typed gate names (`Vec<String>`) should ideally be an enum for compile-time safety.
```

#### 3.1.2 CLI Integration

```bash
# Tier 1: Fast feedback (sub-second)
bashrs gate --tier=1 # Genchi Genbutsu: The `bashrs gate` command must search for `.pmat-gates.toml` in parent directories.
# Respect for People: Error messages for missing config should state where it looked and offer to create a default.
# Built-in Quality: Unit tests are needed for the config loading mechanism to verify parsing of valid/invalid TOML.
# Muda: The current implementation of TOML parsing errors in code loses specific line/column details; these need to be preserved to reduce debugging waste.
# Standardized Work: Inconsistent field naming (`run_clippy` vs `check_coverage`) should be standardized (e.g., `enable_clippy`, `enable_coverage`).
```

---

### 3.2 Tarantula SBFL Fault Localization

**Source:** Adapted from organizational-intelligence-plugin
**Reference:** Jones & Harrold (2005) [2], Abreu et al. (2009) [3]

#### 3.2.1 Theoretical Foundation

Spectrum-Based Fault Localization (SBFL) uses test execution traces to rank code locations by "suspiciousness." The intuition: code executed more by failing tests than passing tests is more likely to contain bugs.

**Tarantula Formula:**
```
suspiciousness(s) = (failed(s)/totalFailed) / ((passed(s)/totalPassed) + (failed(s)/totalFailed))
```

**Ochiai Formula (often superior):**
```
suspiciousness(s) = failed(s) / sqrt(totalFailed × (failed(s) + passed(s)))
```

**DStar Formula (configurable exponent):**
```
suspiciousness(s) = failed(s)^* / (passed(s) + (totalFailed - failed(s)))
```

#### 3.2.2 Data Structures

```rust
/// Statement identifier for fault localization
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatementId {
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub rule_code: Option<String>,  // e.g., "SEC010"
}

/// Coverage data per statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementCoverage {
    pub id: StatementId,
    pub executed_by_passed: usize,
    pub executed_by_failed: usize,
}

/// SBFL formula selection
#[derive(Debug, Clone, Copy, Default)]
pub enum SbflFormula {
    #[default]
    Tarantula,
    Ochiai,
    DStar { exponent: u32 },
}

/// Suspiciousness ranking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousnessRanking {
    pub rank: usize,
    pub statement: StatementId,
    pub suspiciousness: f32,
    pub formula_scores: HashMap<String, f32>,
    pub explanation: String,
}
```

#### 3.2.3 Integration with Linting

```rust
/// Locate most suspicious rules when multiple diagnostics exist
pub fn localize_faults(
    diagnostics: &[Diagnostic],
    test_results: &TestResults,
) -> Vec<SuspiciousnessRanking> {
    // Group diagnostics by rule code
    let rule_coverage = compute_rule_coverage(diagnostics, test_results);

    // Apply SBFL formula
    let rankings = apply_sbfl(rule_coverage, SbflFormula::Ochiai);

    // Return top-N most suspicious
    rankings.into_iter().take(10).collect()
}
```

#### 3.2.4 ASCII Output

```
╔════════════════════════════════════════════════════════════════════════════╗
║                    FAULT LOCALIZATION REPORT (Ochiai)                      ║
╠════════════════════════════════════════════════════════════════════════════╣
║ Rank │ Rule   │ Suspiciousness │ Failed │ Passed │ Explanation             ║
╠══════╪════════╪════════════════╪════════╪════════╪═════════════════════════╣
║  1   │ SC2086 │ ████████░░ 0.94│   31   │    2   │ Quoting prevents 94%    ║
║  2   │ DET001 │ ██████░░░░ 0.72│   12   │    8   │ Random usage blocking   ║
║  3   │ SEC010 │ ████░░░░░░ 0.45│    4   │   12   │ Hardcoded paths         ║
╚══════╧════════╧════════════════╧════════╧════════╧═════════════════════════╝
```

---

### 3.3 Oracle ML-Powered Error Classifier

**Source:** Adapted from ruchy Oracle system
**Reference:** Kim et al. (2013) [4], Le et al. (2016) [5]

#### 3.3.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BASHRS ORACLE ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │   Feature    │───▶│  k-NN + Rule │───▶│  Pattern     │                   │
│  │  Extraction  │    │  Classifier  │    │  Library     │                   │
│  │  (73 feats)  │    │              │    │  (15+ fixes) │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│         │                   │                   │                           │
│         ▼                   ▼                   ▼                           │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        CITL Export (Issue #83)                       │  │
│  │  JSON format for organizational-intelligence-plugin integration      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│         │                                                                   │
│         ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Drift Detection (Hansei)                         │  │
│  │  Monitor fix acceptance rate, retrain when confidence drops          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3.3.2 Error Categories

```rust
/// ML-classified error categories for shell scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellErrorCategory {
    // Security (SEC rules)
    CommandInjection,
    PathTraversal,
    UnsafeExpansion,

    // Determinism (DET rules)
    NonDeterministicRandom,
    TimestampUsage,
    ProcessIdDependency,

    // Idempotency (IDEM rules)
    NonIdempotentOperation,
    MissingGuard,
    UnsafeOverwrite,

    // Quoting (SC2xxx)
    MissingQuotes,
    GlobbingRisk,
    WordSplitting,

    // Other
    SyntaxError,
    StyleViolation,
    Unknown,
}
```

#### 3.3.3 Feature Extraction

```rust
/// Extract 73 features from diagnostic for ML classification
pub fn extract_features(diagnostic: &Diagnostic, source: &str) -> FeatureVector {
    FeatureVector {
        // Lexical features (20)
        code_prefix: extract_code_prefix(&diagnostic.code),
        message_length: diagnostic.message.len(),
        has_variable_reference: diagnostic.message.contains('$'),
        has_path_reference: diagnostic.message.contains('/'),
        // ... 16 more lexical features

        // Structural features (25)
        span_length: diagnostic.span.end_col - diagnostic.span.start_col,
        line_context: extract_line_context(source, diagnostic.span.start_line),
        nesting_depth: compute_nesting_depth(source, diagnostic.span.start_line),
        // ... 22 more structural features

        // Semantic features (28)
        affected_variable: extract_affected_variable(&diagnostic.message),
        operation_type: classify_operation(source, &diagnostic.span),
        control_flow_context: extract_control_flow_context(source, &diagnostic.span),
        // ... 25 more semantic features
    }
}
```

#### 3.3.4 Pattern Library

```rust
/// Fix pattern with success tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPattern {
    pub category: ShellErrorCategory,
    pub pattern_name: String,
    pub regex_match: String,
    pub replacement_template: String,
    pub success_rate: f64,
    pub total_applications: usize,
    pub confidence: f64,
}

/// Bootstrap pattern library (15 initial patterns)
pub fn bootstrap_patterns() -> Vec<FixPattern> {
    vec![
        FixPattern {
            category: ShellErrorCategory::MissingQuotes,
            pattern_name: "quote_variable".to_string(),
            regex_match: r#"\$(\w+)"#.to_string(),
            replacement_template: r#""$${1}""#.to_string(),
            success_rate: 0.94,
            total_applications: 0,
            confidence: 0.90,
        },
        // ... 14 more patterns
    ]
}
```

---

### 3.4 Rich ASCII Reporting and Visualization

**Source:** Adapted from depyler ConvergenceReporter and pmat dashboard
**Reference:** Few (2006) [6], Tufte (2001) [7]

#### 3.4.1 Design Principles

Following Tufte's principles of analytical design [7]:

1. **Show comparisons** - Cluster distributions, before/after
2. **Show causality** - Root cause chains, SBFL rankings
3. **Show multivariate data** - Multiple metrics per diagnostic
4. **Integrate evidence** - Citations, confidence scores
5. **Document everything** - Timestamps, tool versions
6. **Content matters most** - Data density over decoration

#### 3.4.2 Report Components

```rust
/// Rich report with ASCII visualization
pub struct RichLintReport {
    pub header: ReportHeader,
    pub summary: SummaryPanel,
    pub cluster_analysis: ClusterPanel,
    pub fault_localization: SbflPanel,
    pub fix_suggestions: FixPanel,
    pub trend_sparklines: TrendPanel,
    pub footer: ReportFooter,
}

/// ASCII box drawing characters
pub mod box_chars {
    pub const TOP_LEFT: char = '╔';
    pub const TOP_RIGHT: char = '╗';
    pub const BOTTOM_LEFT: char = '╚';
    pub const BOTTOM_RIGHT: char = '╝';
    pub const HORIZONTAL: char = '═';
    pub const VERTICAL: char = '║';
    pub const T_DOWN: char = '╦';
    pub const T_UP: char = '╩';
    pub const T_RIGHT: char = '╠';
    pub const T_LEFT: char = '╣';
    pub const CROSS: char = '╬';
}
```

#### 3.4.3 Sparkline Generation

```rust
/// Generate ASCII sparkline for trend data
pub fn sparkline(data: &[f64], width: usize) -> String {
    const CHARS: &[char] = &[' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    data.iter()
        .map(|&v| {
            let normalized = if range > 0.0 { (v - min) / range } else { 0.5 };
            let index = ((normalized * 7.0).round() as usize).min(7);
            CHARS[index]
        })
        .collect()
}

/// Generate ASCII histogram bar
pub fn histogram_bar(value: f64, max_value: f64, width: usize) -> String {
    let filled = ((value / max_value) * width as f64).round() as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
```

#### 3.4.4 Complete Report Example

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                        BASHRS LINT REPORT v6.42.0                            ║
║                        script.sh │ 2025-12-07 16:45:00                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ SUMMARY                                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Total Issues:    47 │ Errors: 12 │ Warnings: 31 │ Info: 4                   ║
║  Clusters:         3 │ Auto-fixable: 38 (81%) │ Manual: 9 (19%)              ║
║  Confidence:    92.3% │ Est. Fix Time: ~15 min                               ║
║  Trend (7 days):  ▂▃▄▅▆▇█ (improving)                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ ERROR CLUSTERS (Pareto Analysis)                                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Cluster │ Count │ Distribution          │ Category    │ Fix Confidence      ║
╠══════════╪═══════╪═══════════════════════╪═════════════╪═════════════════════╣
║  SC2086  │   31  │ ████████████████████░ │ quoting     │ 94% (auto-fix)      ║
║  DET001  │   12  │ ████████░░░░░░░░░░░░░ │ determinism │ 87% (manual)        ║
║  SEC010  │    4  │ ███░░░░░░░░░░░░░░░░░░ │ security    │ 91% (auto-fix)      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ FAULT LOCALIZATION (Ochiai SBFL)                                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Rank │ Location          │ Suspiciousness │ Root Cause                      ║
╠═══════╪═══════════════════╪════════════════╪═════════════════════════════════╣
║    1  │ script.sh:45      │ ████████░░ 0.94│ Unquoted $RANDOM in loop        ║
║    2  │ script.sh:12-18   │ ██████░░░░ 0.72│ Timestamp in filename           ║
║    3  │ script.sh:89      │ ████░░░░░░ 0.45│ Hardcoded /tmp path             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ RECOMMENDED ACTIONS (Toyota Way: Start with highest impact)                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  1. Run: bashrs lint script.sh --fix                                         ║
║     → Auto-fixes 38 issues (SC2086, SEC010)                                  ║
║                                                                              ║
║  2. Manual review required for DET001 (12 issues)                            ║
║     → Replace $RANDOM with deterministic seed                                ║
║     → Replace $(date) with fixed timestamp parameter                         ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ CITL EXPORT                                                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Export: bashrs lint script.sh --citl-export diagnostics.json                ║
║  Integration: organizational-intelligence-plugin for ML training             ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

### 3.5 Graph Statistics and Control Flow Analysis

**Source:** Adapted from pmat complexity_enhanced.rs
**Reference:** McCabe (1976) [8], Watson & Wallace (1996) [9]

#### 3.5.1 Metrics Computed

| Metric | Formula | Threshold | Reference |
|--------|---------|-----------|-----------|
| Cyclomatic Complexity | E - N + 2P | ≤ 10 | McCabe (1976) [8] |
| Essential Complexity | # of SCCs with >1 node | ≤ 4 | Watson & Wallace (1996) [9] |
| Cognitive Complexity | Weighted nesting depth | ≤ 15 | Shepperd (1988) [10] |
| Halstead Volume | N × log₂(n) | Informational | Halstead (1977) [11] |

#### 3.5.2 Control Flow Graph Generation

```rust
/// Generate CFG for shell script
pub fn build_cfg(ast: &ShellAst) -> ControlFlowGraph {
    let mut graph = DiGraph::new();
    let entry = graph.add_node(CfgNode::Entry);
    let exit = graph.add_node(CfgNode::Exit);

    let mut builder = CfgBuilder::new(graph, entry, exit);
    builder.visit_script(ast);

    ControlFlowGraph {
        graph: builder.graph,
        entry,
        exit,
    }
}

/// Compute graph statistics
pub fn compute_graph_stats(cfg: &ControlFlowGraph) -> GraphStats {
    GraphStats {
        nodes: cfg.graph.node_count(),
        edges: cfg.graph.edge_count(),
        cyclomatic: cfg.cyclomatic_complexity(),
        essential: cfg.essential_complexity(),
        strongly_connected_components: kosaraju_scc(&cfg.graph).len(),
        max_depth: compute_max_depth(&cfg.graph),
    }
}
```

#### 3.5.3 ASCII CFG Visualization

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                     CONTROL FLOW GRAPH - script.sh                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║                              ┌─────────┐                                     ║
║                              │  ENTRY  │                                     ║
║                              └────┬────┘                                     ║
║                                   │                                          ║
║                              ┌────▼────┐                                     ║
║                              │ if cond │                                     ║
║                              └────┬────┘                                     ║
║                         ┌────────┼────────┐                                  ║
║                         │ TRUE   │  FALSE │                                  ║
║                    ┌────▼────┐   │   ┌────▼────┐                             ║
║                    │ block A │   │   │ block B │                             ║
║                    └────┬────┘   │   └────┬────┘                             ║
║                         └────────┼────────┘                                  ║
║                              ┌───▼────┐                                      ║
║                              │  EXIT  │                                      ║
║                              └────────┘                                      ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Nodes: 6 │ Edges: 7 │ Cyclomatic: 2 │ Essential: 0 │ Max Depth: 2           ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

### 3.6 ML Clustering for Error Pattern Discovery

**Reference:** Arthur & Vassilvitskii (2007) [12], Ester et al. (1996) [13]

#### 3.6.1 Clustering Algorithms

```rust
/// Error clustering using k-means++ initialization
pub struct ErrorClusterer {
    pub algorithm: ClusteringAlgorithm,
    pub distance_metric: DistanceMetric,
    pub min_cluster_size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ClusteringAlgorithm {
    KMeansPlusPlus { k: usize },
    DBSCAN { eps: f64, min_samples: usize },
    Hierarchical { linkage: Linkage },
}

#[derive(Debug, Clone, Copy)]
pub enum DistanceMetric {
    Euclidean,
    Cosine,
    Jaccard,
}
```

#### 3.6.2 Cluster Analysis Output

```rust
/// Error cluster with root cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCluster {
    pub cluster_id: usize,
    pub error_code: String,
    pub centroid: FeatureVector,
    pub members: Vec<Diagnostic>,
    pub examples_blocked: Vec<String>,
    pub root_cause: RootCause,
    pub fix_confidence: f64,
    pub sample_errors: Vec<ErrorSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RootCause {
    TranspilerGap { gap_type: String, location: String },
    MissingRule { rule_id: String },
    FalsePositive { reason: String },
    Unknown,
}
```

---

## 4. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

| Task ID | Description | Effort | Priority |
|---------|-------------|--------|----------|
| ML-001 | Implement `.pmat-gates.toml` parser | 4h | P0 |
| ML-002 | Add `bashrs gate` CLI command | 4h | P0 |
| ML-003 | Integrate tiered quality gates | 8h | P0 |

### Phase 2: SBFL Integration (Week 3-4)

| Task ID | Description | Effort | Priority |
|---------|-------------|--------|----------|
| ML-004 | Implement Tarantula/Ochiai formulas | 4h | P1 |
| ML-005 | Add coverage tracking per rule | 8h | P1 |
| ML-006 | Create SBFL ASCII report | 4h | P1 |

### Phase 3: Oracle ML (Week 5-8)

| Task ID | Description | Effort | Priority |
|---------|-------------|--------|----------|
| ML-007 | Implement 73-feature extraction | 8h | P1 |
| ML-008 | Build k-NN classifier | 8h | P1 |
| ML-009 | Create pattern library (15 patterns) | 8h | P1 |
| ML-010 | Add drift detection | 4h | P2 |

### Phase 4: Rich Reporting (Week 9-10)

| Task ID | Description | Effort | Priority |
|---------|-------------|--------|----------|
| ML-011 | Implement ASCII box drawing | 4h | P1 |
| ML-012 | Add sparkline generation | 2h | P1 |
| ML-013 | Create histogram bars | 2h | P1 |
| ML-014 | Build complete rich report | 8h | P1 |

### Phase 5: Graph Analysis (Week 11-12)

| Task ID | Description | Effort | Priority |
|---------|-------------|--------|----------|
| ML-015 | Build shell CFG generator | 8h | P2 |
| ML-016 | Implement complexity metrics | 4h | P2 |
| ML-017 | Add ASCII CFG visualization | 4h | P2 |

---

## 5. Quality Gates

### 5.1 Acceptance Criteria

| Criterion | Threshold | Measurement |
|-----------|-----------|-------------|
| Test Coverage | ≥ 85% | `cargo llvm-cov` |
| Mutation Score | ≥ 80% | `cargo mutants` |
| Cyclomatic Complexity | ≤ 10 | `pmat analyze complexity` |
| SBFL Accuracy | ≥ 70% EXAM score | Benchmark suite |
| Oracle Classification F1 | ≥ 0.85 | Cross-validation |
| Report Render Time | < 100ms | Benchmark |

### 5.2 Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Property: SBFL rankings are deterministic
    proptest! {
        #[test]
        fn prop_sbfl_deterministic(
            diagnostics in prop::collection::vec(arb_diagnostic(), 1..100),
            test_results in arb_test_results(),
        ) {
            let ranking1 = localize_faults(&diagnostics, &test_results);
            let ranking2 = localize_faults(&diagnostics, &test_results);
            prop_assert_eq!(ranking1, ranking2);
        }
    }

    // Property: Rich report never panics
    proptest! {
        #[test]
        fn prop_rich_report_never_panics(
            result in arb_lint_result(),
        ) {
            let report = RichLintReport::from_lint_result(&result);
            let _ = report.render(); // Should not panic
        }
    }
}
```

---

## 6. References

1. Liker, J. K. (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill.

2. Jones, J. A., & Harrold, M. J. (2005). Empirical evaluation of the Tarantula automatic fault-localization technique. *Proceedings of ASE '05*, 273-282. https://doi.org/10.1145/1101908.1101949

3. Abreu, R., Zoeteweij, P., & Van Gemund, A. J. (2009). Spectrum-based multiple fault localization. *Proceedings of ASE '09*, 88-99. https://doi.org/10.1109/ASE.2009.25

4. Kim, D., Tao, Y., Kim, S., & Zeller, A. (2013). Where should we fix this bug? A two-phase recommendation model. *IEEE Transactions on Software Engineering*, 39(11), 1597-1610. https://doi.org/10.1109/TSE.2013.24

5. Le, T. D. B., Lo, D., Le Goues, C., & Grunske, L. (2016). A learning-to-rank based fault localization approach using likely invariants. *Proceedings of ISSTA '16*, 177-188. https://doi.org/10.1145/2931037.2931049

6. Few, S. (2006). *Information Dashboard Design: The Effective Visual Communication of Data*. O'Reilly Media.

7. Tufte, E. R. (2001). *The Visual Display of Quantitative Information* (2nd ed.). Graphics Press.

8. McCabe, T. J. (1976). A complexity measure. *IEEE Transactions on Software Engineering*, SE-2(4), 308-320. https://doi.org/10.1109/TSE.1976.233837

9. Watson, A. H., & Wallace, D. R. (1996). A critique of cyclomatic complexity as a software metric. *NIST Special Publication 500-235*.

10. Shepperd, M. (1988). A critique of cyclomatic complexity as a software metric. *Software Engineering Journal*, 3(2), 30-36. https://doi.org/10.1049/sej.1988.0003

11. Halstead, M. H. (1977). *Elements of Software Science*. Elsevier North-Holland.

12. Arthur, D., & Vassilvitskii, S. (2007). k-means++: The advantages of careful seeding. *Proceedings of SODA '07*, 1027-1035.

13. Ester, M., Kriegel, H. P., Sander, J., & Xu, X. (1996). A density-based algorithm for discovering clusters in large spatial databases with noise. *Proceedings of KDD '96*, 226-231.

---

## Appendix A: ASCII Character Reference

```
Box Drawing (Double):
╔ ═ ╗    TOP_LEFT, HORIZONTAL, TOP_RIGHT
║   ║    VERTICAL
╠ ═ ╣    T_RIGHT, HORIZONTAL, T_LEFT
╚ ═ ╝    BOTTOM_LEFT, HORIZONTAL, BOTTOM_RIGHT
╦ ╩ ╬    T_DOWN, T_UP, CROSS

Progress Bars:
█ Full block     (U+2588)
░ Light shade    (U+2591)
▓ Dark shade     (U+2593)

Sparklines:
 ▂▃▄▅▆▇█  (U+2581 through U+2588)

Status Icons:
✓ Check mark     (U+2713)
✗ X mark         (U+2717)
⚠ Warning        (U+26A0)
● Bullet         (U+25CF)
○ Circle         (U+25CB)
```

---

## Appendix B: CITL Integration Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CITL Export Schema",
  "type": "object",
  "required": ["version", "source_file", "diagnostics"],
  "properties": {
    "version": { "type": "string", "const": "1.0.0" },
    "source_file": { "type": "string" },
    "timestamp": { "type": "integer" },
    "tool": { "type": "string", "const": "bashrs" },
    "tool_version": { "type": "string" },
    "diagnostics": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["error_code", "level", "message"],
        "properties": {
          "error_code": { "type": "string" },
          "level": { "enum": ["error", "warning", "info"] },
          "message": { "type": "string" },
          "oip_category": { "type": "string" },
          "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
          "span": {
            "type": "object",
            "properties": {
              "start_line": { "type": "integer" },
              "start_col": { "type": "integer" },
              "end_line": { "type": "integer" },
              "end_col": { "type": "integer" }
            }
          },
          "suggestion": {
            "type": "object",
            "properties": {
              "replacement": { "type": "string" },
              "description": { "type": "string" },
              "is_safe": { "type": "boolean" }
            }
          }
        }
      }
    },
    "summary": {
      "type": "object",
      "properties": {
        "total": { "type": "integer" },
        "errors": { "type": "integer" },
        "warnings": { "type": "integer" },
        "info": { "type": "integer" }
      }
    }
  }
}
```

---

*Document generated following EXTREME TDD methodology*
*Toyota Way principles applied throughout*