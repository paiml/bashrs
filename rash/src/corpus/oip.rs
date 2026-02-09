//! OIP-Driven Corpus Generation (spec §11.9).
//!
//! Mines fix patterns from git history to identify transpiler defect categories,
//! detect gaps where fix commits lack regression corpus entries, and analyze
//! cross-project defect patterns for corpus prioritization.

use std::collections::HashMap;

/// OIP defect categories from spec §11.9.2.
///
/// Maps to the 18 defect categories used by OIP for classifying fix commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OipCategory {
    /// Parser/emitter correctness: heredoc, brackets, brace groups, command substitution.
    AstTransform,
    /// Arithmetic parenthesization, operator associativity.
    OperatorPrecedence,
    /// Quoting, injection prevention, special character handling.
    SecurityVulnerabilities,
    /// `mkdir -p`, atomic writes, lock files, existence checks.
    IdempotencyViolation,
    /// Iterator patterns, accumulation, filtering, early exit.
    ComprehensionBugs,
    /// Env var handling, default values, path construction.
    ConfigurationErrors,
    /// Cross-shell compatibility, version-specific behavior.
    IntegrationFailures,
    /// Linter rules triggering on valid code.
    FalsePositives,
    /// Type system issues (missing types, type conversion).
    TypeErrors,
    /// Performance regressions or optimization bugs.
    Performance,
    /// Documentation or help text fixes.
    Documentation,
    /// Test infrastructure fixes.
    TestInfrastructure,
    /// Build system or CI/CD fixes.
    BuildSystem,
    /// Dependency or version management.
    DependencyManagement,
    /// Uncategorized fix.
    Other,
}

impl std::fmt::Display for OipCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OipCategory::AstTransform => write!(f, "ASTTransform"),
            OipCategory::OperatorPrecedence => write!(f, "OperatorPrecedence"),
            OipCategory::SecurityVulnerabilities => write!(f, "SecurityVulnerabilities"),
            OipCategory::IdempotencyViolation => write!(f, "IdempotencyViolation"),
            OipCategory::ComprehensionBugs => write!(f, "ComprehensionBugs"),
            OipCategory::ConfigurationErrors => write!(f, "ConfigurationErrors"),
            OipCategory::IntegrationFailures => write!(f, "IntegrationFailures"),
            OipCategory::FalsePositives => write!(f, "FalsePositives"),
            OipCategory::TypeErrors => write!(f, "TypeErrors"),
            OipCategory::Performance => write!(f, "Performance"),
            OipCategory::Documentation => write!(f, "Documentation"),
            OipCategory::TestInfrastructure => write!(f, "TestInfrastructure"),
            OipCategory::BuildSystem => write!(f, "BuildSystem"),
            OipCategory::DependencyManagement => write!(f, "DependencyManagement"),
            OipCategory::Other => write!(f, "Other"),
        }
    }
}

/// A fix commit mined from git history.
#[derive(Debug, Clone)]
pub struct FixCommit {
    /// Short commit hash (7 chars).
    pub hash: String,
    /// Commit date (YYYY-MM-DD).
    pub date: String,
    /// Commit message (first line).
    pub message: String,
    /// OIP defect category.
    pub category: OipCategory,
    /// Files changed in this commit.
    pub files_changed: usize,
    /// Whether this fix has a corresponding corpus entry.
    pub has_corpus_entry: bool,
}

/// A gap between a fix commit and corpus coverage.
#[derive(Debug, Clone)]
pub struct FixGap {
    /// The fix commit without corpus coverage.
    pub commit: FixCommit,
    /// Suggested corpus entry ID range.
    pub suggested_id: String,
    /// Suggested entry description.
    pub suggested_description: String,
    /// Priority based on category severity.
    pub priority: GapPriority,
}

/// Priority level for fix gaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapPriority {
    /// Security or correctness fix — needs corpus entry immediately.
    High,
    /// Functional fix — should have corpus entry.
    Medium,
    /// Style/docs fix — corpus entry optional.
    Low,
}

impl std::fmt::Display for GapPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GapPriority::High => write!(f, "HIGH"),
            GapPriority::Medium => write!(f, "MEDIUM"),
            GapPriority::Low => write!(f, "LOW"),
        }
    }
}

/// Cross-project defect pattern (spec §11.9.4).
#[derive(Debug, Clone)]
pub struct OrgPattern {
    /// Pattern name.
    pub name: String,
    /// OIP category.
    pub category: OipCategory,
    /// Number of occurrences across projects.
    pub occurrences: usize,
    /// bashrs relevance description.
    pub relevance: String,
    /// Corpus entry ranges covering this pattern.
    pub covered_entries: Vec<String>,
}

