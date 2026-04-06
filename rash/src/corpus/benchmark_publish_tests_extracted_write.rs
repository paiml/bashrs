use super::*;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn write_test_splits(dir: &Path) {
    let train_data = r#"{"input":"echo hello","label":0}
{"input":"eval $cmd","label":1}
{"input":"ls -la","label":0}
"#;
    let val_data = r#"{"input":"cat file","label":0}
{"input":"rm -rf /","label":1}
"#;
    let test_data = r#"{"input":"pwd","label":0}
{"input":"chmod 777 /etc/passwd","label":1}
"#;
    std::fs::write(dir.join("train.jsonl"), train_data).expect("write train");
    std::fs::write(dir.join("val.jsonl"), val_data).expect("write val");
    std::fs::write(dir.join("test.jsonl"), test_data).expect("write test");
}

#[test]
fn test_PMAT172_read_splits_valid() {
    let dir = TempDir::new().expect("tmpdir");
    write_test_splits(dir.path());
    let (train, val, test) = read_splits(dir.path()).expect("read");
    assert_eq!(train.len(), 3);
    assert_eq!(val.len(), 2);
    assert_eq!(test.len(), 2);
}

#[test]
fn test_PMAT172_read_splits_labels() {
    let dir = TempDir::new().expect("tmpdir");
    write_test_splits(dir.path());
    let (train, _, _) = read_splits(dir.path()).expect("read");
    assert_eq!(train[0].label, 0); // safe
    assert_eq!(train[1].label, 1); // unsafe
}

#[test]
fn test_PMAT172_read_splits_missing_file() {
    let dir = TempDir::new().expect("tmpdir");
    let result = read_splits(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_PMAT172_compute_summary() {
    let train = vec![
        SplitEntry {
            input: "echo hi".into(),
            label: 0,
        },
        SplitEntry {
            input: "eval $x".into(),
            label: 1,
        },
    ];
    let val = vec![SplitEntry {
        input: "ls".into(),
        label: 0,
    }];
    let test = vec![SplitEntry {
        input: "rm /".into(),
        label: 1,
    }];
    let s = compute_summary(&train, &val, &test);
    assert_eq!(s.total, 4);
    assert_eq!(s.unsafe_count, 2);
    assert_eq!(s.safe_count, 2);
    assert!((s.unsafe_pct - 50.0).abs() < 0.01);
}

#[test]
fn test_PMAT172_generate_dataset_card_has_yaml_frontmatter() {
    let summary = PublishSummary {
        train_count: 22169,
        val_count: 2738,
        test_count: 2935,
        total: 27842,
        unsafe_count: 5849,
        safe_count: 21993,
        unsafe_pct: 21.0,
    };
    let card = generate_dataset_card(&summary, "1.0.0");
    assert!(card.starts_with("---\n"));
    assert!(card.contains("pretty_name: ShellSafetyBench"));
    assert!(card.contains("license: apache-2.0"));
    assert!(card.contains("binary-classification"));
    assert!(card.contains("train.jsonl"));
    assert!(card.contains("validation.jsonl"));
    assert!(card.contains("test.jsonl"));
    assert!(card.contains("22169"));
    assert!(card.contains("2935"));
}

#[test]
fn test_PMAT172_generate_dataset_card_has_cwe_info() {
    let summary = PublishSummary {
        train_count: 100,
        val_count: 10,
        test_count: 10,
        total: 120,
        unsafe_count: 20,
        safe_count: 100,
        unsafe_pct: 16.7,
    };
    let card = generate_dataset_card(&summary, "1.0.0");
    assert!(card.contains("CWE"));
    assert!(card.contains("SEC"));
}

#[test]
fn test_PMAT172_generate_dataset_card_has_baselines() {
    let summary = PublishSummary {
        train_count: 100,
        val_count: 10,
        test_count: 10,
        total: 120,
        unsafe_count: 20,
        safe_count: 100,
        unsafe_pct: 16.7,
    };
    let card = generate_dataset_card(&summary, "1.0.0");
    assert!(card.contains("MLP probe"));
    assert!(card.contains("0.754"));
    assert!(card.contains("Qwen3-4B"));
}

#[test]
fn test_PMAT172_generate_dataset_infos() {
    let summary = PublishSummary {
        train_count: 100,
        val_count: 10,
        test_count: 10,
        total: 120,
        unsafe_count: 20,
        safe_count: 100,
        unsafe_pct: 16.7,
    };
    let infos = generate_dataset_infos(&summary);
    let parsed: serde_json::Value = serde_json::from_str(&infos).expect("valid json");
    assert!(parsed["default"]["splits"]["train"]["num_examples"] == 100);
    assert!(parsed["default"]["splits"]["test"]["num_examples"] == 10);
    assert!(parsed["default"]["features"]["input"]["dtype"] == "string");
}

#[test]
fn test_PMAT172_publish_benchmark_end_to_end() {
    let splits_dir = TempDir::new().expect("tmpdir");
    write_test_splits(splits_dir.path());
    let output_dir = TempDir::new().expect("tmpdir");

    let summary =
        publish_benchmark(splits_dir.path(), output_dir.path(), "1.0.0").expect("publish");

    // Verify files created
    assert!(output_dir.path().join("README.md").exists());
    assert!(output_dir.path().join("train.jsonl").exists());
    assert!(output_dir.path().join("validation.jsonl").exists());
    assert!(output_dir.path().join("test.jsonl").exists());
    assert!(output_dir.path().join("dataset_infos.json").exists());

    // Verify summary
    assert_eq!(summary.total, 7);
    assert_eq!(summary.train_count, 3);
    assert_eq!(summary.val_count, 2);
    assert_eq!(summary.test_count, 2);

    // Verify README has YAML front matter
    let readme = std::fs::read_to_string(output_dir.path().join("README.md")).expect("read readme");
    assert!(readme.starts_with("---\n"));
    assert!(readme.contains("ShellSafetyBench"));

    // Verify validation.jsonl (HF naming, not val.jsonl)
    let val_content = std::fs::read_to_string(output_dir.path().join("validation.jsonl"))
        .expect("read validation");
    assert!(val_content.contains("\"label\""));
}

#[test]
fn test_PMAT172_write_jsonl_round_trip() {
    let dir = TempDir::new().expect("tmpdir");
    let entries = vec![
        SplitEntry {
            input: "echo hello".into(),
            label: 0,
        },
        SplitEntry {
            input: "eval $x".into(),
            label: 1,
        },
    ];
    let path = dir.path().join("test.jsonl");
    write_jsonl(&path, &entries).expect("write");
    let read_back = read_jsonl(&path).expect("read");
    assert_eq!(read_back.len(), 2);
    assert_eq!(read_back[0].input, "echo hello");
    assert_eq!(read_back[1].label, 1);
}
