/// Assign a split based on a deterministic hash of the input text.
///
/// Uses FNV-1a hash (fast, no crypto needed) mod 10:
/// - 0..=7 → Train (80%)
/// - 8     → Val   (10%)
/// - 9     → Test  (10%)
///
/// Stable: same input always maps to same split, even if corpus order changes.
fn assign_split(input: &str) -> Split {
    let hash = fnv1a_hash(input.as_bytes());
    match hash % 10 {
        0..=7 => Split::Train,
        8 => Split::Val,
        _ => Split::Test,
    }
}

/// FNV-1a 64-bit hash — fast, deterministic, no dependencies.
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl fmt::Display for SplitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.train.len() + self.val.len() + self.test.len();
        writeln!(f, "Split Result ({total} total):")?;
        for (name, split, sv) in [
            ("train", &self.train, &self.split_validations[0]),
            ("val", &self.val, &self.split_validations[1]),
            ("test", &self.test, &self.split_validations[2]),
        ] {
            let pct = if total > 0 {
                100.0 * split.len() as f64 / total as f64
            } else {
                0.0
            };
            let status = if sv.passed { "PASS" } else { "FAIL" };
            write!(f, "  {name}: {:>6} ({pct:5.1}%) [{status}]", split.len())?;
            // Show per-class counts inline
            for (i, &c) in sv.class_counts.iter().enumerate() {
                if c > 0 {
                    write!(f, " c{i}={c}")?;
                }
            }
            writeln!(f)?;
        }
        if !self.validation.passed {
            for e in &self.validation.errors {
                writeln!(f, "  ERROR: {e}")?;
            }
        }
        for w in &self.validation.warnings {
            writeln!(f, "  WARN: {w}")?;
        }
        Ok(())
    }
}

/// Multi-label classification row for ML training (SSC-021).
///
/// Format: `{"input": "<shell script>", "labels": [0.0, 1.0, 1.0, 0.0, 0.0]}`
#[derive(Debug, Clone, serde::Serialize)]
pub struct MultiLabelClassificationRow {
    pub input: String,
    pub labels: [f32; 5],
}

/// Safety class labels matching aprender `SafetyClass` enum.
pub const SAFETY_LABELS: [&str; 5] = [
    "safe",              // 0
    "needs-quoting",     // 1
    "non-deterministic", // 2
    "non-idempotent",    // 3
    "unsafe",            // 4
];

/// Dataset metadata for the `dataset-info` command
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    pub total_entries: usize,
    pub format_counts: Vec<(String, usize)>,
    pub schema_fields: Vec<(&'static str, &'static str, &'static str)>,
    pub bashrs_version: String,
    pub date: String,
}

/// Build dataset rows from corpus entries and results
pub fn build_dataset(registry: &CorpusRegistry, score: &CorpusScore) -> Vec<DatasetRow> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let date = current_date();
    let commit = current_commit();

    // KAIZEN-069: O(1) HashMap lookup instead of O(n) linear find per entry.
    // With 17,942 entries, the old O(n²) find wasted ~161M string comparisons.
    let results_by_id: HashMap<&str, &CorpusResult> =
        score.results.iter().map(|r| (r.id.as_str(), r)).collect();

    registry
        .entries
        .iter()
        .map(|entry| {
            let result = results_by_id.get(entry.id.as_str()).copied();
            build_row(entry, result, &version, &commit, &date)
        })
        .collect()
}

fn build_row(
    entry: &CorpusEntry,
    result: Option<&CorpusResult>,
    version: &str,
    commit: &str,
    date: &str,
) -> DatasetRow {
    let (transpiled, output_correct, lint_clean, deterministic, actual, score_val, grade) =
        match result {
            Some(r) => (
                r.transpiled,
                r.output_contains && r.output_exact,
                r.lint_clean,
                r.deterministic,
                r.actual_output.clone().unwrap_or_default(),
                r.score(),
                score_to_grade(r.score()),
            ),
            None => (false, false, false, false, String::new(), 0.0, "F".into()),
        };

    let safety_index = derive_safety_label(&actual, transpiled, lint_clean, deterministic);

    DatasetRow {
        id: entry.id.clone(),
        name: entry.name.clone(),
        tier: entry.tier as u8,
        format: entry.format.to_string(),
        input_rust: entry.input.clone(),
        expected_output: entry.expected_output.clone(),
        actual_output: actual,
        transpiled,
        output_correct,
        lint_clean,
        deterministic,
        score: score_val,
        grade,
        safety_index,
        safety_label: SAFETY_LABELS[safety_index as usize].to_string(),
        bashrs_version: version.to_string(),
        commit_sha: commit.to_string(),
        date: date.to_string(),
    }
}

/// Derive safety class from transpiler output using a decision tree.
///
/// Decision tree (cascading priority):
/// 1. Not transpiled OR not lint-clean → unsafe (4)
/// 2. Not deterministic → non-deterministic (2)
/// 3. Non-idempotent patterns (mkdir without -p, rm without -f, ln without -sf) → non-idempotent (3)
/// 4. Unquoted variable expansion ($VAR without quotes) → needs-quoting (1)
/// 5. Otherwise → safe (0)
///
/// Returns safety class index (0-4).
pub fn derive_safety_label(
    shell_output: &str,
    transpiled: bool,
    lint_clean: bool,
    deterministic: bool,
) -> u8 {
    // Gate 1: failed transpilation or lint → unsafe
    if !transpiled || !lint_clean {
        return 4;
    }

    // Gate 2: non-deterministic → class 2
    if !deterministic {
        return 2;
    }

    // Gate 3: non-idempotent patterns in the shell output
    if has_non_idempotent_pattern(shell_output) {
        return 3;
    }

    // Gate 4: unquoted variable expansion → needs-quoting
    if has_unquoted_variable(shell_output) {
        return 1;
    }

    // Default: safe
    0
}

/// Check for non-idempotent shell patterns.
///
/// Detects:
/// - `mkdir` without `-p` flag
/// - `rm` without `-f` flag (non-force removes fail on missing files)
/// - `ln -s` without `-f` (fails if link exists)
pub fn has_non_idempotent_pattern(script: &str) -> bool {
    for line in script.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // mkdir without -p
        if trimmed.starts_with("mkdir ") && !trimmed.contains("-p") {
            return true;
        }

        // rm without -f (but not rm -rf, rm -f)
        if trimmed.starts_with("rm ") && !trimmed.contains("-f") && !trimmed.contains("-rf") {
            return true;
        }

        // ln -s without -f (non-idempotent symlink creation)
        if trimmed.starts_with("ln ")
            && trimmed.contains("-s")
            && !trimmed.contains("-sf")
            && !trimmed.contains("-f")
        {
            return true;
        }
    }
    false
}

/// Check for unquoted variable expansions in shell script.
///
/// Detects `$VAR` or `${VAR}` that appear outside of double quotes.
/// Simple heuristic: scans for `$` followed by alphanumeric/underscore
/// that is NOT within a double-quoted region on the same line.
pub fn has_unquoted_variable(script: &str) -> bool {
    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line_has_unquoted_var(trimmed) {
            return true;
        }
    }
    false
}

/// Check a single line for unquoted variable references.

include!("dataset_line.rs");
