#![allow(clippy::needless_range_loop)] // Linear algebra code uses indexed loops for clarity
//! Pure-Rust CodeBERT encoder for WASM inference (WASM-004).
//!
//! Implements the RoBERTa/CodeBERT transformer encoder using only f32 operations.
//! No SIMD, no rayon, no external ML libraries — compiles to wasm32-unknown-unknown.
//!
//! Architecture: token_ids → Embedding + Position → LayerNorm → 12×EncoderBlock → [CLS]
//!
//! Weight format: int8 SafeTensors with per-tensor `__scale` factors (from WASM-002).

use safetensors::SafeTensors;

// === Constants (CodeBERT/RoBERTa) ===

const HIDDEN_SIZE: usize = 768;
const NUM_HEADS: usize = 12;
const HEAD_DIM: usize = HIDDEN_SIZE / NUM_HEADS; // 64
const INTERMEDIATE_SIZE: usize = 3072;
const NUM_LAYERS: usize = 12;
const VOCAB_SIZE: usize = 50265;
const MAX_POSITION: usize = 514;
const LAYER_NORM_EPS: f32 = 1e-5;

// === Math primitives ===

/// GELU activation (tanh approximation matching PyTorch).
fn gelu(x: f32) -> f32 {
    let coeff = 0.044715_f32;
    let sqrt_2_over_pi = 0.797_884_6_f32;
    let inner = sqrt_2_over_pi * (x + coeff * x * x * x);
    0.5 * x * (1.0 + inner.tanh())
}

/// In-place softmax over a slice.
fn softmax(x: &mut [f32]) {
    let max = x.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0_f32;
    for v in x.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    if sum > 0.0 {
        for v in x.iter_mut() {
            *v /= sum;
        }
    }
}

/// Layer normalization: x = gamma * (x - mean) / sqrt(var + eps) + beta.
fn layer_norm(x: &mut [f32], gamma: &[f32], beta: &[f32]) {
    let n = x.len();
    let mean = x.iter().sum::<f32>() / n as f32;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n as f32;
    let inv_std = 1.0 / (var + LAYER_NORM_EPS).sqrt();
    for i in 0..n {
        x[i] = gamma[i] * (x[i] - mean) * inv_std + beta[i];
    }
}

/// Dequantize int8 data with per-tensor scale: i8 * scale → f32.
fn dequantize_i8(data: &[u8], scale: f32) -> Vec<f32> {
    data.iter().map(|&b| (b as i8) as f32 * scale).collect()
}

// === Weight structures ===

struct Linear {
    weight: Vec<f32>, // [out_features, in_features] row-major
    bias: Vec<f32>,   // [out_features]
    out_features: usize,
    in_features: usize,
}

impl Linear {
    /// Forward: output[i] = bias[i] + sum_j(weight[i,j] * input[j]).
    #[allow(dead_code)] // Used in tests and as reference implementation
    fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0_f32; self.out_features];
        for i in 0..self.out_features {
            let mut sum = self.bias[i];
            let row = &self.weight[i * self.in_features..(i + 1) * self.in_features];
            for j in 0..self.in_features {
                sum += row[j] * input[j];
            }
            output[i] = sum;
        }
        output
    }

    /// Batched forward: input is [batch, in_features], output is [batch, out_features].
    fn forward_batched(&self, input: &[f32], batch: usize) -> Vec<f32> {
        let mut output = vec![0.0_f32; batch * self.out_features];
        for b in 0..batch {
            let inp = &input[b * self.in_features..(b + 1) * self.in_features];
            let out = &mut output[b * self.out_features..(b + 1) * self.out_features];
            for i in 0..self.out_features {
                let mut sum = self.bias[i];
                let row = &self.weight[i * self.in_features..(i + 1) * self.in_features];
                for j in 0..self.in_features {
                    sum += row[j] * inp[j];
                }
                out[i] = sum;
            }
        }
        output
    }
}

