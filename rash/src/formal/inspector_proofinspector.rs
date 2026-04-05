impl ProofInspector {
    /// Generate a comprehensive verification report
    pub fn inspect(ast: &TinyAst, initial_state: AbstractState) -> VerificationReport {
        // Generate emitted code
        let emitted_code = FormalEmitter::emit(ast);

        // Create execution traces
        let rash_trace = Self::trace_rash_execution(ast, initial_state.clone());
        let posix_trace = Self::trace_posix_execution(&emitted_code, initial_state.clone());

        // Generate annotated AST
        let annotated_ast = Self::annotate_ast(ast, initial_state.clone());

        // Analyze equivalence
        let equivalence_analysis =
            Self::analyze_equivalence(&rash_trace.final_state, &posix_trace.final_state);

        // Generate emitter justifications
        let emitter_justifications = Self::generate_emitter_justifications(ast);

        // Determine verification result
        let verification_result = if equivalence_analysis.are_equivalent {
            VerificationResult::Success { confidence: 1.0 }
        } else {
            VerificationResult::Failure {
                reasons: vec!["States are not equivalent".to_string()],
            }
        };

        VerificationReport {
            ast: ast.clone(),
            emitted_code,
            initial_state,
            annotated_ast,
            rash_trace,
            posix_trace,
            equivalence_analysis,
            emitter_justifications,
            verification_result,
        }
    }

    /// Create an annotated AST with semantic information
    fn annotate_ast(ast: &TinyAst, initial_state: AbstractState) -> AnnotatedAst {
        let postcondition = match rash_semantics::eval_rash(ast, initial_state.clone()) {
            Ok(state) => state,
            Err(_) => initial_state.clone(),
        };

        let transformation = Self::compute_transformation(&initial_state, &postcondition);

        let children = match ast {
            TinyAst::Sequence { commands } => {
                let mut current_state = initial_state.clone();
                let mut annotations = Vec::new();

                for cmd in commands {
                    let annotation = Self::annotate_ast(cmd, current_state.clone());
                    current_state = annotation.postcondition.clone();
                    annotations.push(annotation);
                }

                annotations
            }
            _ => Vec::new(),
        };

        AnnotatedAst {
            node: ast.clone(),
            precondition: initial_state,
            postcondition,
            transformation,
            children,
        }
    }

    /// Compute state transformation description
    fn compute_transformation(
        before: &AbstractState,
        after: &AbstractState,
    ) -> StateTransformation {
        let mut env_changes = HashMap::new();

        // Check for environment changes
        for (key, value) in &after.env {
            match before.env.get(key) {
                Some(old_value) if old_value != value => {
                    env_changes.insert(
                        key.clone(),
                        EnvChange::Modified {
                            old_value: old_value.clone(),
                            new_value: value.clone(),
                        },
                    );
                }
                None => {
                    env_changes.insert(
                        key.clone(),
                        EnvChange::Added {
                            value: value.clone(),
                        },
                    );
                }
                _ => {} // No change
            }
        }

        // Check for removed environment variables
        for (key, value) in &before.env {
            if !after.env.contains_key(key) {
                env_changes.insert(
                    key.clone(),
                    EnvChange::Removed {
                        old_value: value.clone(),
                    },
                );
            }
        }

        // Check for working directory change
        let cwd_change = if before.cwd != after.cwd {
            Some(CwdChange {
                from: before.cwd.to_string_lossy().to_string(),
                to: after.cwd.to_string_lossy().to_string(),
            })
        } else {
            None
        };

        // Check for filesystem changes
        let mut fs_changes = Vec::new();
        for (path, entry) in &after.filesystem {
            if !before.filesystem.contains_key(path) {
                match entry {
                    crate::formal::FileSystemEntry::Directory => {
                        fs_changes.push(FilesystemChange::DirectoryCreated {
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                    crate::formal::FileSystemEntry::File(content) => {
                        fs_changes.push(FilesystemChange::FileCreated {
                            path: path.to_string_lossy().to_string(),
                            content: content.clone(),
                        });
                    }
                }
            }
        }

        // Compute output differences
        let output_produced = after
            .stdout
            .iter()
            .skip(before.stdout.len())
            .cloned()
            .collect();

        let errors_produced = after
            .stderr
            .iter()
            .skip(before.stderr.len())
            .cloned()
            .collect();

        let exit_code_change = if before.exit_code != after.exit_code {
            Some(after.exit_code)
        } else {
            None
        };

        StateTransformation {
            env_changes,
            cwd_change,
            fs_changes,
            output_produced,
            errors_produced,
            exit_code_change,
        }
    }

    /// Trace rash execution step by step
    fn trace_rash_execution(ast: &TinyAst, initial_state: AbstractState) -> ExecutionTrace {
        let mut steps = Vec::new();
        let mut current_state = initial_state.clone();
        let mut step_number = 1;

        Self::trace_rash_recursive(ast, &mut current_state, &mut steps, &mut step_number);

        ExecutionTrace {
            initial_state,
            steps,
            final_state: current_state,
        }
    }

    /// Recursive helper for tracing rash execution
    fn trace_rash_recursive(
        ast: &TinyAst,
        current_state: &mut AbstractState,
        steps: &mut Vec<ExecutionStep>,
        step_number: &mut usize,
    ) {
        let state_before = current_state.clone();

        match ast {
            TinyAst::ExecuteCommand { command_name, args } => {
                let operation = format!("Execute command: {} {}", command_name, args.join(" "));
                let mut errors = Vec::new();

                if let Err(e) = rash_semantics::eval_command(current_state, command_name, args) {
                    errors.push(e);
                }

                steps.push(ExecutionStep {
                    step_number: *step_number,
                    operation,
                    state_before,
                    state_after: current_state.clone(),
                    errors,
                });
                *step_number += 1;
            }

            TinyAst::SetEnvironmentVariable { name, value } => {
                let operation = format!("Set environment variable: {name}={value}");
                current_state.set_env(name.clone(), value.clone());

                steps.push(ExecutionStep {
                    step_number: *step_number,
                    operation,
                    state_before,
                    state_after: current_state.clone(),
                    errors: Vec::new(),
                });
                *step_number += 1;
            }

            TinyAst::ChangeDirectory { path } => {
                let operation = format!("Change directory to: {path}");
                let mut errors = Vec::new();

                if let Err(e) = current_state.change_directory(std::path::PathBuf::from(path)) {
                    errors.push(e);
                }

                steps.push(ExecutionStep {
                    step_number: *step_number,
                    operation,
                    state_before,
                    state_after: current_state.clone(),
                    errors,
                });
                *step_number += 1;
            }

            TinyAst::Sequence { commands } => {
                for cmd in commands {
                    Self::trace_rash_recursive(cmd, current_state, steps, step_number);
                }
            }
        }
    }

    /// Trace POSIX execution step by step
    fn trace_posix_execution(code: &str, initial_state: AbstractState) -> ExecutionTrace {
        let mut steps = Vec::new();
        let mut current_state = initial_state.clone();

        // For simplicity, we'll treat the entire POSIX code as one step
        // In a more sophisticated implementation, we could parse and trace each command
        let state_before = current_state.clone();
        let mut errors = Vec::new();

        if let Err(e) = posix_semantics::eval_posix(code, current_state.clone()) {
            errors.push(e);
        } else if let Ok(final_state) = posix_semantics::eval_posix(code, current_state.clone()) {
            current_state = final_state;
        }

        steps.push(ExecutionStep {
            step_number: 1,
            operation: format!("Execute POSIX code: {code}"),
            state_before,
            state_after: current_state.clone(),
            errors,
        });

        ExecutionTrace {
            initial_state,
            steps,
            final_state: current_state,
        }
    }

    /// Analyze equivalence between two states
    fn analyze_equivalence(
        rash_state: &AbstractState,
        posix_state: &AbstractState,
    ) -> EquivalenceAnalysis {
        let env_comparison = Self::compare_environments(&rash_state.env, &posix_state.env);
        let cwd_comparison = Self::compare_cwd(&rash_state.cwd, &posix_state.cwd);
        let fs_comparison =
            Self::compare_filesystems(&rash_state.filesystem, &posix_state.filesystem);
        let output_comparison = Self::compare_output(
            &rash_state.stdout,
            &rash_state.stderr,
            &posix_state.stdout,
            &posix_state.stderr,
        );
        let exit_code_comparison =
            Self::compare_exit_codes(rash_state.exit_code, posix_state.exit_code);

        let are_equivalent = env_comparison.matches
            && cwd_comparison.matches
            && fs_comparison.matches
            && output_comparison.stdout_matches
            && output_comparison.stderr_matches
            && exit_code_comparison.matches;

        EquivalenceAnalysis {
            are_equivalent,
            env_comparison,
            cwd_comparison,
            fs_comparison,
            output_comparison,
            exit_code_comparison,
        }
    }

    /// Compare environment variables
    fn compare_environments(
        rash_env: &HashMap<String, String>,
        posix_env: &HashMap<String, String>,
    ) -> EnvComparison {
        let mut rash_only = HashMap::new();
        let mut posix_only = HashMap::new();
        let mut different_values = HashMap::new();

        for (key, value) in rash_env {
            match posix_env.get(key) {
                Some(posix_value) if posix_value != value => {
                    different_values.insert(key.clone(), (value.clone(), posix_value.clone()));
                }
                None => {
                    rash_only.insert(key.clone(), value.clone());
                }
                _ => {} // Matches
            }
        }

        for (key, value) in posix_env {
            if !rash_env.contains_key(key) {
                posix_only.insert(key.clone(), value.clone());
            }
        }

        let matches = rash_only.is_empty() && posix_only.is_empty() && different_values.is_empty();

        EnvComparison {
            matches,
            rash_only,
            posix_only,
            different_values,
        }
    }

    /// Compare working directories
    fn compare_cwd(rash_cwd: &std::path::Path, posix_cwd: &std::path::Path) -> CwdComparison {
        CwdComparison {
            matches: rash_cwd == posix_cwd,
            rash_cwd: rash_cwd.to_string_lossy().to_string(),
            posix_cwd: posix_cwd.to_string_lossy().to_string(),
        }
    }

    /// Compare filesystems
    fn compare_filesystems(
        rash_fs: &HashMap<std::path::PathBuf, crate::formal::FileSystemEntry>,
        posix_fs: &HashMap<std::path::PathBuf, crate::formal::FileSystemEntry>,
    ) -> FilesystemComparison {
        let mut differences = Vec::new();

        for (path, entry) in rash_fs {
            match posix_fs.get(path) {
                Some(posix_entry) if posix_entry != entry => {
                    differences.push(format!(
                        "Path {} differs: rash={:?}, posix={:?}",
                        path.display(),
                        entry,
                        posix_entry
                    ));
                }
                None => {
                    differences.push(format!("Path {} only in rash: {:?}", path.display(), entry));
                }
                _ => {} // Matches
            }
        }

        for (path, entry) in posix_fs {
            if !rash_fs.contains_key(path) {
                differences.push(format!(
                    "Path {} only in posix: {:?}",
                    path.display(),
                    entry
                ));
            }
        }

        FilesystemComparison {
            matches: differences.is_empty(),
            differences,
        }
    }

    /// Compare output streams
    fn compare_output(
        rash_stdout: &[String],
        rash_stderr: &[String],
        posix_stdout: &[String],
        posix_stderr: &[String],
    ) -> OutputComparison {
        OutputComparison {
            stdout_matches: rash_stdout == posix_stdout,
            stderr_matches: rash_stderr == posix_stderr,
            rash_stdout: rash_stdout.to_vec(),
            posix_stdout: posix_stdout.to_vec(),
            rash_stderr: rash_stderr.to_vec(),
            posix_stderr: posix_stderr.to_vec(),
        }
    }

    /// Compare exit codes
    fn compare_exit_codes(rash_exit: i32, posix_exit: i32) -> ExitCodeComparison {
        ExitCodeComparison {
            matches: rash_exit == posix_exit,
            rash_exit_code: rash_exit,
            posix_exit_code: posix_exit,
        }
    }

    /// Generate emitter justifications
    fn generate_emitter_justifications(ast: &TinyAst) -> Vec<EmitterJustification> {
        let mut justifications = Vec::new();
        Self::generate_justifications_recursive(ast, &mut justifications);
        justifications
    }

    /// Recursive helper for generating justifications
    fn generate_justifications_recursive(
        ast: &TinyAst,
        justifications: &mut Vec<EmitterJustification>,
    ) {
        match ast {
            TinyAst::ExecuteCommand { command_name, args } => {
                let generated_code = FormalEmitter::emit(ast);
                justifications.push(EmitterJustification {
                    ast_node: format!("ExecuteCommand({command_name}, {args:?})"),
                    generated_code,
                    reasoning: "Command arguments are properly quoted to prevent shell injection"
                        .to_string(),
                    considerations: vec![
                        "Special characters are escaped within double quotes".to_string(),
                        "Empty arguments are preserved as empty quoted strings".to_string(),
                    ],
                });
            }

            TinyAst::SetEnvironmentVariable { name, value } => {
                let generated_code = FormalEmitter::emit(ast);
                justifications.push(EmitterJustification {
                    ast_node: format!("SetEnvironmentVariable({name}, {value})"),
                    generated_code,
                    reasoning: "Variable assignment uses POSIX-compliant syntax with quoted values"
                        .to_string(),
                    considerations: vec![
                        "Value is always quoted to handle spaces and special characters"
                            .to_string(),
                        "Variable name is validated to be POSIX-compliant".to_string(),
                    ],
                });
            }

            TinyAst::ChangeDirectory { path } => {
                let generated_code = FormalEmitter::emit(ast);
                justifications.push(EmitterJustification {
                    ast_node: format!("ChangeDirectory({path})"),
                    generated_code,
                    reasoning: "Change directory uses cd command with quoted path".to_string(),
                    considerations: vec![
                        "Path is quoted to handle spaces and special characters".to_string()
                    ],
                });
            }

            TinyAst::Sequence { commands } => {
                let generated_code = FormalEmitter::emit(ast);
                justifications.push(EmitterJustification {
                    ast_node: "Sequence".to_string(),
                    generated_code,
                    reasoning: "Commands are joined with semicolons for sequential execution"
                        .to_string(),
                    considerations: vec![
                        "Semicolon separator ensures commands execute in order".to_string(),
                        "Each command is independently validated".to_string(),
                    ],
                });

                for cmd in commands {
                    Self::generate_justifications_recursive(cmd, justifications);
                }
            }
        }
    }

    /// Generate a human-readable report
    pub fn generate_report(report: &VerificationReport) -> String {
        let mut output = String::new();

        output.push_str("# Formal Verification Report\n\n");

        // AST and generated code
        output.push_str("## Input AST\n");
        output.push_str(&format!("```\n{:#?}\n```\n\n", report.ast));

        output.push_str("## Generated POSIX Code\n");
        output.push_str(&format!("```bash\n{}\n```\n\n", report.emitted_code));

        // Verification result
        output.push_str("## Verification Result\n");
        match &report.verification_result {
            VerificationResult::Success { confidence } => {
                output.push_str(&format!(
                    "✅ **SUCCESS** (confidence: {:.1}%)\n\n",
                    confidence * 100.0
                ));
            }
            VerificationResult::Failure { reasons } => {
                output.push_str("❌ **FAILURE**\n");
                for reason in reasons {
                    output.push_str(&format!("- {reason}\n"));
                }
                output.push('\n');
            }
            VerificationResult::Partial { issues } => {
                output.push_str("⚠️ **PARTIAL**\n");
                for issue in issues {
                    output.push_str(&format!("- {issue}\n"));
                }
                output.push('\n');
            }
        }

        // Equivalence analysis
        output.push_str("## Equivalence Analysis\n");
        let eq = &report.equivalence_analysis;
        output.push_str(&format!(
            "- Environment variables: {}\n",
            if eq.env_comparison.matches {
                "✅"
            } else {
                "❌"
            }
        ));
        output.push_str(&format!(
            "- Working directory: {}\n",
            if eq.cwd_comparison.matches {
                "✅"
            } else {
                "❌"
            }
        ));
        output.push_str(&format!(
            "- Filesystem: {}\n",
            if eq.fs_comparison.matches {
                "✅"
            } else {
                "❌"
            }
        ));
        output.push_str(&format!(
            "- Standard output: {}\n",
            if eq.output_comparison.stdout_matches {
                "✅"
            } else {
                "❌"
            }
        ));
        output.push_str(&format!(
            "- Standard error: {}\n",
            if eq.output_comparison.stderr_matches {
                "✅"
            } else {
                "❌"
            }
        ));
        output.push_str(&format!(
            "- Exit code: {}\n\n",
            if eq.exit_code_comparison.matches {
                "✅"
            } else {
                "❌"
            }
        ));

        // Emitter justifications
        output.push_str("## Emitter Justifications\n");
        for (i, justification) in report.emitter_justifications.iter().enumerate() {
            output.push_str(&format!("### {}: {}\n", i + 1, justification.ast_node));
            output.push_str(&format!(
                "**Generated:** `{}`\n",
                justification.generated_code
            ));
            output.push_str(&format!("**Reasoning:** {}\n", justification.reasoning));
            if !justification.considerations.is_empty() {
                output.push_str("**Considerations:**\n");
                for consideration in &justification.considerations {
                    output.push_str(&format!("- {consideration}\n"));
                }
            }
            output.push('\n');
        }

        output
    }
}


include!("inspector_tests_proof_inspec.rs");
