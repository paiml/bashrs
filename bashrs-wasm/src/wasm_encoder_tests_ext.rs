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
