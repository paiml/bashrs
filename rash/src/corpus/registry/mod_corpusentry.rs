impl CorpusEntry {
    /// Create a new corpus entry with all verification flags enabled.
    ///
    /// # Expected Output Semantics (Authoring SOP)
    ///
    /// The `expected_output` is checked via **string containment** against the
    /// transpiled output (not the runtime result). Choose patterns accordingly:
    ///
    /// - **Bash**: A shell code pattern in the transpiled script, e.g., `"calc() {"`
    ///   for a function declaration or `"echo $((a + b))"` for an expression.
    /// - **Makefile**: A Makefile syntax pattern, e.g., `"CC := gcc"` or `"all: build test"`.
    /// - **Dockerfile**: A Dockerfile instruction, e.g., `"FROM alpine:3.18"` or `"WORKDIR /app"`.
    ///
    /// **Common mistake**: Using Rust runtime values (e.g., `"42"`) instead of transpiled
    /// output patterns. Always verify with `crate::transpile()` / `crate::transpile_makefile()`
    /// / `crate::transpile_dockerfile()` that the expected output appears in the actual output.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        format: CorpusFormat,
        tier: CorpusTier,
        input: impl Into<String>,
        expected_output: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            format,
            tier,
            input: input.into(),
            expected_output: expected_output.into(),
            shellcheck: matches!(format, CorpusFormat::Bash),
            deterministic: true,
            idempotent: true,
        }
    }
}

/// Registry of all corpus entries, organized by format and tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorpusRegistry {
    /// All registered corpus entries
    pub entries: Vec<CorpusEntry>,
}

impl CorpusRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the registry.
    pub fn add(&mut self, entry: CorpusEntry) {
        self.entries.push(entry);
    }

    /// Get all entries for a specific format.
    pub fn by_format(&self, format: CorpusFormat) -> Vec<&CorpusEntry> {
        self.entries.iter().filter(|e| e.format == format).collect()
    }

    /// Get all entries for a specific tier.
    pub fn by_tier(&self, tier: CorpusTier) -> Vec<&CorpusEntry> {
        self.entries.iter().filter(|e| e.tier == tier).collect()
    }

    /// Get all entries for a specific format and tier.
    pub fn by_format_and_tier(&self, format: CorpusFormat, tier: CorpusTier) -> Vec<&CorpusEntry> {
        self.entries
            .iter()
            .filter(|e| e.format == format && e.tier == tier)
            .collect()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Count entries by format.
    pub fn count_by_format(&self, format: CorpusFormat) -> usize {
        self.entries.iter().filter(|e| e.format == format).count()
    }

    /// Load the built-in Tier 1 corpus for all three formats.
    pub fn load_tier1() -> Self {
        Self::load_set(SET_TIER1)
    }

    /// Load Tier 1 + Tier 2 corpus entries (harder patterns, potential falsifiers).
    pub fn load_tier1_and_tier2() -> Self {
        Self::load_set(SET_TIER1_AND_TIER2)
    }

    /// Load tiers 1-3 for comprehensive testing.
    pub fn load_all() -> Self {
        Self::load_set(SET_ALL)
    }

    /// Load all tiers including adversarial (1-4).
    pub fn load_all_with_adversarial() -> Self {
        Self::load_set(SET_ALL_WITH_ADVERSARIAL)
    }

    /// Load the full corpus (all tiers 1-5) including production entries.
    ///
    /// The corpus ships as data (`corpus_data.jsonl`), not as generated Rust.
    /// See `corpus_data.rs` for why.
    pub fn load_full() -> Self {
        Self {
            entries: corpus_entries().iter().map(|e| e.entry.clone()).collect(),
        }
    }

}

// Corpus data loading methods (split for repository hygiene)
include!("corpus_data.rs");

#[cfg(test)]
#[path = "mod_tests_corpus_reg.rs"]
mod tests_extracted;
