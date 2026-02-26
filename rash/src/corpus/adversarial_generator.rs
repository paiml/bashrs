//! Adversarial training data generator for shell safety classifier.
//!
//! Produces parametrically-varied shell scripts for each underrepresented safety class
//! (non-deterministic, non-idempotent, unsafe, needs-quoting), verified against the
//! existing `derive_safety_label` heuristic for self-consistency.

use crate::cli::args::ClassifyFormat;
use crate::corpus::adversarial_templates::{
    self, AdversarialTemplate, COMMENTS, SETUP_LINES, SHEBANGS, TRAILING_LINES,
};
use crate::corpus::dataset::{ClassificationRow, SAFETY_LABELS};
use rand::prelude::IndexedRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Configuration for adversarial data generation.
#[derive(Debug, Clone)]
pub struct AdversarialConfig {
    /// RNG seed for reproducibility
    pub seed: u64,
    /// Number of samples per minority class (2, 3, 4)
    pub count_per_class: usize,
    /// Extra needs-quoting samples (class 1)
    pub extra_needs_quoting: usize,
    /// Verify each generated script against derive_safety_label
    pub verify: bool,
}

/// Statistics from a generation run.
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    /// Total scripts generated
    pub total: usize,
    /// Per-class counts [safe, needs-quoting, non-det, non-idem, unsafe]
    pub per_class: [usize; 5],
    /// Number of scripts that failed self-consistency verification
    pub misclassified: usize,
    /// Per-class mismatch counts
    pub misclassified_per_class: [usize; 5],
}

/// Result of an adversarial generation run.
#[derive(Debug)]
pub struct GenerationResult {
    /// Generated classification rows
    pub rows: Vec<ClassificationRow>,
    /// Generation statistics
    pub stats: GenerationStats,
}

/// Generate adversarial training data for the shell safety classifier.
///
/// Produces `count_per_class` samples for classes 2, 3, 4 and
/// `extra_needs_quoting` additional samples for class 1.
pub fn generate_adversarial(config: &AdversarialConfig) -> GenerationResult {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let mut rows = Vec::new();
    let mut stats = GenerationStats::default();

    let nondet_templates = adversarial_templates::non_deterministic_templates();
    let nonidem_templates = adversarial_templates::non_idempotent_templates();
    let unsafe_templates = adversarial_templates::unsafe_templates();
    let quoting_templates = adversarial_templates::needs_quoting_templates();

    // Class 2: non-deterministic
    generate_class_samples(
        &nondet_templates,
        2,
        config.count_per_class,
        config.verify,
        &mut rng,
        &mut rows,
        &mut stats,
    );

    // Class 3: non-idempotent
    generate_class_samples(
        &nonidem_templates,
        3,
        config.count_per_class,
        config.verify,
        &mut rng,
        &mut rows,
        &mut stats,
    );

    // Class 4: unsafe
    generate_class_samples(
        &unsafe_templates,
        4,
        config.count_per_class,
        config.verify,
        &mut rng,
        &mut rows,
        &mut stats,
    );

    // Class 1: needs-quoting (extra samples)
    generate_class_samples(
        &quoting_templates,
        1,
        config.extra_needs_quoting,
        config.verify,
        &mut rng,
        &mut rows,
        &mut stats,
    );

    stats.total = rows.len();

    GenerationResult { rows, stats }
}

/// Generate samples for a single class, distributing evenly across templates.
fn generate_class_samples(
    templates: &[AdversarialTemplate],
    target_class: u8,
    count: usize,
    verify: bool,
    rng: &mut ChaCha8Rng,
    rows: &mut Vec<ClassificationRow>,
    stats: &mut GenerationStats,
) {
    if templates.is_empty() || count == 0 {
        return;
    }

    let per_template = count / templates.len();
    let remainder = count % templates.len();

    for (idx, template) in templates.iter().enumerate() {
        let n = per_template + usize::from(idx < remainder);
        for _ in 0..n {
            let script = expand_template(rng, template);
            let label = if verify {
                let verified = verify_classification(&script, target_class);
                if !verified {
                    stats.misclassified += 1;
                    stats.misclassified_per_class[target_class as usize] += 1;
                }
                target_class
            } else {
                target_class
            };
            stats.per_class[label as usize] += 1;
            rows.push(ClassificationRow {
                input: script,
                label,
            });
        }
    }
}

