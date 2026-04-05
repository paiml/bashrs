impl RichLintReport {

    fn render_clusters(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "ERROR CLUSTERS (Pareto Analysis)", width);

        // Header
        let header = "  Cluster │ Count │ Distribution          │ Category    │ Fix Confidence";
        self.render_line(out, header, width);

        // Divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        // Cluster rows
        let max_count = self.clusters.first().map_or(1, |c| c.count);
        for cluster in self.clusters.iter().take(5) {
            let bar = histogram_bar(cluster.count as f64, max_count as f64, 20);
            let confidence_str = if cluster.auto_fixable {
                format!("{:.0}% (auto-fix)", cluster.fix_confidence * 100.0)
            } else {
                format!("{:.0}% (manual)", cluster.fix_confidence * 100.0)
            };

            let line = format!(
                "  {:>7} │ {:>5} │ {} │ {:>11} │ {}",
                cluster.error_code,
                cluster.count,
                bar,
                &cluster.category.name()[..11.min(cluster.category.name().len())],
                confidence_str
            );
            self.render_line(out, &line, width);
        }
    }

    fn render_sbfl(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        if self.sbfl_rankings.is_empty() {
            return;
        }

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "FAULT LOCALIZATION (Ochiai SBFL)", width);

        // Header
        let header = "  Rank │ Location          │ Suspiciousness │ Root Cause";
        self.render_line(out, header, width);

        // Divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        // SBFL rows
        for ranking in self.sbfl_rankings.iter().take(5) {
            let bar = histogram_bar(ranking.score, 1.0, 10);
            let line = format!(
                "  {:>4} │ {:>17} │ {} {:.2}│ {}",
                ranking.rank,
                ranking.location,
                bar,
                ranking.score,
                self.get_root_cause_desc(&ranking.location)
            );
            self.render_line(out, &line, width);
        }
    }

    fn render_recommendations(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(
            out,
            "RECOMMENDED ACTIONS (Toyota Way: Start with highest impact)",
            width,
        );

        // Auto-fix recommendation
        if self.auto_fixable_count > 0 {
            let line1 = format!("  1. Run: bashrs lint {} --fix", self.source_file);
            self.render_line(out, &line1, width);

            let auto_codes: Vec<_> = self
                .clusters
                .iter()
                .filter(|c| c.auto_fixable)
                .take(3)
                .map(|c| c.error_code.as_str())
                .collect();
            let line2 = format!(
                "     → Auto-fixes {} issues ({})",
                self.auto_fixable_count,
                auto_codes.join(", ")
            );
            self.render_line(out, &line2, width);
            self.render_line(out, "", width);
        }

        // Manual review
        if self.manual_count > 0 {
            let manual_clusters: Vec<_> = self
                .clusters
                .iter()
                .filter(|c| !c.auto_fixable)
                .take(2)
                .collect();

            if !manual_clusters.is_empty() {
                let line = format!(
                    "  2. Manual review required for {} ({} issues)",
                    manual_clusters
                        .first()
                        .map_or("", |c| c.error_code.as_str()),
                    self.manual_count
                );
                self.render_line(out, &line, width);
            }
        }
    }

    fn render_footer(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "CITL EXPORT", width);

        let line1 = format!(
            "  Export: bashrs lint {} --citl-export diagnostics.json",
            self.source_file
        );
        self.render_line(out, &line1, width);
        self.render_line(
            out,
            "  Integration: organizational-intelligence-plugin for ML training",
            width,
        );

        // Bottom border
        out.push(box_chars::BOTTOM_LEFT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::BOTTOM_RIGHT);
        out.push('\n');
    }

    fn render_section_title(&self, out: &mut String, title: &str, width: usize) {
        let inner = width - 2;
        out.push(box_chars::VERTICAL);
        out.push(' ');
        out.push_str(title);
        for _ in 0..(inner - title.len() - 1) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    fn render_line(&self, out: &mut String, content: &str, width: usize) {
        let inner = width - 2;
        out.push(box_chars::VERTICAL);
        let truncated = if content.len() > inner {
            &content[..inner]
        } else {
            content
        };
        out.push_str(truncated);
        for _ in 0..(inner.saturating_sub(content.len())) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    fn get_root_cause_desc(&self, location: &str) -> String {
        // Find cluster for location and describe root cause
        self.clusters
            .iter()
            .find(|c| c.error_code == location)
            .map_or_else(|| "Unknown".to_string(), |c| c.category.name().to_string())
    }
}










include!("lint_report_sbfl.rs");
