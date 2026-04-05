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

include!("cfg_incl2.rs");