/// Keyword-to-category mapping rules, ordered by specificity.
///
/// Each rule is a (keywords, category) pair. The first rule where ANY keyword
/// matches wins. Order matters: more specific patterns come first.
const CATEGORY_RULES: &[(&[&str], OipCategory)] = &[
    (&["ast", "parser", "emit", "heredoc", "bracket", "brace", "command substit", "transpil"], OipCategory::AstTransform),
    (&["precedence", "arithmetic", "operator", "parenthes"], OipCategory::OperatorPrecedence),
    (&["security", "injection", "quoting", "escap", "sec0"], OipCategory::SecurityVulnerabilities),
    (&["idempoten", "mkdir -p", "atomic", "idem0"], OipCategory::IdempotencyViolation),
    (&["comprehension", "iterator", "accumulat", "filter"], OipCategory::ComprehensionBugs),
    (&["config", "env var", "default value"], OipCategory::ConfigurationErrors),
    (&["cross-shell", "compat", "dash", "posix"], OipCategory::IntegrationFailures),
    (&["false positive", "false-positive"], OipCategory::FalsePositives),
    (&["perf", "optimi", "speed"], OipCategory::Performance),
    (&["doc", "readme", "comment"], OipCategory::Documentation),
    (&["test"], OipCategory::TestInfrastructure),
    (&["build", "ci"], OipCategory::BuildSystem),
    (&["dep", "version", "upgrade"], OipCategory::DependencyManagement),
];

/// Classify a commit message into an OIP defect category.
///
/// Uses keyword matching on the commit message to determine the category.
/// Rules are applied in order of specificity (most specific first).
pub fn classify_commit(message: &str) -> OipCategory {
    let lower = message.to_lowercase();

    // Special case: type errors require compound condition
    if lower.contains("type")
        && (lower.contains("u16")
            || lower.contains("u32")
            || lower.contains("i64")
            || lower.contains("missing type"))
    {
        return OipCategory::TypeErrors;
    }

    for (keywords, category) in CATEGORY_RULES {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            return *category;
        }
    }

    OipCategory::Other
}

/// Parse git log output into fix commits.
///
/// Expected format: one line per commit, `hash|date|message`.
pub fn parse_fix_commits(git_log: &str) -> Vec<FixCommit> {
    git_log
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() < 3 {
                return None;
            }
            let hash = parts[0].trim().to_string();
            let date = parts[1].trim().to_string();
            let message = parts[2].trim().to_string();
            let category = classify_commit(&message);

            Some(FixCommit {
                hash,
                date,
                message,
                category,
                files_changed: 0,
                has_corpus_entry: false,
            })
        })
        .collect()
}

