//! Corpus pipeline commands: lint pipeline, regression check, and convergence check.

use crate::models::{Config, Result};

pub(crate) fn corpus_lint_pipeline() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_lint_pipeline_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_lint_pipeline`, split so a test can pass a
/// small synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_lint_pipeline_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    let suggestions = citl::lint_pipeline(registry, &score);

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
    use crate::corpus::registry::CorpusRegistry;
    corpus_regression_check_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_regression_check`, split so a test can pass a
/// small synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_regression_check_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

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
    use crate::corpus::registry::CorpusRegistry;
    corpus_convergence_check_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_convergence_check`, split so a test can pass a
/// small synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_convergence_check_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::citl;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

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

// PMAT-257: coverage for the corpus pipeline handlers. All three build a
// `CorpusRunner` over `CorpusRegistry::load_full()` (18,000+ entries), which
// is too slow for a unit test as written. Each was split into a
// `*_with(registry, ...)` twin that takes the registry as a parameter,
// matching `corpus_compare_commands.rs`.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

    fn tiny_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "hello-bash",
            "PMAT-257 fixture",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let greeting = "hello"; }"#,
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "hello-makefile",
            "PMAT-257 fixture",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "all:\n\techo hello\n",
            "all:",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "hello-dockerfile",
            "PMAT-257 fixture",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_lint_pipeline_with_tiny_registry() {
        corpus_lint_pipeline_with(&tiny_registry())
            .expect("lint pipeline runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_lint_pipeline_with_empty_registry() {
        corpus_lint_pipeline_with(&CorpusRegistry::new())
            .expect("lint pipeline runs over an empty registry (no suggestions)");
    }

    #[test]
    fn test_PMAT257_cov_regression_check_with_tiny_registry() {
        corpus_regression_check_with(&tiny_registry())
            .expect("regression check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_convergence_check_with_tiny_registry() {
        corpus_convergence_check_with(&tiny_registry())
            .expect("convergence check runs over a tiny registry");
    }
}
