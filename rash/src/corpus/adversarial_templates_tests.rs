//! Coverage tests for corpus/adversarial_templates.rs.
//!
//! Tests all 4 public template functions and the all_templates() aggregator.
//! No external dependencies needed — all functions return Vec<AdversarialTemplate>.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::adversarial_templates::{
    all_templates, needs_quoting_templates, non_deterministic_templates, non_idempotent_templates,
    unsafe_templates, AdversarialTemplate, ParamSlot, COMMENTS, SETUP_LINES, SHEBANGS,
    TRAILING_LINES,
};

// ── Constants coverage ────────────────────────────────────────────────────

#[test]
fn test_coverage_shebangs_nonempty() {
    assert!(!SHEBANGS.is_empty());
    assert!(SHEBANGS.iter().all(|s| s.starts_with('#')));
}

#[test]
fn test_coverage_comments_nonempty() {
    assert!(!COMMENTS.is_empty());
    assert!(COMMENTS.iter().all(|s| s.starts_with('#')));
}

#[test]
fn test_coverage_setup_lines_nonempty() {
    assert!(!SETUP_LINES.is_empty());
}

#[test]
fn test_coverage_trailing_lines_nonempty() {
    assert!(!TRAILING_LINES.is_empty());
}

// ── non_deterministic_templates ────────────────────────────────────────────

#[test]
fn test_coverage_non_deterministic_templates_returns_vec() {
    let templates = non_deterministic_templates();
    assert!(
        !templates.is_empty(),
        "non_deterministic_templates() must be non-empty"
    );
}

#[test]
fn test_coverage_non_deterministic_templates_target_class() {
    for t in non_deterministic_templates() {
        assert_eq!(
            t.target_class, 2,
            "non-deterministic template {} has wrong class",
            t.family
        );
    }
}

#[test]
fn test_coverage_non_deterministic_templates_families_nonempty() {
    for t in non_deterministic_templates() {
        assert!(!t.family.is_empty(), "template has empty family name");
        assert!(
            !t.template.is_empty(),
            "template {} has empty body",
            t.family
        );
    }
}

#[test]
fn test_coverage_non_deterministic_templates_contains_random_or_date() {
    let templates = non_deterministic_templates();
    let has_random_pattern = templates.iter().any(|t| {
        t.template.contains("$RANDOM") || t.template.contains("date") || t.template.contains("$$")
    });
    assert!(
        has_random_pattern,
        "non-deterministic templates should contain $RANDOM/date/$$"
    );
}

#[test]
fn test_coverage_non_deterministic_templates_params_have_pools() {
    for t in non_deterministic_templates() {
        for slot in t.params {
            assert!(
                !slot.name.is_empty(),
                "param slot has empty name in {}",
                t.family
            );
            assert!(
                !slot.pool.is_empty(),
                "param slot {} has empty pool in {}",
                slot.name,
                t.family
            );
        }
    }
}

#[test]
fn test_coverage_non_deterministic_templates_count_at_least_10() {
    assert!(
        non_deterministic_templates().len() >= 10,
        "expected >=10 non-deterministic templates, got {}",
        non_deterministic_templates().len()
    );
}

// ── non_idempotent_templates ───────────────────────────────────────────────

#[test]
fn test_coverage_non_idempotent_templates_returns_vec() {
    let templates = non_idempotent_templates();
    assert!(
        !templates.is_empty(),
        "non_idempotent_templates() must be non-empty"
    );
}

#[test]
fn test_coverage_non_idempotent_templates_target_class() {
    for t in non_idempotent_templates() {
        assert_eq!(
            t.target_class, 3,
            "non-idempotent template {} has wrong class",
            t.family
        );
    }
}

#[test]
fn test_coverage_non_idempotent_templates_families_nonempty() {
    for t in non_idempotent_templates() {
        assert!(!t.family.is_empty());
        assert!(
            !t.template.is_empty(),
            "template {} has empty body",
            t.family
        );
    }
}

#[test]
fn test_coverage_non_idempotent_templates_contains_mkdir_or_rm() {
    let templates = non_idempotent_templates();
    let has_idempotency_pattern = templates.iter().any(|t| {
        t.template.contains("mkdir") || t.template.contains("rm ") || t.template.contains("ln ")
    });
    assert!(
        has_idempotency_pattern,
        "non-idempotent templates should contain mkdir/rm/ln patterns"
    );
}

#[test]
fn test_coverage_non_idempotent_templates_count_at_least_10() {
    assert!(
        non_idempotent_templates().len() >= 10,
        "expected >=10 non-idempotent templates, got {}",
        non_idempotent_templates().len()
    );
}

// ── unsafe_templates ──────────────────────────────────────────────────────

#[test]
fn test_coverage_unsafe_templates_returns_vec() {
    let templates = unsafe_templates();
    assert!(
        !templates.is_empty(),
        "unsafe_templates() must be non-empty"
    );
}

#[test]
fn test_coverage_unsafe_templates_target_class() {
    for t in unsafe_templates() {
        assert_eq!(
            t.target_class, 4,
            "unsafe template {} has wrong class",
            t.family
        );
    }
}

#[test]
fn test_coverage_unsafe_templates_families_nonempty() {
    for t in unsafe_templates() {
        assert!(!t.family.is_empty());
        assert!(
            !t.template.is_empty(),
            "template {} has empty body",
            t.family
        );
    }
}

#[test]
fn test_coverage_unsafe_templates_count_at_least_10() {
    assert!(
        unsafe_templates().len() >= 10,
        "expected >=10 unsafe templates, got {}",
        unsafe_templates().len()
    );
}

