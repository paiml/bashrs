//! Domain-Specific Corpus Categories (§11.11)
//!
//! Classifies corpus entries into 8 domain categories (A-H) based on
//! entry ID ranges. Provides coverage analysis and quality requirement
//! matrices for each category.
//!
//! Categories:
//! - A: Shell Configuration Files (bashrc/zshrc/profile)
//! - B: Shell One-Liners
//! - C: Provability Corpus (restricted Rust → verified shell)
//! - D: Unix Tool Patterns
//! - E: Language Integration One-Liners
//! - F: System Tooling (cron, daemons)
//! - G: Coreutils Reimplementation
//! - H: Regex Pattern Corpus

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};
use crate::corpus::runner::{CorpusResult, CorpusScore};

/// Domain category for a corpus entry (§11.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainCategory {
    /// A: Shell Configuration Files (B-371..B-380)
    ShellConfig,
    /// B: Shell One-Liners (B-381..B-390)
    OneLiners,
    /// C: Provability Corpus (B-391..B-400)
    Provability,
    /// D: Unix Tool Patterns (B-401..B-410)
    UnixTools,
    /// E: Language Integration (B-411..B-420)
    LangIntegration,
    /// F: System Tooling (B-421..B-430)
    SystemTooling,
    /// G: Coreutils Reimplementation (B-431..B-460)
    Coreutils,
    /// H: Regex Patterns (B-461..B-490)
    RegexPatterns,
    /// General: entries outside domain-specific ranges
    General,
}

impl DomainCategory {
    /// Short label for the category
    pub fn label(self) -> &'static str {
        match self {
            Self::ShellConfig => "A: Shell Config",
            Self::OneLiners => "B: One-Liners",
            Self::Provability => "C: Provability",
            Self::UnixTools => "D: Unix Tools",
            Self::LangIntegration => "E: Lang Integration",
            Self::SystemTooling => "F: System Tooling",
            Self::Coreutils => "G: Coreutils",
            Self::RegexPatterns => "H: Regex Patterns",
            Self::General => "General",
        }
    }

    /// Entry ID range for this category
    pub fn range(self) -> Option<(u32, u32)> {
        match self {
            Self::ShellConfig => Some((371, 380)),
            Self::OneLiners => Some((381, 390)),
            Self::Provability => Some((391, 400)),
            Self::UnixTools => Some((401, 410)),
            Self::LangIntegration => Some((411, 420)),
            Self::SystemTooling => Some((421, 430)),
            Self::Coreutils => Some((431, 460)),
            Self::RegexPatterns => Some((461, 490)),
            Self::General => None,
        }
    }

    /// Maximum entries in this category per spec
    pub fn capacity(self) -> usize {
        match self {
            Self::ShellConfig => 10,
            Self::OneLiners => 10,
            Self::Provability => 10,
            Self::UnixTools => 10,
            Self::LangIntegration => 10,
            Self::SystemTooling => 10,
            Self::Coreutils => 30,
            Self::RegexPatterns => 30,
            Self::General => 0,
        }
    }

    /// All domain-specific categories (excluding General)
    pub fn all_specific() -> &'static [DomainCategory] {
        &[
            Self::ShellConfig,
            Self::OneLiners,
            Self::Provability,
            Self::UnixTools,
            Self::LangIntegration,
            Self::SystemTooling,
            Self::Coreutils,
            Self::RegexPatterns,
        ]
    }
}

/// Quality requirement for the cross-category matrix (§11.11.9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityReq {
    Required,
    NotApplicable,
}

/// Cross-category quality requirements matrix entry
#[derive(Debug, Clone)]
pub struct QualityMatrixRow {
    pub property: &'static str,
    pub requirements: Vec<(DomainCategory, QualityReq)>,
}

/// Per-category statistics
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: DomainCategory,
    pub total: usize,
    pub capacity: usize,
    pub passed: usize,
    pub failed: usize,
    pub fill_pct: f64,
    pub pass_rate: f64,
}

/// Parse the numeric part of a Bash entry ID (e.g., "B-371" → 371)
fn parse_bash_id_num(id: &str) -> Option<u32> {
    id.strip_prefix("B-")?.parse::<u32>().ok()
}

/// Classify a single entry into a domain category
pub fn classify_entry(entry: &CorpusEntry) -> DomainCategory {
    if entry.format != CorpusFormat::Bash {
        return DomainCategory::General;
    }

    let num = match parse_bash_id_num(&entry.id) {
        Some(n) => n,
        None => return DomainCategory::General,
    };

    for cat in DomainCategory::all_specific() {
        if let Some((lo, hi)) = cat.range() {
            if num >= lo && num <= hi {
                return *cat;
            }
        }
    }

    DomainCategory::General
}

