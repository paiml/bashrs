impl ProofInspector {

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
