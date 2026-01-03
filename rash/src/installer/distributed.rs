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

impl Default for InstallerGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate step duration based on action type
fn estimate_step_duration(step: &super::spec::Step) -> f64 {
    // Parse timeout if specified
    if let Some(ref timeout) = step.timing.timeout {
        if let Some(secs) = parse_duration_secs(timeout) {
            // Estimate as 10% of timeout (conservative)
            return secs * 0.1;
        }
    }

    // Default estimates based on action type
    match step.action.as_str() {
        "verify" => 0.5,
        "script" => 5.0,
        "apt-install" => 30.0,
        "apt-remove" => 10.0,
        "file-write" => 0.5,
        "user-group" => 1.0,
        "user-add-to-group" => 1.0,
        _ => 5.0,
    }
}

/// Parse duration string (e.g., "5m", "30s") to seconds
fn parse_duration_secs(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('s') {
        num.parse().ok()
    } else if let Some(num) = s.strip_suffix('m') {
        num.parse::<f64>().ok().map(|m| m * 60.0)
    } else if let Some(num) = s.strip_suffix('h') {
        num.parse::<f64>().ok().map(|h| h * 3600.0)
    } else {
        s.parse().ok()
    }
}

/// Extract required capabilities from step
fn extract_capabilities(step: &super::spec::Step) -> Vec<String> {
    let mut caps = Vec::new();

    match step.action.as_str() {
        "apt-install" | "apt-remove" => caps.push("apt".to_string()),
        "script" => {
            if let Some(ref script) = step.script {
                if script.content.contains("docker") {
                    caps.push("docker".to_string());
                }
            }
        }
        _ => {}
    }

    caps
}

/// sccache client for build artifact caching
#[derive(Debug, Clone)]
pub struct SccacheClient {
    /// Server address
    server: String,
    /// Whether connected
    connected: bool,
}