struct LayerNormParams {
    gamma: Vec<f32>,
    beta: Vec<f32>,
}

struct EncoderLayer {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    attn_ln: LayerNormParams,
    ffn_up: Linear,
    ffn_down: Linear,
    ffn_ln: LayerNormParams,
}

/// Scaled dot-product multi-head attention (bidirectional, no causal mask).
fn multi_head_attention(
    q: &[f32],
    k: &[f32],
    v: &[f32],
    seq_len: usize,
) -> Vec<f32> {
    let scale = 1.0 / (HEAD_DIM as f32).sqrt();
    let mut output = vec![0.0_f32; seq_len * HIDDEN_SIZE];

    for h in 0..NUM_HEADS {
        let mut scores = vec![0.0_f32; seq_len * seq_len];
        for i in 0..seq_len {
            for j in 0..seq_len {
                let mut dot = 0.0_f32;
                for d in 0..HEAD_DIM {
                    dot += q[i * HIDDEN_SIZE + h * HEAD_DIM + d]
                        * k[j * HIDDEN_SIZE + h * HEAD_DIM + d];
                }
                scores[i * seq_len + j] = dot * scale;
            }
        }

        for i in 0..seq_len {
            softmax(&mut scores[i * seq_len..(i + 1) * seq_len]);
        }

        for i in 0..seq_len {
            for d in 0..HEAD_DIM {
                let mut sum = 0.0_f32;
                for j in 0..seq_len {
                    sum += scores[i * seq_len + j] * v[j * HIDDEN_SIZE + h * HEAD_DIM + d];
                }
                output[i * HIDDEN_SIZE + h * HEAD_DIM + d] = sum;
            }
        }
    }

    output
}

/// Add residual connection and apply LayerNorm per position.
fn residual_and_layer_norm(
    x: &[f32],
    residual: &[f32],
    ln: &LayerNormParams,
    seq_len: usize,
) -> Vec<f32> {
    let mut output = vec![0.0_f32; seq_len * HIDDEN_SIZE];
    for i in 0..seq_len {
        let offset = i * HIDDEN_SIZE;
        for j in 0..HIDDEN_SIZE {
            output[offset + j] = x[offset + j] + residual[offset + j];
        }
        layer_norm(
            &mut output[offset..offset + HIDDEN_SIZE],
            &ln.gamma,
            &ln.beta,
        );
    }
    output
}

impl EncoderLayer {
    /// Forward pass (post-norm RoBERTa architecture).
    ///
    /// h = LayerNorm(x + Attention(x))
    /// out = LayerNorm(h + FFN(h))
    fn forward(&self, x: &[f32], seq_len: usize) -> Vec<f32> {
        // Self-attention
        let q = self.q_proj.forward_batched(x, seq_len);
        let k = self.k_proj.forward_batched(x, seq_len);
        let v = self.v_proj.forward_batched(x, seq_len);
        let attn_output = multi_head_attention(&q, &k, &v, seq_len);
        let projected = self.o_proj.forward_batched(&attn_output, seq_len);
        let hidden = residual_and_layer_norm(x, &projected, &self.attn_ln, seq_len);

        // Feed-forward network
        let up = self.ffn_up.forward_batched(&hidden, seq_len);
        let activated: Vec<f32> = up.iter().map(|&x| gelu(x)).collect();
        let down = self.ffn_down.forward_batched(&activated, seq_len);
        residual_and_layer_norm(&hidden, &down, &self.ffn_ln, seq_len)
    }
}

/// Complete CodeBERT encoder for WASM inference.
pub struct WasmEncoder {
    word_embeddings: Vec<f32>,   // [VOCAB_SIZE, HIDDEN_SIZE]
    position_embeddings: Vec<f32>, // [MAX_POSITION, HIDDEN_SIZE]
    token_type_embeddings: Vec<f32>, // [1, HIDDEN_SIZE]
    embed_ln: LayerNormParams,
    layers: Vec<EncoderLayer>,
}

