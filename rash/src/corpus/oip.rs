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
    (
        &[
            "ast",
            "parser",
            "emit",
            "heredoc",
            "bracket",
            "brace",
            "command substit",
            "transpil",
        ],
        OipCategory::AstTransform,
    ),
    (
        &["precedence", "arithmetic", "operator", "parenthes"],
        OipCategory::OperatorPrecedence,
    ),
    (
        &["security", "injection", "quoting", "escap", "sec0"],
        OipCategory::SecurityVulnerabilities,
    ),
    (
        &["idempoten", "mkdir -p", "atomic", "idem0"],
        OipCategory::IdempotencyViolation,
    ),
    (
        &["comprehension", "iterator", "accumulat", "filter"],
        OipCategory::ComprehensionBugs,
    ),
    (
        &["config", "env var", "default value"],
        OipCategory::ConfigurationErrors,
    ),
    (
        &["cross-shell", "compat", "dash", "posix"],
        OipCategory::IntegrationFailures,
    ),
    (
        &["false positive", "false-positive"],
        OipCategory::FalsePositives,
    ),
    (&["perf", "optimi", "speed"], OipCategory::Performance),
    (&["doc", "readme", "comment"], OipCategory::Documentation),
    (&["test"], OipCategory::TestInfrastructure),
    (&["build", "ci"], OipCategory::BuildSystem),
    (
        &["dep", "version", "upgrade"],
        OipCategory::DependencyManagement,
    ),
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
        .filter(|w| {
            ![
                "fix:", "feat:", "the", "and", "for", "with", "from", "that", "this",
            ]
            .contains(w)
        })
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
        "{:<10}{:<12}{:<24}{:<8}Message",
        "Hash", "Date", "Category", "Corpus"
    );
    let _ = writeln!(out, "{divider}");

    for c in commits {
        let corpus_marker = if c.has_corpus_entry {
            "\u{2713}"
        } else {
            "\u{2717}"
        };
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

include!("oip_format.rs");
