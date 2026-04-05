//! Distributed Execution for Installer Framework (§5)
//!
//! This module provides build graph parallelization and distributed execution:
//!
//! - Build graph computation using petgraph
//! - Execution waves for parallel step execution
//! - sccache integration for build artifact caching
//! - Remote executor support for distributed builds
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{InstallerGraph, DistributedConfig};
//!
//! let graph = InstallerGraph::from_spec(&spec);
//! let waves = graph.compute_waves();
//! println!("Execution plan: {} waves", waves.len());
//! ```

use crate::models::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Distributed execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Enable distributed execution
    #[serde(default)]
    pub enabled: bool,

    /// sccache server address (e.g., "10.0.0.50:4226")
    #[serde(default)]
    pub sccache_server: Option<String>,

    /// Remote execution endpoints
    #[serde(default)]
    pub remote_executors: Vec<RemoteExecutor>,

    /// Maximum parallel steps (respecting dependency graph)
    #[serde(default = "default_max_parallel")]
    pub max_parallel_steps: usize,

    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sccache_server: None,
            remote_executors: Vec::new(),
            max_parallel_steps: 4,
            optimization: OptimizationConfig::default(),
        }
    }
}

fn default_max_parallel() -> usize {
    4
}

/// Remote executor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExecutor {
    /// Host address
    pub host: String,
    /// Capabilities (e.g., ["apt", "docker", "gpu"])
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Priority (higher = preferred)
    #[serde(default)]
    pub priority: i32,
}

/// Optimization configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Merge consecutive apt-install steps
    #[serde(default)]
    pub coalesce_package_installs: bool,
    /// Prefetch artifacts during earlier steps
    #[serde(default)]
    pub speculative_download: bool,
}

/// A node in the installer build graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Estimated duration in seconds
    pub estimated_duration_secs: f64,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Exclusive resource (if any, e.g., "apt-lock")
    pub exclusive_resource: Option<String>,
}

/// Build graph for parallel execution planning
#[derive(Debug, Clone)]
pub struct InstallerGraph {
    /// Nodes (steps)
    nodes: Vec<GraphNode>,
    /// Edges (dependencies): (from_idx, to_idx)
    edges: Vec<(usize, usize)>,
    /// Node index by ID
    node_index: HashMap<String, usize>,
}

/// Execution wave (steps that can run in parallel)
#[derive(Debug, Clone)]
pub struct ExecutionWave {
    /// Wave number (0-indexed)
    pub wave_number: usize,
    /// Step IDs in this wave
    pub step_ids: Vec<String>,
    /// Whether this wave is sequential (due to resource constraints)
    pub is_sequential: bool,
    /// Reason for sequential execution (if any)
    pub sequential_reason: Option<String>,
    /// Estimated duration (max of parallel steps)
    pub estimated_duration_secs: f64,
}

/// Execution plan summary
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Execution waves
    pub waves: Vec<ExecutionWave>,
    /// Total estimated duration (parallel)
    pub total_duration_parallel_secs: f64,
    /// Total estimated duration (sequential)
    pub total_duration_sequential_secs: f64,
    /// Speedup factor
    pub speedup_percent: f64,
}

