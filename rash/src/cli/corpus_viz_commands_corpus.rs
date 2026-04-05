/// Schema enforcement layer status per format (spec §11.8).
pub(crate) fn corpus_schema() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let formats = ["Bash", "Makefile", "Dockerfile"];

    println!("{BOLD}Schema Enforcement Layers{RESET} (spec §11.8)");
    println!();
    println!(
        "  {BOLD}{:<12} {:>8} {:>8} {:>8} {:>8} {:>8}{RESET}",
        "Format", "Total", "L1:Lex", "L2:Syn", "L3:Sem", "L4:Beh"
    );

    for fmt_name in &formats {
        let fmt_filter = match *fmt_name {
            "Bash" => crate::corpus::registry::CorpusFormat::Bash,
            "Makefile" => crate::corpus::registry::CorpusFormat::Makefile,
            _ => crate::corpus::registry::CorpusFormat::Dockerfile,
        };

        let entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.format == fmt_filter)
            .collect();

        let total = entries.len();
        let (l1, l2, l3, l4) = schema_layer_counts(&score.results, &entries);

        let l1c = pct_color(l1 as f64 / total.max(1) as f64 * 100.0);
        let l2c = pct_color(l2 as f64 / total.max(1) as f64 * 100.0);
        let l3c = pct_color(l3 as f64 / total.max(1) as f64 * 100.0);
        let l4c = pct_color(l4 as f64 / total.max(1) as f64 * 100.0);

        println!("  {CYAN}{:<12}{RESET} {:>8} {l1c}{:>8}{RESET} {l2c}{:>8}{RESET} {l3c}{:>8}{RESET} {l4c}{:>8}{RESET}",
            fmt_name, total, l1, l2, l3, l4);
    }

    println!();
    println!("  {DIM}L1=Lexical(transpile) L2=Syntactic(lint) L3=Semantic(det+meta) L4=Behavioral(exec+cross-shell){RESET}");

    Ok(())
}

/// ASCII chart of score over iterations from convergence log.
pub(crate) fn corpus_history_chart() -> Result<()> {
    use crate::cli::color::*;

    let log_path = std::path::Path::new(".quality/convergence.log");
    if !log_path.exists() {
        let log_path2 = std::path::Path::new("../.quality/convergence.log");
        if !log_path2.exists() {
            println!("{YELLOW}No convergence log found{RESET}");
            return Ok(());
        }
        return corpus_history_chart_from(log_path2);
    }
    corpus_history_chart_from(log_path)
}

/// Render a single chart cell for history chart.
pub(crate) fn history_chart_cell(
    score: f64,
    row: usize,
    min_score: f64,
    range: f64,
    height: usize,
) {
    use crate::cli::color::*;
    if score <= 0.0 {
        print!(" ");
    } else {
        let normalized = (score - min_score) / range * height as f64;
        if normalized >= row as f64 {
            let color = if score >= 99.0 {
                GREEN
            } else if score >= 95.0 {
                YELLOW
            } else {
                RED
            };
            print!("{color}█{RESET}");
        } else {
            print!(" ");
        }
    }
}

pub(crate) fn corpus_history_chart_from(path: &std::path::Path) -> Result<()> {
    use crate::cli::color::*;

    let content = std::fs::read_to_string(path).map_err(Error::Io)?;

    let entries: Vec<crate::corpus::runner::ConvergenceEntry> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    if entries.is_empty() {
        println!("{YELLOW}No convergence entries found{RESET}");
        return Ok(());
    }

    println!(
        "{BOLD}Score History Chart{RESET} ({} iterations)",
        entries.len()
    );
    println!();

    let scores: Vec<f64> = entries
        .iter()
        .filter_map(|e| if e.score > 0.0 { Some(e.score) } else { None })
        .collect();

    let min_score = scores.iter().copied().fold(f64::MAX, f64::min);
    let max_score = scores.iter().copied().fold(0.0f64, f64::max);
    let chart_height = 10usize;
    let range = (max_score - min_score).max(0.1);

    for row in (0..chart_height).rev() {
        let threshold = min_score + range * row as f64 / chart_height as f64;
        print!("  {DIM}{:>6.1}{RESET} │", threshold);
        for entry in &entries {
            history_chart_cell(entry.score, row, min_score, range, chart_height);
        }
        println!();
    }

    print!("  {DIM}{:>6}{RESET} └", "");
    for _ in &entries {
        print!("─");
    }
    println!();
    print!("  {DIM}{:>6}  ", "");
    for entry in &entries {
        print!("{}", entry.iteration % 10);
    }
    println!("{RESET}");

    println!();
    if let Some(last) = entries.last() {
        println!(
            "  {DIM}Latest: iter {} | {:.1}/100 | {} entries{RESET}",
            last.iteration, last.score, last.total
        );
    }

    Ok(())
}
