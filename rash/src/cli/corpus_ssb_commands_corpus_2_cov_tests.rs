//! PMAT-257 coverage tests for `corpus_ssb_commands_corpus_2.rs` (`include!`d
//! into the `corpus_ssb_commands` module). Kept as a sibling file: the source
//! holds `corpus_merge_data` (cognitive 36) and `corpus_convert_ssb`
//! (cognitive 28), both over the pre-commit complexity gate, so a tests-only
//! change staged directly in that file is refused by the hook. Wired from
//! `commands.rs`.
//!
//! Neither `corpus_merge_data` nor `corpus_convert_ssb` builds a
//! `CorpusRunner` or calls `CorpusRegistry::load_full()` -- both are plain
//! file I/O plus string/JSON transforms, so they're exercised directly
//! against `tempfile::TempDir` fixtures rather than needing a `*_with` twin.
//! `normalize_verificar_entry` is a private helper of `corpus_ssb_commands`,
//! not reachable from this sibling module, so it is covered indirectly via
//! `corpus_merge_data` inputs shaped like verificar mutation output.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_ssb_commands::*;

    fn read_jsonl(path: &std::path::Path) -> Vec<serde_json::Value> {
        std::fs::read_to_string(path)
            .expect("read merged/converted output")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).expect("valid json line"))
            .collect()
    }

    #[test]
    fn test_PMAT257_cov_merge_data_normalizes_verificar_entries() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let extra = dir.path().join("verificar.jsonl");
        std::fs::write(
            &extra,
            concat!(
                "{\"unsafe_script\":\"eval $CMD\",\"safe_script\":\"true\",\"cwe\":\"CWE-78\",",
                "\"vulnerability\":\"command injection\",\"mutation_description\":\"eval of untrusted input\",",
                "\"label\":1,\"findings\":[\"SEC001\"],\"classification\":\"unsafe\"}\n",
                "{\"unsafe_script\":\"echo $x\",\"safe_script\":\"echo \\\"$x\\\"\",",
                "\"label\":0}\n",
                // Already in conversation format -- must pass through untouched.
                "{\"instruction\":\"hi\",\"response\":\"hello\",\"label\":0}\n",
            ),
        )
        .expect("write verificar fixture");

        let output = dir.path().join("merged.jsonl");
        corpus_merge_data(output.clone(), vec![extra], 42)
            .expect("merge over a tiny verificar fixture must succeed");

        let entries = read_jsonl(&output);
        assert_eq!(entries.len(), 3);
        for entry in &entries {
            assert_eq!(
                entry.get("source").and_then(|v| v.as_str()),
                Some("verificar")
            );
        }
        // The two normalized entries must have gained conversation fields.
        let normalized: Vec<_> = entries.iter().filter(|e| e.get("cwe").is_some()).collect();
        assert_eq!(normalized.len(), 2);
        for entry in &normalized {
            assert!(entry.get("instruction").and_then(|v| v.as_str()).is_some());
            assert!(entry.get("response").and_then(|v| v.as_str()).is_some());
        }
    }

    #[test]
    fn test_PMAT257_cov_merge_data_missing_extra_input_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("does-not-exist.jsonl");
        let output = dir.path().join("merged.jsonl");
        let err = corpus_merge_data(output, vec![missing], 1)
            .expect_err("a missing extra input file must fail");
        assert!(format!("{err}").contains("Input file not found"), "{err}");
    }

    #[test]
    fn test_PMAT257_cov_merge_data_no_inputs_writes_empty_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let output = dir.path().join("merged.jsonl");
        // No extra inputs, and the hardcoded corpus conversations path does
        // not exist under the crate-root test cwd -- both loaders no-op.
        corpus_merge_data(output.clone(), vec![], 7)
            .expect("merge with zero inputs must still succeed");
        let entries = read_jsonl(&output);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_PMAT257_cov_convert_ssb_covers_every_branch_to_a_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("split.jsonl");
        std::fs::write(
            &input,
            concat!(
                "not valid json\n",
                "{\"label\":0}\n",
                "{\"input\":\"no code fences here\",\"label\":9}\n",
                "{\"input\":\"\",\"label\":0}\n",
                "{\"input\":\"Evaluate.\\n\\n```bash\\necho hi\\n```\",\"label\":0}\n",
                "{\"input\":\"Evaluate.\\n\\n```bash\\nrm -rf $DIR\\n```\",\"label\":1}\n",
            ),
        )
        .expect("write split fixture");

        let output = dir.path().join("chatml.jsonl");
        corpus_convert_ssb(input, Some(output.clone()), None)
            .expect("convert must skip malformed lines without failing");

        let entries = read_jsonl(&output);
        // Only the last three lines have a well-formed input + valid label.
        assert_eq!(entries.len(), 3);
        let texts: Vec<String> = entries
            .iter()
            .map(|e| e["text"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(texts[0].contains("Classification: safe"));
        assert!(texts[1].contains("Classification: safe"));
        assert!(texts[2].contains("Classification: unsafe"));
    }

    #[test]
    fn test_PMAT257_cov_convert_ssb_respects_limit_and_stdout_branch() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("split.jsonl");
        std::fs::write(
            &input,
            concat!(
                "{\"input\":\"Evaluate.\\n\\n```bash\\necho one\\n```\",\"label\":0}\n",
                "{\"input\":\"Evaluate.\\n\\n```bash\\necho two\\n```\",\"label\":0}\n",
            ),
        )
        .expect("write split fixture");

        // limit=1 and output=None exercises the print-to-stdout branch and
        // the `max = limit.min(lines.len())` clamp.
        corpus_convert_ssb(input, None, Some(1))
            .expect("convert with a limit and no output path must still succeed");
    }
}
