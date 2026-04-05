//! CFG command — `bashrs cfg <file>` (Sprint 5: Formal CFG Construction)
//!
//! Parses a bash script, builds formal control flow graphs, and outputs
//! per-function complexity metrics.

use crate::cli::args::CfgOutputFormat;
use crate::models::{Error, Result};
use crate::quality::cfg_builder::{build_cfg_from_ast, build_per_function_cfgs};
use crate::quality::{render_cfg_ascii, ComplexityMetrics};
use std::fs;
use std::path::Path;

/// Run the `bashrs cfg` command.
pub(crate) fn cfg_command(input: &Path, format: CfgOutputFormat, per_function: bool) -> Result<()> {
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {e}", input.display())))?;

    let ast = {
        let mut parser = crate::bash_parser::BashParser::new(&source)
            .map_err(|e| Error::Internal(format!("Parse error: {e}")))?;
        parser
            .parse()
            .map_err(|e| Error::Internal(format!("Parse error: {e}")))?
    };

    if per_function {
        let cfgs = build_per_function_cfgs(&ast.statements);
        match format {
            CfgOutputFormat::Human => print_per_function_human(&cfgs),
            CfgOutputFormat::Json => print_per_function_json(&cfgs),
        }
    } else {
        let cfg = build_cfg_from_ast(&ast.statements);
        let metrics = ComplexityMetrics::from_cfg(&cfg);
        match format {
            CfgOutputFormat::Human => {
                print!("{}", render_cfg_ascii(&cfg, &metrics, 60));
                println!();
                println!(
                    "Grade: {:?} — {}",
                    metrics.grade(),
                    metrics.grade().description()
                );
            }
            CfgOutputFormat::Json => {
                print_json(&cfg, &metrics, input.display().to_string().as_str());
            }
        }
    }

    Ok(())
}

fn print_per_function_human(cfgs: &[(String, crate::quality::ControlFlowGraph)]) {
    for (name, cfg) in cfgs {
        let metrics = ComplexityMetrics::from_cfg(cfg);
        println!(
            "{name}: cyclomatic={} essential={} cognitive={} depth={} decisions={} loops={} grade={:?}",
            metrics.cyclomatic,
            metrics.essential,
            metrics.cognitive,
            metrics.max_depth,
            metrics.decision_points,
            metrics.loop_count,
            metrics.grade(),
        );
    }
}

fn print_per_function_json(cfgs: &[(String, crate::quality::ControlFlowGraph)]) {
    let entries: Vec<serde_json::Value> = cfgs
        .iter()
        .map(|(name, cfg)| {
            let m = ComplexityMetrics::from_cfg(cfg);
            serde_json::json!({
                "function": name,
                "nodes": cfg.node_count(),
                "edges": cfg.edge_count(),
                "cyclomatic": m.cyclomatic,
                "essential": m.essential,
                "cognitive": m.cognitive,
                "max_depth": m.max_depth,
                "decision_points": m.decision_points,
                "loop_count": m.loop_count,
                "grade": format!("{:?}", m.grade()),
            })
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({ "functions": entries }))
            .unwrap_or_default()
    );
}

fn print_json(cfg: &crate::quality::ControlFlowGraph, metrics: &ComplexityMetrics, filename: &str) {
    let output = serde_json::json!({
        "file": filename,
        "nodes": cfg.node_count(),
        "edges": cfg.edge_count(),
        "cyclomatic": metrics.cyclomatic,
        "essential": metrics.essential,
        "cognitive": metrics.cognitive,
        "max_depth": metrics.max_depth,
        "decision_points": metrics.decision_points,
        "loop_count": metrics.loop_count,
        "grade": format!("{:?}", metrics.grade()),
        "needs_refactoring": metrics.grade().needs_refactoring(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}
