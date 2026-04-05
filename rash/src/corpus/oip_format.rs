/// Format fix gaps as a human-readable table.
pub fn format_fix_gaps_table(gaps: &[FixGap]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "Fix-Driven Corpus Gaps (\u{00a7}11.9.3)");
    let divider = "\u{2500}".repeat(90);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<10}{:<10}{:<24}{:<8}Suggested Description",
        "ID", "Hash", "Category", "Priority"
    );
    let _ = writeln!(out, "{divider}");

    for gap in gaps {
        let _ = writeln!(
            out,
            "{:<10}{:<10}{:<24}{:<8}{}",
            &gap.suggested_id,
            &gap.commit.hash,
            gap.commit.category.to_string(),
            gap.priority.to_string(),
            truncate_message(&gap.suggested_description, 40),
        );
    }

    let _ = writeln!(out, "{divider}");

    let high = gaps
        .iter()
        .filter(|g| g.priority == GapPriority::High)
        .count();
    let medium = gaps
        .iter()
        .filter(|g| g.priority == GapPriority::Medium)
        .count();
    let _ = writeln!(
        out,
        "\n{} gaps total: {} HIGH priority, {} MEDIUM priority",
        gaps.len(),
        high,
        medium,
    );

    out
}

/// Format cross-project org patterns as a human-readable table.
pub fn format_org_patterns_table(patterns: &[OrgPattern]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "Cross-Project Defect Patterns (\u{00a7}11.9.4)");
    let divider = "\u{2500}".repeat(90);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<30}{:<24}{:<8}Relevance",
        "Pattern", "Category", "Count"
    );
    let _ = writeln!(out, "{divider}");

    for p in patterns {
        let _ = writeln!(
            out,
            "{:<30}{:<24}{:<8}{}",
            truncate_message(&p.name, 28),
            p.category.to_string(),
            p.occurrences,
            truncate_message(&p.relevance, 30),
        );
    }

    let _ = writeln!(out, "{divider}");
    let _ = writeln!(out, "{} cross-project patterns identified", patterns.len());

    out
}

/// Get the well-known cross-project patterns from spec §11.9.4.
pub fn known_org_patterns() -> Vec<OrgPattern> {
    vec![
        OrgPattern {
            name: "Off-by-one in range iteration".to_string(),
            category: OipCategory::ComprehensionBugs,
            occurrences: 12,
            relevance: "`for i in $(seq)` boundary values".to_string(),
            covered_entries: vec!["B-005".to_string(), "B-346".to_string()],
        },
        OrgPattern {
            name: "String escaping in code gen".to_string(),
            category: OipCategory::AstTransform,
            occurrences: 24,
            relevance: "Quote handling in shell output".to_string(),
            covered_entries: vec!["B-321".to_string(), "B-336".to_string()],
        },
        OrgPattern {
            name: "Precedence in expression trees".to_string(),
            category: OipCategory::OperatorPrecedence,
            occurrences: 8,
            relevance: "Arithmetic parenthesization".to_string(),
            covered_entries: vec!["B-331".to_string(), "B-335".to_string()],
        },
        OrgPattern {
            name: "Missing error path handling".to_string(),
            category: OipCategory::ConfigurationErrors,
            occurrences: 15,
            relevance: "Shell `set -e` interaction".to_string(),
            covered_entries: vec!["B-055".to_string()],
        },
        OrgPattern {
            name: "Type widening/narrowing errors".to_string(),
            category: OipCategory::TypeErrors,
            occurrences: 6,
            relevance: "u16/i64 type support in emitter".to_string(),
            covered_entries: vec!["D-006".to_string()],
        },
        OrgPattern {
            name: "Macro expansion failures".to_string(),
            category: OipCategory::AstTransform,
            occurrences: 18,
            relevance: "format!/vec! macro handling".to_string(),
            covered_entries: vec!["B-171".to_string()],
        },
        OrgPattern {
            name: "Compound assignment desugar".to_string(),
            category: OipCategory::AstTransform,
            occurrences: 9,
            relevance: "+=/-=/*= operator lowering".to_string(),
            covered_entries: vec![
                "B-036".to_string(),
                "B-037".to_string(),
                "B-038".to_string(),
            ],
        },
        OrgPattern {
            name: "Cross-shell output divergence".to_string(),
            category: OipCategory::IntegrationFailures,
            occurrences: 7,
            relevance: "sh vs dash behavior differences".to_string(),
            covered_entries: vec!["B-143".to_string()],
        },
    ]
}

/// Truncate a message to a maximum length, adding ellipsis.
fn truncate_message(message: &str, max_len: usize) -> String {
    if message.len() <= max_len {
        message.to_string()
    } else {
        format!("{}\u{2026}", &message[..max_len.saturating_sub(1)])
    }
}


include!("oip_tests_classify.rs");
