//! AST-to-CFG Bridge (Sprint 5: Formal Control Flow Graph Construction)
//!
//! Walks the parsed `BashStmt` tree and produces a `ControlFlowGraph`.

use crate::bash_parser::ast::{BashStmt, CaseArm};
use crate::quality::cfg::{CfgBuilder, ControlFlowGraph};

/// Build a control flow graph from a sequence of bash statements.
pub fn build_cfg_from_ast(stmts: &[BashStmt]) -> ControlFlowGraph {
    let mut builder = CfgBuilder::new();
    build_stmts(&mut builder, stmts);
    builder.build()
}

fn build_stmts(builder: &mut CfgBuilder, stmts: &[BashStmt]) {
    for stmt in stmts {
        build_stmt(builder, stmt);
    }
}

fn build_stmt(builder: &mut CfgBuilder, stmt: &BashStmt) {
    match stmt {
        BashStmt::Assignment { span, name, .. } => {
            builder.add_block(&format!("{name}=..."), span.start_line, span.end_line);
        }
        BashStmt::Command { span, name, .. } => {
            builder.add_block(name, span.start_line, span.end_line);
        }
        BashStmt::Comment { .. } => {}
        BashStmt::If {
            span,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            build_if(
                builder,
                span.start_line,
                then_block,
                elif_blocks,
                else_block,
            );
        }
        BashStmt::Case { span, arms, .. } => build_case(builder, span.start_line, arms),
        BashStmt::While { span, body, .. } => build_loop(builder, "while", span.start_line, body),
        BashStmt::Until { span, body, .. } => build_loop(builder, "until", span.start_line, body),
        BashStmt::For {
            span,
            variable,
            body,
            ..
        } => {
            build_loop(builder, &format!("for {variable}"), span.start_line, body);
        }
        BashStmt::ForCStyle { span, body, .. } => {
            build_loop(builder, "for((...;...;...))", span.start_line, body);
        }
        BashStmt::Select {
            span,
            variable,
            body,
            ..
        } => {
            build_loop(
                builder,
                &format!("select {variable}"),
                span.start_line,
                body,
            );
        }
        BashStmt::Pipeline { span, commands, .. } => {
            builder.add_block("(pipe-start)", span.start_line, span.start_line);
            for cmd in commands {
                build_stmt(builder, cmd);
            }
        }
        BashStmt::AndList {
            span, left, right, ..
        } => {
            build_short_circuit(builder, "&&", span.start_line, left, right);
        }
        BashStmt::OrList {
            span, left, right, ..
        } => {
            build_short_circuit(builder, "||", span.start_line, left, right);
        }
        BashStmt::Return { span, .. } => {
            builder.add_block("return", span.start_line, span.end_line);
            let unreachable = builder.add_block("(unreachable)", span.end_line, span.end_line);
            builder.add_edge(unreachable, usize::MAX, Some("return"));
        }
        BashStmt::Function { span, name, body } => {
            let fn_node = builder.add_function(name, span.start_line);
            build_stmts(builder, body);
            builder.set_current(fn_node);
        }
        BashStmt::BraceGroup {
            span,
            body,
            subshell,
        } => {
            if *subshell {
                builder.add_block("(subshell)", span.start_line, span.start_line);
            }
            build_stmts(builder, body);
        }
        BashStmt::Coproc { span, name, .. } => {
            let label = name
                .as_deref()
                .map_or("coproc".to_string(), |n| format!("coproc {n}"));
            builder.add_block(&label, span.start_line, span.end_line);
        }
        BashStmt::Negated { command, .. } => build_stmt(builder, command),
    }
}

fn build_if(
    builder: &mut CfgBuilder,
    line: usize,
    then_block: &[BashStmt],
    elif_blocks: &[(crate::bash_parser::ast::BashExpr, Vec<BashStmt>)],
    else_block: &Option<Vec<BashStmt>>,
) {
    let cond_node = builder.add_conditional("if", line);
    build_stmts(builder, then_block);
    let then_exit = builder.add_block("(then-end)", line, line);

    let mut elif_exits = Vec::new();
    for (i, (_expr, body)) in elif_blocks.iter().enumerate() {
        builder.set_current(cond_node);
        let _elif_cond = builder.add_conditional(&format!("elif-{}", i + 1), line);
        build_stmts(builder, body);
        elif_exits.push(builder.add_block("(elif-end)", line, line));
    }

    let else_exit = if let Some(else_body) = else_block {
        builder.set_current(cond_node);
        let _else_node = builder.add_block("else", line, line);
        build_stmts(builder, else_body);
        Some(builder.add_block("(else-end)", line, line))
    } else {
        builder.set_current(cond_node);
        Some(builder.add_block("(no-else)", line, line))
    };

    let merge = builder.add_block("(if-merge)", line, line);
    builder.add_edge(then_exit, merge, Some("merge"));
    for exit in &elif_exits {
        builder.add_edge(*exit, merge, Some("merge"));
    }
    if let Some(e) = else_exit {
        builder.add_edge(e, merge, Some("merge"));
    }
    builder.set_current(merge);
}

fn build_loop(builder: &mut CfgBuilder, label: &str, line: usize, body: &[BashStmt]) {
    let header = builder.add_loop(label, line);
    build_stmts(builder, body);
    let body_end = builder.add_block("(loop-end)", line, line);
    builder.add_edge(body_end, header, Some("back"));
    let exit = builder.add_block("(loop-exit)", line, line);
    builder.add_edge(header, exit, Some("exit-loop"));
    builder.set_current(exit);
}

