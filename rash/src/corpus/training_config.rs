//! Training configuration export for entrenar (SSC v11 S9, CLF-001).
//!
//! Generates an entrenar-compatible training configuration from live
//! corpus statistics. Includes model architecture, LoRA parameters,
//! optimizer settings, and computed class weights.

use serde::Serialize;

/// Entrenar training configuration for SSC classifier.
#[derive(Debug, Clone, Serialize)]
pub struct TrainingConfig {
    pub model: ModelConfig,
    pub training: TrainingParams,
    pub data: DataConfig,
    pub evaluation: EvalConfig,
}

/// Model architecture configuration.
#[derive(Debug, Clone, Serialize)]
pub struct ModelConfig {
    pub architecture: String,
    pub base_model: String,
    pub num_classes: u32,
    pub hidden_size: u32,
    pub num_layers: u32,
    pub pooling: String,
    pub lora: Option<LoraConfig>,
}

/// LoRA adapter configuration.
#[derive(Debug, Clone, Serialize)]
pub struct LoraConfig {
    pub rank: u32,
    pub alpha: f64,
    pub targets: Vec<String>,
}

/// Training hyperparameters.
#[derive(Debug, Clone, Serialize)]
pub struct TrainingParams {
    pub epochs: u32,
    pub batch_size: u32,
    pub learning_rate: f64,
    pub optimizer: String,
    pub scheduler: String,
    pub warmup_steps: u32,
    pub weight_decay: f64,
    pub max_seq_length: u32,
    pub class_weights: Vec<f64>,
}

/// Data configuration.
#[derive(Debug, Clone, Serialize)]
pub struct DataConfig {
    pub total_entries: usize,
    pub safe_count: usize,
    pub unsafe_count: usize,
    pub split_ratio: String,
    pub split_method: String,
    pub preamble_stripped: bool,
}

/// Evaluation configuration.
#[derive(Debug, Clone, Serialize)]
pub struct EvalConfig {
    pub primary_metric: String,
    pub accuracy_target: f64,
    pub mcc_ci_lower_target: f64,
    pub generalization_target: f64,
    pub generalization_scripts: u32,
}

/// Generate a training configuration from live corpus data.
///
/// Walks the full compiled-in corpus (17,942 entries; transpile + lint each),
/// which costs minutes. Contract `training-config-v1` equation `composition`:
/// this is exactly [`generate_training_config_from`] over
/// `CorpusRegistry::load_full()`, so tests exercise the halves separately.
pub fn generate_training_config() -> TrainingConfig {
    generate_training_config_from(&crate::corpus::registry::CorpusRegistry::load_full())
}

