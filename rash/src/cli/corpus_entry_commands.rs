//! Corpus entry checking, difficulty classification, and risk analysis commands.

use super::corpus_failure_commands::result_fail_dims;
use super::corpus_score_print_commands::stats_bar;
use crate::cli::args::CorpusOutputFormat;
use crate::models::{Config, Error, Result};

pub(crate) fn corpus_check_entry(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry {id} not found")))?;

    let config = Config::default();
    let runner = CorpusRunner::new(config);
    let result = runner.run_single(entry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Metamorphic Check: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 60));
            println!();

            let mr_pass = |name: &str, ok: bool, desc: &str| {
                let mark = if ok {
                    format!("{GREEN}PASS{RESET}")
                } else {
                    format!("{BRIGHT_RED}FAIL{RESET}")
                };
                println!("  {name:<22} {mark}  {DIM}{desc}{RESET}");
            };

            // MR-1: Determinism — transpile twice, same output
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            mr_pass("MR-1 Determinism", mr1, "transpile(X) == transpile(X)");

            // MR-2: Transpilation success
            mr_pass(
                "MR-2 Transpilation",
                result.transpiled,
                "transpile(X) succeeds",
            );

            // MR-3: Containment
            mr_pass(
                "MR-3 Containment",
                result.output_contains,
                "output ⊇ expected_contains",
            );

            // MR-4: Exact match
            mr_pass(
                "MR-4 Exact match",
                result.output_exact,
                "output lines == expected_contains",
            );

            // MR-5: Behavioral execution
            mr_pass(
                "MR-5 Behavioral",
                result.output_behavioral,
                "sh -c output terminates",
            );

            // MR-6: Lint clean
            mr_pass(
                "MR-6 Lint clean",
                result.lint_clean,
                "shellcheck/make -n passes",
            );

            // MR-7: Cross-shell agree
            mr_pass(
                "MR-7 Cross-shell",
                result.cross_shell_agree,
                "sh + dash agree",
            );

            let total = 7u32;
            let passed = [
                mr1,
                result.transpiled,
                result.output_contains,
                result.output_exact,
                result.output_behavioral,
                result.lint_clean,
                result.cross_shell_agree,
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u32;
            let pct = (passed as f64 / total as f64) * 100.0;
            let pc = pct_color(pct);
            println!();
            println!("{BOLD}MR Pass Rate:{RESET} {pc}{passed}/{total} ({pct:.0}%){RESET}");
            println!(
                "{BOLD}V2 Score:{RESET} {pc}{:.1}/100{RESET}",
                result.score()
            );
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct MrCheck {
                name: String,
                passed: bool,
                description: String,
            }
            #[derive(serde::Serialize)]
            struct CheckResult {
                id: String,
                checks: Vec<MrCheck>,
                passed: u32,
                total: u32,
                score: f64,
            }
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            let checks = vec![
                MrCheck {
                    name: "MR-1 Determinism".into(),
                    passed: mr1,
                    description: "transpile(X) == transpile(X)".into(),
                },
                MrCheck {
                    name: "MR-2 Transpilation".into(),
                    passed: result.transpiled,
                    description: "transpile(X) succeeds".into(),
                },
                MrCheck {
                    name: "MR-3 Containment".into(),
                    passed: result.output_contains,
                    description: "output ⊇ expected".into(),
                },
                MrCheck {
                    name: "MR-4 Exact match".into(),
                    passed: result.output_exact,
                    description: "output == expected".into(),
                },
                MrCheck {
                    name: "MR-5 Behavioral".into(),
                    passed: result.output_behavioral,
                    description: "sh -c terminates".into(),
                },
                MrCheck {
                    name: "MR-6 Lint clean".into(),
                    passed: result.lint_clean,
                    description: "linter passes".into(),
                },
                MrCheck {
                    name: "MR-7 Cross-shell".into(),
                    passed: result.cross_shell_agree,
                    description: "sh + dash agree".into(),
                },
            ];
            let passed = checks.iter().filter(|c| c.passed).count() as u32;
            let cr = CheckResult {
                id: id.to_string(),
                checks,
                passed,
                total: 7,
                score: result.score(),
            };
            let json = serde_json::to_string_pretty(&cr)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Truncate a string to max_len, adding "..." if truncated.
pub(crate) fn truncate_line(s: &str, max_len: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

/// Classify a corpus entry's difficulty based on input features (spec §2.3).
/// Returns tier 1-5 with factor breakdown.

include!("corpus_entry_commands_incl2.rs");
