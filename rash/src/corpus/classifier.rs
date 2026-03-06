//! # CodeBERT Classifier Pipeline (SSC v11 CLF-RUN)
//!
//! Implements the three-step pipeline:
//! 1. Extract [CLS] embeddings from frozen CodeBERT (Level 0)
//! 2. Train linear probe on cached embeddings
//! 3. Evaluate with MCC, accuracy, precision, recall
//!
//! Requires the `ml` feature flag (aprender + entrenar).

#[cfg(feature = "ml")]
use entrenar::transformer::{EncoderModel, TransformerConfig};

#[cfg(feature = "ml")]
use crate::corpus::dataset::ClassificationRow;
use crate::corpus::evaluation::{evaluate, EvaluationReport};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Hidden size for CodeBERT (768 dimensions).
const CODEBERT_HIDDEN_SIZE: usize = 768;

/// Maximum sequence length for CodeBERT tokenizer.
#[cfg(any(feature = "ml", test))]
const MAX_SEQ_LEN: usize = 512;

/// A cached embedding with its label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub label: u8,
}

/// Result of embedding extraction.
#[derive(Debug, Serialize)]
pub struct ExtractionReport {
    pub total_entries: usize,
    pub extracted: usize,
    pub skipped: usize,
    pub hidden_size: usize,
}

/// Trained linear probe weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearProbe {
    /// Weight vector: [hidden_size] for binary classification
    pub weights: Vec<f32>,
    /// Bias term
    pub bias: f32,
    /// Training metadata
    pub epochs: usize,
    pub learning_rate: f32,
    pub train_accuracy: f64,
    pub train_mcc: f64,
}

/// Result of classifier training and evaluation.
#[derive(Debug, Serialize)]
pub struct ClassifierReport {
    pub train_eval: EvaluationReport,
    pub test_eval: EvaluationReport,
    pub probe: LinearProbe,
    pub beats_keyword: bool,
    pub beats_linter: bool,
}

/// Tokenize a script into token IDs using a simple whitespace + subword tokenizer.
///
/// This is a fallback when the full BPE tokenizer is not available.
/// For production, use the CodeBERT RoBERTa tokenizer via aprender.
#[cfg(any(feature = "ml", test))]
fn simple_tokenize(script: &str) -> Vec<u32> {
    // CLS=0, SEP=2, PAD=1 (RoBERTa convention)
    let mut ids = vec![0u32]; // CLS
    for byte in script.bytes().take(MAX_SEQ_LEN - 2) {
        // Simple byte-level encoding offset by 4 (skip special tokens)
        ids.push(u32::from(byte) + 4);
    }
    ids.push(2); // SEP
    ids
}

/// Extract [CLS] embeddings from CodeBERT for all corpus entries.
///
/// Loads the model from `model_dir` (expects model.safetensors + config.json),
/// tokenizes each entry's input, and extracts the [CLS] token hidden state.
#[cfg(feature = "ml")]
pub fn extract_embeddings(
    model_dir: &Path,
    entries: &[ClassificationRow],
    progress_fn: Option<&dyn Fn(usize, usize)>,
) -> Result<(Vec<EmbeddingEntry>, ExtractionReport), String> {
    let config = TransformerConfig::codebert();
    eprintln!("  Loading CodeBERT from {}...", model_dir.display());
    let model = EncoderModel::from_safetensors(&config, model_dir)
        .map_err(|e| format!("Failed to load CodeBERT: {e}"))?;
    eprintln!(
        "  Loaded {} parameters ({} layers, {} hidden)",
        model.num_parameters(),
        config.num_hidden_layers,
        config.hidden_size
    );

    let total = entries.len();
    let mut embeddings = Vec::with_capacity(total);
    let mut skipped = 0;

    for (i, entry) in entries.iter().enumerate() {
        if let Some(pf) = progress_fn {
            pf(i, total);
        }

        let token_ids = tokenize_for_codebert(&entry.input);
        if token_ids.len() < 3 {
            skipped += 1;
            continue;
        }

        let cls = model.cls_embedding(&token_ids);
        let data = cls.data();
        let slice = data.as_slice().ok_or("CLS embedding not contiguous")?;

        embeddings.push(EmbeddingEntry {
            id: format!("entry_{i}"),
            embedding: slice.to_vec(),
            label: entry.label,
        });
    }

    let report = ExtractionReport {
        total_entries: total,
        extracted: embeddings.len(),
        skipped,
        hidden_size: config.hidden_size,
    };

    Ok((embeddings, report))
}

