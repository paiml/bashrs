//! Graph-aware corpus prioritization (§11.10.3).
//!
//! Enriches each corpus failure with **construct connectivity** — how many
//! entries exercise each decision type.  Decisions that appear in many entries
//! have higher blast radius, so fixing them yields greater impact.
//!
//! **Priority formula**: `priority(d) = suspiciousness(d) × log2(1 + usage_count(d))`

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How many entries share a given decision type:choice key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConnectivity {
    /// Decision key, e.g. "assignment_value:bool_literal"
    pub decision: String,
    /// Number of traced entries that exercise this decision
    pub usage_count: usize,
    /// Which entry IDs exercise it
    pub entry_ids: Vec<String>,
    /// Whether usage_count exceeds the high-connectivity threshold
    pub is_high_connectivity: bool,
}

/// A decision ranked by graph-weighted priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPriority {
    /// Decision key
    pub decision: String,
    /// Tarantula suspiciousness score (0.0–1.0)
    pub suspiciousness: f64,
    /// Corpus-wide usage count
    pub usage_count: usize,
    /// `suspiciousness × log2(1 + usage_count)`
    pub priority: f64,
    /// "HIGH" / "MEDIUM" / "LOW"
    pub impact: String,
    /// Entry IDs exercising this decision
    pub entry_ids: Vec<String>,
}

/// Full graph analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Priorities sorted descending by priority score
    pub priorities: Vec<GraphPriority>,
    /// Total unique decision keys observed
    pub total_decisions: usize,
    /// Total traced entries
    pub total_entries: usize,
    /// Threshold above which a decision is "high connectivity"
    pub high_connectivity_threshold: usize,
}

/// Default high-connectivity threshold.
const HIGH_CONNECTIVITY_THRESHOLD: usize = 5;

/// Build decision → entry_ids connectivity map from coverage data.
///
/// `coverage_data` is `(entry_id, passed, decision_locations)` as returned
/// by `collect_trace_coverage()`.
pub fn build_connectivity(
    coverage_data: &[(String, bool, Vec<String>)],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (entry_id, _passed, locations) in coverage_data {
        for loc in locations {
            map.entry(loc.clone())
                .or_default()
                .insert(entry_id.clone());
        }
    }
    map
}

/// Compute graph-weighted priorities by combining Tarantula suspiciousness
/// scores with decision connectivity.
///
/// `rankings` come from `localize_faults()` (§11.10.1).
/// `connectivity` comes from `build_connectivity()`.
pub fn compute_graph_priorities(
    rankings: &[crate::quality::sbfl::SuspiciousnessRanking],
    connectivity: &BTreeMap<String, BTreeSet<String>>,
    total_entries: usize,
) -> GraphAnalysis {
    let mut priorities: Vec<GraphPriority> = rankings
        .iter()
        .map(|r| {
            let usage_count = connectivity
                .get(&r.location)
                .map_or(0, |s| s.len());
            let entry_ids: Vec<String> = connectivity
                .get(&r.location)
                .map(|s| {
                    let mut v: Vec<String> = s.iter().cloned().collect();
                    v.sort();
                    v
                })
                .unwrap_or_default();
            let priority = priority_formula(r.score, usage_count);
            let impact = classify_impact(priority);
            GraphPriority {
                decision: r.location.clone(),
                suspiciousness: r.score,
                usage_count,
                priority,
                impact,
                entry_ids,
            }
        })
        .collect();

    priorities.sort_by(|a, b| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    GraphAnalysis {
        total_decisions: connectivity.len(),
        total_entries,
        high_connectivity_threshold: HIGH_CONNECTIVITY_THRESHOLD,
        priorities,
    }
}

/// `suspiciousness × log2(1 + usage_count)`
pub fn priority_formula(suspiciousness: f64, usage_count: usize) -> f64 {
    suspiciousness * (1.0 + usage_count as f64).log2()
}

/// Classify a graph priority score into impact tier.
fn classify_impact(priority: f64) -> String {
    if priority >= 3.0 {
        "HIGH".to_string()
    } else if priority >= 1.5 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    }
}

