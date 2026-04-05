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
}

include!("inspector_proofinspector_compare_environ.rs");
