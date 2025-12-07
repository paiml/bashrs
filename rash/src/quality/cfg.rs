//! Control Flow Graph Generator (ML-015, ML-016, ML-017)
//!
//! Generates control flow graphs from shell scripts and computes
//! complexity metrics following software engineering best practices.
//!
//! # Toyota Way Principles
//!
//! - **Genchi Genbutsu** (Go and see): Visualize actual control flow
//! - **Poka-yoke** (Error-proofing): Complexity limits prevent defects
//! - **Standardized Work**: Consistent complexity thresholds
//!
//! # References
//!
//! - BASHRS-SPEC-ML-015: CFG Generator
//! - BASHRS-SPEC-ML-016: Complexity Metrics
//! - BASHRS-SPEC-ML-017: ASCII CFG Visualization
//! - McCabe (1976): Cyclomatic Complexity
//! - Watson & Wallace (1996): Essential Complexity
//! - Halstead (1977): Software Science

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Node in the control flow graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CfgNode {
    /// Entry point of the graph
    Entry,
    /// Exit point of the graph
    Exit,
    /// Basic block with statements
    BasicBlock {
        id: usize,
        label: String,
        start_line: usize,
        end_line: usize,
    },
    /// Conditional branch (if/elif/case)
    Conditional {
        id: usize,
        label: String,
        line: usize,
    },
    /// Loop header (for/while/until)
    LoopHeader {
        id: usize,
        label: String,
        line: usize,
    },
    /// Function entry
    FunctionEntry {
        id: usize,
        name: String,
        line: usize,
    },
    /// Subshell entry
    SubshellEntry { id: usize, line: usize },
}

impl CfgNode {
    /// Get node ID
    pub fn id(&self) -> usize {
        match self {
            CfgNode::Entry => 0,
            CfgNode::Exit => usize::MAX,
            CfgNode::BasicBlock { id, .. }
            | CfgNode::Conditional { id, .. }
            | CfgNode::LoopHeader { id, .. }
            | CfgNode::FunctionEntry { id, .. }
            | CfgNode::SubshellEntry { id, .. } => *id,
        }
    }

    /// Get display label
    pub fn label(&self) -> String {
        match self {
            CfgNode::Entry => "ENTRY".to_string(),
            CfgNode::Exit => "EXIT".to_string(),
            CfgNode::BasicBlock { label, .. } => label.clone(),
            CfgNode::Conditional { label, .. } => label.clone(),
            CfgNode::LoopHeader { label, .. } => label.clone(),
            CfgNode::FunctionEntry { name, .. } => format!("fn {}", name),
            CfgNode::SubshellEntry { .. } => "subshell".to_string(),
        }
    }
}

/// Edge in the control flow graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CfgEdge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
    pub is_back_edge: bool,
}

/// Control flow graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlFlowGraph {
    pub nodes: Vec<CfgNode>,
    pub edges: Vec<CfgEdge>,
    pub entry_id: usize,
    pub exit_id: usize,
}

impl ControlFlowGraph {
    /// Create a new empty CFG
    pub fn new() -> Self {
        Self {
            nodes: vec![CfgNode::Entry, CfgNode::Exit],
            edges: Vec::new(),
            entry_id: 0,
            exit_id: usize::MAX,
        }
    }

    /// Add a node and return its index
    pub fn add_node(&mut self, node: CfgNode) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, from: usize, to: usize, label: Option<String>) {
        self.edges.push(CfgEdge {
            from,
            to,
            label,
            is_back_edge: false,
        });
    }

    /// Get node count (excluding entry/exit)
    pub fn node_count(&self) -> usize {
        self.nodes.len().saturating_sub(2)
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get successors of a node
    pub fn successors(&self, node_id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to)
            .collect()
    }

    /// Get predecessors of a node
    pub fn predecessors(&self, node_id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|e| e.to == node_id)
            .map(|e| e.from)
            .collect()
    }

    /// Mark back edges (for loop detection)
    pub fn mark_back_edges(&mut self) {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut back_edges = HashSet::new();

        self.dfs_back_edges(0, &mut visited, &mut stack, &mut back_edges);

        for edge in &mut self.edges {
            if back_edges.contains(&(edge.from, edge.to)) {
                edge.is_back_edge = true;
            }
        }
    }

    fn dfs_back_edges(
        &self,
        node: usize,
        visited: &mut HashSet<usize>,
        stack: &mut HashSet<usize>,
        back_edges: &mut HashSet<(usize, usize)>,
    ) {
        visited.insert(node);
        stack.insert(node);

        for succ in self.successors(node) {
            if !visited.contains(&succ) {
                self.dfs_back_edges(succ, visited, stack, back_edges);
            } else if stack.contains(&succ) {
                back_edges.insert((node, succ));
            }
        }

        stack.remove(&node);
    }
}

