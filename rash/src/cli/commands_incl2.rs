/// Dispatch analysis and transformation commands (Lint, Purify, SafetyCheck, Classify, Format).
fn dispatch_analysis(
    command: Commands,
    _target: ShellDialect,
    _verify: VerificationLevel,
    _validation: ValidationLevel,
    _strict: bool,
) -> Result<()> {
    match command {
        Commands::Lint {
            input,
            format,
            fix,
            fix_assumptions,
            output,
            no_ignore,
            ignore_file,
            quiet,
            level,
            ignore,
            exclude,
            citl_export,
            profile,
            graded,
            ci,
            fail_on,
        } => {
            let _ = graded;
            lint_command(LintCommandOptions {
                inputs: &input,
                format,
                fix,
                fix_assumptions,
                output: output.as_deref(),
                no_ignore,
                ignore_file_path: ignore_file.as_deref(),
                quiet,
                level,
                ignore_rules: ignore.as_deref(),
                exclude_rules: exclude.as_deref(),
                citl_export_path: citl_export.as_deref(),
                profile,
                ci,
                fail_on,
            })
        }
        Commands::Purify {
            input,
            output,
            report,
            with_tests,
            property_tests,
            type_check,
            emit_guards,
            type_strict,
            diff,
            verify,
            recursive,
        } => purify_command(PurifyCommandOptions {
            input: &input,
            output: output.as_deref(),
            report,
            with_tests,
            property_tests,
            type_check,
            emit_guards,
            type_strict,
            diff,
            verify,
            recursive,
        }),
        Commands::SafetyCheck {
            input,
            json,
            format,
        } => safety_check_cmds::safety_check_command(&input, json, format.as_ref()),
        Commands::Explain {
            input,
            json,
            format,
            chat_model,
        } => explain_cmds::explain_command(&input, json, format.as_ref(), chat_model.as_deref()),
        Commands::Fix {
            input,
            dry_run,
            assumptions,
            output,
            chat_model,
        } => fix_cmds::fix_command(
            &input,
            dry_run,
            assumptions,
            output.as_deref(),
            chat_model.as_deref(),
        ),
        Commands::Classify {
            input,
            json,
            multi_label,
            format,
            probe,
            mlp_probe,
            model,
        } => classify_cmds::classify_command(
            &input,
            json,
            multi_label,
            format.as_ref(),
            probe.as_deref(),
            mlp_probe.as_deref(),
            model.as_deref(),
        ),
        Commands::Format {
            inputs,
            check,
            dry_run,
            output,
        } => format_command(&inputs, check, dry_run, output.as_deref()),
        _ => unreachable!("dispatch_analysis called with non-analysis command"),
    }
}

/// Dispatch quality and testing commands (Test, Score, Audit, Coverage, Bench, Mutate, Simulate, Gate).
fn dispatch_quality(command: Commands) -> Result<()> {
    match command {
        Commands::Test {
            input,
            format,
            detailed,
            pattern,
        } => test_commands::test_command(&input, format, detailed, pattern.as_deref()),
        Commands::Score {
            input,
            format,
            detailed,
            dockerfile,
            runtime,
            grade,
            profile,
        } => score_commands::score_command(
            &input, format, detailed, dockerfile, runtime, grade, profile,
        ),
        Commands::Audit {
            input,
            format,
            strict,
            detailed,
            min_grade,
        } => audit_commands::audit_command(&input, &format, strict, detailed, min_grade.as_deref()),
        Commands::Coverage {
            input,
            format,
            min,
            detailed,
            output,
        } => coverage_commands::coverage_command(&input, &format, min, detailed, output.as_deref()),
        Commands::Bench {
            scripts,
            warmup,
            iterations,
            output,
            strict,
            verify_determinism,
            show_raw,
            quiet,
            measure_memory,
            csv,
            no_color,
        } => {
            use crate::cli::bench::{bench_command, BenchOptions};
            bench_command(BenchOptions {
                scripts,
                warmup,
                iterations,
                output,
                strict,
                verify_determinism,
                show_raw,
                quiet,
                measure_memory,
                csv,
                no_color,
            })
        }
        Commands::Mutate {
            input,
            config,
            format,
            count,
            show_survivors,
            output,
        } => mutate_command(
            &input,
            config.as_deref(),
            format,
            count,
            show_survivors,
            output.as_deref(),
        ),
        Commands::Simulate {
            input,
            seed,
            verify,
            mock_externals,
            format,
            trace,
        } => simulate_command(&input, seed, verify, mock_externals, format, trace),
        Commands::Gate { tier, report } => gate_cmds::handle_gate_command(tier, report),
        Commands::Cfg {
            input,
            format,
            per_function,
        } => cfg_cmds::cfg_command(&input, format, per_function),
        _ => unreachable!("dispatch_quality called with non-quality command"),
    }
}

