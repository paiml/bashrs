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

/// Trained MLP probe weights (Level 0.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpProbeWeights {
    /// First layer weights [hidden_size × mlp_hidden] flattened row-major
    pub w1: Vec<f32>,
    /// First layer bias [mlp_hidden]
    pub b1: Vec<f32>,
    /// Second layer weights [mlp_hidden × num_classes] flattened row-major
    pub w2: Vec<f32>,
    /// Second layer bias [num_classes]
    pub b2: Vec<f32>,
    pub hidden_size: usize,
    pub mlp_hidden: usize,
    pub num_classes: usize,
    /// Training metadata
    pub epochs: usize,
    pub learning_rate: f32,
    pub train_accuracy: f64,
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

    // Try to load BPE tokenizer from model directory
    let bpe = CodeBertTokenizer::from_model_dir(model_dir);
    if bpe.is_some() {
        eprintln!("  Using RoBERTa BPE tokenizer (vocab.json + merges.txt)");
    } else {
        eprintln!("  Using byte-level fallback tokenizer (vocab files not found)");
    }

    let total = entries.len();
    let mut embeddings = Vec::with_capacity(total);
    let mut skipped = 0;

    for (i, entry) in entries.iter().enumerate() {
        if let Some(pf) = progress_fn {
            pf(i, total);
        }

        let token_ids = tokenize_for_codebert(&entry.input, bpe.as_ref());
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

/// Extract [CLS] embeddings with streaming writes to disk.
///
/// Writes each embedding to the output JSONL file as it's computed,
/// avoiding memory buildup and enabling progress monitoring.
/// The progress callback receives (index, total, elapsed_ms).
#[cfg(feature = "ml")]
pub fn extract_embeddings_streaming(
    model_dir: &Path,
    entries: &[ClassificationRow],
    output: &Path,
    progress_fn: &dyn Fn(usize, usize, u64),
) -> Result<ExtractionReport, String> {
    use std::io::Write;

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

    // Try to load BPE tokenizer from model directory
    let bpe = CodeBertTokenizer::from_model_dir(model_dir);
    if bpe.is_some() {
        eprintln!("  Using RoBERTa BPE tokenizer (vocab.json + merges.txt)");
    } else {
        eprintln!("  Using byte-level fallback tokenizer (vocab files not found)");
    }

    let total = entries.len();
    let mut extracted = 0usize;
    let mut skipped = 0usize;
    let start = std::time::Instant::now();

    let file = std::fs::File::create(output)
        .map_err(|e| format!("Cannot create {}: {e}", output.display()))?;
    let mut writer = std::io::BufWriter::new(file);

    for (i, entry) in entries.iter().enumerate() {
        if i % 10 == 0 {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            progress_fn(i, total, elapsed_ms);
        }

        let token_ids = tokenize_for_codebert(&entry.input, bpe.as_ref());
        if token_ids.len() < 3 {
            skipped += 1;
            continue;
        }

        let cls = model.cls_embedding(&token_ids);
        let data = cls.data();
        let slice = data.as_slice().ok_or("CLS embedding not contiguous")?;

        let emb = EmbeddingEntry {
            id: format!("entry_{i}"),
            embedding: slice.to_vec(),
            label: entry.label,
        };

        let json = serde_json::to_string(&emb).map_err(|e| format!("Serialize error: {e}"))?;
        writeln!(writer, "{json}").map_err(|e| format!("Write error: {e}"))?;

        extracted += 1;
    }

    writer.flush().map_err(|e| format!("Flush error: {e}"))?;

    Ok(ExtractionReport {
        total_entries: total,
        extracted,
        skipped,
        hidden_size: config.hidden_size,
    })
}

/// BPE tokenizer wrapper for CodeBERT (RoBERTa tokenizer).
///
/// Loaded once from vocab.json + merges.txt, reused for all entries.
#[cfg(feature = "ml")]
struct CodeBertTokenizer {
    bpe: aprender::text::bpe::BpeTokenizer,
}

#[cfg(feature = "ml")]
impl CodeBertTokenizer {
    /// Try to load from model directory. Returns None if files missing.
    fn from_model_dir(model_dir: &Path) -> Option<Self> {
        let vocab = model_dir.join("vocab.json");
        let merges = model_dir.join("merges.txt");
        if !vocab.exists() || !merges.exists() {
            return None;
        }
        let bpe = aprender::text::bpe::BpeTokenizer::from_vocab_merges(&vocab, &merges).ok()?;
        Some(Self { bpe })
    }

    /// Tokenize with CLS/SEP wrapping and truncation.
    fn tokenize(&self, script: &str) -> Vec<u32> {
        let mut ids = vec![0u32]; // CLS (RoBERTa: <s>=0)
        let encoded = self.bpe.encode(script);
        let max_body = MAX_SEQ_LEN - 2;
        ids.extend(encoded.iter().take(max_body));
        ids.push(2); // SEP (RoBERTa: </s>=2)
        ids
    }
}

/// Tokenize a shell script for CodeBERT input.
///
/// Uses the provided BPE tokenizer if available, falls back to simple byte-level.
#[cfg(feature = "ml")]
fn tokenize_for_codebert(script: &str, bpe: Option<&CodeBertTokenizer>) -> Vec<u32> {
    match bpe {
        Some(tok) => tok.tokenize(script),
        None => simple_tokenize(script),
    }
}

/// Train a linear probe on pre-extracted embeddings.
///
/// Delegates to `aprender::classification::LogisticRegression` with:
/// - `ClassWeight::Balanced` for sqrt-inverse class weighting (imbalanced data)
/// - L2 regularization (weight decay = 1e-4) to prevent overfitting
///
/// Falls back to a simple hand-rolled SGD when the `ml` feature is disabled.
pub fn train_linear_probe(
    train: &[EmbeddingEntry],
    epochs: usize,
    learning_rate: f32,
) -> LinearProbe {
    train_linear_probe_inner(train, epochs, learning_rate)
}

/// Train a linear probe using online (per-sample) SGD with class weighting and L2.
///
/// Uses `aprender::classification::ClassWeight::Balanced` for sqrt-inverse weighting
/// (upweights minority class). Online SGD is used instead of batch GD because batch
/// averaging dilutes minority signal on imbalanced data (aprender#428).
fn train_linear_probe_inner(
    train: &[EmbeddingEntry],
    epochs: usize,
    learning_rate: f32,
) -> LinearProbe {
    let h = if train.is_empty() {
        CODEBERT_HIDDEN_SIZE
    } else {
        train[0].embedding.len()
    };

    let mut weights = vec![0.0f32; h];
    let mut bias = 0.0f32;

    // Compute class weights: sqrt-inverse (matches aprender ClassWeight::Balanced)
    // w_k = sqrt(n / (2 * n_k))
    let n = train.len() as f32;
    let n_unsafe = train.iter().filter(|e| e.label == 1).count() as f32;
    let n_safe = n - n_unsafe;
    let (w_safe, w_unsafe) = if n_unsafe > 0.0 && n_safe > 0.0 {
        ((n / (2.0 * n_safe)).sqrt(), (n / (2.0 * n_unsafe)).sqrt())
    } else {
        (1.0, 1.0)
    };

    // L2 regularization strength (weight decay)
    let weight_decay: f32 = 1e-4;

    for _epoch in 0..epochs {
        let mut total_loss = 0.0f64;
        for entry in train {
            let logit: f32 = weights
                .iter()
                .zip(entry.embedding.iter())
                .map(|(w, x)| w * x)
                .sum::<f32>()
                + bias;
            let prob = sigmoid(logit);
            let target = entry.label as f32;
            let class_w = if entry.label == 1 { w_unsafe } else { w_safe };

            // Class-weighted gradient
            let grad = class_w * (prob - target);
            total_loss += f64::from(class_w)
                * f64::from(
                    -target * logit.clamp(-100.0, 100.0) + (1.0 + (-logit).exp()).ln().max(0.0),
                );

            // Online SGD with L2 weight decay
            for (w, x) in weights.iter_mut().zip(entry.embedding.iter()) {
                *w -= learning_rate * (grad * x + weight_decay * *w);
            }
            bias -= learning_rate * grad;
        }

        let avg_loss = if train.is_empty() {
            0.0
        } else {
            total_loss / train.len() as f64
        };
        if (_epoch + 1) % 10 == 0 || _epoch == 0 {
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

include!("classifier_evaluate.rs");