/// Build a `Vec<DecisionConnectivity>` from the raw connectivity map,
/// sorted descending by usage count.
pub fn connectivity_table(
    connectivity: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<DecisionConnectivity> {
    let mut table: Vec<DecisionConnectivity> = connectivity
        .iter()
        .map(|(decision, ids)| {
            let mut entry_ids: Vec<String> = ids.iter().cloned().collect();
            entry_ids.sort();
            DecisionConnectivity {
                decision: decision.clone(),
                usage_count: ids.len(),
                entry_ids,
                is_high_connectivity: ids.len() >= HIGH_CONNECTIVITY_THRESHOLD,
            }
        })
        .collect();
    table.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    table
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_coverage() -> Vec<(String, bool, Vec<String>)> {
        vec![
            (
                "B-001".to_string(),
                true,
                vec![
                    "ir_dispatch:Assignment".to_string(),
                    "value_emit:string".to_string(),
                ],
            ),
            (
                "B-002".to_string(),
                true,
                vec![
                    "ir_dispatch:Assignment".to_string(),
                    "ir_dispatch:If".to_string(),
                ],
            ),
            (
                "B-003".to_string(),
                false,
                vec![
                    "ir_dispatch:Assignment".to_string(),
                    "assignment_value:bool_literal".to_string(),
                ],
            ),
        ]
    }

    #[test]
    fn test_build_connectivity_counts() {
        let data = sample_coverage();
        let conn = build_connectivity(&data);
        assert_eq!(conn.get("ir_dispatch:Assignment").unwrap().len(), 3);
        assert_eq!(conn.get("value_emit:string").unwrap().len(), 1);
        assert_eq!(conn.get("ir_dispatch:If").unwrap().len(), 1);
        assert_eq!(
            conn.get("assignment_value:bool_literal").unwrap().len(),
            1
        );
    }

    #[test]
    fn test_priority_formula_basic() {
        // log2(1 + 0) = 0, so priority = 0
        assert!((priority_formula(0.9, 0) - 0.0).abs() < 1e-10);
        // log2(1 + 1) = 1.0, so priority = 0.9
        assert!((priority_formula(0.9, 1) - 0.9).abs() < 1e-10);
        // log2(1 + 7) = 3.0, so priority = 0.9 * 3.0 = 2.7
        assert!((priority_formula(0.9, 7) - 2.7).abs() < 1e-10);
    }

    #[test]
    fn test_priority_formula_zero_suspiciousness() {
        assert!((priority_formula(0.0, 100) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_connectivity_table_sorted_descending() {
        let data = sample_coverage();
        let conn = build_connectivity(&data);
        let table = connectivity_table(&conn);
        assert!(table[0].usage_count >= table[1].usage_count);
        assert_eq!(table[0].decision, "ir_dispatch:Assignment");
        assert_eq!(table[0].usage_count, 3);
    }

    #[test]
    fn test_connectivity_table_high_connectivity_flag() {
        let data = sample_coverage();
        let conn = build_connectivity(&data);
        let table = connectivity_table(&conn);
        // With only 3 entries, none should exceed threshold of 5
        for row in &table {
            assert!(!row.is_high_connectivity);
        }
    }

    #[test]
    fn test_classify_impact_tiers() {
        assert_eq!(classify_impact(4.0), "HIGH");
        assert_eq!(classify_impact(3.0), "HIGH");
        assert_eq!(classify_impact(2.0), "MEDIUM");
        assert_eq!(classify_impact(1.5), "MEDIUM");
        assert_eq!(classify_impact(1.0), "LOW");
        assert_eq!(classify_impact(0.0), "LOW");
    }

    #[test]
    fn test_compute_graph_priorities_integration() {
        use crate::quality::sbfl::{CoverageData, SuspiciousnessRanking};

        let data = sample_coverage();
        let conn = build_connectivity(&data);

        let rankings = vec![
            SuspiciousnessRanking {
                location: "ir_dispatch:Assignment".to_string(),
                score: 0.5,
                coverage: CoverageData::default(),
                rank: 1,
            },
            SuspiciousnessRanking {
                location: "assignment_value:bool_literal".to_string(),
                score: 0.9,
                coverage: CoverageData::default(),
                rank: 2,
            },
        ];

        let analysis = compute_graph_priorities(&rankings, &conn, 3);
        assert_eq!(analysis.total_entries, 3);
        assert_eq!(analysis.priorities.len(), 2);

        // ir_dispatch:Assignment: 0.5 * log2(4) = 0.5 * 2.0 = 1.0
        // assignment_value:bool_literal: 0.9 * log2(2) = 0.9 * 1.0 = 0.9
        // So ir_dispatch:Assignment should rank first (higher priority)
        assert_eq!(analysis.priorities[0].decision, "ir_dispatch:Assignment");
        assert!((analysis.priorities[0].priority - 1.0).abs() < 1e-10);
        assert_eq!(
            analysis.priorities[1].decision,
            "assignment_value:bool_literal"
        );
        assert!((analysis.priorities[1].priority - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_compute_graph_priorities_empty() {
        let conn = BTreeMap::new();
        let rankings = vec![];
        let analysis = compute_graph_priorities(&rankings, &conn, 0);
        assert!(analysis.priorities.is_empty());
        assert_eq!(analysis.total_entries, 0);
        assert_eq!(analysis.total_decisions, 0);
    }

    #[test]
    fn test_build_connectivity_empty() {
        let data: Vec<(String, bool, Vec<String>)> = vec![];
        let conn = build_connectivity(&data);
        assert!(conn.is_empty());
    }

    #[test]
    fn test_connectivity_table_entry_ids_sorted() {
        let data = vec![
            ("B-003".to_string(), true, vec!["x:y".to_string()]),
            ("B-001".to_string(), true, vec!["x:y".to_string()]),
            ("B-002".to_string(), true, vec!["x:y".to_string()]),
        ];
        let conn = build_connectivity(&data);
        let table = connectivity_table(&conn);
        assert_eq!(table[0].entry_ids, vec!["B-001", "B-002", "B-003"]);
    }
}