/// Classify all entries and compute per-category stats
pub fn categorize_corpus(
    registry: &CorpusRegistry,
    results: &[CorpusResult],
) -> Vec<CategoryStats> {
    let result_map: std::collections::HashMap<&str, &CorpusResult> =
        results.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut stats_map: std::collections::HashMap<DomainCategory, (usize, usize, usize)> =
        std::collections::HashMap::new();

    for entry in &registry.entries {
        let cat = classify_entry(entry);
        let (total, passed, failed) = stats_map.entry(cat).or_insert((0, 0, 0));
        *total += 1;
        if let Some(result) = result_map.get(entry.id.as_str()) {
            if result.transpiled {
                *passed += 1;
            } else {
                *failed += 1;
            }
        }
    }

    // Build stats for all specific categories (even if empty)
    let mut all_stats: Vec<CategoryStats> = DomainCategory::all_specific()
        .iter()
        .map(|cat| {
            let (total, passed, failed) = stats_map.get(cat).copied().unwrap_or((0, 0, 0));
            let capacity = cat.capacity();
            let fill_pct = if capacity > 0 {
                (total as f64 / capacity as f64) * 100.0
            } else {
                0.0
            };
            let pass_rate = if total > 0 {
                (passed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            CategoryStats {
                category: *cat,
                total,
                capacity,
                passed,
                failed,
                fill_pct,
                pass_rate,
            }
        })
        .collect();

    // Add General category
    if let Some(&(total, passed, failed)) = stats_map.get(&DomainCategory::General) {
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        all_stats.push(CategoryStats {
            category: DomainCategory::General,
            total,
            capacity: 0,
            passed,
            failed,
            fill_pct: 0.0,
            pass_rate,
        });
    }

    all_stats
}

/// Format the domain categories report
pub fn format_categories_report(stats: &[CategoryStats]) -> String {
    let mut out = String::new();

    out.push_str("Domain-Specific Corpus Categories (\u{00a7}11.11)\n");
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );
    out.push_str(&format!(
        "{:<22} {:>8} {:>8} {:>8} {:>8} {:>10}\n",
        "Category", "Entries", "Capacity", "Fill %", "Passed", "Pass Rate"
    ));
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );

    let mut domain_total = 0usize;
    let mut domain_passed = 0usize;
    let mut domain_capacity = 0usize;

    for s in stats {
        if s.category == DomainCategory::General {
            continue;
        }
        let fill_str = if s.capacity > 0 {
            format!("{:.0}%", s.fill_pct)
        } else {
            "-".to_string()
        };
        let rate_str = if s.total > 0 {
            format!("{:.1}%", s.pass_rate)
        } else {
            "-".to_string()
        };
        out.push_str(&format!(
            "{:<22} {:>8} {:>8} {:>8} {:>8} {:>10}\n",
            s.category.label(),
            s.total,
            s.capacity,
            fill_str,
            s.passed,
            rate_str,
        ));
        domain_total += s.total;
        domain_passed += s.passed;
        domain_capacity += s.capacity;
    }

    // General row
    if let Some(gen) = stats.iter().find(|s| s.category == DomainCategory::General) {
        let rate_str = if gen.total > 0 {
            format!("{:.1}%", gen.pass_rate)
        } else {
            "-".to_string()
        };
        out.push_str(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
             \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
        );
        out.push_str(&format!(
            "{:<22} {:>8} {:>8} {:>8} {:>8} {:>10}\n",
            "General", gen.total, "-", "-", gen.passed, rate_str,
        ));
    }

    // Summary
    let total_entries = domain_total
        + stats
            .iter()
            .find(|s| s.category == DomainCategory::General)
            .map_or(0, |s| s.total);
    let total_passed = domain_passed
        + stats
            .iter()
            .find(|s| s.category == DomainCategory::General)
            .map_or(0, |s| s.passed);
    let fill_pct = if domain_capacity > 0 {
        (domain_total as f64 / domain_capacity as f64) * 100.0
    } else {
        0.0
    };

    out.push_str(&format!(
        "\nTotal: {} entries ({} domain-specific, {:.0}% of capacity {})\n",
        total_entries, domain_total, fill_pct, domain_capacity,
    ));
    out.push_str(&format!(
        "Pass rate: {}/{} ({:.1}%)\n",
        total_passed,
        total_entries,
        if total_entries > 0 {
            total_passed as f64 / total_entries as f64 * 100.0
        } else {
            0.0
        }
    ));

    out
}