/// Generate a training configuration from a specific registry.
///
/// Contract `training-config-v1` equation `counts`: `total_entries` equals
/// `registry.len()` and `safe_count + unsafe_count == total_entries`.
pub fn generate_training_config_from(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> TrainingConfig {
    let owned = crate::corpus::baselines::corpus_baseline_entries_from(registry);
    let total = owned.len();
    let safe_count = owned.iter().filter(|(_, l)| *l == 0).count();
    let unsafe_count = owned.iter().filter(|(_, l)| *l == 1).count();
    training_config_from_counts(total, safe_count, unsafe_count)
}

/// Build the training configuration from class counts alone.
///
/// Pure: no I/O and no corpus access. Contract `training-config-v1` equations
/// `weights` (sqrt-inverse class weights, unsafe second, zero guard) and
/// `constants` (model and evaluation sections do not depend on the counts).
pub fn training_config_from_counts(
    total: usize,
    safe_count: usize,
    unsafe_count: usize,
) -> TrainingConfig {
    let w_safe = compute_sqrt_inverse_weight(safe_count, total);
    let w_unsafe = compute_sqrt_inverse_weight(unsafe_count, total);

    TrainingConfig {
        model: ModelConfig {
            architecture: "encoder".to_string(),
            base_model: "microsoft/codebert-base".to_string(),
            num_classes: 2,
            hidden_size: 768,
            num_layers: 12,
            pooling: "cls".to_string(),
            lora: None,
        },
        training: TrainingParams {
            epochs: 3,
            batch_size: 32,
            learning_rate: 2e-4,
            optimizer: "AdamW".to_string(),
            scheduler: "linear_warmup".to_string(),
            warmup_steps: 100,
            weight_decay: 0.01,
            max_seq_length: 512,
            class_weights: vec![w_safe, w_unsafe],
        },
        data: DataConfig {
            total_entries: total,
            safe_count,
            unsafe_count,
            split_ratio: "80/10/10".to_string(),
            split_method: "FNV-1a hash deterministic".to_string(),
            preamble_stripped: true,
        },
        evaluation: EvalConfig {
            primary_metric: "MCC".to_string(),
            accuracy_target: 0.935,
            mcc_ci_lower_target: 0.2,
            generalization_target: 0.50,
            generalization_scripts: 50,
        },
    }
}

/// Compute sqrt-inverse class weight.
fn compute_sqrt_inverse_weight(class_count: usize, total: usize) -> f64 {
    if class_count == 0 || total == 0 {
        return 1.0;
    }
    let freq = class_count as f64 / total as f64;
    (1.0 / freq).sqrt()
}

/// Format training config as YAML (hand-formatted, no serde_yaml dependency).
pub fn format_yaml(config: &TrainingConfig) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(2048);

    let _ = writeln!(
        out,
        "# SSC v11 Training Configuration (entrenar-compatible)"
    );
    let _ = writeln!(out, "# Generated by bashrs corpus training-config");
    let _ = writeln!(out);

    // Model
    let _ = writeln!(out, "model:");
    let _ = writeln!(out, "  architecture: {}", config.model.architecture);
    let _ = writeln!(out, "  base_model: {}", config.model.base_model);
    let _ = writeln!(out, "  num_classes: {}", config.model.num_classes);
    let _ = writeln!(out, "  hidden_size: {}", config.model.hidden_size);
    let _ = writeln!(out, "  num_layers: {}", config.model.num_layers);
    let _ = writeln!(out, "  pooling: {}", config.model.pooling);
    if let Some(ref lora) = config.model.lora {
        let _ = writeln!(out, "  lora:");
        let _ = writeln!(out, "    rank: {}", lora.rank);
        let _ = writeln!(out, "    alpha: {}", lora.alpha);
        let _ = writeln!(out, "    targets:");
        for t in &lora.targets {
            let _ = writeln!(out, "    - {t}");
        }
    }
    let _ = writeln!(out);

    // Training
    let _ = writeln!(out, "training:");
    let _ = writeln!(out, "  epochs: {}", config.training.epochs);
    let _ = writeln!(out, "  batch_size: {}", config.training.batch_size);
    let _ = writeln!(out, "  learning_rate: {}", config.training.learning_rate);
    let _ = writeln!(out, "  optimizer: {}", config.training.optimizer);
    let _ = writeln!(out, "  scheduler: {}", config.training.scheduler);
    let _ = writeln!(out, "  warmup_steps: {}", config.training.warmup_steps);
    let _ = writeln!(out, "  weight_decay: {}", config.training.weight_decay);
    let _ = writeln!(out, "  max_seq_length: {}", config.training.max_seq_length);
    let _ = writeln!(out, "  class_weights:");
    for (i, w) in config.training.class_weights.iter().enumerate() {
        let _ = writeln!(out, "  - {w:.3}  # class {i}");
    }
    let _ = writeln!(out);

    // Data
    let _ = writeln!(out, "data:");
    let _ = writeln!(out, "  total_entries: {}", config.data.total_entries);
    let _ = writeln!(out, "  safe_count: {}", config.data.safe_count);
    let _ = writeln!(out, "  unsafe_count: {}", config.data.unsafe_count);
    let _ = writeln!(out, "  split_ratio: \"{}\"", config.data.split_ratio);
    let _ = writeln!(out, "  split_method: \"{}\"", config.data.split_method);
    let _ = writeln!(
        out,
        "  preamble_stripped: {}",
        config.data.preamble_stripped
    );
    let _ = writeln!(out);

    // Evaluation
    let _ = writeln!(out, "evaluation:");
    let _ = writeln!(
        out,
        "  primary_metric: {}",
        config.evaluation.primary_metric
    );
    let _ = writeln!(
        out,
        "  accuracy_target: {}",
        config.evaluation.accuracy_target
    );
    let _ = writeln!(
        out,
        "  mcc_ci_lower_target: {}",
        config.evaluation.mcc_ci_lower_target
    );
    let _ = writeln!(
        out,
        "  generalization_target: {}",
        config.evaluation.generalization_target
    );
    let _ = writeln!(
        out,
        "  generalization_scripts: {}",
        config.evaluation.generalization_scripts
    );

    out
}