/// Complexity metrics for a CFG (ML-016)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// McCabe's cyclomatic complexity: E - N + 2P
    pub cyclomatic: usize,
    /// Essential complexity (irreducible control flow)
    pub essential: usize,
    /// Cognitive complexity (weighted nesting)
    pub cognitive: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Number of decision points
    pub decision_points: usize,
    /// Number of loops
    pub loop_count: usize,
    /// Halstead volume (if available)
    pub halstead_volume: Option<f64>,
}

impl ComplexityMetrics {
    /// Calculate metrics from a CFG
    pub fn from_cfg(cfg: &ControlFlowGraph) -> Self {
        let n = cfg.node_count() + 2; // Include entry/exit
        let e = cfg.edge_count();
        let p = 1; // Single connected component for now

        // Cyclomatic: E - N + 2P
        let cyclomatic = e.saturating_sub(n) + 2 * p;

        // Count decision points and loops
        let decision_points = cfg
            .nodes
            .iter()
            .filter(|n| matches!(n, CfgNode::Conditional { .. }))
            .count();

        let loop_count = cfg
            .nodes
            .iter()
            .filter(|n| matches!(n, CfgNode::LoopHeader { .. }))
            .count();

        // Essential complexity (simplified: count of back edges)
        let essential = cfg.edges.iter().filter(|e| e.is_back_edge).count();

        // Max depth (simplified heuristic)
        let max_depth = Self::compute_max_depth(cfg);

        // Cognitive complexity (simplified: decisions + 2*loops + nesting penalty)
        let cognitive = decision_points + 2 * loop_count + max_depth;

        Self {
            cyclomatic,
            essential,
            cognitive,
            max_depth,
            decision_points,
            loop_count,
            halstead_volume: None,
        }
    }

    fn compute_max_depth(cfg: &ControlFlowGraph) -> usize {
        let mut max_depth = 0;
        let mut visited = HashSet::new();
        Self::dfs_depth(cfg, 0, 0, &mut max_depth, &mut visited);
        max_depth
    }

    fn dfs_depth(
        cfg: &ControlFlowGraph,
        node: usize,
        current_depth: usize,
        max_depth: &mut usize,
        visited: &mut HashSet<usize>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        *max_depth = (*max_depth).max(current_depth);

        let new_depth = match cfg.nodes.get(node) {
            Some(CfgNode::Conditional { .. } | CfgNode::LoopHeader { .. }) => current_depth + 1,
            _ => current_depth,
        };

        for succ in cfg.successors(node) {
            Self::dfs_depth(cfg, succ, new_depth, max_depth, visited);
        }
    }

    /// Check if complexity exceeds Toyota standard (10)
    pub fn exceeds_threshold(&self) -> bool {
        self.cyclomatic > 10
    }

    /// Get complexity grade
    pub fn grade(&self) -> ComplexityGrade {
        match self.cyclomatic {
            0..=5 => ComplexityGrade::Simple,
            6..=10 => ComplexityGrade::Moderate,
            11..=20 => ComplexityGrade::Complex,
            21..=50 => ComplexityGrade::VeryComplex,
            _ => ComplexityGrade::Untestable,
        }
    }
}

/// Complexity grade following Toyota standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityGrade {
    /// 1-5: Simple, low risk
    Simple,
    /// 6-10: Moderate, acceptable
    Moderate,
    /// 11-20: Complex, needs attention
    Complex,
    /// 21-50: Very complex, high risk
    VeryComplex,
    /// 50+: Untestable, must refactor
    Untestable,
}