/// Format domain coverage gap analysis
pub fn format_domain_coverage(stats: &[CategoryStats], score: &CorpusScore) -> String {
    let mut out = String::new();

    out.push_str("Domain Coverage Analysis (\u{00a7}11.11)\n");
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );

    // Overall score context
    out.push_str(&format!(
        "Corpus Score: {:.1}/100 ({})\n\n",
        score.score, score.grade
    ));

    // Per-category coverage with gap identification
    out.push_str(&format!(
        "{:<22} {:>6}/{:<6} {:>7}  {}\n",
        "Category", "Have", "Need", "Fill", "Status"
    ));
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );

    let mut gaps = Vec::new();

    for s in stats {
        if s.category == DomainCategory::General {
            continue;
        }
        let fill_str = format!("{:.0}%", s.fill_pct);
        let status = coverage_status(s);
        out.push_str(&format!(
            "{:<22} {:>6}/{:<6} {:>7}  {}\n",
            s.category.label(),
            s.total,
            s.capacity,
            fill_str,
            status,
        ));
        if s.total < s.capacity {
            gaps.push((s.category, s.capacity - s.total));
        }
    }

    // Gap summary
    if gaps.is_empty() {
        out.push_str("\nAll domain categories fully populated.\n");
    } else {
        let total_gap: usize = gaps.iter().map(|(_, g)| g).sum();
        out.push_str(&format!(
            "\nCoverage Gaps: {} entries needed across {} categories\n",
            total_gap,
            gaps.len()
        ));
        for (cat, gap) in &gaps {
            if let Some((lo, hi)) = cat.range() {
                out.push_str(&format!(
                    "  {} : {} entries needed (B-{}..B-{})\n",
                    cat.label(),
                    gap,
                    lo,
                    hi
                ));
            }
        }
    }

    out
}

/// Determine coverage status string for a category
fn coverage_status(s: &CategoryStats) -> &'static str {
    if s.total == 0 {
        "EMPTY"
    } else if s.total >= s.capacity && s.failed == 0 {
        "COMPLETE"
    } else if s.total >= s.capacity {
        "FULL (has failures)"
    } else if s.fill_pct >= 50.0 {
        "PARTIAL"
    } else {
        "SPARSE"
    }
}

/// Quality requirements matrix properties (from §11.11.9)
const QUALITY_PROPERTIES: &[(&str, [QualityReq; 8])] = &[
    (
        "Idempotent",
        [
            QualityReq::Required,      // A: Config
            QualityReq::NotApplicable, // B: One-liner
            QualityReq::Required,      // C: Provability
            QualityReq::NotApplicable, // D: Unix tools
            QualityReq::NotApplicable, // E: Lang integ
            QualityReq::Required,      // F: System
            QualityReq::Required,      // G: Coreutils
            QualityReq::Required,      // H: Regex
        ],
    ),
    (
        "POSIX",
        [
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
        ],
    ),
    (
        "Deterministic",
        [
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
        ],
    ),
    (
        "Miri-verifiable",
        [
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
        ],
    ),
    (
        "Cross-shell",
        [
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
        ],
    ),
    (
        "Shellcheck-clean",
        [
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::Required,
        ],
    ),
    (
        "Pipeline-safe",
        [
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::Required,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::Required,
        ],
    ),
    (
        "1:1 parity",
        [
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
        ],
    ),
    (
        "Signal-aware",
        [
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
        ],
    ),
    (
        "Terminates",
        [
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::NotApplicable,
            QualityReq::Required,
        ],
    ),
];

/// Format the cross-category quality requirements matrix (§11.11.9)
pub fn format_quality_matrix(stats: &[CategoryStats]) -> String {
    let mut out = String::new();
    let cats = DomainCategory::all_specific();
    let cat_labels: Vec<&str> = cats
        .iter()
        .map(|c| match c {
            DomainCategory::ShellConfig => "Config",
            DomainCategory::OneLiners => "1-Liner",
            DomainCategory::Provability => "Prove",
            DomainCategory::UnixTools => "Unix",
            DomainCategory::LangIntegration => "Lang",
            DomainCategory::SystemTooling => "System",
            DomainCategory::Coreutils => "Core",
            DomainCategory::RegexPatterns => "Regex",
            DomainCategory::General => "Gen",
        })
        .collect();

    out.push_str("Cross-Category Quality Matrix (\u{00a7}11.11.9)\n");
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );

    // Header row
    out.push_str(&format!("{:<18}", "Property"));
    for label in &cat_labels {
        out.push_str(&format!(" {:>8}", label));
    }
    out.push('\n');
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );

    // Data rows
    for (prop, reqs) in QUALITY_PROPERTIES {
        out.push_str(&format!("{:<18}", prop));
        for req in reqs {
            let sym = match req {
                QualityReq::Required => "REQ",
                QualityReq::NotApplicable => "N/A",
            };
            out.push_str(&format!(" {:>8}", sym));
        }
        out.push('\n');
    }

    // Summary: count required per category
    out.push_str(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n",
    );
    out.push_str(&format!("{:<18}", "Required count"));
    for i in 0..8 {
        let count = QUALITY_PROPERTIES
            .iter()
            .filter(|(_, reqs)| reqs[i] == QualityReq::Required)
            .count();
        out.push_str(&format!(" {:>8}", format!("{}/10", count)));
    }
    out.push('\n');

    // Entry count per category
    out.push_str(&format!("{:<18}", "Entries"));
    for cat in cats {
        let count = stats
            .iter()
            .find(|s| s.category == *cat)
            .map_or(0, |s| s.total);
        out.push_str(&format!(" {:>8}", count));
    }
    out.push('\n');

    out
}

#[cfg(test)]
#[path = "domain_categories_tests_extracted.rs"]
mod tests_extracted;
