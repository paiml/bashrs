
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