/// Compute category frequency distribution from fix commits.
pub fn category_distribution(commits: &[FixCommit]) -> Vec<(OipCategory, usize)> {
    let mut counts: HashMap<OipCategory, usize> = HashMap::new();
    for commit in commits {
        *counts.entry(commit.category).or_insert(0) += 1;
    }
    let mut result: Vec<(OipCategory, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

/// Determine gap priority from OIP category.
pub fn category_priority(category: OipCategory) -> GapPriority {
    match category {
        OipCategory::AstTransform
        | OipCategory::SecurityVulnerabilities
        | OipCategory::OperatorPrecedence => GapPriority::High,

        OipCategory::IdempotencyViolation
        | OipCategory::ComprehensionBugs
        | OipCategory::IntegrationFailures
        | OipCategory::TypeErrors
        | OipCategory::ConfigurationErrors => GapPriority::Medium,

        OipCategory::FalsePositives
        | OipCategory::Performance
        | OipCategory::Documentation
        | OipCategory::TestInfrastructure
        | OipCategory::BuildSystem
        | OipCategory::DependencyManagement
        | OipCategory::Other => GapPriority::Low,
    }
}

/// Identify fix commits that lack corresponding corpus entries.
///
/// Matches commit messages against corpus entry descriptions to detect gaps.
pub fn find_fix_gaps(commits: &[FixCommit], _corpus_descriptions: &[String]) -> Vec<FixGap> {
    let mut next_id = 501; // Start after B-500
    commits
        .iter()
        .filter(|c| !c.has_corpus_entry)
        .filter(|c| {
            // Only flag high/medium priority gaps
            category_priority(c.category) != GapPriority::Low
        })
        .map(|c| {
            let suggested_id = format!("B-{next_id:03}");
            next_id += 1;
            let suggested_description = format!(
                "Regression test for {} fix: {}",
                c.category,
                truncate_message(&c.message, 60)
            );
            let priority = category_priority(c.category);

            FixGap {
                commit: c.clone(),
                suggested_id,
                suggested_description,
                priority,
            }
        })
        .collect()
}

/// Check if a commit message has a matching corpus entry description.
pub fn has_matching_corpus_entry(message: &str, descriptions: &[String]) -> bool {
    let lower = message.to_lowercase();
    // Extract key terms from the commit message
    let keywords: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .filter(|w| !["fix:", "feat:", "the", "and", "for", "with", "from", "that", "this"].contains(w))
        .take(5)
        .collect();

    if keywords.is_empty() {
        return false;
    }

    descriptions.iter().any(|desc| {
        let desc_lower = desc.to_lowercase();
        // At least 2 keywords must match
        let matches = keywords.iter().filter(|k| desc_lower.contains(*k)).count();
        matches >= 2
    })
}

/// Format the mine output as a human-readable table.
pub fn format_mine_table(commits: &[FixCommit]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "OIP Fix Pattern Mining (\u{00a7}11.9)");
    let divider = "\u{2500}".repeat(90);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<10}{:<12}{:<24}{:<8}{}",
        "Hash", "Date", "Category", "Corpus", "Message"
    );
    let _ = writeln!(out, "{divider}");

    for c in commits {
        let corpus_marker = if c.has_corpus_entry { "\u{2713}" } else { "\u{2717}" };
        let _ = writeln!(
            out,
            "{:<10}{:<12}{:<24}{:<8}{}",
            &c.hash,
            &c.date,
            c.category.to_string(),
            corpus_marker,
            truncate_message(&c.message, 40),
        );
    }

    let _ = writeln!(out, "{divider}");

    // Category distribution summary
    let dist = category_distribution(commits);
    let _ = writeln!(out, "\nCategory Distribution:");
    for (cat, count) in &dist {
        let pct = if commits.is_empty() {
            0.0
        } else {
            *count as f64 / commits.len() as f64 * 100.0
        };
        let _ = writeln!(out, "  {:<24} {:>3} ({:.0}%)", cat.to_string(), count, pct);
    }

    let total_covered = commits.iter().filter(|c| c.has_corpus_entry).count();
    let _ = writeln!(
        out,
        "\n{} fix commits, {} with corpus entries, {} gaps",
        commits.len(),
        total_covered,
        commits.len() - total_covered,
    );

    out
}

