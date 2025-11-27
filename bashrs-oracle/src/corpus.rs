//! Training corpus management for ML model.

use crate::categories::ErrorCategory;
use crate::features::ErrorFeatures;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Training example with features and label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    /// Exit code from command execution.
    pub exit_code: i32,
    /// Standard error output.
    pub stderr: String,
    /// Optional command that was executed.
    pub command: Option<String>,
    /// Error category label.
    pub category: ErrorCategory,
}

/// Training corpus management.
pub struct Corpus {
    examples: Vec<TrainingExample>,
}

impl Default for Corpus {
    fn default() -> Self {
        Self::new()
    }
}

impl Corpus {
    /// Create empty corpus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
        }
    }

    /// Create corpus from examples.
    #[must_use]
    pub fn from_examples(examples: Vec<TrainingExample>) -> Self {
        Self { examples }
    }

    /// Load corpus from JSON file.
    ///
    /// # Errors
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let examples: Vec<TrainingExample> = serde_json::from_str(&content)?;
        Ok(Self { examples })
    }

    /// Save corpus to JSON file.
    ///
    /// # Errors
    /// Returns error if file cannot be written.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.examples)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a training example.
    pub fn add(&mut self, example: TrainingExample) {
        self.examples.push(example);
    }

    /// Generate synthetic training data.
    #[must_use]
    pub fn generate_synthetic(count: usize) -> Self {
        let mut examples = Vec::with_capacity(count);
        let mut rng_seed = 42u64;

        // Template: (exit_code, stderr, category)
        let templates: &[(i32, &str, ErrorCategory)] = &[
            // Syntax errors
            (
                1,
                "bash: syntax error near unexpected token 'done'",
                ErrorCategory::SyntaxUnexpectedToken,
            ),
            (
                1,
                "bash: unexpected EOF while looking for matching '\"'",
                ErrorCategory::SyntaxQuoteMismatch,
            ),
            (
                1,
                "bash: syntax error: unexpected end of file",
                ErrorCategory::SyntaxBracketMismatch,
            ),
            (
                1,
                "bash: syntax error near unexpected token ')'",
                ErrorCategory::SyntaxBracketMismatch,
            ),
            (
                2,
                "bash: line 5: syntax error: operand expected",
                ErrorCategory::SyntaxMissingOperand,
            ),
            // Command errors
            (
                127,
                "bash: foobar: command not found",
                ErrorCategory::CommandNotFound,
            ),
            (
                127,
                "zsh: command not found: nonexistent",
                ErrorCategory::CommandNotFound,
            ),
            (
                126,
                "bash: ./script.sh: Permission denied",
                ErrorCategory::CommandPermissionDenied,
            ),
            (
                1,
                "grep: invalid option -- 'z'",
                ErrorCategory::CommandInvalidOption,
            ),
            (
                1,
                "ls: option requires an argument -- 'w'",
                ErrorCategory::CommandMissingArgument,
            ),
            // File errors
            (
                1,
                "cat: /nonexistent: No such file or directory",
                ErrorCategory::FileNotFound,
            ),
            (
                1,
                "rm: cannot remove '/root/secret': Permission denied",
                ErrorCategory::FilePermissionDenied,
            ),
            (
                1,
                "cat: /tmp: Is a directory",
                ErrorCategory::FileIsDirectory,
            ),
            (
                1,
                "cd: /etc/passwd: Not a directory",
                ErrorCategory::FileNotDirectory,
            ),
            (
                1,
                "bash: cannot redirect: Too many open files",
                ErrorCategory::FileTooManyOpen,
            ),
            // Variable errors
            (
                1,
                "bash: VAR: unbound variable",
                ErrorCategory::VariableUnbound,
            ),
            (
                1,
                "bash: PATH: readonly variable",
                ErrorCategory::VariableReadonly,
            ),
            (
                1,
                "bash: ${foo: bad substitution",
                ErrorCategory::VariableBadSubstitution,
            ),
            // Process errors
            (141, "", ErrorCategory::PipeBroken), // SIGPIPE = 128 + 13
            (137, "Killed", ErrorCategory::ProcessSignaled), // SIGKILL = 128 + 9
            (
                1,
                "Command exited with status 1",
                ErrorCategory::ProcessExitNonZero,
            ),
            (
                124,
                "timeout: the monitored command timed out",
                ErrorCategory::ProcessTimeout,
            ),
            // Redirect errors
            (
                1,
                "bash: /dev/full: No space left on device",
                ErrorCategory::RedirectFailed,
            ),
            (
                1,
                "bash: warning: here-document delimited by end-of-file (wanted 'EOF')",
                ErrorCategory::HereDocUnterminated,
            ),
        ];

        for i in 0..count {
            // Simple LCG for reproducibility
            rng_seed = rng_seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let idx = (rng_seed as usize) % templates.len();
            // Safety: idx is always within bounds due to modulo, but use unwrap_or for clippy
            let (exit_code, stderr, category) =
                templates
                    .get(idx)
                    .copied()
                    .unwrap_or((1, "unknown error", ErrorCategory::Unknown));

            // Add variation to make examples more diverse
            let varied_stderr = match i % 5 {
                0 => format!("{stderr} (variant {i})"),
                1 => format!("line {}: {stderr}", (rng_seed % 100) + 1),
                2 => stderr.to_uppercase(),
                3 => format!("{stderr}\nAdditional context line"),
                _ => stderr.to_string(),
            };

            let command = if i % 3 == 0 {
                Some(format!("test_command_{}", i % 10))
            } else {
                None
            };

            examples.push(TrainingExample {
                exit_code,
                stderr: varied_stderr,
                command,
                category,
            });
        }

        Self { examples }
    }

    /// Convert to feature matrix (X) and labels (y).
    #[must_use]
    pub fn to_training_data(&self) -> (Vec<Vec<f32>>, Vec<u8>) {
        let mut x = Vec::with_capacity(self.examples.len());
        let mut y = Vec::with_capacity(self.examples.len());

        for example in &self.examples {
            let features = ErrorFeatures::extract(
                example.exit_code,
                &example.stderr,
                example.command.as_deref(),
            );
            x.push(features.features);
            y.push(example.category.to_label_index() as u8);
        }

        (x, y)
    }

    /// Number of examples in corpus.
    #[must_use]
    pub fn len(&self) -> usize {
        self.examples.len()
    }

    /// Check if corpus is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    /// Get examples as slice.
    #[must_use]
    pub fn examples(&self) -> &[TrainingExample] {
        &self.examples
    }

    /// Get category distribution for analysis.
    #[must_use]
    pub fn category_distribution(&self) -> std::collections::HashMap<ErrorCategory, usize> {
        let mut dist = std::collections::HashMap::new();
        for example in &self.examples {
            *dist.entry(example.category).or_insert(0) += 1;
        }
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_synthetic() {
        let corpus = Corpus::generate_synthetic(100);
        assert_eq!(corpus.len(), 100);
    }

    #[test]
    fn test_to_training_data() {
        let corpus = Corpus::generate_synthetic(50);
        let (x, y) = corpus.to_training_data();

        assert_eq!(x.len(), 50);
        assert_eq!(y.len(), 50);
        assert_eq!(x[0].len(), ErrorFeatures::SIZE);
    }

    #[test]
    fn test_category_distribution() {
        let corpus = Corpus::generate_synthetic(1000);
        let dist = corpus.category_distribution();

        // Should have multiple categories represented
        assert!(dist.len() > 5, "Expected diverse categories");
    }

    #[test]
    fn test_corpus_save_load() {
        let corpus = Corpus::generate_synthetic(10);
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_corpus.json");

        corpus.save(&path).expect("Failed to save");
        let loaded = Corpus::load(&path).expect("Failed to load");

        assert_eq!(corpus.len(), loaded.len());
    }

    #[test]
    fn test_training_labels_valid() {
        let corpus = Corpus::generate_synthetic(100);
        let (_, y) = corpus.to_training_data();

        for label in y {
            // Labels should be valid indices (0-22)
            assert!(label < ErrorCategory::COUNT as u8);
        }
    }

    #[test]
    fn test_example_serialization() {
        let example = TrainingExample {
            exit_code: 127,
            stderr: "command not found".to_string(),
            command: Some("foo".to_string()),
            category: ErrorCategory::CommandNotFound,
        };

        let json = serde_json::to_string(&example).expect("Failed to serialize");
        let parsed: TrainingExample = serde_json::from_str(&json).expect("Failed to parse");

        assert_eq!(parsed.exit_code, 127);
        assert_eq!(parsed.category, ErrorCategory::CommandNotFound);
    }
}
