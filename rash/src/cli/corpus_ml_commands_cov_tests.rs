//! PMAT-257 coverage tests for `corpus_ml_commands.rs`.
//!
//! `ml` is not a default feature (see `rash/Cargo.toml`: `ml = ["aprender",
//! "entrenar"]`), so in the default `--lib` build:
//! - `corpus_extract_embeddings` and `corpus_run_classifier` compile only
//!   their `#[cfg(not(feature = "ml"))]` bodies, which always return an
//!   `Err` explaining the missing feature -- no model is ever loaded, no
//!   network access happens.
//! - `corpus_train_classifier` is NOT feature-gated; it always compiles and
//!   exercises real training/evaluation code from `corpus::classifier`. Its
//!   `mlp` branch calls `train_and_evaluate_mlp`, whose `#[cfg(not(feature =
//!   "ml"))]` fallback always returns an `Err` (real MLP training requires
//!   the `ml` feature).
//!
//! All fixtures use `tempfile::TempDir` and hand-built embedding JSONL
//! files; nothing is downloaded and no real model is loaded.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_ml_commands::*;
    use crate::corpus::classifier::{save_embeddings, EmbeddingEntry};
    use std::path::PathBuf;

    fn fixture_embeddings(n: usize, dim: usize) -> Vec<EmbeddingEntry> {
        (0..n)
            .map(|i| {
                let label = u8::from(i % 2 == 0);
                let mut emb = vec![0.0f32; dim];
                for (j, v) in emb.iter_mut().enumerate() {
                    *v = if label == 0 {
                        if j < dim / 2 {
                            1.0
                        } else {
                            -1.0
                        }
                    } else if j < dim / 2 {
                        -1.0
                    } else {
                        1.0
                    };
                }
                EmbeddingEntry {
                    id: format!("entry_{i}"),
                    embedding: emb,
                    label,
                }
            })
            .collect()
    }

    #[test]
    fn test_PMAT257_cov_extract_embeddings_without_ml_feature_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let model = dir.path().join("model");
        let output = dir.path().join("embeddings.jsonl");
        let err = corpus_extract_embeddings(model, output, None, None)
            .expect_err("extract-embeddings requires the ml feature");
        assert!(format!("{err}").contains("ml"));
    }

    #[test]
    fn test_PMAT257_cov_extract_embeddings_with_limit_and_input_jsonl_still_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let model = dir.path().join("model");
        let output = dir.path().join("embeddings.jsonl");
        let input = dir.path().join("input.jsonl");
        std::fs::write(&input, "{\"input\":\"echo hi\",\"label\":0}\n").expect("write fixture");
        let err = corpus_extract_embeddings(model, output, Some(5), Some(input))
            .expect_err("extract-embeddings requires the ml feature regardless of args");
        assert!(format!("{err}").contains("ml"));
    }

    #[test]
    fn test_PMAT257_cov_run_classifier_without_ml_feature_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let model = dir.path().join("model");
        let output = dir.path().join("out");
        let err = corpus_run_classifier(model, output, 5, 0.01, 42)
            .expect_err("run-classifier requires the ml feature");
        assert!(format!("{err}").contains("ml"));
    }

    #[test]
    fn test_PMAT257_cov_train_classifier_linear_probe_end_to_end() {
        let dir = tempfile::tempdir().expect("tempdir");
        let embeddings_path = dir.path().join("embeddings.jsonl");
        save_embeddings(&fixture_embeddings(30, 16), &embeddings_path)
            .expect("save fixture embeddings");
        let output = dir.path().join("out");

        corpus_train_classifier(
            embeddings_path,
            output.clone(),
            5,
            0.01,
            42,
            None,
            Vec::new(),
            false,
            8,
        )
        .expect("linear probe training runs over a tiny fixture");

        assert!(output.join("probe.json").exists());
        assert!(output.join("evaluation.json").exists());
        assert!(!output.join("mlp_probe.json").exists());
    }

    #[test]
    fn test_PMAT257_cov_train_classifier_respects_max_entries_and_augment() {
        let dir = tempfile::tempdir().expect("tempdir");
        let embeddings_path = dir.path().join("embeddings.jsonl");
        save_embeddings(&fixture_embeddings(20, 8), &embeddings_path)
            .expect("save fixture embeddings");
        let augment_path = dir.path().join("augment.jsonl");
        save_embeddings(&fixture_embeddings(6, 8), &augment_path).expect("save augment fixture");
        let output = dir.path().join("out2");

        corpus_train_classifier(
            embeddings_path,
            output.clone(),
            3,
            0.01,
            7,
            Some(10),
            vec![augment_path],
            false,
            4,
        )
        .expect("training with max_entries + augment runs");

        assert!(output.join("probe.json").exists());
        assert!(output.join("evaluation.json").exists());
    }

    #[test]
    fn test_PMAT257_cov_train_classifier_mlp_without_ml_feature_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let embeddings_path = dir.path().join("embeddings.jsonl");
        save_embeddings(&fixture_embeddings(10, 8), &embeddings_path)
            .expect("save fixture embeddings");
        let output = dir.path().join("out3");

        let err = corpus_train_classifier(
            embeddings_path,
            output,
            2,
            0.01,
            1,
            None,
            Vec::new(),
            true,
            4,
        )
        .expect_err("MLP training requires the ml feature in this build");
        assert!(format!("{err}").contains("ml"));
    }

    #[test]
    fn test_PMAT257_cov_train_classifier_missing_embeddings_file_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing: PathBuf = dir.path().join("does-not-exist.jsonl");
        let output = dir.path().join("out4");

        let err = corpus_train_classifier(missing, output, 2, 0.01, 1, None, Vec::new(), false, 4)
            .expect_err("missing embeddings file must fail to load");
        assert!(
            format!("{err}").to_lowercase().contains("cannot open")
                || format!("{err}").to_lowercase().contains("no such file")
        );
    }
}