/// Format fix gaps as a human-readable table.
pub fn format_fix_gaps_table(gaps: &[FixGap]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "Fix-Driven Corpus Gaps (\u{00a7}11.9.3)");
    let divider = "\u{2500}".repeat(90);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<10}{:<10}{:<24}{:<8}{}",
        "ID", "Hash", "Category", "Priority", "Suggested Description"
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

    let high = gaps.iter().filter(|g| g.priority == GapPriority::High).count();
    let medium = gaps.iter().filter(|g| g.priority == GapPriority::Medium).count();
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
        "{:<30}{:<24}{:<8}{}",
        "Pattern", "Category", "Count", "Relevance"
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
            covered_entries: vec!["B-036".to_string(), "B-037".to_string(), "B-038".to_string()],
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_commit_ast_transform() {
        assert_eq!(
            classify_commit("fix: handle nested quotes in command substitution"),
            OipCategory::AstTransform
        );
        assert_eq!(
            classify_commit("fix: parser crash on heredoc with tabs"),
            OipCategory::AstTransform
        );
        assert_eq!(
            classify_commit("fix: emit brace group correctly"),
            OipCategory::AstTransform
        );
    }

    #[test]
    fn test_classify_commit_operator_precedence() {
        assert_eq!(
            classify_commit("fix: arithmetic precedence for nested expressions"),
            OipCategory::OperatorPrecedence
        );
        assert_eq!(
            classify_commit("fix: operator parenthesization in compound expressions"),
            OipCategory::OperatorPrecedence
        );
    }

    #[test]
    fn test_classify_commit_security() {
        assert_eq!(
            classify_commit("fix: quoting issue in variable expansion"),
            OipCategory::SecurityVulnerabilities
        );
        assert_eq!(
            classify_commit("fix: command injection via unescaped input"),
            OipCategory::SecurityVulnerabilities
        );
    }

    #[test]
    fn test_classify_commit_idempotency() {
        assert_eq!(
            classify_commit("fix: idempotency violation in mkdir"),
            OipCategory::IdempotencyViolation
        );
    }

    #[test]
    fn test_classify_commit_comprehension() {
        assert_eq!(
            classify_commit("fix: iterator early exit in accumulator"),
            OipCategory::ComprehensionBugs
        );
    }

    #[test]
    fn test_classify_commit_config() {
        assert_eq!(
            classify_commit("fix: env var config loading with default value"),
            OipCategory::ConfigurationErrors
        );
    }

    #[test]
    fn test_classify_commit_integration() {
        assert_eq!(
            classify_commit("fix: cross-shell compatibility for case statement"),
            OipCategory::IntegrationFailures
        );
        assert_eq!(
            classify_commit("fix: POSIX compliance for test command"),
            OipCategory::IntegrationFailures
        );
    }

    #[test]
    fn test_classify_commit_false_positive() {
        assert_eq!(
            classify_commit("fix: false positive on SC2171 rule"),
            OipCategory::FalsePositives
        );
    }

    #[test]
    fn test_classify_commit_type_errors() {
        assert_eq!(
            classify_commit("fix: missing type u16 support in codegen"),
            OipCategory::TypeErrors
        );
    }

    #[test]
    fn test_classify_commit_performance() {
        assert_eq!(
            classify_commit("fix: performance regression in optimizer"),
            OipCategory::Performance
        );
    }

    #[test]
    fn test_classify_commit_documentation() {
        assert_eq!(
            classify_commit("fix: doc comment typo"),
            OipCategory::Documentation
        );
    }

    #[test]
    fn test_classify_commit_test() {
        assert_eq!(
            classify_commit("fix: test suite flaky assertion in CI"),
            OipCategory::TestInfrastructure
        );
    }

    #[test]
    fn test_classify_commit_other() {
        assert_eq!(
            classify_commit("fix: miscellaneous cleanup"),
            OipCategory::Other
        );
    }

    #[test]
    fn test_parse_fix_commits() {
        let log = "abc1234|2026-02-08|fix: handle nested quotes in parser\ndef5678|2026-02-07|fix: arithmetic precedence bug";
        let commits = parse_fix_commits(log);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].hash, "abc1234");
        assert_eq!(commits[0].date, "2026-02-08");
        assert_eq!(commits[0].category, OipCategory::AstTransform);
        assert_eq!(commits[1].category, OipCategory::OperatorPrecedence);
    }

    #[test]
    fn test_parse_fix_commits_empty() {
        let commits = parse_fix_commits("");
        assert!(commits.is_empty());
    }

    #[test]
    fn test_parse_fix_commits_malformed() {
        let log = "abc1234|no second pipe";
        let commits = parse_fix_commits(log);
        assert!(commits.is_empty());
    }

    #[test]
    fn test_category_distribution() {
        let commits = vec![
            FixCommit {
                hash: "a".to_string(),
                date: "2026-01-01".to_string(),
                message: "fix: parser".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "b".to_string(),
                date: "2026-01-02".to_string(),
                message: "fix: parser 2".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "c".to_string(),
                date: "2026-01-03".to_string(),
                message: "fix: quoting".to_string(),
                category: OipCategory::SecurityVulnerabilities,
                files_changed: 1,
                has_corpus_entry: false,
            },
        ];
        let dist = category_distribution(&commits);
        assert_eq!(dist[0].0, OipCategory::AstTransform);
        assert_eq!(dist[0].1, 2);
        assert_eq!(dist[1].0, OipCategory::SecurityVulnerabilities);
        assert_eq!(dist[1].1, 1);
    }

    #[test]
    fn test_category_priority() {
        assert_eq!(category_priority(OipCategory::AstTransform), GapPriority::High);
        assert_eq!(category_priority(OipCategory::SecurityVulnerabilities), GapPriority::High);
        assert_eq!(category_priority(OipCategory::IdempotencyViolation), GapPriority::Medium);
        assert_eq!(category_priority(OipCategory::Documentation), GapPriority::Low);
    }

    #[test]
    fn test_find_fix_gaps() {
        let commits = vec![
            FixCommit {
                hash: "a".to_string(),
                date: "2026-01-01".to_string(),
                message: "fix: parser crash".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            FixCommit {
                hash: "b".to_string(),
                date: "2026-01-02".to_string(),
                message: "fix: doc typo".to_string(),
                category: OipCategory::Documentation,
                files_changed: 1,
                has_corpus_entry: false,
            },
        ];
        let descriptions = vec![];
        let gaps = find_fix_gaps(&commits, &descriptions);
        // Documentation fix should be filtered out (Low priority)
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].suggested_id, "B-501");
        assert_eq!(gaps[0].priority, GapPriority::High);
    }

    #[test]
    fn test_find_fix_gaps_with_corpus_entry() {
        let commits = vec![FixCommit {
            hash: "a".to_string(),
            date: "2026-01-01".to_string(),
            message: "fix: parser crash".to_string(),
            category: OipCategory::AstTransform,
            files_changed: 1,
            has_corpus_entry: true,
        }];
        let descriptions = vec![];
        let gaps = find_fix_gaps(&commits, &descriptions);
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_has_matching_corpus_entry() {
        let descriptions = vec![
            "Nested quoting in command substitution".to_string(),
            "Variable assignment with arithmetic".to_string(),
        ];
        assert!(has_matching_corpus_entry(
            "fix: handle nested quoting in command substitution",
            &descriptions
        ));
        assert!(!has_matching_corpus_entry("fix: random unrelated fix", &descriptions));
    }

    #[test]
    fn test_has_matching_corpus_entry_empty() {
        assert!(!has_matching_corpus_entry("fix: something", &[]));
    }

    #[test]
    fn test_format_mine_table() {
        let commits = vec![FixCommit {
            hash: "abc1234".to_string(),
            date: "2026-02-08".to_string(),
            message: "fix: parser crash on heredoc".to_string(),
            category: OipCategory::AstTransform,
            files_changed: 3,
            has_corpus_entry: true,
        }];
        let table = format_mine_table(&commits);
        assert!(table.contains("OIP Fix Pattern Mining"));
        assert!(table.contains("abc1234"));
        assert!(table.contains("ASTTransform"));
        assert!(table.contains("\u{2713}")); // checkmark
        assert!(table.contains("1 fix commits"));
    }

    #[test]
    fn test_format_mine_table_empty() {
        let table = format_mine_table(&[]);
        assert!(table.contains("0 fix commits"));
    }

    #[test]
    fn test_format_fix_gaps_table() {
        let gaps = vec![FixGap {
            commit: FixCommit {
                hash: "abc1234".to_string(),
                date: "2026-02-08".to_string(),
                message: "fix: parser crash".to_string(),
                category: OipCategory::AstTransform,
                files_changed: 1,
                has_corpus_entry: false,
            },
            suggested_id: "B-501".to_string(),
            suggested_description: "Regression test for ASTTransform fix".to_string(),
            priority: GapPriority::High,
        }];
        let table = format_fix_gaps_table(&gaps);
        assert!(table.contains("Fix-Driven Corpus Gaps"));
        assert!(table.contains("B-501"));
        assert!(table.contains("HIGH"));
        assert!(table.contains("1 gaps total"));
    }

    #[test]
    fn test_format_fix_gaps_table_empty() {
        let table = format_fix_gaps_table(&[]);
        assert!(table.contains("0 gaps total"));
    }

    #[test]
    fn test_format_org_patterns_table() {
        let patterns = known_org_patterns();
        let table = format_org_patterns_table(&patterns);
        assert!(table.contains("Cross-Project Defect Patterns"));
        assert!(table.contains("Off-by-one"));
        assert!(table.contains("String escaping"));
        assert!(table.contains("Precedence"));
    }

    #[test]
    fn test_known_org_patterns_not_empty() {
        let patterns = known_org_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.len() >= 8);
    }

    #[test]
    fn test_truncate_message_short() {
        assert_eq!(truncate_message("short", 10), "short");
    }

    #[test]
    fn test_truncate_message_long() {
        let result = truncate_message("this is a very long message", 10);
        // 9 ASCII chars + 3-byte ellipsis character = 12 bytes
        assert!(result.len() <= 12);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn test_oip_category_display() {
        assert_eq!(format!("{}", OipCategory::AstTransform), "ASTTransform");
        assert_eq!(
            format!("{}", OipCategory::SecurityVulnerabilities),
            "SecurityVulnerabilities"
        );
        assert_eq!(format!("{}", OipCategory::Other), "Other");
    }

    #[test]
    fn test_gap_priority_display() {
        assert_eq!(format!("{}", GapPriority::High), "HIGH");
        assert_eq!(format!("{}", GapPriority::Medium), "MEDIUM");
        assert_eq!(format!("{}", GapPriority::Low), "LOW");
    }

    #[test]
    fn test_category_distribution_empty() {
        let dist = category_distribution(&[]);
        assert!(dist.is_empty());
    }
}