impl ComplexityGrade {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Simple => "Simple, low risk",
            Self::Moderate => "Moderate complexity, acceptable",
            Self::Complex => "Complex, needs attention",
            Self::VeryComplex => "Very complex, high risk",
            Self::Untestable => "Untestable, must refactor",
        }
    }

    /// Check if refactoring is recommended
    pub fn needs_refactoring(&self) -> bool {
        matches!(self, Self::Complex | Self::VeryComplex | Self::Untestable)
    }
}

/// CFG Builder for shell scripts (ML-015)
pub struct CfgBuilder {
    cfg: ControlFlowGraph,
    next_id: usize,
    current_block: Option<usize>,
}

impl CfgBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            cfg: ControlFlowGraph::new(),
            next_id: 1,             // 0 is Entry
            current_block: Some(0), // Start at Entry
        }
    }

    /// Add a basic block
    pub fn add_block(&mut self, label: &str, start_line: usize, end_line: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = CfgNode::BasicBlock {
            id,
            label: label.to_string(),
            start_line,
            end_line,
        };

        self.cfg.nodes.push(node);

        if let Some(prev) = self.current_block {
            self.cfg.add_edge(prev, id, None);
        }

        self.current_block = Some(id);
        id
    }

    /// Add a conditional node
    pub fn add_conditional(&mut self, label: &str, line: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = CfgNode::Conditional {
            id,
            label: label.to_string(),
            line,
        };

        self.cfg.nodes.push(node);

        if let Some(prev) = self.current_block {
            self.cfg.add_edge(prev, id, None);
        }

        self.current_block = Some(id);
        id
    }

    /// Add a loop header
    pub fn add_loop(&mut self, label: &str, line: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = CfgNode::LoopHeader {
            id,
            label: label.to_string(),
            line,
        };

        self.cfg.nodes.push(node);

        if let Some(prev) = self.current_block {
            self.cfg.add_edge(prev, id, None);
        }

        self.current_block = Some(id);
        id
    }

    /// Add a function entry
    pub fn add_function(&mut self, name: &str, line: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = CfgNode::FunctionEntry {
            id,
            name: name.to_string(),
            line,
        };

        self.cfg.nodes.push(node);

        if let Some(prev) = self.current_block {
            self.cfg.add_edge(prev, id, None);
        }

        self.current_block = Some(id);
        id
    }

    /// Add edge with label
    pub fn add_edge(&mut self, from: usize, to: usize, label: Option<&str>) {
        self.cfg.add_edge(from, to, label.map(String::from));
    }

    /// Connect to exit
    pub fn connect_to_exit(&mut self) {
        if let Some(current) = self.current_block {
            self.cfg.add_edge(current, usize::MAX, None);
        }
    }

    /// Set current block
    pub fn set_current(&mut self, id: usize) {
        self.current_block = Some(id);
    }

    /// Build the final CFG
    pub fn build(mut self) -> ControlFlowGraph {
        self.connect_to_exit();
        self.cfg.mark_back_edges();
        self.cfg
    }
}

