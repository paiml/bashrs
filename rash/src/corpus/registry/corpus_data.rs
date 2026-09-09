// Corpus data, shipped as data rather than as generated Rust.
//
// # Why this is a `.jsonl` file and not Rust source
//
// The 17,942-entry corpus used to live here as **9,406 generated `load_*`
// functions across 277,890 lines (9.7 MB) of Rust**. That shape cost 7.3 GB
// peak RSS to compile and, linked into 113 workspace test binaries, blew the
// CI test job's 60-minute timeout (#284, PR #285). It is also what made the
// corpus tempting to stub out in the first place — a 9.7 MB source file tanks
// every file-level quality metric the repo tracks, and the "fix" for that was
// to replace the loaders with no-op stubs, which silently emptied the corpus
// for five months.
//
// Encoded as data, `rustc` sees one string literal instead of 9,406 functions.
// The entries are parsed once on first use and cached.
//
// To regenerate after adding entries, see `docs/audits/impl-PMAT-245-receipt.md`.

/// Membership in [`CorpusRegistry::load_all`].
const SET_ALL: u8 = 1;
/// Membership in [`CorpusRegistry::load_all_with_adversarial`].
const SET_ALL_WITH_ADVERSARIAL: u8 = 2;
/// Membership in [`CorpusRegistry::load_tier1`].
const SET_TIER1: u8 = 4;
/// Membership in [`CorpusRegistry::load_tier1_and_tier2`].
const SET_TIER1_AND_TIER2: u8 = 8;

/// Every corpus entry, one JSON object per line, compiled into the binary.
const CORPUS_JSONL: &str = include_str!("corpus_data.jsonl");

/// A corpus entry plus the bitmask of which named loads include it.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct StoredEntry {
    #[serde(flatten)]
    pub(crate) entry: CorpusEntry,
    pub(crate) sets: u8,
}

/// Parse the corpus once, then hand out references to it.
///
/// # Panics
///
/// Panics if `corpus_data.jsonl` does not parse. That file is compiled in, so a
/// failure here is a build-time defect in the data, not a runtime condition —
/// failing loudly is correct, and is what #284 did not do.
pub(crate) fn corpus_entries() -> &'static [StoredEntry] {
    static ENTRIES: std::sync::OnceLock<Vec<StoredEntry>> = std::sync::OnceLock::new();
    ENTRIES.get_or_init(|| {
        CORPUS_JSONL
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, line)| {
                serde_json::from_str(line).unwrap_or_else(|e| {
                    panic!("corpus_data.jsonl line {} is not a valid entry: {e}", i + 1)
                })
            })
            .collect()
    })
}

impl CorpusRegistry {
    /// Load every entry whose membership bitmask includes `set`.
    pub(crate) fn load_set(set: u8) -> Self {
        Self {
            entries: corpus_entries()
                .iter()
                .filter(|e| e.sets & set != 0)
                .map(|e| e.entry.clone())
                .collect(),
        }
    }
}