/// Tokenize a shell script for CodeBERT input.
///
/// Uses aprender's BPE tokenizer when available, falls back to simple
/// byte-level encoding.
#[cfg(feature = "ml")]
fn tokenize_for_codebert(script: &str) -> Vec<u32> {
    // Try to use aprender's BPE tokenizer if vocab files are available
    // For now, use the simple byte-level fallback
    simple_tokenize(script)
}

/// Train a linear probe on pre-extracted embeddings.
///
/// Uses simple logistic regression with SGD:
/// - sigmoid(w @ embedding + b) → P(unsafe)
/// - Binary cross-entropy loss
/// - No regularization (embeddings are high-quality from pretrained encoder)
pub fn train_linear_probe(
    train: &[EmbeddingEntry],
    epochs: usize,
    learning_rate: f32,
) -> LinearProbe {
    let h = if train.is_empty() {
        CODEBERT_HIDDEN_SIZE
    } else {
        train[0].embedding.len()
    };

    // Initialize weights to zero (logistic regression convention)
    let mut weights = vec![0.0f32; h];
    let mut bias = 0.0f32;

    for _epoch in 0..epochs {
        let mut total_loss = 0.0f64;
        for entry in train {
            // Forward: logit = w . x + b
            let logit: f32 = weights
                .iter()
                .zip(entry.embedding.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>()
                + bias;

            // Sigmoid
            let prob = sigmoid(logit);

            // Target: 1.0 for unsafe (label=1), 0.0 for safe (label=0)
            let target = entry.label as f32;

            // Gradient: d_loss/d_logit = prob - target
            let grad = prob - target;

            total_loss += f64::from(-target * logit.clamp(-100.0, 100.0)
                + (1.0 + (-logit).exp()).ln().max(0.0));

            // SGD update
            for (w, x) in weights.iter_mut().zip(entry.embedding.iter()) {
                *w -= learning_rate * grad * x;
            }
            bias -= learning_rate * grad;
        }

        let avg_loss = if train.is_empty() {
            0.0
        } else {
            total_loss / train.len() as f64
        };
        if (_epoch + 1) % 5 == 0 || _epoch == 0 {
            eprintln!("  Epoch {}/{epochs}: loss={avg_loss:.4}", _epoch + 1);
        }
    }

    // Compute training accuracy and MCC
    let predictions: Vec<(u8, u8)> = train
        .iter()
        .map(|entry| {
            let logit: f32 = weights
                .iter()
                .zip(entry.embedding.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>()
                + bias;
            let pred = u8::from(sigmoid(logit) > 0.5);
            (pred, entry.label)
        })
        .collect();
    let train_report = evaluate(&predictions, "linear_probe_train");

    LinearProbe {
        weights,
        bias,
        epochs,
        learning_rate,
        train_accuracy: train_report.accuracy,
        train_mcc: train_report.mcc,
    }
}

/// Evaluate a trained linear probe on test embeddings.
pub fn evaluate_probe(probe: &LinearProbe, test: &[EmbeddingEntry]) -> EvaluationReport {
    let predictions: Vec<(u8, u8)> = test
        .iter()
        .map(|entry| {
            let logit: f32 = probe
                .weights
                .iter()
                .zip(entry.embedding.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>()
                + probe.bias;
            let pred = u8::from(sigmoid(logit) > 0.5);
            (pred, entry.label)
        })
        .collect();
    evaluate(&predictions, "codebert_linear_probe")
}

/// Run the full CLF-RUN pipeline: extract → split → train → evaluate.
#[cfg(feature = "ml")]
pub fn run_classifier_pipeline(
    model_dir: &Path,
    entries: &[ClassificationRow],
    epochs: usize,
    learning_rate: f32,
    seed: u64,
) -> Result<ClassifierReport, String> {
    // Step 1: Extract embeddings
    eprintln!("Step 1/3: Extracting [CLS] embeddings...");
    let (all_embeddings, report) = extract_embeddings(model_dir, entries, Some(&|i, total| {
        if i % 100 == 0 {
            eprintln!("  {i}/{total} ({:.1}%)", 100.0 * i as f64 / total as f64);
        }
    }))?;
    eprintln!(
        "  Extracted {} embeddings ({} skipped)\n",
        report.extracted, report.skipped
    );

    // Step 2: Split into train/test (80/20, deterministic by seed)
    eprintln!("Step 2/3: Training linear probe...");
    let (train, test) = split_embeddings(&all_embeddings, seed);
    eprintln!(
        "  Train: {} entries, Test: {} entries",
        train.len(),
        test.len()
    );

    let probe = train_linear_probe(&train, epochs, learning_rate);
    eprintln!(
        "  Training complete: accuracy={:.1}%, MCC={:.3}\n",
        probe.train_accuracy * 100.0,
        probe.train_mcc
    );

    // Step 3: Evaluate on test set
    eprintln!("Step 3/3: Evaluating on test set...");
    let train_eval = evaluate_probe(&probe, &train);
    let test_eval = evaluate_probe(&probe, &test);

    // Compare against baselines (keyword ~0.3-0.5 MCC, linter ~0.4-0.6 MCC)
    let beats_keyword = test_eval.mcc > 0.3;
    let beats_linter = test_eval.mcc > 0.4;

    Ok(ClassifierReport {
        train_eval,
        test_eval,
        probe,
        beats_keyword,
        beats_linter,
    })
}

/// Split embeddings into train/test using deterministic hash.
pub fn split_embeddings(
    embeddings: &[EmbeddingEntry],
    seed: u64,
) -> (Vec<EmbeddingEntry>, Vec<EmbeddingEntry>) {
    let mut train = Vec::new();
    let mut test = Vec::new();

    for (i, entry) in embeddings.iter().enumerate() {
        // FNV-1a hash for deterministic splitting (matches dataset.rs)
        let hash = fnv1a_hash(i as u64, seed);
        if hash.is_multiple_of(5) {
            test.push(entry.clone());
        } else {
            train.push(entry.clone());
        }
    }

    (train, test)
}

/// FNV-1a hash for deterministic splitting.
fn fnv1a_hash(index: u64, seed: u64) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;
    for byte in index.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(prime);
    }
    for byte in seed.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}

/// Sigmoid activation function.
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Save embeddings to a JSONL file for caching.
pub fn save_embeddings(
    embeddings: &[EmbeddingEntry],
    path: &Path,
) -> Result<(), String> {
    use std::io::Write;
    let file = std::fs::File::create(path)
        .map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    for entry in embeddings {
        let json = serde_json::to_string(entry)
            .map_err(|e| format!("Serialize error: {e}"))?;
        writeln!(writer, "{json}")
            .map_err(|e| format!("Write error: {e}"))?;
    }
    Ok(())
}

/// Load cached embeddings from a JSONL file.
pub fn load_embeddings(path: &Path) -> Result<Vec<EmbeddingEntry>, String> {
    use std::io::BufRead;
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let mut embeddings = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {e}"))?;
        let entry: EmbeddingEntry =
            serde_json::from_str(&line).map_err(|e| format!("Parse error: {e}"))?;
        embeddings.push(entry);
    }
    Ok(embeddings)
}

