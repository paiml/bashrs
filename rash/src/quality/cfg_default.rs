impl Default for CfgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a padded line inside box borders: `║<content padded to inner>║\n`
fn write_box_line(out: &mut String, content: &str, inner: usize) {
    out.push('║');
    out.push_str(content);
    for _ in 0..inner.saturating_sub(content.len()) {
        out.push(' ');
    }
    out.push_str("║\n");
}

/// Write a horizontal rule: `<left><fill * inner><right>\n`
fn write_hrule(out: &mut String, left: char, fill: char, right: &str, inner: usize) {
    out.push(left);
    for _ in 0..inner {
        out.push(fill);
    }
    out.push_str(right);
    out.push('\n');
}

/// ASCII CFG visualization (ML-017)
pub fn render_cfg_ascii(
    cfg: &ControlFlowGraph,
    metrics: &ComplexityMetrics,
    width: usize,
) -> String {
    let mut out = String::new();
    let inner = width - 2;

    write_hrule(&mut out, '╔', '═', "╗", inner);

    // Title centered
    let title = "CONTROL FLOW GRAPH";
    let padding = (inner.saturating_sub(title.len())) / 2;
    let mut title_line = String::new();
    for _ in 0..padding {
        title_line.push(' ');
    }
    title_line.push_str(title);
    for _ in 0..(inner - padding - title.len()) {
        title_line.push(' ');
    }
    write_box_line(&mut out, &title_line, inner);

    write_hrule(&mut out, '╠', '═', "╣", inner);

    // Entry node
    let entry_lines = [
        "                          ┌─────────┐",
        "                          │  ENTRY  │",
        "                          └────┬────┘",
        "                               │",
    ];
    for line in &entry_lines {
        write_box_line(&mut out, line, inner);
    }

    // Show conditional if present
    if metrics.decision_points > 0 {
        let cond_lines = [
            "                          ┌────▼────┐",
            "                          │ if cond │",
            "                          └────┬────┘",
            "                     ┌────────┼────────┐",
            "                     │ TRUE   │  FALSE │",
            "                     └────────┼────────┘",
            "                               │",
        ];
        for line in &cond_lines {
            write_box_line(&mut out, line, inner);
        }
    }

    // Exit node
    let exit_lines = [
        "                          ┌───▼────┐",
        "                          │  EXIT  │",
        "                          └────────┘",
    ];
    for line in &exit_lines {
        write_box_line(&mut out, line, inner);
    }

    // Metrics section
    write_hrule(&mut out, '╠', '═', "╣", inner);

    let metrics_line = format!(
        "  Nodes: {} │ Edges: {} │ Cyclomatic: {} │ Essential: {} │ Max Depth: {}",
        cfg.node_count(),
        cfg.edge_count(),
        metrics.cyclomatic,
        metrics.essential,
        metrics.max_depth
    );
    let truncated = if metrics_line.len() > inner {
        &metrics_line[..inner]
    } else {
        &metrics_line
    };
    write_box_line(&mut out, truncated, inner);

    write_hrule(&mut out, '╚', '═', "╝", inner);

    out
}

#[cfg(test)]
#[path = "cfg_tests_ml_015.rs"]
mod tests_extracted;