#[test]
fn test_coverage_unsafe_templates_params_valid() {
    for t in unsafe_templates() {
        for slot in t.params {
            assert!(!slot.name.is_empty(), "empty param name in {}", t.family);
            assert!(
                !slot.pool.is_empty(),
                "empty pool for {} in {}",
                slot.name,
                t.family
            );
        }
    }
}

// ── needs_quoting_templates ───────────────────────────────────────────────

#[test]
fn test_coverage_needs_quoting_templates_returns_vec() {
    let templates = needs_quoting_templates();
    assert!(
        !templates.is_empty(),
        "needs_quoting_templates() must be non-empty"
    );
}

#[test]
fn test_coverage_needs_quoting_templates_target_class() {
    for t in needs_quoting_templates() {
        assert_eq!(
            t.target_class, 1,
            "needs-quoting template {} has wrong class",
            t.family
        );
    }
}

#[test]
fn test_coverage_needs_quoting_templates_families_nonempty() {
    for t in needs_quoting_templates() {
        assert!(!t.family.is_empty());
        assert!(
            !t.template.is_empty(),
            "template {} has empty body",
            t.family
        );
    }
}

#[test]
fn test_coverage_needs_quoting_templates_count_at_least_10() {
    assert!(
        needs_quoting_templates().len() >= 10,
        "expected >=10 needs-quoting templates, got {}",
        needs_quoting_templates().len()
    );
}

// ── all_templates ─────────────────────────────────────────────────────────

#[test]
fn test_coverage_all_templates_returns_all_groups() {
    let all = all_templates();
    assert!(!all.is_empty(), "all_templates() must be non-empty");
}

#[test]
fn test_coverage_all_templates_includes_all_classes() {
    let all = all_templates();
    let has_class1 = all.iter().any(|t| t.target_class == 1);
    let has_class2 = all.iter().any(|t| t.target_class == 2);
    let has_class3 = all.iter().any(|t| t.target_class == 3);
    let has_class4 = all.iter().any(|t| t.target_class == 4);
    assert!(
        has_class1,
        "all_templates() missing class 1 (needs-quoting)"
    );
    assert!(
        has_class2,
        "all_templates() missing class 2 (non-deterministic)"
    );
    assert!(
        has_class3,
        "all_templates() missing class 3 (non-idempotent)"
    );
    assert!(has_class4, "all_templates() missing class 4 (unsafe)");
}

#[test]
fn test_coverage_all_templates_count_at_least_40() {
    let all = all_templates();
    assert!(
        all.len() >= 40,
        "expected >=40 total templates, got {}",
        all.len()
    );
}

#[test]
fn test_coverage_all_templates_sum_of_groups() {
    let expected = needs_quoting_templates().len()
        + non_deterministic_templates().len()
        + non_idempotent_templates().len()
        + unsafe_templates().len();
    let actual = all_templates().len();
    assert_eq!(
        actual, expected,
        "all_templates() count mismatch: expected {expected}, got {actual}"
    );
}

#[test]
fn test_coverage_all_templates_unique_families() {
    let all = all_templates();
    let mut families: Vec<&str> = all.iter().map(|t| t.family).collect();
    let total = families.len();
    families.sort();
    families.dedup();
    assert_eq!(
        families.len(),
        total,
        "found duplicate family names in all_templates()"
    );
}

// ── AdversarialTemplate struct fields ────────────────────────────────────

#[test]
fn test_coverage_adversarial_template_debug_clone() {
    let templates = non_deterministic_templates();
    let t = &templates[0];
    let cloned = t.clone();
    assert_eq!(t.family, cloned.family);
    assert_eq!(t.target_class, cloned.target_class);
    // Debug format should work
    let _ = format!("{:?}", t);
}

#[test]
fn test_coverage_param_slot_debug_clone() {
    let slot = ParamSlot {
        name: "TEST",
        pool: &["a", "b", "c"],
    };
    let cloned = slot.clone();
    assert_eq!(slot.name, cloned.name);
    assert_eq!(slot.pool.len(), cloned.pool.len());
    let _ = format!("{:?}", slot);
}

#[test]
fn test_coverage_template_body_contains_placeholder() {
    // Templates with params should have placeholders in template body
    for t in all_templates() {
        if !t.params.is_empty() {
            // At least one param's name should appear as {NAME} in the template
            let has_placeholder = t
                .params
                .iter()
                .any(|p| t.template.contains(&format!("{{{}}}", p.name)));
            assert!(
                has_placeholder,
                "template {} has params but no matching {{PARAM}} placeholder",
                t.family
            );
        }
    }
}

#[test]
fn test_coverage_non_deterministic_specific_families() {
    let templates = non_deterministic_templates();
    let families: Vec<&str> = templates.iter().map(|t| t.family).collect();
    // All families should contain "NONDET" prefix based on the module implementation
    assert!(
        families.iter().any(|f| f.contains("NONDET")),
        "expected NONDET prefix in families"
    );
}

#[test]
fn test_coverage_non_idempotent_specific_families() {
    let templates = non_idempotent_templates();
    let families: Vec<&str> = templates.iter().map(|t| t.family).collect();
    assert!(
        families.iter().any(|f| f.contains("NONIDEM")),
        "expected NONIDEM prefix in families"
    );
}

#[test]
fn test_coverage_unsafe_specific_families() {
    let templates = unsafe_templates();
    let families: Vec<&str> = templates.iter().map(|t| t.family).collect();
    assert!(
        families.iter().any(|f| f.contains("UNSAFE")),
        "expected UNSAFE prefix in families"
    );
}

#[test]
fn test_coverage_needs_quoting_specific_families() {
    let templates = needs_quoting_templates();
    let families: Vec<&str> = templates.iter().map(|t| t.family).collect();
    assert!(
        families.iter().any(|f| f.contains("QUOTE")),
        "expected QUOTE prefix in families"
    );
}