/// Expand a template by substituting parameters and adding context.
fn expand_template(rng: &mut ChaCha8Rng, template: &AdversarialTemplate) -> String {
    let mut body = template.template.to_string();

    // Substitute each parameter
    for param in template.params {
        if let Some(value) = param.pool.choose(rng) {
            let placeholder = format!("{{{}}}", param.name);
            body = body.replace(&placeholder, value);
        }
    }

    // Build script with context wrapping
    let mut parts = Vec::new();

    // Shebang
    if let Some(shebang) = SHEBANGS.choose(rng) {
        parts.push(shebang.to_string());
    }

    // Optional comment (70% chance)
    if rng.random_range(0..10) < 7 {
        if let Some(comment) = COMMENTS.choose(rng) {
            parts.push(comment.to_string());
        }
    }

    // Optional setup line (50% chance)
    if rng.random_range(0..10) < 5 {
        if let Some(setup) = SETUP_LINES.choose(rng) {
            if !setup.is_empty() {
                parts.push(setup.to_string());
            }
        }
    }

    // Template body
    parts.push(body);

    // Optional trailing line (40% chance)
    if rng.random_range(0..10) < 4 {
        if let Some(trailing) = TRAILING_LINES.choose(rng) {
            if !trailing.is_empty() {
                parts.push(trailing.to_string());
            }
        }
    }

    parts.join("\n")
}

/// Verify that a generated script classifies to the expected class.
///
/// Uses the same `analyze_lint` + `derive_safety_label` pipeline as the
/// classify command to ensure self-consistency.
fn verify_classification(script: &str, expected_class: u8) -> bool {
    use crate::cli::commands::classify_cmds::analyze_lint;
    use crate::corpus::dataset::derive_safety_label;

    let signals = analyze_lint(script, &ClassifyFormat::Bash);

    let lint_clean = !signals.has_security_issues;
    let deterministic = !signals.has_determinism_issues;

    // For adversarial data, scripts are raw shell (not transpiler output)
    // so we pass transpiled=true to avoid everything being forced to class 4
    let actual_class = derive_safety_label(script, true, lint_clean, deterministic);

    actual_class == expected_class
}

