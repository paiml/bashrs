use crate::models::{Error, Result};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "playground")]
use petgraph::visit::EdgeRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub enum ComputeNode {
    Parse {
        range: ByteRange,
        version: u64,
    },
    Validate {
        ast_node: NodeId,
        rules: u64, // BitSet as u64 for simplicity
    },
    Transpile {
        ir_node: NodeId,
        dialect: crate::models::ShellDialect,
    },
    Highlight {
        line_range: LineRange,
        theme_id: u32,
    },
}

#[derive(Debug)]
pub struct Dependency {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: u32,
}

/// Lock-free incremental computation graph
pub struct ComputationGraph {
    #[cfg(feature = "playground")]
    nodes: dashmap::DashMap<NodeId, ComputeNode>,

    #[cfg(feature = "playground")]
    edges: std::sync::Mutex<petgraph::Graph<NodeId, Dependency>>,

    #[cfg(feature = "playground")]
    dirty_set: crossbeam::queue::SegQueue<NodeId>,

    #[cfg(not(feature = "playground"))]
    _placeholder: std::marker::PhantomData<()>,
}

impl ComputationGraph {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "playground")]
        {
            Ok(Self {
                nodes: dashmap::DashMap::new(),
                edges: std::sync::Mutex::new(petgraph::Graph::new()),
                dirty_set: crossbeam::queue::SegQueue::new(),
            })
        }

        #[cfg(not(feature = "playground"))]
        {
            Ok(Self {
                _placeholder: std::marker::PhantomData,
            })
        }
    }

    pub fn mark_full_rebuild(&self) {
        #[cfg(feature = "playground")]
        {
            // Mark all nodes as dirty
            for entry in self.nodes.iter() {
                self.dirty_set.push(*entry.key());
            }
        }
    }

    #[cfg(feature = "playground")]
    pub fn add_node(&self, node: ComputeNode) -> NodeId {
        let id = NodeId::new();
        self.nodes.insert(id, node);
        self.dirty_set.push(id); // New nodes are dirty by default
        id
    }

    #[cfg(feature = "playground")]
    pub fn add_dependency(&self, source: NodeId, target: NodeId) -> Result<()> {
        let mut graph = self
            .edges
            .lock()
            .map_err(|_| Error::Internal("Failed to lock dependency graph".to_string()))?;

        // Find or create graph nodes
        let source_idx = graph
            .node_indices()
            .find(|&idx| graph[idx] == source)
            .unwrap_or_else(|| graph.add_node(source));

        let target_idx = graph
            .node_indices()
            .find(|&idx| graph[idx] == target)
            .unwrap_or_else(|| graph.add_node(target));

        // Add edge
        graph.add_edge(
            source_idx,
            target_idx,
            Dependency {
                source,
                target,
                weight: 1,
            },
        );

        Ok(())
    }

    #[cfg(feature = "playground")]
    pub fn mark_dirty(&self, edit_range: &ByteRange) {
        // Find affected parse nodes via range overlap
        for entry in self.nodes.iter() {
            if let ComputeNode::Parse { range, .. } = entry.value() {
                if ranges_overlap(range, edit_range) {
                    self.dirty_set.push(*entry.key());

                    // Propagate dirtiness through dependency edges
                    self.propagate_dirtiness(*entry.key());
                }
            }
        }
    }

    #[cfg(feature = "playground")]
    fn propagate_dirtiness(&self, node_id: NodeId) {
        if let Ok(graph) = self.edges.lock() {
            let mut to_visit = VecDeque::new();
            to_visit.push_back(node_id);

            while let Some(current_id) = to_visit.pop_front() {
                // Find node in graph
                if let Some(node_idx) = graph.node_indices().find(|&idx| graph[idx] == current_id) {
                    // Add all dependent nodes to visit queue
                    for edge in graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
                        let target_node_id = graph[edge.target()];
                        self.dirty_set.push(target_node_id);
                        to_visit.push_back(target_node_id);
                    }
                }
            }
        }
    }

    pub fn process_pending(&self) -> Result<()> {
        #[cfg(feature = "playground")]
        {
            use rayon::prelude::*;

            // Collect dirty nodes
            let mut dirty_nodes = Vec::new();
            while let Some(node_id) = self.dirty_set.pop() {
                dirty_nodes.push(node_id);
            }

            if dirty_nodes.is_empty() {
                return Ok(());
            }

            // Process in parallel using rayon
            dirty_nodes
                .par_iter()
                .try_for_each(|&node_id| -> Result<()> {
                    if let Some(entry) = self.nodes.get(&node_id) {
                        match entry.value() {
                            ComputeNode::Parse { range, version } => {
                                self.reparse_range(range, *version)?;
                            }
                            ComputeNode::Validate { ast_node, rules } => {
                                self.revalidate_node(*ast_node, *rules)?;
                            }
                            ComputeNode::Transpile { ir_node, dialect } => {
                                self.retranspile_node(*ir_node, dialect)?;
                            }
                            ComputeNode::Highlight {
                                line_range,
                                theme_id,
                            } => {
                                self.rehighlight_range(line_range, *theme_id)?;
                            }
                        }
                    }
                    Ok(())
                })?;
        }

        Ok(())
    }

    #[cfg(feature = "playground")]
    fn reparse_range(&self, _range: &ByteRange, _version: u64) -> Result<()> {
        // TODO: Implement incremental parsing with tree-sitter
        tracing::debug!("Reparsing range: {:?}", _range);
        Ok(())
    }

    #[cfg(feature = "playground")]
    fn revalidate_node(&self, _ast_node: NodeId, _rules: u64) -> Result<()> {
        // TODO: Implement incremental validation
        tracing::debug!("Revalidating node: {:?}", _ast_node);
        Ok(())
    }

    #[cfg(feature = "playground")]
    fn retranspile_node(
        &self,
        _ir_node: NodeId,
        _dialect: &crate::models::ShellDialect,
    ) -> Result<()> {
        // TODO: Implement incremental transpilation
        tracing::debug!("Retranspiling node: {:?}", _ir_node);
        Ok(())
    }

    #[cfg(feature = "playground")]
    fn rehighlight_range(&self, _line_range: &LineRange, _theme_id: u32) -> Result<()> {
        // TODO: Implement incremental syntax highlighting
        tracing::debug!("Rehighlighting range: {:?}", _line_range);
        Ok(())
    }
}

#[cfg(feature = "playground")]
fn ranges_overlap(a: &ByteRange, b: &ByteRange) -> bool {
    !(a.end <= b.start || b.end <= a.start)
}
