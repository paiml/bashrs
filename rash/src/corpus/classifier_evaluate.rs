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
    let (all_embeddings, report) = extract_embeddings(
        model_dir,
        entries,
        Some(&|i, total| {
            if i % 100 == 0 {
                eprintln!("  {i}/{total} ({:.1}%)", 100.0 * i as f64 / total as f64);
            }
        }),
    )?;
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
///
/// Uses FNV-1a hash on global index for deterministic, reproducible splits.
/// Approximately 80/20 train/test ratio (hash % 5 == 0 → test).
pub fn split_embeddings(
    embeddings: &[EmbeddingEntry],
    seed: u64,
) -> (Vec<EmbeddingEntry>, Vec<EmbeddingEntry>) {
    let mut train = Vec::new();
    let mut test = Vec::new();

    for (i, entry) in embeddings.iter().enumerate() {
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

/// Classify a single script using CodeBERT + linear probe.
///
/// Loads the model, tokenizes, extracts [CLS], applies probe weights.
/// Returns (predicted_label, confidence) or None on failure.
#[cfg(feature = "ml")]
pub fn classify_with_probe(
    source: &str,
    probe: &LinearProbe,
    model_dir: &Path,
) -> Option<(u8, f64)> {
    let config = TransformerConfig::codebert();
    let model = EncoderModel::from_safetensors(&config, model_dir).ok()?;

    // Try BPE tokenizer, fall back to byte-level
    let bpe = CodeBertTokenizer::from_model_dir(model_dir);
    let token_ids = tokenize_for_codebert(source, bpe.as_ref());

    if token_ids.len() < 3 {
        return None;
    }

    let cls = model.cls_embedding(&token_ids);
    let data = cls.data();
    let slice = data.as_slice()?;

    // Apply probe: logit = w . embedding + b
    let logit: f32 = probe
        .weights
        .iter()
        .zip(slice.iter())
        .map(|(w, x)| w * x)
        .sum::<f32>()
        + probe.bias;

    let prob = sigmoid(logit);
    let label = u8::from(prob > 0.5);
    let confidence = if label == 1 {
        f64::from(prob)
    } else {
        f64::from(1.0 - prob)
    };

    Some((label, confidence))
}

/// Save embeddings to a JSONL file for caching.
pub fn save_embeddings(embeddings: &[EmbeddingEntry], path: &Path) -> Result<(), String> {
    use std::io::Write;
    let file = std::fs::File::create(path)
        .map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    for entry in embeddings {
        let json = serde_json::to_string(entry).map_err(|e| format!("Serialize error: {e}"))?;
        writeln!(writer, "{json}").map_err(|e| format!("Write error: {e}"))?;
    }
    Ok(())
}

/// Load cached embeddings from a JSONL file.
pub fn load_embeddings(path: &Path) -> Result<Vec<EmbeddingEntry>, String> {
    use std::io::BufRead;
    let file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
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
    let json = serde_json::to_string_pretty(probe).map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Write error: {e}"))?;
    Ok(())
}

/// Load linear probe weights from JSON.
pub fn load_probe(path: &Path) -> Result<LinearProbe, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read error: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))
}

/// Load MLP probe weights from JSON.
pub fn load_mlp_probe(path: &Path) -> Result<MlpProbeWeights, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read error: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))
}

/// Classify using MLP probe: embedding → Linear → ReLU → Linear → softmax.
#[cfg(any(feature = "ml", test))]
fn mlp_forward(weights: &MlpProbeWeights, embedding: &[f32]) -> (u8, f64) {
    // Layer 1: hidden = ReLU(W1 @ embedding + b1)
    let mut hidden = vec![0.0f32; weights.mlp_hidden];
    for i in 0..weights.mlp_hidden {
        let mut sum = weights.b1[i];
        for j in 0..weights.hidden_size {
            sum += weights.w1[i * weights.hidden_size + j] * embedding[j];
        }
        hidden[i] = sum.max(0.0); // ReLU
    }

    // Layer 2: logits = W2 @ hidden + b2
    let mut logits = vec![0.0f32; weights.num_classes];
    for i in 0..weights.num_classes {
        let mut sum = weights.b2[i];
        for j in 0..weights.mlp_hidden {
            sum += weights.w2[i * weights.mlp_hidden + j] * hidden[j];
        }
        logits[i] = sum;
    }

    // Binary classification: use sigmoid on logit difference
    let prob_unsafe =
        sigmoid(logits.get(1).copied().unwrap_or(0.0) - logits.first().copied().unwrap_or(0.0));
    let label = u8::from(prob_unsafe > 0.5);
    let confidence = if label == 1 {
        f64::from(prob_unsafe)
    } else {
        f64::from(1.0 - prob_unsafe)
    };
    (label, confidence)
}