impl InstallerGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_index: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) {
        let idx = self.nodes.len();
        self.node_index.insert(node.id.clone(), idx);
        self.nodes.push(node);
    }

    /// Add an edge (dependency) to the graph
    pub fn add_edge(&mut self, from_id: &str, to_id: &str) -> Result<()> {
        let from_idx = self
            .node_index
            .get(from_id)
            .ok_or_else(|| Error::Validation(format!("Unknown step: {}", from_id)))?;
        let to_idx = self
            .node_index
            .get(to_id)
            .ok_or_else(|| Error::Validation(format!("Unknown step: {}", to_id)))?;
        self.edges.push((*from_idx, *to_idx));
        Ok(())
    }

    /// Build graph from installer spec
    pub fn from_spec(spec: &super::spec::InstallerSpec) -> Result<Self> {
        let mut graph = Self::new();

        // Add nodes for each step
        for step in &spec.step {
            let node = GraphNode {
                id: step.id.clone(),
                name: step.name.clone(),
                estimated_duration_secs: estimate_step_duration(step),
                capabilities: extract_capabilities(step),
                exclusive_resource: step.constraints.exclusive_resource.clone(),
            };
            graph.add_node(node);
        }

        // Add edges for dependencies
        for step in &spec.step {
            for dep_id in &step.depends_on {
                graph.add_edge(dep_id, &step.id)?;
            }
        }

        Ok(graph)
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }

    /// Get incoming dependencies for a node
    fn incoming_deps(&self, node_idx: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(_, to)| *to == node_idx)
            .map(|(from, _)| *from)
            .collect()
    }

    /// Compute execution waves (steps that can run in parallel)
    #[allow(clippy::indexing_slicing)] // Indices are from 0..self.nodes.len(), guaranteed valid
    pub fn compute_waves(&self) -> Vec<ExecutionWave> {
        if self.nodes.is_empty() {
            return Vec::new();
        }

        let mut waves = Vec::new();
        let mut completed: HashSet<usize> = HashSet::new();
        let mut wave_number = 0;

        while completed.len() < self.nodes.len() {
            // Find all nodes whose dependencies are all completed
            let ready: Vec<usize> = (0..self.nodes.len())
                .filter(|idx| !completed.contains(idx))
                .filter(|idx| {
                    self.incoming_deps(*idx)
                        .iter()
                        .all(|dep| completed.contains(dep))
                })
                .collect();

            if ready.is_empty() {
                // Cycle detection - should not happen with valid spec
                break;
            }

            // Check for exclusive resources
            let mut resource_groups: HashMap<String, Vec<usize>> = HashMap::new();
            let mut no_resource: Vec<usize> = Vec::new();

            for &idx in &ready {
                if let Some(ref resource) = self.nodes[idx].exclusive_resource {
                    resource_groups
                        .entry(resource.clone())
                        .or_default()
                        .push(idx);
                } else {
                    no_resource.push(idx);
                }
            }

            // Create wave(s) for this level
            // Steps without exclusive resources can run together
            if !no_resource.is_empty() {
                let step_ids: Vec<String> = no_resource
                    .iter()
                    .map(|idx| self.nodes[*idx].id.clone())
                    .collect();
                let max_duration = no_resource
                    .iter()
                    .map(|idx| self.nodes[*idx].estimated_duration_secs)
                    .fold(0.0_f64, |a, b| a.max(b));

                waves.push(ExecutionWave {
                    wave_number,
                    step_ids,
                    is_sequential: false,
                    sequential_reason: None,
                    estimated_duration_secs: max_duration,
                });
                wave_number += 1;

                for idx in no_resource {
                    completed.insert(idx);
                }
            }

            // Steps with exclusive resources must run sequentially within their group
            for (resource, indices) in resource_groups {
                for idx in indices {
                    waves.push(ExecutionWave {
                        wave_number,
                        step_ids: vec![self.nodes[idx].id.clone()],
                        is_sequential: true,
                        sequential_reason: Some(format!("Exclusive resource: {}", resource)),
                        estimated_duration_secs: self.nodes[idx].estimated_duration_secs,
                    });
                    wave_number += 1;
                    completed.insert(idx);
                }
            }
        }

        waves
    }

    /// Create execution plan from waves
    pub fn create_plan(&self) -> ExecutionPlan {
        let waves = self.compute_waves();

        let total_sequential: f64 = self.nodes.iter().map(|n| n.estimated_duration_secs).sum();

        let total_parallel: f64 = waves.iter().map(|w| w.estimated_duration_secs).sum();

        let speedup = if total_sequential > 0.0 {
            ((total_sequential - total_parallel) / total_sequential) * 100.0
        } else {
            0.0
        };

        ExecutionPlan {
            waves,
            total_duration_parallel_secs: total_parallel,
            total_duration_sequential_secs: total_sequential,
            speedup_percent: speedup,
        }
    }

    /// Generate Mermaid diagram of build graph
    #[allow(clippy::indexing_slicing)] // Edge indices come from validated add_edge calls
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");

        // Add nodes
        for node in &self.nodes {
            let label = node.name.replace('"', "'");
            mermaid.push_str(&format!("    {}[\"{}\"]\n", node.id, label));
        }

        // Add edges
        for (from_idx, to_idx) in &self.edges {
            mermaid.push_str(&format!(
                "    {} --> {}\n",
                self.nodes[*from_idx].id, self.nodes[*to_idx].id
            ));
        }

        mermaid
    }

    /// Generate DOT format for graphviz
    #[allow(clippy::indexing_slicing)] // Edge indices come from validated add_edge calls
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph InstallerGraph {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=box];\n");

        // Add nodes
        for node in &self.nodes {
            let label = node.name.replace('"', "'");
            dot.push_str(&format!(
                "    {} [label=\"{}\\n({:.1}s)\"];\n",
                node.id, label, node.estimated_duration_secs
            ));
        }

        // Add edges
        for (from_idx, to_idx) in &self.edges {
            dot.push_str(&format!(
                "    {} -> {};\n",
                self.nodes[*from_idx].id, self.nodes[*to_idx].id
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

include!("distributed_incl2.rs");
