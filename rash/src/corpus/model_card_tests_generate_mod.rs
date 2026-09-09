use super::*;
use crate::corpus::registry::CorpusRegistry;

// Contract: contracts/corpus-derived-generators-v1.yaml (F-CDG-001, F-CDG-006).
// No test here calls generate_model_card(): it walks all 17,942 corpus entries
// and cost 567-825 s per test in CI (run 34368312280). The card is exercised on
// the 30-entry tier-1 set instead.

fn tier1_card() -> String {
    generate_model_card_from(&CorpusRegistry::load_tier1())
}

/// F-CDG-001: the card reports the counts of the registry it is given.
#[test]
fn test_F_CDG_001_model_card_reports_tier1_counts() {
    let registry = CorpusRegistry::load_tier1();
    assert_eq!(registry.len(), 30);
    let card = generate_model_card_from(&registry);
    let line = card
        .lines()
        .find(|l| l.contains("Total entries"))
        .unwrap_or("");
    assert!(
        line.contains("30"),
        "Total entries line must report the tier-1 count: {line:?}"
    );
}

/// F-CDG-006: the public entry point is exactly the _from form over load_full().
#[test]
fn test_F_CDG_006_model_card_entry_point_is_full_registry_path() {
    let src = include_str!("model_card.rs");
    let head = "pub fn generate_model_card() -> String {";
    let Some(start) = src.find(head) else {
        panic!("generate_model_card() not found in source");
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
            "generate_model_card_from(&CorpusRegistry::load_full())",
            "}"
        ],
        "generate_model_card() must be exactly the _from path over load_full()"
    );
}

#[test]
fn test_generate_model_card_has_yaml_front_matter() {
    let card = tier1_card();
    assert!(
        card.starts_with("---\n"),
        "Must start with YAML front matter"
    );
    assert!(
        card.contains("---\n\n#"),
        "Must have closing --- before markdown"
    );
}

#[test]
fn test_generate_model_card_has_required_sections() {
    let card = tier1_card();
    assert!(card.contains("## Dataset"), "Must have Dataset section");
    assert!(card.contains("### Splits"), "Must have Splits section");
    assert!(
        card.contains("### Class Weights"),
        "Must have Class Weights section"
    );
    assert!(card.contains("## Baselines"), "Must have Baselines section");
    assert!(
        card.contains("## Intended Use"),
        "Must have Intended Use section"
    );
    assert!(
        card.contains("## Limitations"),
        "Must have Limitations section"
    );
    assert!(
        card.contains("## Honesty Requirements"),
        "Must have Honesty section"
    );
    assert!(card.contains("## Citation"), "Must have Citation section");
}

#[test]
fn test_generate_model_card_has_honesty_requirements() {
    let card = tier1_card();
    assert!(card.contains("synthetic data derived from rule-based linter"));
    assert!(card.contains("known patterns"));
    assert!(card.contains("Not a replacement for security audit"));
}

#[test]
fn test_generate_model_card_has_live_corpus_data() {
    let card = tier1_card();
    // Must contain actual corpus size (>100 entries)
    assert!(card.contains("Total entries"), "Must report total entries");
    // Must contain split data
    assert!(card.contains("Train"), "Must have train split");
    assert!(card.contains("Val"), "Must have val split");
    assert!(card.contains("Test"), "Must have test split");
}

#[test]
fn test_generate_model_card_has_class_weights() {
    let card = tier1_card();
    assert!(card.contains("safe (0)"), "Must report safe weight");
    assert!(card.contains("unsafe (1)"), "Must report unsafe weight");
}

#[test]
fn test_compute_class_weight_balanced() {
    // 50/50 split → weight ≈ 1.414
    let w = compute_class_weight(50, 100);
    assert!((w - std::f64::consts::SQRT_2).abs() < 0.001);
}

#[test]
fn test_compute_class_weight_minority() {
    // 2% minority → weight should be > 5
    let w = compute_class_weight(2, 100);
    assert!(w > 5.0, "Minority class weight should be high: {w}");
}

#[test]
fn test_compute_class_weight_zero_guard() {
    assert!((compute_class_weight(0, 100) - 1.0).abs() < 1e-9);
    assert!((compute_class_weight(100, 0) - 1.0).abs() < 1e-9);
}

#[test]
fn test_generate_model_card_yaml_metadata() {
    let card = tier1_card();
    assert!(card.contains("license: apache-2.0"));
    assert!(card.contains("- shell-safety"));
    assert!(card.contains("task_categories:"));
    assert!(card.contains("- text-classification"));
}
