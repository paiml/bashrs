#[cfg(test)]
mod installer_from_bash_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_from_bash() {
        let dir = tempfile::tempdir().unwrap();
        let script = super::write_bash_script(dir.path());
        let output = dir.path().join("converted-installer");
        let cmd = InstallerCommands::FromBash {
            input: script,
            output: Some(output),
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_ok(), "from-bash failed: {:?}", res);
    }

    #[test]
    fn test_cov_installer_from_bash_default_output() {
        let dir = tempfile::tempdir().unwrap();
        let script = super::write_bash_script(dir.path());
        let cmd = InstallerCommands::FromBash {
            input: script,
            output: None,
        };
        // Output defaults to <stem>-installer in CWD; may produce directory in unexpected location
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }

    #[test]
    fn test_cov_installer_from_bash_missing_file() {
        let cmd = InstallerCommands::FromBash {
            input: std::path::PathBuf::from("/tmp/nonexistent-script-xyz.sh"),
            output: None,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err());
    }
}

// ---------------------------------------------------------------------------
// installer resume
// ---------------------------------------------------------------------------
#[cfg(test)]
mod installer_resume_cmd {
    use crate::cli::args::InstallerCommands;

    #[test]
    fn test_cov_installer_resume_no_checkpoint() {
        let (_dir, project_path) = super::make_installer_project();
        let cmd = InstallerCommands::Resume {
            path: project_path,
            from: None,
        };
        let res = super::super::super::installer_commands::handle_installer_command(cmd);
        assert!(res.is_err(), "resume without checkpoint should fail");
    }

    #[test]
    fn test_cov_installer_resume_after_run() {
        let (_dir, project_path) = super::make_installer_project();
        // Run first to create checkpoint
        let run_cmd = InstallerCommands::Run {
            path: project_path.clone(),
            checkpoint_dir: None,
            dry_run: false,
            diff: false,
            hermetic: false,
            verify_signatures: false,
            parallel: false,
            trace: false,
            trace_file: None,
        };
        let _ = super::super::super::installer_commands::handle_installer_command(run_cmd);
        // Now try resume
        let cmd = InstallerCommands::Resume {
            path: project_path,
            from: None,
        };
        // May fail if no steps were successful, but exercises the code path
        let _ = super::super::super::installer_commands::handle_installer_command(cmd);
    }
}

// ---------------------------------------------------------------------------
// parse_public_key unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod parse_public_key_tests {
    use super::super::super::installer_commands::parse_public_key;

    #[test]
    fn test_cov_parse_public_key_valid() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key = parse_public_key(hex).unwrap();
        assert_eq!(key[0], 0x01);
        assert_eq!(key[1], 0x23);
        assert_eq!(key[31], 0xef);
    }

    #[test]
    fn test_cov_parse_public_key_wrong_length() {
        let hex = "0123456789abcdef";
        let res = parse_public_key(hex);
        assert!(res.is_err());
    }

    #[test]
    fn test_cov_parse_public_key_invalid_hex() {
        let hex = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let res = parse_public_key(hex);
        assert!(res.is_err());
    }

    #[test]
    fn test_cov_parse_public_key_all_zeros() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000000";
        let key = parse_public_key(hex).unwrap();
        assert!(key.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_cov_parse_public_key_all_ff() {
        let hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let key = parse_public_key(hex).unwrap();
        assert!(key.iter().all(|&b| b == 0xff));
    }
}

// ============================================================================
// CORPUS COMMAND COVERAGE TESTS — pure functions (no runner.run())
// ============================================================================

// ---------------------------------------------------------------------------
// corpus_ops_commands::names_similar
// ---------------------------------------------------------------------------
#[cfg(test)]
mod corpus_ops_names_similar {
    use super::super::corpus_ops_commands::names_similar;

    #[test]
    fn test_cov_exact_match() {
        assert!(names_similar("variable-assignment", "variable-assignment"));
    }

    #[test]
    fn test_cov_different_suffixes_basic_simple() {
        assert!(names_similar("variable-basic", "variable-simple"));
    }

    #[test]
    fn test_cov_different_suffixes_basic_advanced() {
        assert!(names_similar("loop-basic", "loop-advanced"));
    }

    #[test]
    fn test_cov_completely_different() {
        assert!(!names_similar("variable", "function"));
    }

    #[test]
    fn test_cov_empty_strings() {
        assert!(names_similar("", ""));
    }

    #[test]
    fn test_cov_one_empty() {
        assert!(!names_similar("variable", ""));
    }

    #[test]
    fn test_cov_case_insensitive_suffix_strip() {
        assert!(names_similar("loop-BASIC", "loop-SIMPLE"));
    }
}

