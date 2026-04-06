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
#[path = "domain_categories_tests_make_entry.rs"]
mod tests_extracted;