impl SccacheClient {
    /// Create new sccache client
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            connected: false,
        }
    }

    /// Connect to sccache server
    pub fn connect(&mut self) -> Result<()> {
        // In a real implementation, this would establish connection
        // For now, just validate the address format
        if self.server.contains(':') {
            self.connected = true;
            Ok(())
        } else {
            Err(Error::Validation(format!(
                "Invalid sccache server address: {}",
                self.server
            )))
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get server address
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Get cache stats (placeholder)
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            size_bytes: 0,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache size in bytes
    pub size_bytes: u64,
}

impl CacheStats {
    /// Hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Format execution plan for display
pub fn format_execution_plan(plan: &ExecutionPlan, max_parallel: usize) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Execution Plan ({} waves, max parallelism: {})\n",
        plan.waves.len(),
        max_parallel
    ));
    output.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n\n",
    );

    for wave in &plan.waves {
        let wave_type = if wave.is_sequential {
            format!(
                "sequential - {}",
                wave.sequential_reason.as_deref().unwrap_or("constraint")
            )
        } else {
            "parallel".to_string()
        };

        output.push_str(&format!("Wave {} ({}):\n", wave.wave_number + 1, wave_type));

        for step_id in &wave.step_ids {
            output.push_str(&format!("  • {}\n", step_id));
        }

        output.push_str(&format!(
            "  Estimated: {:.1}s\n\n",
            wave.estimated_duration_secs
        ));
    }

    output.push_str(&format!(
        "Estimated total: {:.1}s (vs {:.1}s sequential = {:.0}% speedup)\n",
        plan.total_duration_parallel_secs,
        plan.total_duration_sequential_secs,
        plan.speedup_percent
    ));
    output.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n",
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DIST_001_graph_new() {
        let graph = InstallerGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_DIST_002_graph_add_node() {
        let mut graph = InstallerGraph::new();
        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First Step".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "step-1");
    }

    #[test]
    fn test_DIST_003_graph_add_edge() {
        let mut graph = InstallerGraph::new();
        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 1.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "step-2".to_string(),
            name: "Second".to_string(),
            estimated_duration_secs: 2.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        graph.add_edge("step-1", "step-2").expect("Should add edge");
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_DIST_004_graph_add_edge_invalid() {
        let mut graph = InstallerGraph::new();
        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 1.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let result = graph.add_edge("step-1", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_DIST_005_compute_waves_empty() {
        let graph = InstallerGraph::new();
        let waves = graph.compute_waves();
        assert!(waves.is_empty());
    }

    #[test]
    fn test_DIST_006_compute_waves_single() {
        let mut graph = InstallerGraph::new();
        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "Only Step".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let waves = graph.compute_waves();
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].step_ids, vec!["step-1"]);
    }

    #[test]
    fn test_DIST_007_compute_waves_parallel() {
        let mut graph = InstallerGraph::new();

        // Two independent steps should be in same wave
        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "step-2".to_string(),
            name: "Second".to_string(),
            estimated_duration_secs: 3.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let waves = graph.compute_waves();
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].step_ids.len(), 2);
        assert!(!waves[0].is_sequential);
    }

    #[test]
    fn test_DIST_008_compute_waves_sequential() {
        let mut graph = InstallerGraph::new();

        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "step-2".to_string(),
            name: "Second".to_string(),
            estimated_duration_secs: 3.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        // step-2 depends on step-1
        graph.add_edge("step-1", "step-2").unwrap();

        let waves = graph.compute_waves();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0].step_ids, vec!["step-1"]);
        assert_eq!(waves[1].step_ids, vec!["step-2"]);
    }

    #[test]
    fn test_DIST_009_compute_waves_exclusive_resource() {
        let mut graph = InstallerGraph::new();

        // Two steps with same exclusive resource should be sequential
        graph.add_node(GraphNode {
            id: "apt-1".to_string(),
            name: "Install A".to_string(),
            estimated_duration_secs: 10.0,
            capabilities: vec!["apt".to_string()],
            exclusive_resource: Some("apt-lock".to_string()),
        });
        graph.add_node(GraphNode {
            id: "apt-2".to_string(),
            name: "Install B".to_string(),
            estimated_duration_secs: 15.0,
            capabilities: vec!["apt".to_string()],
            exclusive_resource: Some("apt-lock".to_string()),
        });

        let waves = graph.compute_waves();

        // Should have 2 waves, each sequential
        assert_eq!(waves.len(), 2);
        assert!(waves[0].is_sequential);
        assert!(waves[1].is_sequential);
    }

    #[test]
    fn test_DIST_010_compute_waves_mixed() {
        let mut graph = InstallerGraph::new();

        // Step 1: no deps
        graph.add_node(GraphNode {
            id: "check".to_string(),
            name: "Check OS".to_string(),
            estimated_duration_secs: 0.5,
            capabilities: vec![],
            exclusive_resource: None,
        });

        // Steps 2 & 3: both depend on step 1, can run in parallel
        graph.add_node(GraphNode {
            id: "download-a".to_string(),
            name: "Download A".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "download-b".to_string(),
            name: "Download B".to_string(),
            estimated_duration_secs: 3.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        graph.add_edge("check", "download-a").unwrap();
        graph.add_edge("check", "download-b").unwrap();

        // Step 4: depends on both downloads
        graph.add_node(GraphNode {
            id: "install".to_string(),
            name: "Install".to_string(),
            estimated_duration_secs: 30.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        graph.add_edge("download-a", "install").unwrap();
        graph.add_edge("download-b", "install").unwrap();

        let waves = graph.compute_waves();

        // Wave 0: check
        // Wave 1: download-a, download-b (parallel)
        // Wave 2: install
        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0].step_ids, vec!["check"]);
        assert_eq!(waves[1].step_ids.len(), 2);
        assert!(waves[1].step_ids.contains(&"download-a".to_string()));
        assert!(waves[1].step_ids.contains(&"download-b".to_string()));
        assert_eq!(waves[2].step_ids, vec!["install"]);
    }

    #[test]
    fn test_DIST_011_create_plan() {
        let mut graph = InstallerGraph::new();

        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 5.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "step-2".to_string(),
            name: "Second".to_string(),
            estimated_duration_secs: 3.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let plan = graph.create_plan();

        assert!(!plan.waves.is_empty());
        assert!(plan.total_duration_sequential_secs > 0.0);
        assert!(plan.total_duration_parallel_secs > 0.0);
    }

    #[test]
    fn test_DIST_012_to_mermaid() {
        let mut graph = InstallerGraph::new();

        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First Step".to_string(),
            estimated_duration_secs: 1.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "step-2".to_string(),
            name: "Second Step".to_string(),
            estimated_duration_secs: 2.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_edge("step-1", "step-2").unwrap();

        let mermaid = graph.to_mermaid();

        assert!(mermaid.starts_with("graph TD"));
        assert!(mermaid.contains("step-1"));
        assert!(mermaid.contains("step-2"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_DIST_013_to_dot() {
        let mut graph = InstallerGraph::new();

        graph.add_node(GraphNode {
            id: "step-1".to_string(),
            name: "First".to_string(),
            estimated_duration_secs: 1.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let dot = graph.to_dot();

        assert!(dot.starts_with("digraph"));
        assert!(dot.contains("step-1"));
    }

    #[test]
    fn test_DIST_014_parse_duration_secs() {
        assert_eq!(parse_duration_secs("30s"), Some(30.0));
        assert_eq!(parse_duration_secs("5m"), Some(300.0));
        assert_eq!(parse_duration_secs("1h"), Some(3600.0));
        assert_eq!(parse_duration_secs("45"), Some(45.0));
        assert_eq!(parse_duration_secs("invalid"), None);
    }

    #[test]
    fn test_DIST_015_sccache_client_new() {
        let client = SccacheClient::new("10.0.0.50:4226");
        assert_eq!(client.server(), "10.0.0.50:4226");
        assert!(!client.is_connected());
    }

    #[test]
    fn test_DIST_016_sccache_client_connect() {
        let mut client = SccacheClient::new("10.0.0.50:4226");
        client.connect().expect("Should connect");
        assert!(client.is_connected());
    }

    #[test]
    fn test_DIST_017_sccache_client_invalid() {
        let mut client = SccacheClient::new("invalid-address");
        let result = client.connect();
        assert!(result.is_err());
    }

    #[test]
    fn test_DIST_018_cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 80,
            misses: 20,
            size_bytes: 1000,
        };
        assert!((stats.hit_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_DIST_019_cache_stats_hit_rate_zero() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_DIST_020_distributed_config_default() {
        let config = DistributedConfig::default();
        assert!(!config.enabled);
        assert!(config.sccache_server.is_none());
        assert!(config.remote_executors.is_empty());
        assert_eq!(config.max_parallel_steps, 4);
    }

    #[test]
    fn test_DIST_021_format_execution_plan() {
        let plan = ExecutionPlan {
            waves: vec![ExecutionWave {
                wave_number: 0,
                step_ids: vec!["step-1".to_string()],
                is_sequential: false,
                sequential_reason: None,
                estimated_duration_secs: 5.0,
            }],
            total_duration_parallel_secs: 5.0,
            total_duration_sequential_secs: 5.0,
            speedup_percent: 0.0,
        };

        let output = format_execution_plan(&plan, 4);
        assert!(output.contains("Execution Plan"));
        assert!(output.contains("Wave 1"));
        assert!(output.contains("step-1"));
    }

    #[test]
    fn test_DIST_022_from_spec() {
        use super::super::spec::InstallerSpec;

        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "step-1"
name = "First Step"
action = "script"
depends_on = []

[step.script]
content = "echo hello"

[[step]]
id = "step-2"
name = "Second Step"
action = "script"
depends_on = ["step-1"]

[step.script]
content = "echo world"
"#;

        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let graph = InstallerGraph::from_spec(&spec).expect("Should build graph");

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_DIST_023_speedup_calculation() {
        let mut graph = InstallerGraph::new();

        // Three parallel steps
        graph.add_node(GraphNode {
            id: "a".to_string(),
            name: "A".to_string(),
            estimated_duration_secs: 10.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "b".to_string(),
            name: "B".to_string(),
            estimated_duration_secs: 10.0,
            capabilities: vec![],
            exclusive_resource: None,
        });
        graph.add_node(GraphNode {
            id: "c".to_string(),
            name: "C".to_string(),
            estimated_duration_secs: 10.0,
            capabilities: vec![],
            exclusive_resource: None,
        });

        let plan = graph.create_plan();

        // Sequential: 30s, Parallel: 10s, Speedup: ~66%
        assert!((plan.total_duration_sequential_secs - 30.0).abs() < 0.01);
        assert!((plan.total_duration_parallel_secs - 10.0).abs() < 0.01);
        assert!(plan.speedup_percent > 60.0);
    }
}
