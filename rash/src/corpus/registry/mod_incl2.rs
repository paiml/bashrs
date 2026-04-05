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
        let mut registry = Self::new();
        registry.load_tier1_bash();
        registry.load_tier1_makefile();
        registry.load_tier1_dockerfile();
        registry
    }

    /// Load Tier 1 + Tier 2 corpus entries (harder patterns, potential falsifiers).
    pub fn load_tier1_and_tier2() -> Self {
        let mut registry = Self::load_tier1();
        registry.load_tier2_bash();
        registry.load_tier2_makefile();
        registry.load_tier2_dockerfile();
        registry
    }

    /// Load tiers 1-3 for comprehensive testing.
    pub fn load_all() -> Self {
        let mut registry = Self::load_tier1_and_tier2();
        registry.load_tier3_bash();
        registry.load_tier3_makefile();
        registry.load_tier3_dockerfile();
        registry
    }

    /// Load all tiers including adversarial (1-4).
    pub fn load_all_with_adversarial() -> Self {
        let mut registry = Self::load_all();
        registry.load_tier4_bash();
        registry.load_tier4_makefile();
        registry.load_tier4_dockerfile();
        registry
    }

    /// Load the full corpus (all tiers 1-5) including production entries.
    pub fn load_full() -> Self {
        let mut registry = Self::load_all_with_adversarial();
        registry.load_tier5_bash();
        registry.load_tier5_makefile();
        registry.load_tier5_dockerfile();
        registry.load_expansion_bash();
        registry.load_expansion_makefile();
        registry.load_expansion_dockerfile();
        registry.load_expansion2_bash();
        registry.load_expansion2_makefile();
        registry.load_expansion2_dockerfile();
        registry.load_expansion3_bash();
        registry.load_expansion3_makefile();
        registry.load_expansion3_dockerfile();
        registry.load_expansion4_bash();
        registry.load_expansion4_makefile();
        registry.load_expansion4_dockerfile();
        registry.load_expansion5_bash();
        registry.load_expansion5_makefile();
        registry.load_expansion5_dockerfile();
        registry.load_expansion6_bash();
        registry.load_expansion7_bash();
        registry.load_expansion8_bash();
        registry.load_expansion9_bash();
        registry.load_expansion10_bash();
        registry.load_expansion6_makefile();
        registry.load_expansion6_dockerfile();
        registry.load_expansion7_makefile();
        registry.load_expansion7_dockerfile();
        registry.load_expansion8_makefile();
        registry.load_expansion8_dockerfile();
        registry.load_expansion11_bash();
        registry.load_expansion9_dockerfile();
        registry.load_expansion12_bash();
        registry.load_expansion9_makefile();
        registry.load_expansion10_dockerfile();
        registry.load_expansion13_bash();
        registry.load_expansion14_bash();
        registry.load_expansion15_bash();
        registry.load_expansion10_makefile();
        registry.load_expansion11_dockerfile();
        registry.load_expansion16_bash();
        registry.load_expansion17_bash();
        registry.load_expansion12_dockerfile();
        registry.load_expansion18_bash();
        registry.load_expansion11_makefile();
        registry.load_expansion13_dockerfile();
        registry.load_expansion19_bash();
        registry.load_expansion12_makefile();
        registry.load_expansion14_dockerfile();
        registry.load_expansion20_bash();
        registry.load_expansion13_makefile();
        registry.load_expansion15_dockerfile();
        registry.load_expansion21_bash();
        registry.load_expansion14_makefile();
        registry.load_expansion22_bash();
        registry.load_expansion15_makefile();
        registry.load_expansion16_dockerfile();
        registry.load_expansion23_bash();
        registry.load_expansion16_makefile_ext();
        registry.load_expansion24_bash();
        registry.load_expansion25_bash();
        registry.load_expansion26_bash();
        registry.load_expansion17_makefile();
        registry.load_expansion17_dockerfile();
        registry.load_expansion27_bash();
        registry.load_expansion18_makefile();
        registry.load_expansion18_dockerfile();
        registry.load_expansion28_bash();
        registry.load_expansion19_makefile();
        registry.load_expansion19_dockerfile();
        registry.load_expansion29_bash();
        registry.load_expansion30_bash();
        registry.load_expansion20_makefile();
        registry.load_expansion20_dockerfile();
        registry.load_expansion31_bash();
        registry.load_expansion32_bash();
        registry.load_expansion33_bash();
        registry.load_expansion21_makefile();
        registry.load_expansion21_dockerfile();
        registry.load_expansion34_bash();
        registry.load_expansion22_makefile();
        registry.load_expansion22_dockerfile();
        registry.load_expansion35_bash();
        registry.load_expansion23_makefile();
        registry.load_expansion23_dockerfile();
        registry.load_expansion36_bash();
        registry.load_expansion24_makefile();
        registry.load_expansion24_dockerfile();
        registry.load_expansion37_bash();
        registry.load_expansion25_makefile();
        registry.load_expansion25_dockerfile();
        registry.load_expansion38_bash();
        registry.load_expansion26_makefile();
        registry.load_expansion26_dockerfile();
        registry.load_expansion39_bash();
        registry.load_expansion27_makefile();
        registry.load_expansion27_dockerfile();
        registry.load_expansion40_bash();
        registry.load_expansion28_makefile();
        registry.load_expansion28_dockerfile();
        registry.load_expansion41_bash();
        registry.load_expansion29_makefile();
        registry.load_expansion29_dockerfile();
        registry.load_expansion42_bash();
        registry.load_expansion30_makefile();
        registry.load_expansion30_dockerfile();
        registry.load_expansion43_bash();
        registry.load_expansion31_makefile();
        registry.load_expansion31_dockerfile();
        registry.load_expansion44_bash();
        registry.load_expansion32_makefile();
        registry.load_expansion32_dockerfile();
        registry.load_expansion45_bash();
        registry.load_expansion33_makefile();
        registry.load_expansion33_dockerfile();
        registry.load_expansion46_bash();
        registry.load_expansion34_makefile();
        registry.load_expansion34_dockerfile();
        registry.load_expansion47_bash();
        registry.load_expansion35_makefile();
        registry.load_expansion35_dockerfile();
        registry.load_expansion48_bash();
        registry.load_expansion36_makefile();
        registry.load_expansion36_dockerfile();
        registry.load_expansion49_bash();
        registry.load_expansion37_makefile();
        registry.load_expansion37_dockerfile();
        registry.load_expansion50_bash();
        registry.load_expansion38_makefile();
        registry.load_expansion38_dockerfile();
        registry.load_expansion51_bash();
        registry.load_expansion39_makefile();
        registry.load_expansion39_dockerfile();
        registry.load_expansion52_bash();
        registry.load_expansion40_makefile();
        registry.load_expansion40_dockerfile();
        registry.load_expansion53_bash();
        registry.load_expansion41_makefile();
        registry.load_expansion41_dockerfile();
        registry.load_expansion54_bash();
        registry.load_expansion42_makefile();
        registry.load_expansion42_dockerfile();
        registry.load_expansion55_bash();
        registry.load_expansion43_makefile();
        registry.load_expansion43_dockerfile();
        registry.load_expansion56_bash();
        registry.load_expansion44_makefile();
        registry.load_expansion44_dockerfile();
        registry.load_expansion57_bash();
        registry.load_expansion45_makefile();
        registry.load_expansion45_dockerfile();
        registry.load_expansion58_bash();
        registry.load_expansion46_makefile();
        registry.load_expansion46_dockerfile();
        registry.load_expansion59_bash();
        registry.load_expansion47_makefile();
        registry.load_expansion47_dockerfile();
        registry.load_expansion60_bash();
        registry.load_expansion61_bash();
        registry.load_expansion62_bash();
        registry.load_expansion63_bash();
        registry.load_expansion64_bash();
        registry.load_expansion65_bash();
        registry.load_expansion65a_bash();
        registry.load_expansion66_bash();
        registry.load_expansion67_bash();
        registry.load_expansion68_bash();
        registry.load_expansion69_bash();
        registry.load_expansion70_bash();
        registry.load_expansion71_bash();
        registry.load_expansion72_bash();
        registry.load_expansion73_bash();
        registry.load_expansion74_bash();
        registry.load_expansion75_bash();
        registry.load_expansion76_bash();
        registry.load_expansion77_bash();
        registry.load_expansion78_bash();
        registry.load_expansion79_bash();
        registry.load_expansion80_bash();
        registry.load_expansion81_bash();
        registry.load_expansion82_bash();
        registry.load_expansion83_bash();
        registry.load_expansion84_bash();
        registry.load_expansion85_bash();
        registry.load_expansion86_bash();
        registry.load_expansion87_bash();
        registry.load_expansion88_bash();
        registry.load_expansion89_bash();
        registry.load_expansion90_bash();
        registry.load_expansion91_bash();
        registry.load_expansion92_bash();
        registry.load_expansion93_bash();
        registry.load_expansion94_bash();
        registry.load_expansion95_bash();
        registry.load_expansion96_bash();
        registry.load_expansion97_bash();
        registry.load_expansion98_bash();
        registry.load_expansion99_bash();
        registry.load_expansion100_bash();
        registry.load_expansion101_bash();
        registry.load_expansion102_bash();
        registry.load_expansion103_bash();
        registry.load_expansion104_bash();
        registry.load_expansion105_bash();
        registry.load_expansion106_bash();
        registry.load_expansion107_bash();
        registry.load_expansion108_bash();
        registry.load_expansion109_bash();
        registry.load_expansion110_bash();
        registry.load_expansion111_bash();
        registry.load_expansion112_bash();
        registry.load_expansion113_bash();
        registry.load_expansion114_bash();
        registry.load_expansion115_bash();
        registry.load_expansion116_bash();
        registry.load_expansion117_bash();
        registry.load_expansion118_bash();
        registry.load_expansion119_bash();
        registry.load_expansion120_bash();
        registry.load_expansion121_bash();
        registry.load_expansion122_bash();
        registry.load_expansion123_bash();
        registry.load_expansion124_bash();
        registry.load_expansion125_bash();
        registry.load_expansion126_bash();
        registry.load_expansion127_bash();
        registry.load_expansion128_bash();
        registry.load_expansion129_bash();
        registry.load_expansion130_bash();
        registry.load_expansion131_bash();
        registry.load_expansion132_bash();
        registry.load_expansion133_bash();
        registry.load_expansion134_bash();
        registry.load_expansion135_bash();
        registry.load_expansion136_bash();
        registry.load_expansion137_bash();
        registry.load_expansion138_bash();
        registry.load_expansion139_bash();
        registry.load_expansion140_bash();
        registry.load_expansion141_bash();
        registry.load_expansion142_bash();
        registry.load_expansion143_bash();
        registry.load_expansion144_bash();
        registry.load_expansion145_bash();
        registry.load_expansion146_bash();
        registry.load_expansion147_bash();
        registry.load_expansion148_bash();
        registry.load_expansion149_bash();
        registry.load_expansion150_bash();
        registry.load_expansion151_bash();
        registry.load_expansion152_bash();
        registry.load_expansion153_bash();
        registry.load_expansion154_bash();
        registry.load_expansion155_bash();
        registry.load_expansion156_bash();
        registry.load_expansion157_bash();
        registry.load_expansion158_bash();
        registry.load_expansion159_bash();
        registry.load_expansion160_bash();
        registry.load_expansion161_bash();
        registry.load_expansion162_bash();
        registry.load_expansion163_bash();
        registry.load_expansion164_bash();
        registry.load_expansion165_bash();
        registry.load_expansion166_bash();
        registry.load_expansion167_bash();
        registry.load_expansion168_bash();
        registry.load_expansion169_bash();
        registry.load_expansion170_bash();
        registry.load_expansion171_bash();
        registry.load_expansion172_bash();
        registry.load_expansion173_bash();
        registry.load_expansion174_bash();
        registry.load_expansion175_bash();
        registry.load_expansion176_bash();
        registry.load_expansion177_bash();
        registry.load_expansion178_bash();
        registry.load_expansion179_bash();
        registry.load_expansion179_makefile();
        registry.load_expansion179_dockerfile();
        registry.load_expansion180_bash();
        registry.load_expansion180_makefile();
        registry.load_expansion180_dockerfile();
        registry.load_expansion181_bash();
        registry.load_expansion181_makefile();
        registry.load_expansion181_dockerfile();
        registry.load_expansion182_bash();
        registry.load_expansion182_makefile();
        registry.load_expansion182_dockerfile();
        registry.load_expansion183_bash();
        registry.load_expansion183_makefile();
        registry.load_expansion183_dockerfile();
        registry.load_expansion184_bash();
        registry.load_expansion184_makefile();
        registry.load_expansion184_dockerfile();
        registry.load_expansion185_bash();
        registry.load_expansion185_makefile();
        registry.load_expansion185_dockerfile();
        registry.load_expansion186_bash();
        registry.load_expansion186_makefile();
        registry.load_expansion186_dockerfile();
        registry.load_expansion187_bash();
        registry.load_expansion187_makefile();
        registry.load_expansion187_dockerfile();
        registry.load_expansion188_bash();
        registry.load_expansion188_makefile();
        registry.load_expansion188_dockerfile();
        registry.load_expansion189_bash();
        registry.load_expansion189_makefile();
        registry.load_expansion189_dockerfile();
        registry.load_expansion190_bash();
        registry.load_expansion190_makefile();
        registry.load_expansion190_dockerfile();
        registry.load_expansion191_bash();
        registry.load_expansion191_makefile();
        registry.load_expansion191_dockerfile();
        registry.load_expansion192_bash();
        registry.load_expansion192_makefile();
        registry.load_expansion192_dockerfile();
        registry.load_expansion193_bash();
        registry.load_expansion193_makefile();
        registry.load_expansion193_dockerfile();
        registry.load_expansion194_bash();
        registry.load_expansion194_makefile();
        registry.load_expansion194_dockerfile();
        registry.load_expansion195_bash();
        registry.load_expansion195_makefile();
        registry.load_expansion195_dockerfile();
        registry.load_expansion196_bash();
        registry.load_expansion196_makefile();
        registry.load_expansion196_dockerfile();
        registry.load_expansion197_bash();
        registry.load_expansion197_makefile();
        registry.load_expansion197_dockerfile();
        registry.load_expansion198_bash();
        registry.load_expansion198_makefile();
        registry.load_expansion198_dockerfile();
        registry.load_expansion199_bash();
        registry.load_expansion199_makefile();
        registry.load_expansion199_dockerfile();
        registry.load_expansion200_bash();
        registry.load_expansion200_makefile();
        registry.load_expansion200_dockerfile();
        registry.load_expansion201_bash();
        registry.load_expansion201_makefile();
        registry.load_expansion201_dockerfile();
        registry.load_expansion202_bash();
        registry.load_expansion202_makefile();
        registry.load_expansion202_dockerfile();
        registry.load_expansion203_bash();
        registry.load_expansion203_makefile();
        registry.load_expansion203_dockerfile();
        registry.load_expansion204_bash();
        registry.load_expansion204_makefile();
        registry.load_expansion204_dockerfile();
        registry
    }
}

// Corpus data loading methods (split for repository hygiene)
include!("corpus_data.rs");

#[cfg(test)]
#[path = "mod_tests_extracted.rs"]
mod tests_extracted;