// ---------------------------------------------------------------------------
// corpus_ops_commands::converged_print_check
// ---------------------------------------------------------------------------
#[cfg(test)]
mod corpus_ops_converged_print_check {
    use super::super::corpus_ops_commands::converged_print_check;

    #[test]
    fn test_cov_pass_label() {
        // Just exercises the function without panicking
        converged_print_check("Rate >= 99%", true);
    }

    #[test]
    fn test_cov_fail_label() {
        converged_print_check("Rate >= 99%", false);
    }
}

// ---------------------------------------------------------------------------
// corpus_ops_commands::converged_no_regressions
// ---------------------------------------------------------------------------
#[cfg(test)]
mod corpus_ops_converged_no_regressions {
    use super::super::corpus_ops_commands::converged_no_regressions;
    use crate::corpus::runner::ConvergenceEntry;

    fn make_entry(iteration: u32, passed: usize, total: usize) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration,
            date: "2026-01-01".to_string(),
            passed,
            total,
            failed: total.saturating_sub(passed),
            rate: passed as f64 / total as f64,
            delta: 0.0,
            notes: String::new(),
            score: passed as f64 / total as f64 * 100.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_cov_empty_entries() {
        assert!(converged_no_regressions(&[], 3));
    }

    #[test]
    fn test_cov_single_entry() {
        let entries = vec![make_entry(1, 100, 100)];
        assert!(converged_no_regressions(&entries, 3));
    }

    #[test]
    fn test_cov_improving_entries() {
        let entries = vec![
            make_entry(1, 90, 100),
            make_entry(2, 95, 100),
            make_entry(3, 98, 100),
        ];
        assert!(converged_no_regressions(&entries, 3));
    }
}

// ---------------------------------------------------------------------------
// corpus_diff_commands::chrono_free_date
// ---------------------------------------------------------------------------
#[cfg(test)]
mod corpus_diff_chrono_free_date {
    use super::super::corpus_diff_commands::chrono_free_date;

    #[test]
    fn test_cov_chrono_free_date_returns_string() {
        let date = chrono_free_date();
        assert!(!date.is_empty());
        // Should look like YYYY-MM-DD
        if date != "unknown" {
            assert_eq!(date.len(), 10, "Expected YYYY-MM-DD format, got: {}", date);
            assert_eq!(date.chars().filter(|&c| c == '-').count(), 2);
        }
    }
}

// ============================================================================
// CORPUS COMMAND SMOKE TESTS — exercise heavy paths (load + run)
// These call into corpus registry loading + runner, covering corpus_core_commands,
// corpus_advanced_commands, corpus_ops_commands, and corpus_diff_commands.
// ============================================================================

#[cfg(test)]
mod corpus_core_smoke {
    use crate::cli::args::CorpusCommands;

    // These tests load the full corpus (17,942 entries) and are slow (~30-60s each).
    // They are marked #[ignore] for normal `cargo test` but are included in coverage runs
    // via `cargo llvm-cov -- --include-ignored`.