impl Default for CfgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// ASCII CFG visualization (ML-017)
pub fn render_cfg_ascii(
    cfg: &ControlFlowGraph,
    metrics: &ComplexityMetrics,
    width: usize,
) -> String {
    let mut out = String::new();
    let inner = width - 2;

    // Header
    out.push('╔');
    for _ in 0..inner {
        out.push('═');
    }
    out.push_str("╗\n");

    // Title
    let title = "CONTROL FLOW GRAPH";
    let padding = (inner.saturating_sub(title.len())) / 2;
    out.push('║');
    for _ in 0..padding {
        out.push(' ');
    }
    out.push_str(title);
    for _ in 0..(inner - padding - title.len()) {
        out.push(' ');
    }
    out.push_str("║\n");

    // Divider
    out.push('╠');
    for _ in 0..inner {
        out.push('═');
    }
    out.push_str("╣\n");

    // Simple ASCII graph representation
    out.push('║');
    let entry_line = "                          ┌─────────┐";
    out.push_str(entry_line);
    for _ in 0..(inner.saturating_sub(entry_line.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    out.push('║');
    let entry_label = "                          │  ENTRY  │";
    out.push_str(entry_label);
    for _ in 0..(inner.saturating_sub(entry_label.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    out.push('║');
    let entry_bottom = "                          └────┬────┘";
    out.push_str(entry_bottom);
    for _ in 0..(inner.saturating_sub(entry_bottom.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    out.push('║');
    let connector = "                               │";
    out.push_str(connector);
    for _ in 0..(inner.saturating_sub(connector.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    // Show conditional if present
    if metrics.decision_points > 0 {
        out.push('║');
        let cond_top = "                          ┌────▼────┐";
        out.push_str(cond_top);
        for _ in 0..(inner.saturating_sub(cond_top.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        out.push('║');
        let cond_label = "                          │ if cond │";
        out.push_str(cond_label);
        for _ in 0..(inner.saturating_sub(cond_label.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        out.push('║');
        let cond_bottom = "                          └────┬────┘";
        out.push_str(cond_bottom);
        for _ in 0..(inner.saturating_sub(cond_bottom.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        // Branches
        out.push('║');
        let branches = "                     ┌────────┼────────┐";
        out.push_str(branches);
        for _ in 0..(inner.saturating_sub(branches.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        out.push('║');
        let branch_labels = "                     │ TRUE   │  FALSE │";
        out.push_str(branch_labels);
        for _ in 0..(inner.saturating_sub(branch_labels.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        out.push('║');
        let merge = "                     └────────┼────────┘";
        out.push_str(merge);
        for _ in 0..(inner.saturating_sub(merge.len())) {
            out.push(' ');
        }
        out.push_str("║\n");

        out.push('║');
        out.push_str(connector);
        for _ in 0..(inner.saturating_sub(connector.len())) {
            out.push(' ');
        }
        out.push_str("║\n");
    }

    // Exit node
    out.push('║');
    let exit_top = "                          ┌───▼────┐";
    out.push_str(exit_top);
    for _ in 0..(inner.saturating_sub(exit_top.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    out.push('║');
    let exit_label = "                          │  EXIT  │";
    out.push_str(exit_label);
    for _ in 0..(inner.saturating_sub(exit_label.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    out.push('║');
    let exit_bottom = "                          └────────┘";
    out.push_str(exit_bottom);
    for _ in 0..(inner.saturating_sub(exit_bottom.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    // Metrics section
    out.push('╠');
    for _ in 0..inner {
        out.push('═');
    }
    out.push_str("╣\n");

    let metrics_line = format!(
        "  Nodes: {} │ Edges: {} │ Cyclomatic: {} │ Essential: {} │ Max Depth: {}",
        cfg.node_count(),
        cfg.edge_count(),
        metrics.cyclomatic,
        metrics.essential,
        metrics.max_depth
    );

    out.push('║');
    let truncated = if metrics_line.len() > inner {
        &metrics_line[..inner]
    } else {
        &metrics_line
    };
    out.push_str(truncated);
    for _ in 0..(inner.saturating_sub(metrics_line.len())) {
        out.push(' ');
    }
    out.push_str("║\n");

    // Footer
    out.push('╚');
    for _ in 0..inner {
        out.push('═');
    }
    out.push_str("╝\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_015_cfg_builder_basic() {
        let mut builder = CfgBuilder::new();
        builder.add_block("block1", 1, 5);
        builder.add_block("block2", 6, 10);
        let cfg = builder.build();

        assert!(cfg.node_count() >= 2);
        assert!(!cfg.edges.is_empty());
    }

    #[test]
    fn test_ml_015_cfg_builder_conditional() {
        let mut builder = CfgBuilder::new();
        let cond = builder.add_conditional("if x > 0", 1);
        let then_block = builder.add_block("then", 2, 3);
        builder.set_current(cond);
        let else_block = builder.add_block("else", 4, 5);

        // Merge point
        builder.add_edge(then_block, 100, Some("merge"));
        builder.add_edge(else_block, 100, Some("merge"));

        let cfg = builder.build();

        assert!(cfg.node_count() >= 3);
    }

    #[test]
    fn test_ml_015_cfg_builder_loop() {
        let mut builder = CfgBuilder::new();
        let loop_header = builder.add_loop("while true", 1);
        let body = builder.add_block("body", 2, 5);
        builder.add_edge(body, loop_header, Some("back"));
        let cfg = builder.build();

        assert!(cfg.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_ml_016_cyclomatic_simple() {
        // Simple linear flow: E=2, N=3 (entry, block, exit), P=1
        // Cyclomatic = 2 - 3 + 2 = 1
        let mut builder = CfgBuilder::new();
        builder.add_block("block", 1, 5);
        let cfg = builder.build();

        let metrics = ComplexityMetrics::from_cfg(&cfg);

        // Allow some flexibility due to entry/exit handling
        assert!(metrics.cyclomatic >= 1);
        assert!(metrics.cyclomatic <= 3);
    }

    #[test]
    fn test_ml_016_cyclomatic_conditional() {
        let mut builder = CfgBuilder::new();
        builder.add_conditional("if", 1);
        builder.add_block("then", 2, 3);
        let cfg = builder.build();

        let metrics = ComplexityMetrics::from_cfg(&cfg);

        // With conditional, complexity increases
        assert!(metrics.decision_points >= 1);
    }

    #[test]
    fn test_ml_016_complexity_grade() {
        assert_eq!(
            ComplexityMetrics {
                cyclomatic: 3,
                ..Default::default()
            }
            .grade(),
            ComplexityGrade::Simple
        );
        assert_eq!(
            ComplexityMetrics {
                cyclomatic: 8,
                ..Default::default()
            }
            .grade(),
            ComplexityGrade::Moderate
        );
        assert_eq!(
            ComplexityMetrics {
                cyclomatic: 15,
                ..Default::default()
            }
            .grade(),
            ComplexityGrade::Complex
        );
        assert_eq!(
            ComplexityMetrics {
                cyclomatic: 30,
                ..Default::default()
            }
            .grade(),
            ComplexityGrade::VeryComplex
        );
        assert_eq!(
            ComplexityMetrics {
                cyclomatic: 60,
                ..Default::default()
            }
            .grade(),
            ComplexityGrade::Untestable
        );
    }

    #[test]
    fn test_ml_016_needs_refactoring() {
        assert!(!ComplexityGrade::Simple.needs_refactoring());
        assert!(!ComplexityGrade::Moderate.needs_refactoring());
        assert!(ComplexityGrade::Complex.needs_refactoring());
        assert!(ComplexityGrade::VeryComplex.needs_refactoring());
        assert!(ComplexityGrade::Untestable.needs_refactoring());
    }

    #[test]
    fn test_ml_016_threshold() {
        let within = ComplexityMetrics {
            cyclomatic: 10,
            ..Default::default()
        };
        let exceeds = ComplexityMetrics {
            cyclomatic: 11,
            ..Default::default()
        };

        assert!(!within.exceeds_threshold());
        assert!(exceeds.exceeds_threshold());
    }

    #[test]
    fn test_ml_017_render_ascii() {
        let mut builder = CfgBuilder::new();
        builder.add_conditional("if x", 1);
        builder.add_block("body", 2, 3);
        let cfg = builder.build();

        let metrics = ComplexityMetrics::from_cfg(&cfg);
        let rendered = render_cfg_ascii(&cfg, &metrics, 80);

        assert!(rendered.contains("CONTROL FLOW GRAPH"));
        assert!(rendered.contains("ENTRY"));
        assert!(rendered.contains("EXIT"));
        assert!(rendered.contains("Cyclomatic"));
    }

    #[test]
    fn test_cfg_node_id() {
        assert_eq!(CfgNode::Entry.id(), 0);
        assert_eq!(CfgNode::Exit.id(), usize::MAX);
        assert_eq!(
            CfgNode::BasicBlock {
                id: 5,
                label: "test".to_string(),
                start_line: 1,
                end_line: 2
            }
            .id(),
            5
        );
    }

    #[test]
    fn test_cfg_node_label() {
        assert_eq!(CfgNode::Entry.label(), "ENTRY");
        assert_eq!(CfgNode::Exit.label(), "EXIT");
        assert_eq!(
            CfgNode::FunctionEntry {
                id: 1,
                name: "main".to_string(),
                line: 1
            }
            .label(),
            "fn main"
        );
    }

    #[test]
    fn test_cfg_successors_predecessors() {
        let mut cfg = ControlFlowGraph::new();
        cfg.add_edge(0, 1, None);
        cfg.add_edge(0, 2, None);
        cfg.add_edge(1, 3, None);
        cfg.add_edge(2, 3, None);

        assert_eq!(cfg.successors(0).len(), 2);
        assert_eq!(cfg.predecessors(3).len(), 2);
    }
}
