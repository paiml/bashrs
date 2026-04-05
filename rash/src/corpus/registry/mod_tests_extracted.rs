#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_CORPUS_REG_001_tier_weights() {
        assert!((CorpusTier::Trivial.weight() - 1.0).abs() < f64::EPSILON);
        assert!((CorpusTier::Standard.weight() - 1.5).abs() < f64::EPSILON);
        assert!((CorpusTier::Complex.weight() - 2.0).abs() < f64::EPSILON);
        assert!((CorpusTier::Adversarial.weight() - 2.5).abs() < f64::EPSILON);
        assert!((CorpusTier::Production.weight() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_REG_002_grade_from_score() {
        assert_eq!(Grade::from_score(100.0), Grade::APlus);
        assert_eq!(Grade::from_score(97.0), Grade::APlus);
        assert_eq!(Grade::from_score(95.0), Grade::A);
        assert_eq!(Grade::from_score(85.0), Grade::B);
        assert_eq!(Grade::from_score(75.0), Grade::C);
        assert_eq!(Grade::from_score(65.0), Grade::D);
        assert_eq!(Grade::from_score(50.0), Grade::F);
    }

    #[test]
    fn test_CORPUS_REG_003_load_tier1_all_formats() {
        let registry = CorpusRegistry::load_tier1();
        assert_eq!(registry.count_by_format(CorpusFormat::Bash), 10);
        assert_eq!(registry.count_by_format(CorpusFormat::Makefile), 10);
        assert_eq!(registry.count_by_format(CorpusFormat::Dockerfile), 10);
        assert_eq!(registry.len(), 30);
    }

    #[test]
    fn test_CORPUS_REG_004_filter_by_format() {
        let registry = CorpusRegistry::load_tier1();
        let bash_entries = registry.by_format(CorpusFormat::Bash);
        assert_eq!(bash_entries.len(), 10);
        for entry in &bash_entries {
            assert_eq!(entry.format, CorpusFormat::Bash);
        }
    }

    #[test]
    fn test_CORPUS_REG_005_filter_by_tier() {
        let registry = CorpusRegistry::load_tier1();
        let tier1 = registry.by_tier(CorpusTier::Trivial);
        assert_eq!(tier1.len(), 30); // All tier 1
    }

    #[test]
    fn test_CORPUS_REG_006_entry_defaults() {
        let entry = CorpusEntry::new(
            "T-001",
            "test",
            "Test entry",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() {}",
            "#!/bin/sh",
        );
        assert!(entry.shellcheck);
        assert!(entry.deterministic);
        assert!(entry.idempotent);
    }

    #[test]
    fn test_CORPUS_REG_007_makefile_entry_no_shellcheck() {
        let entry = CorpusEntry::new(
            "M-001",
            "test",
            "Test entry",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "fn main() {}",
            "CC := gcc",
        );
        assert!(!entry.shellcheck);
    }

    #[test]
    fn test_CORPUS_REG_008_grade_display() {
        assert_eq!(format!("{}", Grade::APlus), "A+");
        assert_eq!(format!("{}", Grade::F), "F");
    }

    #[test]
    fn test_CORPUS_REG_009_format_display() {
        assert_eq!(format!("{}", CorpusFormat::Bash), "bash");
        assert_eq!(format!("{}", CorpusFormat::Makefile), "makefile");
        assert_eq!(format!("{}", CorpusFormat::Dockerfile), "dockerfile");
    }

    #[test]
    fn test_CORPUS_REG_010_tier_target_rates() {
        assert!((CorpusTier::Trivial.target_rate() - 1.0).abs() < f64::EPSILON);
        assert!((CorpusTier::Standard.target_rate() - 0.99).abs() < f64::EPSILON);
        assert!((CorpusTier::Adversarial.target_rate() - 0.95).abs() < f64::EPSILON);
    }

    /// Poka-Yoke: every corpus entry ID must be unique across the full registry.
    /// Prevents the expansion 113/114 duplicate ID class of defect (Five Whys root cause).
    #[test]
    fn test_CORPUS_REG_011_no_duplicate_ids() {
        let registry = CorpusRegistry::load_full();
        let mut seen = std::collections::HashSet::new();
        for entry in &registry.entries {
            assert!(
                seen.insert(&entry.id),
                "Duplicate corpus entry ID detected: {} (slug: {}). \
                 Every entry must have a unique ID.",
                entry.id,
                entry.name
            );
        }
    }

    /// Verify total entry count is within expected bounds (regression guard).
    /// Catches accidental deletions or massive unintended additions.
    #[test]
    fn test_CORPUS_REG_012_entry_count_bounds() {
        let registry = CorpusRegistry::load_full();
        // Lower bound: we know we have at least 16,676 entries (through expansion 204)
        assert!(
            registry.len() >= 16_676,
            "Corpus entry count {} is below expected minimum 16,676",
            registry.len()
        );
    }

    /// Genchi Genbutsu: verify all expansion 204 entries transpile successfully,
    /// contain their expected output, and produce deterministic output.
    /// (Go and see, Toyota Way principle.)
    #[test]
    fn test_CORPUS_REG_013_expansion204_transpile_containment() {
        let registry = CorpusRegistry::load_full();
        let config = crate::models::Config::default();

        // Collect expansion 204 IDs: B-16617..B-16636, M-16637..M-16656, D-16657..D-16676
        let exp204_entries: Vec<&CorpusEntry> = registry
            .entries
            .iter()
            .filter(|e| {
                let id_num: Option<u32> = e.id[2..].parse().ok();
                matches!(id_num, Some(n) if (16617..=16676).contains(&n))
            })
            .collect();

        assert_eq!(
            exp204_entries.len(),
            60,
            "Expected 60 expansion 204 entries, found {}",
            exp204_entries.len()
        );

        let mut failures = Vec::new();

        let transpile_entry = |entry: &CorpusEntry, cfg: &crate::models::Config| match entry.format
        {
            CorpusFormat::Bash => crate::transpile(&entry.input, cfg),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, cfg),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(&entry.input, cfg),
        };

        for entry in &exp204_entries {
            let result = transpile_entry(entry, &config);

            match result {
                Ok(output) => {
                    // B_L1: Containment check
                    if !output.contains(&entry.expected_output) {
                        failures.push(format!(
                            "{} ({}): containment failed — expected '{}' not in output",
                            entry.id, entry.name, entry.expected_output
                        ));
                        continue;
                    }
                    // E: Determinism check — transpile again, compare
                    if let Ok(output2) = transpile_entry(entry, &config) {
                        if output != output2 {
                            failures.push(format!(
                                "{} ({}): non-deterministic — two runs differ",
                                entry.id, entry.name
                            ));
                        }
                    }
                }
                Err(e) => {
                    failures.push(format!(
                        "{} ({}): transpilation failed: {}",
                        entry.id, entry.name, e
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "Expansion 204 Genchi Genbutsu: {} of 60 entries failed:\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}
