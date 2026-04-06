
use super::super::*;

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
    use crate::installer::spec::InstallerSpec;

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
