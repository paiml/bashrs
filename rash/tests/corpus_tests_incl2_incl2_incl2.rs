fn test_CORPUS_037_falsify_lint_failures() {
    // Transpile the 4 lint-failing entries and show what lint flags
    let config = bashrs::Config::default();
    let failing_inputs = vec![
        (
            "B-046",
            r#"fn main() { let mut x = 0; let mut y = 10; while x < 5 && y > 0 { x = x + 1; y = y - 1; } }"#,
        ),
        (
            "B-116",
            r#"fn main() { let mut a = 0; let mut b = 100; while a < 50 || b > 50 { a += 1; b -= 1; } }"#,
        ),
        (
            "B-138",
            r#"fn main() { let max_retries = 3; let mut attempts = 0; let mut success = false; while attempts < max_retries && !success { attempts += 1; if attempts >= 2 { success = true; } } }"#,
        ),
        (
            "B-160",
            r#"fn main() { let mut interrupted = false; let mut completed = false; let mut retries = 0; while !completed && !interrupted { retries += 1; if retries >= 5 { completed = true; } } }"#,
        ),
    ];

    for (id, input) in &failing_inputs {
        match bashrs::transpile(input, &config) {
            Ok(output) => {
                let lint = bashrs::linter::rules::lint_shell(&output);
                eprintln!("\n=== {} (lint_clean={}) ===", id, !lint.has_errors());
                eprintln!("--- output ---\n{}", output);
                if lint.has_errors() {
                    eprintln!("--- lint violations ---");
                    for d in &lint.diagnostics {
                        eprintln!("  {}: {} ({})", d.code, d.message, d.severity);
                    }
                }
            }
            Err(e) => eprintln!("\n=== {} TRANSPILE FAILED: {} ===", id, e),
        }
    }
}

#[test]
fn test_CORPUS_038_falsify_cross_shell_failures() {
    // Transpile B-001, B-021, B-023 with Posix and Bash dialect configs
    let entries = vec![
        (
            "B-001",
            r#"fn main() { let greeting = "hello"; } "#,
            "greeting='hello'",
        ),
        (
            "B-021",
            r#"fn main() { let x = 5; if x > 10 { let r = "big"; } else if x > 3 { let r = "medium"; } else { let r = "small"; } }"#,
            "elif",
        ),
    ];

    for (id, input, expected) in &entries {
        let posix_config = bashrs::Config {
            target: bashrs::models::ShellDialect::Posix,
            ..bashrs::Config::default()
        };
        let bash_config = bashrs::Config {
            target: bashrs::models::ShellDialect::Bash,
            ..bashrs::Config::default()
        };

        let posix_out = bashrs::transpile(input, &posix_config);
        let bash_out = bashrs::transpile(input, &bash_config);

        eprintln!("\n=== {} (expected contains: '{}') ===", id, expected);
        match &posix_out {
            Ok(o) => eprintln!(
                "POSIX: contains={} len={}\n{}",
                o.contains(expected),
                o.len(),
                o
            ),
            Err(e) => eprintln!("POSIX: FAILED - {}", e),
        }
        match &bash_out {
            Ok(o) => eprintln!(
                "BASH:  contains={} len={}\n{}",
                o.contains(expected),
                o.len(),
                o
            ),
            Err(e) => eprintln!("BASH:  FAILED - {}", e),
        }
    }
}

#[test]
fn test_CORPUS_039_b1_containment_failures() {
    let registry = bashrs::corpus::CorpusRegistry::load_full();
    let config = bashrs::Config::default();
    let runner = bashrs::corpus::CorpusRunner::new(config);
    let score = runner.run(&registry);

    let entry_map: std::collections::HashMap<&str, &bashrs::corpus::CorpusEntry> = registry
        .entries
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect();

    let b1_failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| r.transpiled && !r.output_contains)
        .collect();

    for r in &b1_failures {
        if let Some(entry) = entry_map.get(r.id.as_str()) {
            eprintln!(
                "\nB1 FAIL: {} expected='{}' actual=\n{}",
                r.id,
                entry.expected_output,
                r.actual_output.as_deref().unwrap_or("(none)")
            );
        }
    }

    // B1 containment rate must be >= 95% (currently ~95.8% at 16K+ entries)
    let transpiled: Vec<_> = score.results.iter().filter(|r| r.transpiled).collect();
    let b1_rate = if transpiled.is_empty() {
        0.0
    } else {
        (transpiled.len() - b1_failures.len()) as f64 / transpiled.len() as f64 * 100.0
    };
    assert!(
        b1_rate >= 95.0,
        "B1 containment rate {:.1}% below 95% threshold ({} failures out of {} transpiled)",
        b1_rate,
        b1_failures.len(),
        transpiled.len()
    );
}