/// Dispatch interactive and misc commands (Repl, Playbook, GenerateAdversarial, ExplainError).
fn dispatch_interactive(command: Commands) -> Result<()> {
    match command {
        Commands::Repl {
            debug,
            sandboxed,
            max_memory,
            timeout,
            max_depth,
        } => handle_repl_command(debug, sandboxed, max_memory, timeout, max_depth),
        Commands::Playbook {
            input,
            run,
            format,
            verbose,
            dry_run,
        } => playbook_command(&input, run, format, verbose, dry_run),
        Commands::GenerateAdversarial {
            output,
            seed,
            count_per_class,
            extra_needs_quoting,
            verify,
            stats,
        } => adversarial_cmds::generate_adversarial_command(
            &output,
            seed,
            count_per_class,
            extra_needs_quoting,
            verify,
            stats,
        ),
        #[cfg(feature = "oracle")]
        Commands::ExplainError {
            error,
            command,
            shell,
            format,
            detailed,
        } => explain_error_command(&error, command.as_deref(), &shell, format, detailed),
        _ => unreachable!("dispatch_interactive called with non-interactive command"),
    }
}

// ---------------------------------------------------------------------------
// Core functions (small, kept in commands.rs)
// ---------------------------------------------------------------------------

/// Explain shell error using ML classification (v6.40.0)
#[cfg(feature = "oracle")]
fn explain_error_command(
    error: &str,
    command: Option<&str>,
    _shell: &str,
    format: ExplainErrorFormat,
    detailed: bool,
) -> Result<()> {
    use bashrs_oracle::{ErrorFeatures, Oracle};

    // Load or train the oracle (cached model for performance)
    let oracle = Oracle::load_or_train()
        .map_err(|e| Error::Internal(format!("Failed to load ML oracle: {e}")))?;

    // Extract exit code from error message if present (e.g., "exit code 127")
    let exit_code = extract_exit_code(error);

    // Classify the error
    let features = ErrorFeatures::extract(exit_code, error, command);
    let result = oracle
        .classify(&features)
        .map_err(|e| Error::Internal(format!("Classification failed: {e}")))?;

    match format {
        ExplainErrorFormat::Human => {
            println!("Category: {}", result.category.name());
            println!("Confidence: {:.1}%", result.confidence * 100.0);
            println!();
            if let Some(fix) = &result.suggested_fix {
                println!("Suggested Fix:");
                println!("  {fix}");
            } else {
                println!("Suggested Fix:");
                println!("  {}", result.category.fix_suggestion());
            }

            if detailed && !result.related_patterns.is_empty() {
                println!();
                println!("Related Patterns:");
                for pattern in &result.related_patterns {
                    println!("  - {pattern}");
                }
            }

            if detailed {
                println!();
                println!("Error Analysis:");
                println!("  Exit code: {exit_code}");
                if let Some(cmd) = command {
                    println!("  Command: {cmd}");
                }
            }
        }
        ExplainErrorFormat::Json => {
            let output = serde_json::json!({
                "category": result.category.name(),
                "confidence": result.confidence,
                "suggested_fix": result.suggested_fix.as_deref()
                    .unwrap_or_else(|| result.category.fix_suggestion()),
                "related_patterns": result.related_patterns,
                "exit_code": exit_code,
                "command": command,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?
            );
        }
    }

    Ok(())
}

// extract_exit_code moved to cli/logic.rs

/// Wrap an error with file path and source code context for rich diagnostics
fn with_context(error: Error, file: &Path, source: &str) -> Error {
    Error::WithContext {
        inner: Box::new(error),
        file: Some(file.display().to_string()),
        source_code: Some(source.to_string()),
    }
}


include!("commands_incl2_incl2.rs");
