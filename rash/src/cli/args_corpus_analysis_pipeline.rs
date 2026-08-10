//! Corpus analysis subcommands, group 3 of 3 (original variants 43-63:
//! `merge-data` .. `batch-eval`).
//!
//! GH-215: `clap_derive` emits one `let __clap_app = __clap_app.subcommand({..});`
//! binding per variant inside `augment_subcommands`. At opt-level 0 each binding
//! gets its own stack slot (~12KB), so a single 63-variant enum needed ~768KB of
//! frame and overflowed the 2MB stack `cargo test` gives each test thread.
//! `CorpusAnalysisCommands` is now a shell of `#[command(flatten)]` groups; clap
//! calls each group's `augment_subcommands` as a sequential statement, so peak
//! stack is parent + MAX(group) rather than parent + SUM(variants).
//!
//! `#[command(flatten)]` keeps every subcommand at the same CLI level: this is a
//! pure code-organisation change with zero effect on the user-visible surface.
//! Variant order across the groups MUST match the original declaration order so
//! `--help` ordering is unchanged.

use clap::Subcommand;

/// Corpus analysis subcommands, group 3 of 3 (flattened into `CorpusAnalysisCommands`).
#[derive(Subcommand)]
pub enum CorpusAnalysisPipelineCommands {
    /// Merge corpus + verificar data into unified training JSONL (SSC v12 S14 Step 7.4)
    MergeData {
        /// Output file
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Additional JSONL input files to merge (e.g., verificar-labeled.jsonl)
        #[arg(short, long)]
        input: Vec<std::path::PathBuf>,

        /// Random seed for shuffling
        #[arg(long, default_value = "42")]
        seed: u64,
    },

    /// Cross-validate bashrs labels against ShellCheck (SSC v12 S14.9 Step 7.4e)
    ShellcheckValidate {
        /// Number of samples to validate
        #[arg(long, default_value = "500")]
        samples: usize,

        /// Random seed
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run eval harness on benchmark predictions (SSC v12 S14.5)
    EvalBenchmark {
        /// Predictions JSONL file
        #[arg(short, long)]
        predictions: std::path::PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Label external JSONL with linter findings + CWE mappings (SSC v12 pipeline)
    Label {
        /// Input JSONL file (one shell script per line, field: "script" or "text")
        #[arg(short, long)]
        input: std::path::PathBuf,

        /// Output JSONL file (labeled entries)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Audit safety label accuracy (SSC v11 S5.3, C-LABEL-001)
    LabelAudit {
        /// Maximum unsafe entries to audit (default: 100)
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },

    /// Run out-of-distribution generalization tests (SSC v11 S5.6)
    GeneralizationTests,

    /// Validate tokenizer quality on shell constructs (SSC v11 S5.2, C-TOK-001)
    TokenizerValidation,

    /// Run all SSC contract validations (pre-training gate)
    ValidateContracts,

    /// Export dataset with train/val/test splits for ML training
    ExportSplits {
        /// Output directory for split JSONL files
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Input merged JSONL (if omitted, uses corpus transpilation)
        #[arg(long)]
        input: Option<std::path::PathBuf>,
    },

    /// Show comprehensive SSC v11 readiness report
    SscReport {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// CI gate mode: exit 1 if any section fails
        #[arg(long)]
        gate: bool,
    },

    /// Generate HuggingFace model card for SSC dataset/classifier (S6.5, S9)
    ModelCard {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Export entrenar-compatible training configuration (S9 CLF-001)
    TrainingConfig {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output as JSON instead of YAML
        #[arg(long)]
        json: bool,
    },

    /// Export complete HuggingFace-ready dataset directory (S9 GEN-003)
    PublishDataset {
        /// Output directory (required)
        #[arg(short, long)]
        output: std::path::PathBuf,
    },

    /// Publish ShellSafetyBench to HuggingFace (SSC v12 S14.7, Phase 10)
    PublishBenchmark {
        /// Directory containing SSB split files (train.jsonl, val.jsonl, test.jsonl)
        #[arg(short = 'i', long)]
        input: std::path::PathBuf,

        /// Output directory for HuggingFace-ready repository
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Version tag (e.g., "1.0.0")
        #[arg(long, default_value = "1.0.0")]
        version: String,
    },

    /// Generate expansion entries for ShellSafetyBench (Phase 9 #10: 27K -> 50K+)
    GenerateExpansion {
        /// Script format to generate
        #[arg(short, long, value_parser = ["bash", "makefile", "dockerfile"])]
        format: String,

        /// Number of entries to generate
        #[arg(short, long, default_value = "5000")]
        count: usize,

        /// Output JSONL file
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Random seed for reproducibility
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Export HuggingFace-ready conversation dataset (S6.6 paiml/shell-safety-conversations)
    PublishConversations {
        /// Output directory (required)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Random seed for variant selection
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Convert SSB splits to entrenar ChatML JSONL for chat model training (PMAT-167)
    ConvertSsb {
        /// Input JSONL file (SSB format: {"input":"...","label":0|1})
        #[arg(short, long)]
        input: std::path::PathBuf,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Maximum entries to convert
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Extract [CLS] embeddings from CodeBERT for all corpus entries (CLF-RUN step 1)
    ExtractEmbeddings {
        /// Path to CodeBERT model directory (must contain model.safetensors)
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Output file for cached embeddings (JSONL)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Maximum number of entries to process (for testing)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Extract from JSONL file instead of corpus (format: {"input":"...","label":N})
        #[arg(long)]
        input_jsonl: Option<std::path::PathBuf>,
    },

    /// Train linear probe classifier on cached embeddings (CLF-RUN step 2-3)
    TrainClassifier {
        /// Path to cached embeddings JSONL (from extract-embeddings)
        #[arg(short, long)]
        embeddings: std::path::PathBuf,

        /// Output directory for probe weights and evaluation report
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Training epochs
        #[arg(long, default_value = "30")]
        epochs: usize,

        /// Learning rate
        #[arg(long, default_value = "0.01")]
        learning_rate: f32,

        /// Random seed for train/test split
        #[arg(short, long, default_value = "42")]
        seed: u64,

        /// Maximum entries to use (caps training data to avoid data labeling gaps)
        #[arg(long)]
        max_entries: Option<usize>,

        /// Additional embedding JSONL files to augment training data (e.g. adversarial entries)
        #[arg(long)]
        augment: Vec<std::path::PathBuf>,

        /// Use MLP probe (2-layer with ReLU) instead of linear probe
        #[arg(long)]
        mlp: bool,

        /// MLP hidden layer size (only with --mlp)
        #[arg(long, default_value = "128")]
        mlp_hidden: usize,
    },

    /// Run full CLF-RUN pipeline: extract embeddings -> train -> evaluate (requires --features ml)
    RunClassifier {
        /// Path to CodeBERT model directory
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Output directory for all artifacts
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Training epochs
        #[arg(long, default_value = "30")]
        epochs: usize,

        /// Learning rate
        #[arg(long, default_value = "0.01")]
        learning_rate: f32,

        /// Random seed
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Run batch inference on test split using a trained model checkpoint (SSC v12 S14)
    BatchEval {
        /// Path to model directory (config.json + safetensors + optional LoRA adapter)
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Path to test JSONL file (entries with "input" and "label" fields)
        #[arg(short, long)]
        test_data: std::path::PathBuf,

        /// Output predictions JSONL path (compatible with eval-benchmark)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Maximum tokens to generate per entry (default: 128)
        #[arg(long, default_value = "128")]
        max_tokens: usize,
    },
}