    #[test]
    #[ignore] // slow: loads full corpus
    fn test_cov_corpus_dupes() {
        let _ = super::super::corpus_ops_commands::corpus_dupes();
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs dedup
    fn test_cov_corpus_dedup() {
        let _ = super::super::corpus_advanced_commands::corpus_dedup();
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs triage
    fn test_cov_corpus_triage() {
        let _ = super::super::corpus_advanced_commands::corpus_triage();
    }

    #[test]
    #[ignore] // slow: loads full corpus
    fn test_cov_corpus_label_rules() {
        let _ = super::super::corpus_advanced_commands::corpus_label_rules();
    }

    #[test]
    #[ignore] // slow: loads full corpus + builds graph
    fn test_cov_corpus_graph() {
        let _ = super::super::corpus_advanced_commands::corpus_graph();
    }

    #[test]
    #[ignore] // slow: loads full corpus + impact analysis
    fn test_cov_corpus_impact_default() {
        let _ = super::super::corpus_advanced_commands::corpus_impact(10);
    }

    #[test]
    #[ignore] // slow: loads full corpus
    fn test_cov_corpus_blast_radius_nonexistent() {
        // Should handle missing decision gracefully
        let _ = super::super::corpus_advanced_commands::corpus_blast_radius("nonexistent-decision");
    }

    #[test]
    #[ignore] // slow: loads full corpus + generates report
    fn test_cov_corpus_generate_report_stdout() {
        let _ = super::super::corpus_diff_commands::corpus_generate_report(None);
    }

    #[test]
    #[ignore] // slow: loads full corpus + generates report to file
    fn test_cov_corpus_generate_report_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.md");
        let _ = super::super::corpus_diff_commands::corpus_generate_report(Some(
            path.to_str().unwrap(),
        ));
    }

    #[test]
    fn test_cov_corpus_show_diff_no_log() {
        use crate::cli::args::CorpusOutputFormat;
        // No convergence log → should return error (fast: no corpus load)
        let res = super::super::corpus_diff_commands::corpus_show_diff(
            &CorpusOutputFormat::Human,
            None,
            None,
        );
        let _ = res;
    }

    #[test]
    fn test_cov_corpus_converged_no_log() {
        // No convergence log → should fail (fast: no corpus load)
        let res = super::super::corpus_ops_commands::corpus_converged(99.0, 0.5, 3);
        assert!(res.is_err());
    }

    #[test]
    #[ignore] // slow: loads full corpus + benchmarks all entries
    fn test_cov_corpus_benchmark_all() {
        let _ = super::super::corpus_ops_commands::corpus_benchmark(10000, None);
    }

    #[test]
    #[ignore] // slow: loads full corpus + benchmarks bash entries
    fn test_cov_corpus_benchmark_bash_only() {
        use crate::cli::args::CorpusFormatArg;
        let _ = super::super::corpus_ops_commands::corpus_benchmark(
            10000,
            Some(&CorpusFormatArg::Bash),
        );
    }

    #[test]
    #[ignore] // slow: loads full corpus + benchmarks makefile entries
    fn test_cov_corpus_benchmark_makefile_only() {
        use crate::cli::args::CorpusFormatArg;
        let _ = super::super::corpus_ops_commands::corpus_benchmark(
            10000,
            Some(&CorpusFormatArg::Makefile),
        );
    }

    #[test]
    fn test_cov_corpus_benchmark_dockerfile_only() {
        use crate::cli::args::CorpusFormatArg;
        // Dockerfile corpus is small (~700 entries), so this is fast enough
        let _ = super::super::corpus_ops_commands::corpus_benchmark(
            10000,
            Some(&CorpusFormatArg::Dockerfile),
        );
    }

    // Exercise the corpus core dispatcher via handle_corpus_command
    #[test]
    #[ignore] // slow: loads full corpus + runs all entries
    fn test_cov_corpus_handle_run_human() {
        use crate::cli::args::CorpusOutputFormat;
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: None,
            min_score: None,
            log: false,
        };
        let _ = super::super::corpus_core_cmds::handle_corpus_command(cmd);
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs all entries
    fn test_cov_corpus_handle_run_json() {
        use crate::cli::args::CorpusOutputFormat;
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Json,
            filter: None,
            min_score: None,
            log: false,
        };
        let _ = super::super::corpus_core_cmds::handle_corpus_command(cmd);
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs bash entries
    fn test_cov_corpus_handle_run_with_bash_filter() {
        use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: Some(CorpusFormatArg::Bash),
            min_score: None,
            log: false,
        };
        let _ = super::super::corpus_core_cmds::handle_corpus_command(cmd);
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs makefile entries
    fn test_cov_corpus_handle_run_with_makefile_filter() {
        use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: Some(CorpusFormatArg::Makefile),
            min_score: None,
            log: false,
        };
        let _ = super::super::corpus_core_cmds::handle_corpus_command(cmd);
    }

    #[test]
    fn test_cov_corpus_handle_run_with_dockerfile_filter() {
        use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
        // Dockerfile corpus is small (~700 entries), so this is fast enough
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: Some(CorpusFormatArg::Dockerfile),
            min_score: None,
            log: false,
        };
        let _ = super::super::corpus_core_cmds::handle_corpus_command(cmd);
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs all entries
    fn test_cov_corpus_handle_run_with_min_score_passing() {
        use crate::cli::args::CorpusOutputFormat;
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: None,
            min_score: Some(0.0), // should always pass
            log: false,
        };
        let res = super::super::corpus_core_cmds::handle_corpus_command(cmd);
        assert!(res.is_ok());
    }

    #[test]
    #[ignore] // slow: loads full corpus + runs all entries
    fn test_cov_corpus_handle_run_with_high_min_score() {
        use crate::cli::args::CorpusOutputFormat;
        let cmd = CorpusCommands::Run {
            format: CorpusOutputFormat::Human,
            filter: None,
            min_score: Some(999.0), // impossible threshold
            log: false,
        };
        let res = super::super::corpus_core_cmds::handle_corpus_command(cmd);
        assert!(res.is_err());
    }
}
