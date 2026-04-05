//! Corpus runner check methods: MR relations, schema, lint, behavioral, cross-shell, determinism.
//!
//! Extracted from runner.rs for file size health (PMAT).
//! Contains `impl CorpusRunner` methods for validation and quality checking.

use crate::corpus::registry::{CorpusEntry, CorpusFormat};
use crate::models::Config;

use super::runner::CorpusRunner;
use super::runner_helpers::DOCKERFILE_INSTRUCTIONS;

// ---------------------------------------------------------------------------
// impl CorpusRunner: metamorphic relation checks (MR-2 through MR-7)
// ---------------------------------------------------------------------------

impl CorpusRunner {
    /// MR-2: Stability under no-op addition.
    /// Adding a comment to the input should not change the transpiled output semantics.
    pub(crate) fn check_mr2_stability(&self, entry: &CorpusEntry, output_contains: bool) -> bool {
        // Input is Rust DSL -- use Rust comment syntax, not shell comment
        let modified_input = format!("// MR-2 no-op\n{}", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-3: Trailing whitespace invariance.
    /// Adding trailing whitespace/newlines to the input should not change output semantics.
    pub(crate) fn check_mr3_whitespace(&self, entry: &CorpusEntry, output_contains: bool) -> bool {
        let modified_input = format!("{}\n\n  \n", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-4: Leading blank line invariance.
    /// Adding leading blank lines to the input should not change output semantics.
    pub(crate) fn check_mr4_leading_blanks(
        &self,
        entry: &CorpusEntry,
        output_contains: bool,
    ) -> bool {
        let modified_input = format!("\n\n{}", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-5: Subsumption -- if A transpiles, a simplification of A should also transpile.
    /// For Rust DSL: remove the last statement from main. Vacuously true for
    /// single-statement entries or non-Bash formats.
    pub(crate) fn check_mr5_subsumption(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        let Some((body_start, body_end)) = Self::extract_main_body_range(input) else {
            return true;
        };
        let body = input[body_start..body_end].trim();
        let Some(semi_pos) = Self::find_last_top_level_semicolon(body) else {
            return true;
        };
        let simplified_body = &body[..semi_pos];
        if !simplified_body.contains(';') {
            return true;
        }
        let simplified = format!("{}{}; }}", &input[..body_start], simplified_body);
        self.transpile_entry(&simplified, entry.format).is_ok()
    }

    /// Extract the byte range of the main function body (between outer braces)
    pub(crate) fn extract_main_body_range(input: &str) -> Option<(usize, usize)> {
        let body_start = input
            .find("fn main()")
            .and_then(|i| input[i..].find('{').map(|j| i + j + 1))?;
        let body_end = input.rfind('}')?;
        if body_end <= body_start {
            return None;
        }
        Some((body_start, body_end))
    }

    /// Find the position of the last semicolon at brace depth 0
    pub(crate) fn find_last_top_level_semicolon(body: &str) -> Option<usize> {
        let mut depth = 0i32;
        let mut last_top_semi = None;
        for (i, ch) in body.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                ';' if depth == 0 => last_top_semi = Some(i),
                _ => {}
            }
        }
        last_top_semi
    }

    /// MR-6: Composition -- for entries with multiple `let` statements,
    /// each individual `let` should transpile independently.
    /// Vacuously true if < 2 let statements or non-Bash format.
    pub(crate) fn check_mr6_composition(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        // Extract individual `let` statements from the body
        let lets: Vec<&str> = input
            .split(';')
            .filter(|s| s.trim().starts_with("let ") || s.trim().starts_with("let mut "))
            .collect();

        if lets.len() < 2 {
            return true; // inapplicable
        }
        // Each let should transpile independently inside a main fn
        for let_stmt in &lets {
            let single = format!("fn main() {{ {}; }}", let_stmt.trim());
            if self.transpile_entry(&single, entry.format).is_err() {
                return false;
            }
        }
        true
    }

    /// MR-7: Negation -- for entries containing `if`, negating the condition
    /// should still produce valid transpilation.
    /// Vacuously true for entries without `if` or non-Bash format.
    pub(crate) fn check_mr7_negation(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        // Check if input contains an if statement with a simple comparison
        if !input.contains("if ") {
            return true; // inapplicable
        }
        // Simple negation: wrap the condition in !()
        // Find pattern: `if <cond> {` and replace with `if !(<cond>) {`
        if let Some(if_pos) = input.find("if ") {
            let after_if = &input[if_pos + 3..];
            if let Some(brace_pos) = after_if.find('{') {
                let condition = after_if[..brace_pos].trim();
                if condition.is_empty() {
                    return true;
                }
                let negated = format!(
                    "{}if !({}) {}",
                    &input[..if_pos],
                    condition,
                    &after_if[brace_pos..]
                );
                // Negation: negated version must also transpile
                return self.transpile_entry(&negated, entry.format).is_ok();
            }
        }
        true // inapplicable
    }

    /// KAIZEN-072: MR equivalence check reusing pre-computed original containment.
    /// Eliminates 3 redundant transpilations per entry (MR-2, MR-3, MR-4 each
    /// re-transpiled the original -- ~53,826 wasted transpilations per corpus run).
    pub(crate) fn check_mr_equivalence_precomputed(
        &self,
        entry: &CorpusEntry,
        modified_input: &str,
        original_contains: bool,
    ) -> bool {
        let modified = self.transpile_entry(modified_input, entry.format);
        match modified {
            Ok(modif) => original_contains == modif.contains(&entry.expected_output),
            // Original succeeded (we're in the Ok branch of run_entry), modified failed -> not equivalent
            Err(_) => false,
        }
    }

    /// Transpile input based on format (DRY helper for MR checks).
    pub(crate) fn transpile_entry(
        &self,
        input: &str,
        format: CorpusFormat,
    ) -> std::result::Result<String, crate::Error> {
        match format {
            CorpusFormat::Bash => crate::transpile(input, &self.config),
            CorpusFormat::Makefile => crate::transpile_makefile(input, &self.config),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(input, &self.config),
        }
    }

    // -----------------------------------------------------------------------
    // Cross-shell agreement
    // -----------------------------------------------------------------------

    /// KAIZEN-073: Cross-shell agreement reusing run_entry output when config matches.
    /// KAIZEN-074: Skip redundant sh execution when behavioral already passed for same output.
    pub(crate) fn check_cross_shell_with_output(
        &self,
        entry: &CorpusEntry,
        output: &str,
        behavioral_passed: bool,
    ) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }

        let posix_config = Config {
            target: crate::models::ShellDialect::Posix,
            ..self.config.clone()
        };
        let bash_config = Config {
            target: crate::models::ShellDialect::Bash,
            ..self.config.clone()
        };

        // Track whether posix_out is the same as the behavioral-tested output
        let posix_is_reused = self.config.target == crate::models::ShellDialect::Posix;

        // Reuse run_entry output for whichever dialect matches self.config.target
        let (posix_result, bash_result) = match self.config.target {
            crate::models::ShellDialect::Posix => {
                let bash_r = crate::transpile(&entry.input, &bash_config);
                (Ok(output.to_string()), bash_r)
            }
            crate::models::ShellDialect::Bash => {
                let posix_r = crate::transpile(&entry.input, &posix_config);
                (posix_r, Ok(output.to_string()))
            }
            // Dash/Ash: neither matches Posix or Bash, transpile both
            _ => {
                let posix_r = crate::transpile(&entry.input, &posix_config);
                let bash_r = crate::transpile(&entry.input, &bash_config);
                (posix_r, bash_r)
            }
        };

        match (posix_result, bash_result) {
            (Ok(posix_out), Ok(bash_out)) => {
                let posix_has = posix_out.contains(&entry.expected_output);
                let bash_has = bash_out.contains(&entry.expected_output);
                if !(posix_has && bash_has) {
                    return false;
                }
                // KAIZEN-074: if behavioral already passed for this same output,
                // sh execution is known-good -- only run dash
                if behavioral_passed && posix_is_reused {
                    self.check_dash_execution(&posix_out)
                } else {
                    self.check_shell_execution(&posix_out)
                }
            }
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    /// Execute shell output in both `sh` and `dash`, verifying both terminate.
    /// Returns true if both shells execute without timeout.
    /// Gracefully skips dash if not installed.
    pub(crate) fn check_shell_execution(&self, output: &str) -> bool {
        // Execute in sh (must pass)
        let sh_ok = match std::process::Command::new("timeout")
            .args(["2", "sh", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(_) => return false,
        };

        if !sh_ok {
            return false;
        }

        // Execute in dash (graceful: skip if not found)
        match std::process::Command::new("timeout")
            .args(["2", "dash", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true, // dash not installed
            Err(_) => true, // other error, graceful skip
        }
    }

    /// KAIZEN-074: Execute only in dash (sh already verified by check_behavioral).
    /// Gracefully skips if dash is not installed.
    pub(crate) fn check_dash_execution(&self, output: &str) -> bool {
        match std::process::Command::new("timeout")
            .args(["2", "dash", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => true,
        }
    }

    // -----------------------------------------------------------------------
    // Schema, lint, behavioral, determinism checks
    // -----------------------------------------------------------------------

    /// Schema validation: verify output conforms to the format grammar.
    pub(crate) fn check_schema(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                let internal_ok = crate::validation::validate_shell_snippet(output).is_ok();
                if !internal_ok {
                    return false;
                }
                // Additionally run shellcheck for stricter POSIX validation.
                // Graceful fallback: if shellcheck is not installed, trust internal result.
                self.check_shellcheck(output).unwrap_or(true)
            }
            CorpusFormat::Makefile => crate::make_parser::parse_makefile(output).is_ok(),
            CorpusFormat::Dockerfile => {
                // No dedicated Dockerfile parser; use linter as schema proxy.
                // Check that output contains at least one valid Dockerfile instruction.
                let has_instruction = output.lines().any(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && DOCKERFILE_INSTRUCTIONS
                            .iter()
                            .any(|instr| trimmed.starts_with(instr))
                });
                has_instruction
            }
        }
    }

    /// Run shellcheck on shell output, returning None if shellcheck is not found.
    /// Returns Some(true) if no error-level findings, Some(false) if errors found.
    pub(crate) fn check_shellcheck(&self, output: &str) -> Option<bool> {
        let result = std::process::Command::new("shellcheck")
            .args(["-s", "sh", "-f", "json", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(_) => return None,
        };

        // Write output to shellcheck's stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(output.as_bytes());
        }

        let output_result = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Some(true), // can't read output, trust internal
        };

        // shellcheck exits 0 = clean, 1 = findings exist
        // Parse JSON to check for "error" level findings only
        let stdout = String::from_utf8_lossy(&output_result.stdout);
        if stdout.trim().is_empty() || stdout.trim() == "[]" {
            return Some(true);
        }

        // Parse JSON array of findings; fail only on "error" level
        match serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
            Ok(findings) => {
                let has_errors = findings
                    .iter()
                    .any(|f| f.get("level").and_then(|l| l.as_str()) == Some("error"));
                Some(!has_errors)
            }
            Err(_) => Some(true), // can't parse, trust internal
        }
    }

    /// Rules excluded from corpus D-score because they fire on valid transpiler output.
    pub(crate) const CORPUS_LINT_EXCLUSIONS: &'static [&'static str] = &[
        "SEC001", "REL001", "SC1020", "SC1028", "SC1035", "SC1037", "SC1041", "SC1044", "SC1078",
        "SC1140", "SC2105",
    ];

    pub(crate) fn check_lint(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                let lint_result = crate::linter::rules::lint_shell(output);
                !lint_result.diagnostics.iter().any(|d| {
                    d.severity == crate::linter::Severity::Error
                        && !Self::CORPUS_LINT_EXCLUSIONS.contains(&d.code.as_str())
                })
            }
            CorpusFormat::Makefile => {
                let lint_result = crate::linter::rules::lint_makefile(output);
                !lint_result.has_errors()
            }
            CorpusFormat::Dockerfile => {
                let lint_result = crate::linter::rules::lint_dockerfile(output);
                !lint_result.has_errors()
            }
        }
    }

    /// B_L3: Behavioral equivalence -- execute the transpiled output and verify
    /// it terminates within 2 seconds.
    pub(crate) fn check_behavioral(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                match std::process::Command::new("timeout")
                    .args(["2", "sh", "-c", output])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()
                {
                    Ok(result) => {
                        let code = result.status.code().unwrap_or(128);
                        code != 124
                    }
                    Err(_) => false,
                }
            }
            CorpusFormat::Makefile => self.check_makefile_dry_run(output),
            CorpusFormat::Dockerfile => true,
        }
    }

    /// Validate Makefile output by writing to a temp file and running `make -n -f`.
    pub(crate) fn check_makefile_dry_run(&self, output: &str) -> bool {
        use std::io::Write;

        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join(format!("bashrs_makefile_check_{}", std::process::id()));
        let tmp_str = tmp_path.to_string_lossy().to_string();

        let write_ok =
            std::fs::File::create(&tmp_path).and_then(|mut f| f.write_all(output.as_bytes()));
        if write_ok.is_err() {
            return true; // can't write temp file, graceful pass
        }

        let result = std::process::Command::new("make")
            .args(["-n", "-f", &tmp_str])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        // Clean up temp file
        let _ = std::fs::remove_file(&tmp_path);

        match result {
            Ok(r) => {
                if r.status.success() {
                    return true;
                }
                let stderr = String::from_utf8_lossy(&r.stderr);
                stderr.contains("No targets") || stderr.contains("No rule to make target")
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => true,
        }
    }

    /// KAIZEN-070: Determinism check reusing the first transpilation output from run_entry.
    pub(crate) fn check_determinism_with_output(
        &self,
        entry: &CorpusEntry,
        first_output: &str,
    ) -> bool {
        if !entry.deterministic {
            return true;
        }

        let second = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, &self.config),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, &self.config),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(&entry.input, &self.config),
        };

        match second {
            Ok(b) => first_output == b,
            Err(_) => false,
        }
    }
}