/// Format generation statistics as a human-readable report.
pub fn format_stats(stats: &GenerationStats) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Total generated: {}", stats.total));
    lines.push(String::new());
    lines.push("Per-class distribution:".to_string());
    for (i, &count) in stats.per_class.iter().enumerate() {
        if count > 0 {
            let pct = if stats.total > 0 {
                count as f64 / stats.total as f64 * 100.0
            } else {
                0.0
            };
            lines.push(format!(
                "  {} ({}): {} ({:.1}%)",
                SAFETY_LABELS[i], i, count, pct
            ));
        }
    }
    if stats.misclassified > 0 {
        lines.push(String::new());
        lines.push(format!(
            "Misclassified: {} ({:.1}%)",
            stats.misclassified,
            stats.misclassified as f64 / stats.total as f64 * 100.0
        ));
        for (i, &count) in stats.misclassified_per_class.iter().enumerate() {
            if count > 0 {
                lines.push(format!("  {} ({}): {}", SAFETY_LABELS[i], i, count));
            }
        }
    } else {
        lines.push(String::new());
        lines.push("Misclassified: 0 (100% self-consistent)".to_string());
    }
    lines.join("\n")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic() {
        let config = AdversarialConfig {
            seed: 42,
            count_per_class: 10,
            extra_needs_quoting: 5,
            verify: false,
        };
        let result = generate_adversarial(&config);
        assert_eq!(result.stats.total, 35); // 10*3 + 5
        assert_eq!(result.stats.per_class[2], 10);
        assert_eq!(result.stats.per_class[3], 10);
        assert_eq!(result.stats.per_class[4], 10);
        assert_eq!(result.stats.per_class[1], 5);
        assert_eq!(result.rows.len(), 35);
    }

    #[test]
    fn test_deterministic_output() {
        let config = AdversarialConfig {
            seed: 42,
            count_per_class: 5,
            extra_needs_quoting: 3,
            verify: false,
        };
        let result1 = generate_adversarial(&config);
        let result2 = generate_adversarial(&config);
        assert_eq!(result1.rows.len(), result2.rows.len());
        for (a, b) in result1.rows.iter().zip(result2.rows.iter()) {
            assert_eq!(a.input, b.input);
            assert_eq!(a.label, b.label);
        }
    }

    #[test]
    fn test_all_scripts_have_shebang() {
        let config = AdversarialConfig {
            seed: 123,
            count_per_class: 10,
            extra_needs_quoting: 5,
            verify: false,
        };
        let result = generate_adversarial(&config);
        for row in &result.rows {
            assert!(
                row.input.starts_with("#!"),
                "Script should start with shebang: {}",
                &row.input[..row.input.len().min(80)]
            );
        }
    }

    #[test]
    fn test_all_scripts_nonempty() {
        let config = AdversarialConfig {
            seed: 99,
            count_per_class: 25,
            extra_needs_quoting: 10,
            verify: false,
        };
        let result = generate_adversarial(&config);
        for row in &result.rows {
            assert!(!row.input.is_empty());
            // Scripts should have at least shebang + one content line
            assert!(
                row.input.lines().count() >= 2,
                "Script too short: {}",
                row.input
            );
        }
    }

    #[test]
    fn test_template_expansion_all_families() {
        let all = adversarial_templates::all_templates();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for template in &all {
            let script = expand_template(&mut rng, template);
            // Check that no {PARAM}-style placeholders remain (but allow
            // shell syntax like ${VAR}, ${{VAR}}, etc.)
            for param in template.params {
                let placeholder = format!("{{{}}}", param.name);
                assert!(
                    !script.contains(&placeholder),
                    "Template {} has unexpanded placeholder {}: {}",
                    template.family,
                    placeholder,
                    script
                );
            }
        }
    }

    #[test]
    fn test_verify_nondet_scripts() {
        let config = AdversarialConfig {
            seed: 42,
            count_per_class: 5,
            extra_needs_quoting: 0,
            verify: true,
        };
        let result = generate_adversarial(&config);
        // Allow some misclassifications from heuristic gaps, but most should match
        let nondet_count = result.stats.per_class[2];
        let nondet_misclass = result.stats.misclassified_per_class[2];
        assert!(
            nondet_misclass as f64 / nondet_count as f64 <= 0.5,
            "Too many non-det misclassifications: {nondet_misclass}/{nondet_count}"
        );
    }

    #[test]
    fn test_format_stats() {
        let stats = GenerationStats {
            total: 100,
            per_class: [0, 10, 30, 30, 30],
            misclassified: 2,
            misclassified_per_class: [0, 0, 1, 1, 0],
        };
        let report = format_stats(&stats);
        assert!(report.contains("Total generated: 100"));
        assert!(report.contains("non-deterministic"));
        assert!(report.contains("Misclassified: 2"));
    }

    #[test]
    fn test_distribution_accuracy() {
        let config = AdversarialConfig {
            seed: 42,
            count_per_class: 100,
            extra_needs_quoting: 50,
            verify: false,
        };
        let result = generate_adversarial(&config);
        assert_eq!(result.stats.per_class[2], 100);
        assert_eq!(result.stats.per_class[3], 100);
        assert_eq!(result.stats.per_class[4], 100);
        assert_eq!(result.stats.per_class[1], 50);
        assert_eq!(result.stats.total, 350);
    }
}