impl WasmEncoder {
    /// Load encoder from int8 SafeTensors bytes.
    ///
    /// Dequantizes all weights to f32 during loading.
    pub fn from_safetensors_bytes(data: &[u8]) -> Result<Self, String> {
        let tensors = SafeTensors::deserialize(data)
            .map_err(|e| format!("SafeTensors parse error: {e}"))?;

        let load = |name: &str| -> Result<Vec<f32>, String> {
            let tensor = tensors
                .tensor(name)
                .map_err(|e| format!("Missing tensor {name}: {e}"))?;
            let scale_name = format!("{name}.__scale");
            let scale_tensor = tensors
                .tensor(&scale_name)
                .map_err(|e| format!("Missing scale {scale_name}: {e}"))?;
            let scale_bytes = scale_tensor.data();
            let scale = f32::from_le_bytes([
                scale_bytes[0],
                scale_bytes[1],
                scale_bytes[2],
                scale_bytes[3],
            ]);
            Ok(dequantize_i8(tensor.data(), scale))
        };

        let word_embeddings = load("embeddings.word_embeddings.weight")?;
        let position_embeddings = load("embeddings.position_embeddings.weight")?;
        let token_type_embeddings = load("embeddings.token_type_embeddings.weight")?;
        let embed_ln = LayerNormParams {
            gamma: load("embeddings.LayerNorm.weight")?,
            beta: load("embeddings.LayerNorm.bias")?,
        };

        let mut layers = Vec::with_capacity(NUM_LAYERS);
        for i in 0..NUM_LAYERS {
            let p = format!("encoder.layer.{i}");
            let layer = EncoderLayer {
                q_proj: Linear {
                    weight: load(&format!("{p}.attention.self.query.weight"))?,
                    bias: load(&format!("{p}.attention.self.query.bias"))?,
                    out_features: HIDDEN_SIZE,
                    in_features: HIDDEN_SIZE,
                },
                k_proj: Linear {
                    weight: load(&format!("{p}.attention.self.key.weight"))?,
                    bias: load(&format!("{p}.attention.self.key.bias"))?,
                    out_features: HIDDEN_SIZE,
                    in_features: HIDDEN_SIZE,
                },
                v_proj: Linear {
                    weight: load(&format!("{p}.attention.self.value.weight"))?,
                    bias: load(&format!("{p}.attention.self.value.bias"))?,
                    out_features: HIDDEN_SIZE,
                    in_features: HIDDEN_SIZE,
                },
                o_proj: Linear {
                    weight: load(&format!("{p}.attention.output.dense.weight"))?,
                    bias: load(&format!("{p}.attention.output.dense.bias"))?,
                    out_features: HIDDEN_SIZE,
                    in_features: HIDDEN_SIZE,
                },
                attn_ln: LayerNormParams {
                    gamma: load(&format!("{p}.attention.output.LayerNorm.weight"))?,
                    beta: load(&format!("{p}.attention.output.LayerNorm.bias"))?,
                },
                ffn_up: Linear {
                    weight: load(&format!("{p}.intermediate.dense.weight"))?,
                    bias: load(&format!("{p}.intermediate.dense.bias"))?,
                    out_features: INTERMEDIATE_SIZE,
                    in_features: HIDDEN_SIZE,
                },
                ffn_down: Linear {
                    weight: load(&format!("{p}.output.dense.weight"))?,
                    bias: load(&format!("{p}.output.dense.bias"))?,
                    out_features: HIDDEN_SIZE,
                    in_features: INTERMEDIATE_SIZE,
                },
                ffn_ln: LayerNormParams {
                    gamma: load(&format!("{p}.output.LayerNorm.weight"))?,
                    beta: load(&format!("{p}.output.LayerNorm.bias"))?,
                },
            };
            layers.push(layer);
        }

        Ok(Self {
            word_embeddings,
            position_embeddings,
            token_type_embeddings,
            embed_ln,
            layers,
        })
    }