/// Classify a single script using CodeBERT + MLP probe.
#[cfg(feature = "ml")]
pub fn classify_with_mlp_probe(
    source: &str,
    weights: &MlpProbeWeights,
    model_dir: &Path,
) -> Option<(u8, f64)> {
    let config = TransformerConfig::codebert();
    let model = EncoderModel::from_safetensors(&config, model_dir).ok()?;

    let bpe = CodeBertTokenizer::from_model_dir(model_dir);
    let token_ids = tokenize_for_codebert(source, bpe.as_ref());

    if token_ids.len() < 3 {
        return None;
    }

    let cls = model.cls_embedding(&token_ids);
    let data = cls.data();
    let slice = data.as_slice()?;

    Some(mlp_forward(weights, slice))
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
                        if j < dim / 2 {
                            1.0
                        } else {
                            -1.0
                        }
                    } else {
                        if j < dim / 2 {
                            -1.0
                        } else {
                            1.0
                        }
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
    fn test_mlp_forward_basic() {
        // Simple 2-dim input, 2-hidden, 2-class MLP
        let weights = MlpProbeWeights {
            w1: vec![1.0, 0.0, 0.0, 1.0], // identity-like
            b1: vec![0.0, 0.0],
            w2: vec![1.0, -1.0, -1.0, 1.0], // class 0 prefers dim0, class 1 prefers dim1
            b2: vec![0.0, 0.0],
            hidden_size: 2,
            mlp_hidden: 2,
            num_classes: 2,
            epochs: 1,
            learning_rate: 0.01,
            train_accuracy: 1.0,
        };
        // Input [1.0, 0.0] → hidden [1.0, 0.0] → logits [1.0, -1.0] → class 0 (safe)
        let (label, conf) = mlp_forward(&weights, &[1.0, 0.0]);
        assert_eq!(label, 0);
        assert!(conf > 0.8);

        // Input [0.0, 1.0] → hidden [0.0, 1.0] → logits [-1.0, 1.0] → class 1 (unsafe)
        let (label, conf) = mlp_forward(&weights, &[0.0, 1.0]);
        assert_eq!(label, 1);
        assert!(conf > 0.8);
    }

    #[test]
    fn test_mlp_forward_relu() {
        // Negative inputs should be zeroed by ReLU
        let weights = MlpProbeWeights {
            w1: vec![1.0, 0.0, 0.0, 1.0],
            b1: vec![0.0, 0.0],
            w2: vec![1.0, 0.0, 0.0, 1.0],
            b2: vec![0.0, 0.0],
            hidden_size: 2,
            mlp_hidden: 2,
            num_classes: 2,
            epochs: 1,
            learning_rate: 0.01,
            train_accuracy: 1.0,
        };
        // Input [-5.0, -5.0] → hidden [0.0, 0.0] (ReLU) → logits [0.0, 0.0] → sigmoid(0)=0.5
        let (_, conf) = mlp_forward(&weights, &[-5.0, -5.0]);
        assert!((conf - 0.5).abs() < 0.01); // 50% confidence = no signal
    }

    #[test]
    fn test_mlp_probe_weights_roundtrip() {
        let weights = MlpProbeWeights {
            w1: vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6],
            b1: vec![0.01, -0.02],
            w2: vec![0.7, -0.8, 0.9, -1.0],
            b2: vec![0.03, -0.04],
            hidden_size: 3,
            mlp_hidden: 2,
            num_classes: 2,
            epochs: 50,
            learning_rate: 0.0001,
            train_accuracy: 0.95,
        };
        let json = serde_json::to_string_pretty(&weights).unwrap();
        let loaded: MlpProbeWeights = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.w1, weights.w1);
        assert_eq!(loaded.b1, weights.b1);
        assert_eq!(loaded.w2, weights.w2);
        assert_eq!(loaded.b2, weights.b2);
        assert_eq!(loaded.hidden_size, 3);
        assert_eq!(loaded.mlp_hidden, 2);
        assert_eq!(loaded.num_classes, 2);
        assert_eq!(loaded.epochs, 50);
        assert!((loaded.train_accuracy - 0.95).abs() < 1e-6);
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