fn build_case(builder: &mut CfgBuilder, line: usize, arms: &[CaseArm]) {
    let case_node = builder.add_conditional("case", line);
    let mut arm_exits = Vec::new();
    for (i, arm) in arms.iter().enumerate() {
        builder.set_current(case_node);
        let pat = arm.patterns.first().map_or("*".to_string(), |p| p.clone());
        builder.add_block(&format!("pattern-{i}: {pat}"), line, line);
        build_stmts(builder, &arm.body);
        arm_exits.push(builder.add_block(&format!("(arm-{i}-end)"), line, line));
    }
    let merge = builder.add_block("(case-merge)", line, line);
    for exit in &arm_exits {
        builder.add_edge(*exit, merge, Some("merge"));
    }
    builder.set_current(merge);
}

fn build_short_circuit(
    builder: &mut CfgBuilder,
    op: &str,
    line: usize,
    left: &BashStmt,
    right: &BashStmt,
) {
    build_stmt(builder, left);
    let branch = builder.add_conditional(op, line);
    build_stmt(builder, right);
    let right_exit = builder.add_block(&format!("({op}-end)"), line, line);
    builder.set_current(branch);
    let skip = builder.add_block(&format!("({op}-skip)"), line, line);
    let merge = builder.add_block(&format!("({op}-merge)"), line, line);
    builder.add_edge(right_exit, merge, Some("merge"));
    builder.add_edge(skip, merge, Some("short-circuit"));
    builder.set_current(merge);
}

/// Build a CFG for each function in a script, returning (name, CFG) pairs.
pub fn build_per_function_cfgs(stmts: &[BashStmt]) -> Vec<(String, ControlFlowGraph)> {
    let mut results = Vec::new();
    let top_level: Vec<BashStmt> = stmts
        .iter()
        .filter(|s| !matches!(s, BashStmt::Function { .. }))
        .cloned()
        .collect();
    if !top_level.is_empty() {
        results.push(("<main>".to_string(), build_cfg_from_ast(&top_level)));
    }
    for stmt in stmts {
        if let BashStmt::Function { name, body, .. } = stmt {
            results.push((name.clone(), build_cfg_from_ast(body)));
        }
    }
    results
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::bash_parser::ast::{BashExpr, Span};
    use crate::quality::ComplexityMetrics;

    fn span() -> Span {
        Span::dummy()
    }
    fn cmd(name: &str) -> BashStmt {
        BashStmt::Command {
            name: name.to_string(),
            args: vec![],
            redirects: vec![],
            span: span(),
        }
    }

    #[test]
    fn test_cfg_linear() {
        let cfg = build_cfg_from_ast(&[cmd("echo"), cmd("ls")]);
        assert!(cfg.node_count() >= 2);
    }

    #[test]
    fn test_cfg_empty() {
        let cfg = build_cfg_from_ast(&[]);
        assert_eq!(cfg.nodes.len(), 2); // Entry + Exit
    }

    #[test]
    fn test_cfg_if_else() {
        let stmts = vec![BashStmt::If {
            condition: BashExpr::Literal("true".into()),
            then_block: vec![cmd("a")],
            elif_blocks: vec![],
            else_block: Some(vec![cmd("b")]),
            span: span(),
        }];
        let m = ComplexityMetrics::from_cfg(&build_cfg_from_ast(&stmts));
        assert!(m.decision_points >= 1);
    }

    #[test]
    fn test_cfg_while_back_edge() {
        let stmts = vec![BashStmt::While {
            condition: BashExpr::Literal("true".into()),
            body: vec![cmd("echo")],
            span: span(),
        }];
        let cfg = build_cfg_from_ast(&stmts);
        assert!(cfg.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_cfg_case() {
        let stmts = vec![BashStmt::Case {
            word: BashExpr::Variable("x".into()),
            arms: vec![
                CaseArm {
                    patterns: vec!["a".into()],
                    body: vec![cmd("a")],
                },
                CaseArm {
                    patterns: vec!["*".into()],
                    body: vec![cmd("default")],
                },
            ],
            span: span(),
        }];
        let m = ComplexityMetrics::from_cfg(&build_cfg_from_ast(&stmts));
        assert!(m.decision_points >= 1);
    }

    #[test]
    fn test_cfg_and_list() {
        let stmts = vec![BashStmt::AndList {
            left: Box::new(cmd("test")),
            right: Box::new(cmd("echo")),
            span: span(),
        }];
        let m = ComplexityMetrics::from_cfg(&build_cfg_from_ast(&stmts));
        assert!(m.decision_points >= 1);
    }

    #[test]
    fn test_cfg_per_function() {
        let stmts = vec![
            cmd("setup"),
            BashStmt::Function {
                name: "helper".into(),
                body: vec![cmd("echo")],
                span: span(),
            },
        ];
        let cfgs = build_per_function_cfgs(&stmts);
        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].0, "<main>");
        assert_eq!(cfgs[1].0, "helper");
    }

    #[test]
    fn test_cfg_nested() {
        let stmts = vec![BashStmt::While {
            condition: BashExpr::Literal("true".into()),
            body: vec![BashStmt::If {
                condition: BashExpr::Literal("c".into()),
                then_block: vec![cmd("a")],
                elif_blocks: vec![],
                else_block: Some(vec![cmd("b")]),
                span: span(),
            }],
            span: span(),
        }];
        let m = ComplexityMetrics::from_cfg(&build_cfg_from_ast(&stmts));
        assert!(m.loop_count >= 1);
        assert!(m.decision_points >= 1);
    }
}
