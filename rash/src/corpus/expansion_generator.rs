//! Template-based script generator for SSB expansion (Phase 9 #10).
//!
//! Generates labeled shell/Makefile/Dockerfile scripts from parameterized
//! templates. Each template produces safe and unsafe variants, labeled by
//! the bashrs linter. Output is JSONL compatible with `merge-data` and
//! `export-splits`.
//!
//! Target: expand ShellSafetyBench from 27,842 to 50,000+ entries.

use crate::corpus::dataset::{strip_shell_preamble, ClassificationRow};
use crate::linter::{self, LintProfile};
use crate::models::{Error, Result};
use std::io::Write;
use std::path::Path;

/// Format of generated scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenFormat {
    Bash,
    Makefile,
    Dockerfile,
}

impl GenFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenFormat::Bash => "bash",
            GenFormat::Makefile => "makefile",
            GenFormat::Dockerfile => "dockerfile",
        }
    }
}

/// Summary of generated entries.
#[derive(Debug)]
pub struct GenSummary {
    pub total: usize,
    pub safe: usize,
    pub unsafe_count: usize,
    pub format: GenFormat,
}

/// Label a raw script using the bashrs linter.
///
/// Returns (label, is_unsafe): 0=safe, 1=unsafe.
fn label_script(script: &str, format: GenFormat) -> u8 {
    let has_security_finding = match format {
        GenFormat::Bash => {
            let result = linter::lint_shell(script);
            has_sec_det_idem(&result)
        }
        GenFormat::Makefile => {
            // Makefile linter uses MAKE* rules, not SEC/DET/IDEM.
            // Also lint recipe lines as shell to catch security issues.
            let result = linter::lint_makefile(script);
            if has_sec_det_idem(&result) {
                return 1;
            }
            // Extract recipe lines (after tab) and lint as shell
            let recipe_lines: String = script
                .lines()
                .filter(|l| l.starts_with('\t'))
                .map(|l| l.trim_start_matches('\t'))
                .collect::<Vec<_>>()
                .join("\n");
            if !recipe_lines.is_empty() {
                has_sec_det_idem(&linter::lint_shell(&recipe_lines))
            } else {
                false
            }
        }
        GenFormat::Dockerfile => {
            // Dockerfile linter uses DOCKER* rules.
            // Also lint RUN commands as shell.
            let result = linter::lint_dockerfile_with_profile(script, LintProfile::Standard);
            if has_sec_det_idem(&result) {
                return 1;
            }
            // Extract RUN command bodies and lint as shell
            let run_lines: String = script
                .lines()
                .filter_map(|l| {
                    let trimmed = l.trim();
                    if trimmed.starts_with("RUN ") {
                        Some(trimmed.trim_start_matches("RUN "))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !run_lines.is_empty() {
                has_sec_det_idem(&linter::lint_shell(&run_lines))
            } else {
                false
            }
        }
    };
    if has_security_finding {
        1
    } else {
        0
    }
}

/// Check if lint result contains SEC/DET/IDEM rules.
fn has_sec_det_idem(result: &linter::LintResult) -> bool {
    result.diagnostics.iter().any(|d| {
        d.code.starts_with("SEC") || d.code.starts_with("DET") || d.code.starts_with("IDEM")
    })
}

/// Generate a batch of labeled scripts for a given format.
pub fn generate_expansion(format: GenFormat, count: usize, seed: u64) -> Vec<ClassificationRow> {
    let templates = match format {
        GenFormat::Bash => generate_bash_templates(count, seed),
        GenFormat::Makefile => generate_makefile_templates(count, seed),
        GenFormat::Dockerfile => generate_dockerfile_templates(count, seed),
    };

    templates
        .into_iter()
        .map(|script| {
            let label = label_script(&script, format);
            ClassificationRow {
                input: strip_shell_preamble(&script),
                label,
            }
        })
        .collect()
}

/// Write generated entries to JSONL file.
pub fn write_expansion(entries: &[ClassificationRow], output: &Path) -> Result<GenSummary> {
    let file = std::fs::File::create(output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;
    let mut writer = std::io::BufWriter::new(file);

    let mut safe = 0;
    let mut unsafe_count = 0;
    for entry in entries {
        if entry.label == 0 {
            safe += 1;
        } else {
            unsafe_count += 1;
        }
        let json = serde_json::to_string(entry)
            .map_err(|e| Error::Validation(format!("JSON error: {e}")))?;
        writeln!(writer, "{json}").map_err(|e| Error::Validation(format!("Write error: {e}")))?;
    }

    Ok(GenSummary {
        total: entries.len(),
        safe,
        unsafe_count,
        format: GenFormat::Bash, // overridden by caller
    })
}

// ============================================================================
// Bash script templates
// ============================================================================

fn generate_bash_templates(count: usize, seed: u64) -> Vec<String> {
    let mut scripts = Vec::with_capacity(count);
    let mut idx = seed;

    // Safe patterns
    let safe_commands = [
        "echo", "printf", "cat", "ls", "pwd", "date", "whoami", "hostname", "uname", "id", "env",
        "wc", "sort", "uniq", "head", "tail", "tee", "tr", "cut", "paste", "comm", "diff", "patch",
        "test", "true", "false",
    ];
    let safe_flags = [
        "-n",
        "-e",
        "-l",
        "-a",
        "-r",
        "-v",
        "-h",
        "--help",
        "--version",
    ];
    let safe_paths = [
        "/tmp/output.txt",
        "/var/log/app.log",
        "\"$HOME/data\"",
        "\"${TMPDIR:-/tmp}/work\"",
        "\"$OUTPUT_DIR/result.txt\"",
    ];

    // Unsafe patterns (will trigger SEC/DET/IDEM rules)
    let unsafe_eval_patterns = [
        "eval $USER_INPUT",
        "eval \"$1\"",
        "eval $(cat /tmp/cmd)",
        "eval \"${COMMAND}\"",
    ];
    let unsafe_unquoted = [
        "rm -rf $DIR",
        "cp $SRC $DST",
        "mv $OLD $NEW",
        "cat $FILE",
        "chmod 755 $PATH",
    ];
    let unsafe_random = [
        "echo $RANDOM",
        "SEED=$RANDOM",
        "FILE=/tmp/test_$RANDOM",
        "TOKEN=$(head -c 16 /dev/urandom | xxd -p)",
    ];
    let unsafe_timestamp = [
        "echo $(date)",
        "LOG=/tmp/log_$(date +%s)",
        "STAMP=$(date +%Y%m%d)",
    ];
    let unsafe_mkdir = ["mkdir /tmp/workdir", "mkdir $DIR"];
    let unsafe_pid = ["echo $$", "PIDFILE=/tmp/app_$$.pid"];

    // Generate mix of safe and unsafe
    while scripts.len() < count {
        idx = idx
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let variant = (idx >> 33) as usize;

        let script = match variant % 12 {
            // Safe: simple commands with quoted vars
            0 => {
                let cmd = safe_commands[variant % safe_commands.len()];
                let flag = safe_flags[variant % safe_flags.len()];
                let path = safe_paths[variant % safe_paths.len()];
                format!("{cmd} {flag} {path}")
            }
            // Safe: variable assignment + echo
            1 => {
                let var_name = ["NAME", "VALUE", "COUNT", "RESULT", "STATUS"][variant % 5];
                let val = ["hello", "42", "ok", "true", "done"][variant % 5];
                format!("{var_name}=\"{val}\"\necho \"${{{var_name}}}\"")
            }
            // Safe: conditional with quoted vars
            2 => {
                let test_op = ["-f", "-d", "-e", "-z", "-n"][variant % 5];
                let path = safe_paths[variant % safe_paths.len()];
                format!("if [ {test_op} {path} ]; then\n  echo \"exists\"\nfi")
            }
            // Safe: for loop with safe iteration
            3 => {
                let items = ["a b c", "1 2 3", "*.txt", "\"$@\""][variant % 4];
                let cmd = safe_commands[variant % safe_commands.len()];
                format!("for item in {items}; do\n  {cmd} \"$item\"\ndone")
            }
            // Safe: function definition
            4 => {
                let fn_name = ["setup", "cleanup", "validate", "process", "report"][variant % 5];
                let body_cmd = safe_commands[variant % safe_commands.len()];
                format!("{fn_name}() {{\n  {body_cmd} \"$1\"\n}}")
            }
            // Safe: pipe chain
            5 => {
                let c1 = safe_commands[variant % safe_commands.len()];
                let c2 = safe_commands[(variant + 1) % safe_commands.len()];
                let c3 = safe_commands[(variant + 2) % safe_commands.len()];
                format!("{c1} | {c2} | {c3}")
            }
            // Unsafe: eval injection
            6 => {
                let pat = unsafe_eval_patterns[variant % unsafe_eval_patterns.len()];
                pat.to_string()
            }
            // Unsafe: unquoted variable
            7 => {
                let pat = unsafe_unquoted[variant % unsafe_unquoted.len()];
                pat.to_string()
            }
            // Unsafe: non-deterministic ($RANDOM)
            8 => {
                let pat = unsafe_random[variant % unsafe_random.len()];
                pat.to_string()
            }
            // Unsafe: timestamp
            9 => {
                let pat = unsafe_timestamp[variant % unsafe_timestamp.len()];
                pat.to_string()
            }
            // Unsafe: non-idempotent mkdir
            10 => {
                let pat = unsafe_mkdir[variant % unsafe_mkdir.len()];
                pat.to_string()
            }
            // Unsafe: PID-dependent
            _ => {
                let pat = unsafe_pid[variant % unsafe_pid.len()];
                pat.to_string()
            }
        };

        scripts.push(script);
    }

    scripts
}

// ============================================================================
// Makefile templates
// ============================================================================

fn generate_makefile_templates(count: usize, seed: u64) -> Vec<String> {
    let mut scripts = Vec::with_capacity(count);
    let mut idx = seed;

    let targets = [
        "all",
        "build",
        "test",
        "clean",
        "install",
        "lint",
        "fmt",
        "check",
        "release",
        "deploy",
        "docs",
        "bench",
        "coverage",
        "docker-build",
        "docker-push",
        "ci",
        "setup",
        "run",
        "dev",
    ];
    let vars = [
        ("CC", "gcc"),
        ("CXX", "g++"),
        ("CFLAGS", "-Wall -Werror"),
        ("PREFIX", "/usr/local"),
        ("DESTDIR", ""),
        ("VERSION", "1.0.0"),
        ("CARGO", "cargo"),
        ("PYTHON", "python3"),
        ("NODE", "node"),
        ("GO", "go"),
    ];
    let commands_safe = [
        "echo \"Building...\"",
        "$(CARGO) build --release",
        "$(CARGO) test",
        "$(CARGO) clippy -- -D warnings",
        "$(PYTHON) -m pytest tests/",
        "$(GO) build ./...",
        "rm -rf \"$(BUILD_DIR)\"",
        "mkdir -p \"$(BUILD_DIR)\"",
        "install -m 755 target/release/app \"$(DESTDIR)$(PREFIX)/bin/\"",
        "cp -r docs/ \"$(DESTDIR)$(PREFIX)/share/doc/\"",
    ];
    let commands_unsafe = [
        "rm -rf $(BUILD_DIR)",
        "cp $(SRC) $(DST)",
        "chmod 777 $(TARGET)",
        "eval $(SHELL_CMD)",
        "mkdir $(OUTPUT)",
        "cat $INPUT",
    ];
    let phony_targets = [
        ".PHONY: all build test clean",
        ".PHONY: install lint fmt check",
        ".PHONY: release deploy docs",
    ];

    while scripts.len() < count {
        idx = idx
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let variant = (idx >> 33) as usize;

        let script = match variant % 10 {
            // Safe: simple target with safe command
            0 => {
                let target = targets[variant % targets.len()];
                let cmd = commands_safe[variant % commands_safe.len()];
                let phony = phony_targets[variant % phony_targets.len()];
                format!("{phony}\n\n{target}:\n\t{cmd}")
            }
            // Safe: variable definition + target
            1 => {
                let (var, val) = vars[variant % vars.len()];
                let target = targets[variant % targets.len()];
                let cmd = commands_safe[variant % commands_safe.len()];
                format!("{var} := {val}\n\n{target}:\n\t{cmd}")
            }
            // Safe: multi-target with dependencies
            2 => {
                let t1 = targets[variant % targets.len()];
                let t2 = targets[(variant + 1) % targets.len()];
                let c1 = commands_safe[variant % commands_safe.len()];
                let c2 = commands_safe[(variant + 1) % commands_safe.len()];
                format!(".PHONY: {t1} {t2}\n\n{t1}: {t2}\n\t{c1}\n\n{t2}:\n\t{c2}")
            }
            // Safe: conditional with ifdef
            3 => {
                let (var, val) = vars[variant % vars.len()];
                let cmd = commands_safe[variant % commands_safe.len()];
                format!("ifdef {var}\n{var} := {val}\nendif\n\nbuild:\n\t{cmd}")
            }
            // Safe: pattern rule
            4 => {
                let ext1 = [".o", ".so", ".a", ".bin"][variant % 4];
                let ext2 = [".c", ".cpp", ".rs", ".go"][variant % 4];
                format!("%{ext1}: %{ext2}\n\t$(CC) $(CFLAGS) -o \"$@\" \"$<\"")
            }
            // Safe: help target
            5 => {
                let t1 = targets[variant % targets.len()];
                let t2 = targets[(variant + 1) % targets.len()];
                format!(
                    ".PHONY: help\n\nhelp:\n\t@echo \"Available targets:\"\n\t@echo \"  {t1} - Build the project\"\n\t@echo \"  {t2} - Run tests\""
                )
            }
            // Unsafe: unquoted variable in command
            6 => {
                let target = targets[variant % targets.len()];
                let cmd = commands_unsafe[variant % commands_unsafe.len()];
                format!("{target}:\n\t{cmd}")
            }
            // Unsafe: eval in Makefile
            7 => {
                let target = targets[variant % targets.len()];
                format!("{target}:\n\teval $(SHELL_CMD)\n\techo $(USER_INPUT)")
            }
            // Unsafe: chmod 777
            8 => {
                let target = targets[variant % targets.len()];
                format!("{target}:\n\tchmod 777 $(OUTPUT)")
            }
            // Unsafe: unquoted rm
            _ => {
                let target = targets[variant % targets.len()];
                format!("{target}:\n\trm -rf ${{BUILD}}")
            }
        };

        scripts.push(script);
    }

    scripts
}

// ============================================================================
// Dockerfile templates
// ============================================================================

include!("expansion_generator_generate.rs");