/// Format training config as JSON.
pub fn format_json(config: &TrainingConfig) -> String {
    serde_json::to_string_pretty(config).unwrap_or_else(|_| format!("{config:#?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::corpus::registry::CorpusRegistry;

    // Contract: contracts/training-config-v1.yaml. None of these tests call
    // `generate_training_config()`: it walks all 17,942 corpus entries and cost
    // 477-660 s per test in CI, which is what pushed the 60-minute test job over
    // its limit (run 34368312280). The pure half is tested on fixed counts and
    // the corpus-backed half on the 30-entry tier-1 set.

    /// Counts measured on the full corpus (CHANGELOG 7.0.2 / SSC #172).
    const FULL: (usize, usize, usize) = (17_942, 17_794, 148);

    /// F-TC-004: model and evaluation constants do not depend on the counts.
    #[test]
    fn test_generate_training_config_structure() {
        let a = training_config_from_counts(FULL.0, FULL.1, FULL.2);
        let b = training_config_from_counts(7, 3, 4);

        assert_eq!(a.model.architecture, "encoder");
        assert_eq!(a.model.num_classes, 2);
        assert_eq!(a.model.hidden_size, 768);
        assert_eq!(a.training.epochs, 3);
        assert_eq!(a.evaluation.primary_metric, "MCC");

        assert_eq!(a.model.architecture, b.model.architecture);
        assert_eq!(a.model.base_model, b.model.base_model);
        assert_eq!(a.model.num_classes, b.model.num_classes);
        assert_eq!(a.model.hidden_size, b.model.hidden_size);
        assert_eq!(a.model.num_layers, b.model.num_layers);
        assert_eq!(a.training.epochs, b.training.epochs);
        assert_eq!(a.training.max_seq_length, b.training.max_seq_length);
        assert_eq!(a.evaluation.primary_metric, b.evaluation.primary_metric);
        assert_eq!(a.evaluation.accuracy_target, b.evaluation.accuracy_target);
        // The counts, by contrast, must flow through.
        assert_eq!(a.data.total_entries, FULL.0);
        assert_eq!(b.data.total_entries, 7);
    }

    /// F-TC-001: the corpus-backed path counts every entry of its registry.
    #[test]
    fn test_F_TC_001_counts_match_tier1_registry() {
        let registry = CorpusRegistry::load_tier1();
        assert_eq!(
            registry.len(),
            30,
            "tier-1 set size (corpus_data.jsonl sets bit 4)"
        );

        let config = generate_training_config_from(&registry);
        assert_eq!(config.data.total_entries, registry.len());
        assert_eq!(
            config.data.safe_count + config.data.unsafe_count,
            config.data.total_entries,
            "every entry is labelled exactly once"
        );
        assert!(config.data.preamble_stripped);
        assert_eq!(config.training.class_weights.len(), 2);
        assert!(config.training.class_weights.iter().all(|w| w.is_finite()));
    }

    /// F-TC-002: class weights are sqrt-inverse of class frequency, unsafe second.
    #[test]
    fn test_F_TC_002_weights_are_sqrt_inverse() {
        let (total, safe, unsafe_) = FULL;
        let config = training_config_from_counts(total, safe, unsafe_);
        let w = &config.training.class_weights;
        assert_eq!(w.len(), 2);
        let expect_safe = (total as f64 / safe as f64).sqrt();
        let expect_unsafe = (total as f64 / unsafe_ as f64).sqrt();
        assert!((w[0] - expect_safe).abs() < 1e-9, "safe weight {}", w[0]);
        assert!(
            (w[1] - expect_unsafe).abs() < 1e-9,
            "unsafe weight {}",
            w[1]
        );
        assert!(
            w[1] > w[0],
            "the minority (unsafe) class carries the larger weight"
        );
        assert_eq!(config.data.safe_count, safe);
        assert_eq!(config.data.unsafe_count, unsafe_);
    }

    /// F-TC-003: an empty class or an empty corpus degrades to weight 1.0.
    #[test]
    fn test_F_TC_003_weights_guard_empty_classes() {
        let empty = training_config_from_counts(0, 0, 0);
        assert_eq!(empty.training.class_weights, vec![1.0, 1.0]);

        let one_sided = training_config_from_counts(10, 10, 0);
        assert!((one_sided.training.class_weights[0] - 1.0).abs() < 1e-9);
        assert!((one_sided.training.class_weights[1] - 1.0).abs() < 1e-9);
        assert!(one_sided
            .training
            .class_weights
            .iter()
            .all(|w| w.is_finite()));
    }

    /// F-TC-005: YAML formatting is a function of the config alone.
    #[test]
    fn test_format_yaml_produces_yaml() {
        let config = training_config_from_counts(FULL.0, FULL.1, FULL.2);
        let yaml = format_yaml(&config);
        assert!(yaml.contains("architecture: encoder"), "Must produce YAML");
        assert!(yaml.contains("codebert"), "Must reference CodeBERT");
        assert!(yaml.contains("class_weights:"), "Must have class weights");
    }

    /// F-TC-006: JSON formatting is valid JSON.
    #[test]
    fn test_format_json_produces_json() {
        let config = training_config_from_counts(FULL.0, FULL.1, FULL.2);
        let json = format_json(&config);
        assert!(json.contains("\"architecture\""), "Must produce JSON");
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Must produce valid JSON");
    }

    /// F-TC-007: the public entry point is the corpus-backed path over the full
    /// registry. Checked on the source text because executing it costs ~8 min.
    #[test]
    fn test_F_TC_007_public_entry_point_is_full_registry_path() {
        let src = include_str!("training_config.rs");
        let head = "pub fn generate_training_config() -> TrainingConfig {";
        let Some(start) = src.find(head) else {
            panic!("generate_training_config() not found in source");
        };
        let body: Vec<&str> = src[start + head.len()..]
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .take(2)
            .collect();
        assert_eq!(
            body,
            vec![
                "generate_training_config_from(&crate::corpus::registry::CorpusRegistry::load_full())",
                "}",
            ],
            "generate_training_config() must be exactly the _from path over load_full()"
        );
    }

    /// Full-corpus data properties (unsafe entries exist, corpus is large).
    /// Walks all 17,942 entries: ~8 minutes. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore = "walks the full corpus (~8 min); the tier-1 path is covered by F-TC-001"]
    fn test_full_corpus_training_config_data() {
        let config = generate_training_config();
        assert!(
            config.data.total_entries >= 17_942,
            "Must have the whole corpus"
        );
        assert!(config.data.safe_count > 0);
        assert!(config.data.unsafe_count > 0);
        assert!(
            config.training.class_weights[1] > config.training.class_weights[0],
            "Unsafe weight should be higher than safe weight"
        );
    }

    #[test]
    fn test_compute_sqrt_inverse_weight_balanced() {
        let w = compute_sqrt_inverse_weight(50, 100);
        assert!((w - std::f64::consts::SQRT_2).abs() < 0.001);
    }

    #[test]
    fn test_compute_sqrt_inverse_weight_guards() {
        assert!((compute_sqrt_inverse_weight(0, 100) - 1.0).abs() < 1e-9);
        assert!((compute_sqrt_inverse_weight(50, 0) - 1.0).abs() < 1e-9);
    }
}