    /// Forward pass: token_ids → hidden states [seq_len, HIDDEN_SIZE].
    pub fn forward(&self, token_ids: &[u32]) -> Vec<f32> {
        let seq_len = token_ids.len();
        let mut hidden = vec![0.0_f32; seq_len * HIDDEN_SIZE];

        // Combine embeddings: word + position + token_type
        for (i, &tok) in token_ids.iter().enumerate() {
            let tok_idx = (tok as usize).min(VOCAB_SIZE - 1);
            let pos_idx = i.min(MAX_POSITION - 1);
            let offset = i * HIDDEN_SIZE;
            for j in 0..HIDDEN_SIZE {
                hidden[offset + j] = self.word_embeddings[tok_idx * HIDDEN_SIZE + j]
                    + self.position_embeddings[pos_idx * HIDDEN_SIZE + j]
                    + self.token_type_embeddings[j]; // type 0 for all tokens
            }
        }

        // Embedding LayerNorm
        for i in 0..seq_len {
            let offset = i * HIDDEN_SIZE;
            layer_norm(
                &mut hidden[offset..offset + HIDDEN_SIZE],
                &self.embed_ln.gamma,
                &self.embed_ln.beta,
            );
        }

        // Encoder layers
        for layer in &self.layers {
            hidden = layer.forward(&hidden, seq_len);
        }

        hidden
    }

    /// Extract [CLS] embedding (position 0), the summary representation for classification.
    pub fn cls_embedding(&self, token_ids: &[u32]) -> Vec<f32> {
        let hidden = self.forward(token_ids);
        hidden[..HIDDEN_SIZE].to_vec()
    }
}

/// MLP probe weights for binary classification (safe/unsafe).
pub struct MlpProbe {
    w1: Vec<f32>,    // [hidden, HIDDEN_SIZE]
    b1: Vec<f32>,    // [hidden]
    w2: Vec<f32>,    // [2, hidden]
    b2: Vec<f32>,    // [2]
    hidden: usize,
}

impl MlpProbe {
    /// Load MLP probe from JSON bytes.
    ///
    /// Expected format: `{"w1": [...], "b1": [...], "w2": [...], "b2": [...], "hidden_size": 768, "mlp_hidden": 32, "num_classes": 2}`
    pub fn from_json(data: &[u8]) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_slice(data).map_err(|e| format!("Probe JSON parse: {e}"))?;

        let hidden = v["mlp_hidden"]
            .as_u64()
            .ok_or("missing mlp_hidden")? as usize;

        let w1: Vec<f32> = v["w1"]
            .as_array()
            .ok_or("missing w1")?
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();
        let b1: Vec<f32> = v["b1"]
            .as_array()
            .ok_or("missing b1")?
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();
        let w2: Vec<f32> = v["w2"]
            .as_array()
            .ok_or("missing w2")?
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();
        let b2: Vec<f32> = v["b2"]
            .as_array()
            .ok_or("missing b2")?
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();

