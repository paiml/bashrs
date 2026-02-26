//! Corpus pipeline commands: lint pipeline, regression check, and convergence check.

use crate::models::{Config, Result};

pub(crate) fn corpus_lint_pipeline() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let suggestions = citl::lint_pipeline(&registry, &score);

    println!("{BOLD}CITL Lint Pipeline (\u{00a7}7.3){RESET}");
    println!();

    let table = citl::format_lint_pipeline(&suggestions);
    for line in table.lines() {
        let colored = line
            .replace("SEC", &format!("{RED}SEC{RESET}"))
            .replace("DET", &format!("{YELLOW}DET{RESET}"))
            .replace("IDEM", &format!("{YELLOW}IDEM{RESET}"))
            .replace("MAKE", &format!("{CYAN}MAKE{RESET}"))
            .replace("DOCKER", &format!("{GREEN}DOCKER{RESET}"))
            .replace("CITL loop clean", &format!("{GREEN}CITL loop clean{RESET}"));
        println!("  {colored}");
    }

    if suggestions.is_empty() {
        println!("  {GREEN}\u{2713} All transpiled output passes lint{RESET}");
    } else {
        println!();
        println!(
            "  {YELLOW}\u{26a0} {} lint violation(s) \u{2192} corpus entry suggestion(s){RESET}",
            suggestions.len()
        );
        println!("  Fix the transpiler, then re-run (\u{00a7}1.2: fix transpiler, never corpus)");
    }

    Ok(())
}

pub(crate) fn corpus_regression_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let log_path = std::path::Path::new(".quality/convergence.log");
    let history = CorpusRunner::load_convergence_log(log_path).unwrap_or_default();

    let report = citl::check_regressions(&score, &history);

    println!("{BOLD}Jidoka Regression Check (\u{00a7}5.3){RESET}");
    println!();

    let table = citl::format_regression_report(&report);
    for line in table.lines() {
        let colored = line
            .replace("No regressions", &format!("{GREEN}No regressions{RESET}"))
            .replace(
                "REGRESSIONS DETECTED",
                &format!("{RED}REGRESSIONS DETECTED{RESET}"),
            )
            .replace("ANDON CORD", &format!("{BRIGHT_RED}ANDON CORD{RESET}"))
            .replace("Status: OK", &format!("{GREEN}Status: OK{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_convergence_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let log_path = std::path::Path::new(".quality/convergence.log");
    let history = CorpusRunner::load_convergence_log(log_path).unwrap_or_default();

    let criteria = citl::check_convergence(&score, &history);

    println!("{BOLD}Convergence Criteria Check (\u{00a7}5.2){RESET}");
    println!();

    let table = citl::format_convergence_criteria(&criteria);
    for line in table.lines() {
        let colored = line
            .replace("\u{2713} PASS", &format!("{GREEN}\u{2713} PASS{RESET}"))
            .replace("\u{2717} FAIL", &format!("{RED}\u{2717} FAIL{RESET}"))
            .replace("CONVERGED:", &format!("{GREEN}CONVERGED:{RESET}"))
            .replace("NOT CONVERGED:", &format!("{RED}NOT CONVERGED:{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}