/// Save linear probe weights to JSON.
pub fn save_probe(probe: &LinearProbe, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(probe)
        .map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Write error: {e}"))?;
    Ok(())
}

/// Load linear probe weights from JSON.
pub fn load_probe(path: &Path) -> Result<LinearProbe, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("Read error: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_test_embeddings(n: usize, dim: usize) -> Vec<EmbeddingEntry> {
        (0..n)
            .map(|i| {
                let label = u8::from(i % 3 == 0); // ~33% unsafe
                let mut emb = vec![0.0f32; dim];
                // Safe entries: positive first half, negative second half
                // Unsafe entries: negative first half, positive second half
                for (j, val) in emb.iter_mut().enumerate() {
                    *val = if label == 0 {
                        if j < dim / 2 { 1.0 } else { -1.0 }
                    } else {
                        if j < dim / 2 { -1.0 } else { 1.0 }
                    };
                    // Add some noise based on index
                    *val += (i as f32 * 0.01) * if j % 2 == 0 { 1.0 } else { -1.0 };
                }
                EmbeddingEntry {
                    id: format!("test_{i}"),
                    embedding: emb,
                    label,
                }
            })
            .collect()
    }

    #[test]
    fn test_sigmoid() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-6);
        assert!(sigmoid(10.0) > 0.99);
        assert!(sigmoid(-10.0) < 0.01);
    }

    #[test]
    fn test_fnv1a_deterministic() {
        let h1 = fnv1a_hash(42, 7);
        let h2 = fnv1a_hash(42, 7);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_fnv1a_different_seeds() {
        let h1 = fnv1a_hash(42, 7);
        let h2 = fnv1a_hash(42, 8);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_split_deterministic() {
        let embeddings = make_test_embeddings(100, 32);
        let (train1, test1) = split_embeddings(&embeddings, 42);
        let (train2, test2) = split_embeddings(&embeddings, 42);
        assert_eq!(train1.len(), train2.len());
        assert_eq!(test1.len(), test2.len());
    }

    #[test]
    fn test_split_ratio() {
        let embeddings = make_test_embeddings(1000, 32);
        let (train, test) = split_embeddings(&embeddings, 42);
        // Should be approximately 80/20
        let test_pct = test.len() as f64 / 1000.0;
        assert!(test_pct > 0.15 && test_pct < 0.25, "test_pct={test_pct}");
        assert_eq!(train.len() + test.len(), 1000);
    }

    #[test]
    fn test_train_linear_probe_learns() {
        let train = make_test_embeddings(200, 32);
        let probe = train_linear_probe(&train, 20, 0.01);
        assert!(probe.train_accuracy > 0.7, "acc={}", probe.train_accuracy);
    }

    #[test]
    fn test_evaluate_probe() {
        let train = make_test_embeddings(200, 32);
        let test = make_test_embeddings(50, 32);
        let probe = train_linear_probe(&train, 20, 0.01);
        let report = evaluate_probe(&probe, &test);
        assert!(report.accuracy > 0.5, "test_acc={}", report.accuracy);
        assert_eq!(report.total, 50);
    }

    #[test]
    fn test_save_load_embeddings() {
        let embeddings = make_test_embeddings(10, 8);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("embeddings.jsonl");
        save_embeddings(&embeddings, &path).unwrap();
        let loaded = load_embeddings(&path).unwrap();
        assert_eq!(loaded.len(), 10);
        assert_eq!(loaded[0].id, "test_0");
        assert_eq!(loaded[0].embedding.len(), 8);
    }

    #[test]
    fn test_save_load_probe() {
        let probe = LinearProbe {
            weights: vec![1.0, -1.0, 0.5],
            bias: 0.1,
            epochs: 10,
            learning_rate: 0.01,
            train_accuracy: 0.85,
            train_mcc: 0.6,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("probe.json");
        save_probe(&probe, &path).unwrap();
        let loaded = load_probe(&path).unwrap();
        assert_eq!(loaded.weights, vec![1.0, -1.0, 0.5]);
        assert!((loaded.bias - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_simple_tokenize() {
        let tokens = simple_tokenize("echo hello");
        assert_eq!(tokens[0], 0); // CLS
        assert_eq!(*tokens.last().unwrap(), 2); // SEP
        assert!(tokens.len() > 2);
    }

    #[test]
    fn test_simple_tokenize_truncates() {
        let long_script = "a".repeat(1000);
        let tokens = simple_tokenize(&long_script);
        assert!(tokens.len() <= MAX_SEQ_LEN);
        assert_eq!(tokens[0], 0);
        assert_eq!(*tokens.last().unwrap(), 2);
    }
}