        Ok(Self { w1, b1, w2, b2, hidden })
    }

    /// Forward pass: embedding → Linear → ReLU → Linear → sigmoid → (label, confidence).
    pub fn classify(&self, embedding: &[f32]) -> (u8, f64) {
        let input_dim = embedding.len();

        // Layer 1: hidden = ReLU(W1 @ embedding + b1)
        let mut hidden = vec![0.0_f32; self.hidden];
        for i in 0..self.hidden {
            let mut sum = self.b1[i];
            for j in 0..input_dim {
                sum += self.w1[i * input_dim + j] * embedding[j];
            }
            hidden[i] = sum.max(0.0); // ReLU
        }

        // Layer 2: logits = W2 @ hidden + b2
        let mut logits = [0.0_f32; 2];
        for i in 0..2 {
            let mut sum = self.b2[i];
            for j in 0..self.hidden {
                sum += self.w2[i * self.hidden + j] * hidden[j];
            }
            logits[i] = sum;
        }

        // Sigmoid on logit difference
        let prob_unsafe = sigmoid(logits[1] - logits[0]);
        let label = u8::from(prob_unsafe > 0.5);
        let confidence = if label == 1 {
            f64::from(prob_unsafe)
        } else {
            f64::from(1.0 - prob_unsafe)
        };
        (label, confidence)
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Simple whitespace tokenizer for WASM (no BPE vocab files needed).
///
/// Converts each byte to its token ID. This matches the fallback tokenizer
/// used in the CLI when BPE files are unavailable.
pub fn simple_tokenize(script: &str, max_len: usize) -> Vec<u32> {
    let mut tokens = Vec::with_capacity(max_len.min(512));
    tokens.push(0); // <s> BOS token

    // Byte-level tokenization (matching RoBERTa byte encoder)
    for &b in script.as_bytes() {
        if tokens.len() >= max_len - 1 {
            break;
        }
        // RoBERTa byte tokens start at index 4 (0=<s>, 1=<pad>, 2=</s>, 3=<unk>)
        tokens.push(u32::from(b) + 4);
    }

    tokens.push(2); // </s> EOS token
    tokens
}

/// Classification result from CodeBERT + MLP probe.
pub struct ClassificationResult {
    pub label: u8,       // 0=safe, 1=unsafe
    pub confidence: f64, // 0.0-1.0
}

/// Run full CodeBERT classification pipeline.
///
/// 1. Tokenize script (simple byte-level)
/// 2. Run encoder forward pass → [CLS] embedding
/// 3. Apply MLP probe → (label, confidence)
pub fn classify_with_codebert(
    encoder: &WasmEncoder,
    probe: &MlpProbe,
    script: &str,
) -> ClassificationResult {
    let tokens = simple_tokenize(script, 128);
    let cls = encoder.cls_embedding(&tokens);
    let (label, confidence) = probe.classify(&cls);
    ClassificationResult { label, confidence }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_gelu_zero() {
        assert!((gelu(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_gelu_positive() {
        let val = gelu(1.0);
        assert!(val > 0.8 && val < 1.0, "gelu(1.0) = {val}");
    }

    #[test]
    fn test_softmax_uniform() {
        let mut x = vec![1.0, 1.0, 1.0];
        softmax(&mut x);
        for v in &x {
            assert!((v - 1.0 / 3.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let mut x = vec![1.0, 2.0, 3.0];
        softmax(&mut x);
        let sum: f32 = x.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_layer_norm() {
        let mut x = vec![1.0, 2.0, 3.0, 4.0];
        let gamma = vec![1.0; 4];
        let beta = vec![0.0; 4];
        layer_norm(&mut x, &gamma, &beta);
        // Mean should be ~0, variance ~1
        let mean: f32 = x.iter().sum::<f32>() / 4.0;
        assert!(mean.abs() < 1e-5, "mean = {mean}");
    }

    #[test]
    fn test_sigmoid_bounds() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-6);
        assert!(sigmoid(10.0) > 0.99);
        assert!(sigmoid(-10.0) < 0.01);
    }

    #[test]
    fn test_linear_forward() {
        let linear = Linear {
            weight: vec![1.0, 0.0, 0.0, 1.0], // 2x2 identity
            bias: vec![0.5, -0.5],
            out_features: 2,
            in_features: 2,
        };
        let output = linear.forward(&[3.0, 4.0]);
        assert!((output[0] - 3.5).abs() < 1e-5);
        assert!((output[1] - 3.5).abs() < 1e-5);
    }

    #[test]
    fn test_simple_tokenize() {
        let tokens = simple_tokenize("echo hi", 128);
        assert_eq!(tokens[0], 0); // BOS
        assert_eq!(*tokens.last().unwrap(), 2); // EOS
        assert_eq!(tokens.len(), 9); // BOS + 7 bytes + EOS
    }

    #[test]
    fn test_simple_tokenize_truncation() {
        let long = "x".repeat(1000);
        let tokens = simple_tokenize(&long, 128);
        assert_eq!(tokens.len(), 128); // truncated to max_len
        assert_eq!(tokens[0], 0);
        assert_eq!(*tokens.last().unwrap(), 2);
    }

    #[test]
    fn test_dequantize_i8() {
        let data: Vec<u8> = vec![127, 0, 128]; // i8: 127, 0, -128
        let result = dequantize_i8(&data, 0.01);
        assert!((result[0] - 1.27).abs() < 1e-5);
        assert!((result[1] - 0.0).abs() < 1e-5);
        assert!((result[2] - (-1.28)).abs() < 1e-5);
    }

    #[test]
    fn test_mlp_probe_classify() {
        // Tiny probe: 2-dim input → 2-dim hidden → 2-dim output
        let probe = MlpProbe {
            w1: vec![1.0, 0.0, 0.0, 1.0], // identity
            b1: vec![0.0, 0.0],
            w2: vec![1.0, 0.0, 0.0, 1.0], // identity
            b2: vec![0.0, 0.0],
            hidden: 2,
        };
        let embedding = vec![0.5, 1.5]; // class 1 logit > class 0
        let (label, confidence) = probe.classify(&embedding);
        assert_eq!(label, 1); // unsafe (logit[1] > logit[0])
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_encoder_from_safetensors_invalid() {
        let result = WasmEncoder::from_safetensors_bytes(b"invalid data");
        assert!(result.is_err());
    }

    #[test]
    fn test_encoder_from_safetensors_real() {
        // Only runs if the int8 model exists
        let path = std::path::Path::new("/tmp/codebert-base/model_int8.safetensors");
        if !path.exists() {
            return; // Skip if model not available
        }
        let data = std::fs::read(path).unwrap();
        let encoder = WasmEncoder::from_safetensors_bytes(&data).unwrap();

        // Quick smoke test: forward pass on 4 tokens
        let tokens = vec![0, 100, 200, 2]; // BOS, two tokens, EOS
        let cls = encoder.cls_embedding(&tokens);
        assert_eq!(cls.len(), HIDDEN_SIZE);
        assert!(cls.iter().all(|v| v.is_finite()), "CLS must be finite");
    }

    #[test]
    fn test_encoder_deterministic() {
        let path = std::path::Path::new("/tmp/codebert-base/model_int8.safetensors");
        if !path.exists() {
            return;
        }
        let data = std::fs::read(path).unwrap();
        let encoder = WasmEncoder::from_safetensors_bytes(&data).unwrap();

        let tokens = vec![0, 50, 100, 2];
        let cls1 = encoder.cls_embedding(&tokens);
        let cls2 = encoder.cls_embedding(&tokens);
        assert_eq!(cls1, cls2, "CLS must be deterministic");
    }

    #[test]
    fn test_encoder_benchmark() {
        let path = std::path::Path::new("/tmp/codebert-base/model_int8.safetensors");
        if !path.exists() {
            return;
        }
        let data = std::fs::read(path).unwrap();
        let encoder = WasmEncoder::from_safetensors_bytes(&data).unwrap();

        // Benchmark: classify a short script
        let script = "#!/bin/bash\neval \"$user_input\"\n";
        let tokens = simple_tokenize(script, 128);
        let start = std::time::Instant::now();
        let _cls = encoder.cls_embedding(&tokens);
        let elapsed = start.elapsed();

        // Document the result — this informs kill criterion 5
        eprintln!(
            "WASM-004 benchmark: {} tokens, {:.1}ms native CPU",
            tokens.len(),
            elapsed.as_secs_f64() * 1000.0
        );
        // WASM is typically 3-5x slower than native
        // Kill criterion: >2000ms in browser
    }
}
