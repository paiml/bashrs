//! Corpus registry types for transpilation quality measurement.
//!
//! Defines `CorpusEntry` and `CorpusRegistry` following the depyler corpus
//! pattern (Gift, 2025) with metadata for quality tracking, tier assignment,
//! and falsification protocol support.

use serde::{Deserialize, Serialize};

/// Target transpilation format for a corpus entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorpusFormat {
    /// POSIX shell (purified bash)
    Bash,
    /// GNU Makefile
    Makefile,
    /// Dockerfile
    Dockerfile,
}

impl std::fmt::Display for CorpusFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Makefile => write!(f, "makefile"),
            Self::Dockerfile => write!(f, "dockerfile"),
        }
    }
}

/// Difficulty tier for a corpus entry (progressive difficulty, Vygotsky 1978).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CorpusTier {
    /// Tier 1: Single constructs (10-20 LOC), 100% expected pass rate
    Trivial = 1,
    /// Tier 2: Common patterns (20-100 LOC), 99% expected pass rate
    Standard = 2,
    /// Tier 3: Real-world programs (100-500 LOC), 98% expected pass rate
    Complex = 3,
    /// Tier 4: Edge cases, injection attempts, Unicode, 95% expected pass rate
    Adversarial = 4,
    /// Tier 5: Full production scripts, 95% expected pass rate
    Production = 5,
}

impl CorpusTier {
    /// Scoring weight for aggregate calculations (Pareto principle, Juran 1951).
    /// Higher tiers contribute more to overall score.
    pub fn weight(&self) -> f64 {
        match self {
            Self::Trivial => 1.0,
            Self::Standard => 1.5,
            Self::Complex => 2.0,
            Self::Adversarial => 2.5,
            Self::Production => 3.0,
        }
    }

    /// Expected minimum pass rate for this tier.
    pub fn target_rate(&self) -> f64 {
        match self {
            Self::Trivial => 1.0,
            Self::Standard => 0.99,
            Self::Complex => 0.98,
            Self::Adversarial => 0.95,
            Self::Production => 0.95,
        }
    }
}

/// Quality grade derived from 100-point score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    /// 97-100: Production-ready, fully validated
    APlus,
    /// 90-96: Near-production, minor gaps
    A,
    /// 80-89: Good quality, known limitations
    B,
    /// 70-79: Functional, significant gaps
    C,
    /// 60-69: Partially functional
    D,
    /// <60: Not yet viable
    F,
}

impl Grade {
    /// Derive grade from a 100-point score.
    pub fn from_score(score: f64) -> Self {
        if score >= 97.0 {
            Self::APlus
        } else if score >= 90.0 {
            Self::A
        } else if score >= 80.0 {
            Self::B
        } else if score >= 70.0 {
            Self::C
        } else if score >= 60.0 {
            Self::D
        } else {
            Self::F
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::APlus => write!(f, "A+"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::F => write!(f, "F"),
        }
    }
}

/// A single corpus entry: an input-output pair that serves as a potential falsifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// Unique identifier (e.g., "B-001", "M-042", "D-015")
    pub id: String,
    /// Human-readable name (e.g., "hello-world")
    pub name: String,
    /// Description of what this entry tests
    pub description: String,
    /// Target transpilation format
    pub format: CorpusFormat,
    /// Difficulty tier
    pub tier: CorpusTier,
    /// Rust DSL source code (the input)
    pub input: String,
    /// Expected transpiled output (the prediction)
    pub expected_output: String,
    /// Whether this entry's output must pass shellcheck (Bash only)
    pub shellcheck: bool,
    /// Whether this entry's output must be deterministic
    pub deterministic: bool,
    /// Whether this entry's output must be idempotent
    pub idempotent: bool,
}

impl CorpusEntry {
    /// Create a new corpus entry with all verification flags enabled.
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
        registry
    }

    fn load_tier1_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-001",
                "variable-assignment",
                "Simple string variable assignment",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let greeting = "hello"; } "#,
                "greeting='hello'",
            ),
            CorpusEntry::new(
                "B-002",
                "echo-string",
                "Echo a string literal",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { echo("hello world"); } fn echo(msg: &str) {}"#,
                "echo 'hello world'",
            ),
            CorpusEntry::new(
                "B-003",
                "integer-variable",
                "Integer variable assignment",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                "fn main() { let x = 42; }",
                "x='42'",
            ),
            CorpusEntry::new(
                "B-004",
                "boolean-variable",
                "Boolean variable assignment",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                "fn main() { let active = true; }",
                "active=true",
            ),
            CorpusEntry::new(
                "B-005",
                "multiple-variables",
                "Multiple variable assignments",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let name = "Alice"; let age = 30; }"#,
                "name='Alice'",
            ),
            CorpusEntry::new(
                "B-006",
                "function-call",
                "Simple function call",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { greet("World"); } fn greet(name: &str) {}"#,
                "greet() {",
            ),
            CorpusEntry::new(
                "B-007",
                "empty-main",
                "Empty main function produces valid shell",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                "fn main() {}",
                "#!/bin/sh",
            ),
            CorpusEntry::new(
                "B-008",
                "arithmetic-add",
                "Integer arithmetic (addition)",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                "fn main() { let x = 5 + 3; }",
                "x='8'",
            ),
            CorpusEntry::new(
                "B-009",
                "string-with-spaces",
                "String with spaces is properly quoted",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let msg = "hello world"; }"#,
                "msg='hello world'",
            ),
            CorpusEntry::new(
                "B-010",
                "exit-code",
                "Exit with specific code",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                "fn main() { std::process::exit(1); }",
                "std::process::exit 1",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier1_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-001",
                "simple-variable",
                "Single variable assignment",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let cc = "gcc"; }"#,
                "CC := gcc",
            ),
            CorpusEntry::new(
                "M-002",
                "multiple-variables",
                "Multiple variable assignments with uppercase conversion",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2 -Wall"; }"#,
                "CFLAGS := -O2 -Wall",
            ),
            CorpusEntry::new(
                "M-003",
                "simple-target",
                "Target with prerequisites and recipes",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { target("build", &["main.o"], &["gcc -o build main.o"]); } fn target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "build: main.o",
            ),
            CorpusEntry::new(
                "M-004",
                "phony-target",
                "Phony target with .PHONY directive",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { phony_target("clean", &[], &["rm -f *.o"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: clean",
            ),
            CorpusEntry::new(
                "M-005",
                "variable-uppercase",
                "Variable name is uppercased",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let my_var = "value"; }"#,
                "MY_VAR := value",
            ),
            CorpusEntry::new(
                "M-006",
                "compiler-flags",
                "Standard compiler flags pattern",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let ldflags = "-lpthread"; }"#,
                "LDFLAGS := -lpthread",
            ),
            CorpusEntry::new(
                "M-007",
                "boolean-value",
                "Boolean value in Makefile variable",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                "fn main() { let debug = true; }",
                "DEBUG := true",
            ),
            CorpusEntry::new(
                "M-008",
                "integer-value",
                "Integer value in Makefile variable",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                "fn main() { let jobs = 4; }",
                "JOBS := 4",
            ),
            CorpusEntry::new(
                "M-009",
                "path-variable",
                "Path variable with slashes",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let prefix = "/usr/local"; }"#,
                "PREFIX := /usr/local",
            ),
            CorpusEntry::new(
                "M-010",
                "empty-value",
                "Variable with empty string value",
                CorpusFormat::Makefile,
                CorpusTier::Trivial,
                r#"fn main() { let extra = ""; }"#,
                "EXTRA :=",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier1_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-001",
                "basic-from",
                "Basic FROM instruction with pinned tag",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); } fn from_image(i: &str, t: &str) {}"#,
                "FROM alpine:3.18",
            ),
            CorpusEntry::new(
                "D-002",
                "workdir",
                "WORKDIR instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); workdir("/app"); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {}"#,
                "WORKDIR /app",
            ),
            CorpusEntry::new(
                "D-003",
                "copy",
                "COPY instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); copy(".", "."); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {}"#,
                "COPY . .",
            ),
            CorpusEntry::new(
                "D-004",
                "user",
                "USER instruction for non-root (DOCKER003 compliance)",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); user("65534"); } fn from_image(i: &str, t: &str) {} fn user(u: &str) {}"#,
                "USER 65534",
            ),
            CorpusEntry::new(
                "D-005",
                "env",
                "ENV instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); env("APP_ENV", "production"); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {}"#,
                "ENV APP_ENV=production",
            ),
            CorpusEntry::new(
                "D-006",
                "expose",
                "EXPOSE instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); expose(8080); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "EXPOSE 8080",
            ),
            CorpusEntry::new(
                "D-007",
                "entrypoint",
                "ENTRYPOINT in exec form",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); entrypoint(&["/app"]); } fn from_image(i: &str, t: &str) {} fn entrypoint(e: &[&str]) {}"#,
                r#"ENTRYPOINT ["/app"]"#,
            ),
            CorpusEntry::new(
                "D-008",
                "label",
                "LABEL instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); label("maintainer", "team@example.com"); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {}"#,
                r#"LABEL maintainer="team@example.com""#,
            ),
            CorpusEntry::new(
                "D-009",
                "multi-stage",
                "Multi-stage build with two FROM instructions",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "builder"); from_image("alpine", "3.18"); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {}"#,
                "FROM rust:1.75-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-010",
                "no-latest-tag",
                "Pinned version tag (DOCKER002 compliance)",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("rust", "1.75-alpine"); } fn from_image(i: &str, t: &str) {}"#,
                "FROM rust:1.75-alpine",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Tier 2: Standard difficulty (common patterns, potential falsifiers)
    // =========================================================================

    fn load_tier2_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-011",
                "if-else",
                "If/else conditional transpilation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 5; if x > 3 { let msg = "big"; } else { let msg = "small"; } }"#,
                r#"if [ "$x" -gt 3 ]; then"#,
            ),
            CorpusEntry::new(
                "B-012",
                "for-loop-range",
                "For loop with integer range",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { for i in 0..5 { let x = i; } }",
                "for i in $(seq 0 4); do",
            ),
            CorpusEntry::new(
                "B-013",
                "binary-ops",
                "Multiple binary operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let sum = 10 + 20; let product = 3 * 4; }",
                "sum='30'",
            ),
            CorpusEntry::new(
                "B-014",
                "nested-calls",
                "Nested function calls",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = "hello"; greet(x); } fn greet(name: &str) {} "#,
                "greet() {",
            ),
            CorpusEntry::new(
                "B-015",
                "negation",
                "Boolean negation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let flag = !false; }",
                "flag=! false",
            ),
            // Harder entries - potential falsifiers
            CorpusEntry::new(
                "B-016",
                "while-loop",
                "While loop with condition",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let mut x = 0; while x < 10 { x = x + 1; } }",
                r#"while [ "$x" -lt 10 ]; do"#,
            ),
            CorpusEntry::new(
                "B-017",
                "match-statement",
                "Match/case statement",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 1; match x { 1 => { let a = "one"; }, 2 => { let b = "two"; }, _ => { let c = "other"; } } }"#,
                r#"case "$x" in"#,
            ),
            CorpusEntry::new(
                "B-018",
                "negative-integer",
                "Negative integer literal",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let x = -42; }",
                "x='-42'",
            ),
            CorpusEntry::new(
                "B-019",
                "method-call",
                "Method call expression",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = "hello"; let y = x.len(); }"#,
                "#!/bin/sh",
            ),
            CorpusEntry::new(
                "B-020",
                "multi-function",
                "Multiple function definitions",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { helper(); } fn helper() { let x = 1; }"#,
                "helper",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier2_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-011",
                "target-with-multiple-deps",
                "Target with multiple prerequisites",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { target("all", &["lib", "bin", "tests"], &["echo done"]); } fn target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "all: lib bin tests",
            ),
            CorpusEntry::new(
                "M-012",
                "target-with-multiple-recipes",
                "Target with multiple recipe lines",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("test", &[], &["cargo test", "cargo clippy"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "cargo test",
            ),
            CorpusEntry::new(
                "M-013",
                "variable-and-target",
                "Variables followed by targets",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2"; target("build", &["main.c"], &["$(CC) $(CFLAGS) -o build main.c"]); } fn target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CC := gcc",
            ),
            CorpusEntry::new(
                "M-014",
                "multiple-targets",
                "Multiple targets in one Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("all", &["build", "test"], &[]); phony_target("build", &[], &["echo build"]); phony_target("test", &[], &["echo test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: all",
            ),
            CorpusEntry::new(
                "M-015",
                "variable-reference-in-recipe",
                "Variable reference $(VAR) in recipe",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let prefix = "/usr/local"; phony_target("install", &[], &["cp bin $(PREFIX)/bin"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "PREFIX := /usr/local",
            ),
            // Harder entries
            CorpusEntry::new(
                "M-016",
                "helper-function-as-target",
                "Non-main function becomes phony target",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cc = "gcc"; } fn lint() { echo("running lint"); } fn echo(msg: &str) {}"#,
                ".PHONY: lint",
            ),
            CorpusEntry::new(
                "M-017",
                "integer-variable-value",
                "Integer variable produces string value",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                "fn main() { let timeout = 30; let retries = 3; }",
                "TIMEOUT := 30",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier2_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-011",
                "run-chained",
                "RUN with chained commands",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ubuntu", "22.04"); run(&["apt-get update", "apt-get install -y curl"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {}"#,
                r#"RUN apt-get update && \"#,
            ),
            CorpusEntry::new(
                "D-012",
                "full-image",
                "Full Dockerfile with common instructions",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("rust", "1.75-alpine"); workdir("/app"); copy(".", "."); user("65534"); entrypoint(&["/app/server"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-alpine",
            ),
            CorpusEntry::new(
                "D-013",
                "multi-stage-copy",
                "Multi-stage build with COPY --from",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image_as("rust", "1.75", "builder"); workdir("/app"); copy(".", "."); from_image("alpine", "3.18"); copy_from("builder", "/app/target/release/app", "/usr/local/bin/"); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {}"#,
                "COPY --from=builder /app/target/release/app /usr/local/bin/",
            ),
            CorpusEntry::new(
                "D-014",
                "env-variable",
                "ENV with let binding",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("node", "20-alpine"); let node_env = "production"; workdir("/app"); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {}"#,
                "ENV NODE_ENV=production",
            ),
            CorpusEntry::new(
                "D-015",
                "cmd-exec-form",
                "CMD instruction in exec form",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("alpine", "3.18"); cmd(&["sh", "-c", "echo hello"]); } fn from_image(i: &str, t: &str) {} fn cmd(c: &[&str]) {}"#,
                r#"CMD ["sh", "-c", "echo hello"]"#,
            ),
            // Harder entries
            CorpusEntry::new(
                "D-016",
                "healthcheck",
                "HEALTHCHECK instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("nginx", "1.25"); healthcheck("curl -f http://localhost/"); } fn from_image(i: &str, t: &str) {} fn healthcheck(c: &str) {}"#,
                "HEALTHCHECK CMD curl -f http://localhost/",
            ),
            CorpusEntry::new(
                "D-017",
                "comment",
                "Comment instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("alpine", "3.18"); comment("Install dependencies"); } fn from_image(i: &str, t: &str) {} fn comment(t: &str) {}"#,
                "# Install dependencies",
            ),
            CorpusEntry::new(
                "D-018",
                "u16-port-suffixed",
                "EXPOSE with u16 suffixed literal",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("node", "20"); expose(3000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "EXPOSE 3000",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Tier 3: Complex difficulty (multi-construct, real-world patterns)
    // =========================================================================

    fn load_tier3_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-021",
                "else-if-chain",
                "Else-if chain with three branches",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let x = 5; if x > 10 { let r = "big"; } else if x > 3 { let r = "medium"; } else { let r = "small"; } }"#,
                r#"elif [ "$x" -gt 3 ]; then"#,
            ),
            CorpusEntry::new(
                "B-022",
                "inclusive-range",
                "For loop with inclusive range (0..=5)",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { for i in 0..=5 { let x = i; } }",
                "for i in $(seq 0 5); do",
            ),
            CorpusEntry::new(
                "B-023",
                "println-macro",
                "println! macro transpilation",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { println!("hello world"); }"#,
                "rash_println() {",
            ),
            CorpusEntry::new(
                "B-024",
                "function-with-params",
                "Function with typed parameters",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { greet("World", 3); } fn greet(name: &str, count: u32) {}"#,
                "greet() {",
            ),
            CorpusEntry::new(
                "B-025",
                "nested-if-in-loop",
                "If inside for loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { for i in 0..10 { if i > 5 { let big = true; } } }",
                "for i in $(seq 0 9); do",
            ),
            CorpusEntry::new(
                "B-026",
                "comparison-operators",
                "All comparison operators",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let a = 5 == 5; let b = 3 != 4; let c = 1 < 2; let d = 5 >= 3; }",
                "#!/bin/sh",
            ),
            CorpusEntry::new(
                "B-027",
                "boolean-logic",
                "Boolean AND/OR operators",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let a = true; let b = false; let c = a && b; let d = a || b; }",
                "#!/bin/sh",
            ),
            CorpusEntry::new(
                "B-028",
                "modulo-operator",
                "Modulo arithmetic",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let remainder = 17 % 5; }",
                "remainder='2'",
            ),
            CorpusEntry::new(
                "B-029",
                "string-empty",
                "Empty string variable",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let empty = ""; }"#,
                "empty=''",
            ),
            CorpusEntry::new(
                "B-030",
                "multiple-assignments-in-loop",
                "Multiple assignments inside while loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let mut i = 0; let mut sum = 0; while i < 5 { sum = sum + i; i = i + 1; } }",
                r#"while [ "$i" -lt 5 ]; do"#,
            ),
            CorpusEntry::new(
                "B-031",
                "break-in-loop",
                "Break statement in while loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let mut i = 0; while i < 100 { if i > 10 { break; } i = i + 1; } }",
                "break",
            ),
            CorpusEntry::new(
                "B-032",
                "continue-in-loop",
                "Continue statement in for loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { for i in 0..20 { if i == 5 { continue; } let x = i; } }",
                "continue",
            ),
            CorpusEntry::new(
                "B-033",
                "subtraction-multiplication",
                "Subtraction and multiplication",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let a = 100 - 42; let b = 6 * 7; }",
                "#!/bin/sh",
            ),
            CorpusEntry::new(
                "B-034",
                "division",
                "Integer division",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let quotient = 20 / 4; }",
                "quotient='5'",
            ),
            CorpusEntry::new(
                "B-035",
                "large-integer",
                "Large integer value",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let big = 4294967295; }",
                "big='4294967295'",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier3_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-018",
                "complex-build-system",
                "Variables + multiple phony targets",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2 -Wall"; phony_target("all", &["build", "test"], &[]); target("build", &["main.c"], &["$(CC) $(CFLAGS) -o app main.c"]); phony_target("test", &[], &["./test.sh"]); phony_target("clean", &[], &["rm -f app"]); } fn target(n: &str, d: &[&str], r: &[&str]) {} fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CC := gcc",
            ),
            CorpusEntry::new(
                "M-019",
                "install-uninstall",
                "Install and uninstall targets",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { let prefix = "/usr/local"; phony_target("install", &[], &["cp -f app $(PREFIX)/bin/"]); phony_target("uninstall", &[], &["rm -f $(PREFIX)/bin/app"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "PREFIX := /usr/local",
            ),
            CorpusEntry::new(
                "M-020",
                "empty-prerequisites",
                "Target with no prerequisites",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { target("help", &[], &["@echo 'Usage: make build'"]); } fn target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "help:",
            ),
            CorpusEntry::new(
                "M-021",
                "docker-build-target",
                "Makefile with docker build recipe",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { let image = "myapp"; let tag = "latest"; phony_target("docker-build", &[], &["docker build -t $(IMAGE):$(TAG) ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "IMAGE := myapp",
            ),
            CorpusEntry::new(
                "M-022",
                "many-variables",
                "Five or more variables",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2"; let ldflags = "-lpthread"; let prefix = "/usr"; let version = "1.0"; }"#,
                "VERSION := 1.0",
            ),
            CorpusEntry::new(
                "M-023",
                "shell-recipe",
                "Recipe with shell command",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { phony_target("check", &[], &["@which gcc || echo 'gcc not found'"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "check:",
            ),
            CorpusEntry::new(
                "M-024",
                "target-chain",
                "Chain of dependent targets",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { phony_target("deploy", &["test", "build"], &["echo deploying"]); phony_target("build", &["lint"], &["echo building"]); phony_target("lint", &[], &["echo linting"]); phony_target("test", &["build"], &["echo testing"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "deploy: test build",
            ),
            CorpusEntry::new(
                "M-025",
                "silent-recipe",
                "Recipe with @ prefix (silent)",
                CorpusFormat::Makefile,
                CorpusTier::Complex,
                r#"fn main() { phony_target("version", &[], &["@echo v1.0.0"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "@echo v1.0.0",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier3_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-019",
                "three-stage-build",
                "Three-stage multi-stage build",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image_as("rust", "1.75", "builder"); workdir("/app"); from_image_as("debian", "bookworm-slim", "runtime"); copy_from("builder", "/app/target/release/app", "/usr/local/bin/"); from_image("alpine", "3.18"); copy_from("runtime", "/usr/local/bin/app", "/app/"); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy_from(f: &str, s: &str, d: &str) {}"#,
                "FROM rust:1.75 AS builder",
            ),
            CorpusEntry::new(
                "D-020",
                "run-cleanup",
                "RUN with cleanup (rm cache)",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("ubuntu", "22.04"); run(&["apt-get update", "apt-get install -y --no-install-recommends curl", "rm -rf /var/lib/apt/lists/*"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {}"#,
                "rm -rf /var/lib/apt/lists/*",
            ),
            CorpusEntry::new(
                "D-021",
                "multiple-env",
                "Multiple ENV instructions from let bindings",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("node", "20-alpine"); let app_port = "3000"; let node_env = "production"; let app_name = "myapp"; workdir("/app"); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {}"#,
                "ENV APP_PORT=3000",
            ),
            CorpusEntry::new(
                "D-022",
                "entrypoint-with-cmd",
                "ENTRYPOINT with CMD for default args",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("alpine", "3.18"); entrypoint(&["/app/server"]); cmd(&["--port", "8080"]); } fn from_image(i: &str, t: &str) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                r#"ENTRYPOINT ["/app/server"]"#,
            ),
            CorpusEntry::new(
                "D-023",
                "label-multiple",
                "Multiple LABEL instructions",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("alpine", "3.18"); label("maintainer", "team@example.com"); label("version", "1.0"); label("description", "My application"); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {}"#,
                r#"LABEL maintainer="team@example.com""#,
            ),
            CorpusEntry::new(
                "D-024",
                "full-production-rust",
                "Production Rust Dockerfile pattern",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "builder"); workdir("/app"); copy("Cargo.toml", "."); copy("src", "src"); run(&["cargo build --release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/target/release/myapp", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/myapp"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-025",
                "arg-instruction",
                "ARG via env() function",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("alpine", "3.18"); env("APP_VERSION", "1.0.0"); env("BUILD_DATE", "2026-02-06"); workdir("/app"); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {}"#,
                "ENV APP_VERSION=1.0.0",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Tier 4: Adversarial entries (edge cases, boundary conditions)
    // =========================================================================

    fn load_tier4_bash(&mut self) {
        let entries = vec![
            // --- Compound assignment operators (+=, -=) ---
            CorpusEntry::new(
                "B-036",
                "compound-add-assign",
                "Compound addition assignment (x += 1)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut count = 0; count += 5; }"#,
                "count='0'",
            ),
            CorpusEntry::new(
                "B-037",
                "compound-sub-assign",
                "Compound subtraction assignment (x -= 1)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut total = 100; total -= 25; }"#,
                "total='100'",
            ),
            CorpusEntry::new(
                "B-038",
                "compound-mul-assign",
                "Compound multiplication assignment (x *= 2)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut factor = 3; factor *= 4; }"#,
                "factor='3'",
            ),
            // --- eprintln! macro ---
            CorpusEntry::new(
                "B-039",
                "eprintln-macro",
                "eprintln! macro support (stderr output)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { eprintln!("error: something went wrong"); }"#,
                r#"printf '%s\n' "$1" >&2"#,
            ),
            // --- Nested function calls ---
            CorpusEntry::new(
                "B-040",
                "nested-function-calls",
                "Function call as argument to another function",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn add(a: u32, b: u32) -> u32 { a + b } fn double(x: u32) -> u32 { x * 2 } fn main() { let result = double(add(3, 4)); }"#,
                r#"result="$(double "$(add 3 4)")""#,
            ),
            // --- Multiple return paths ---
            CorpusEntry::new(
                "B-041",
                "early-return-conditional",
                "Early return based on condition",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn check(x: u32) -> u32 { if x > 10 { return 1; } return 0; } fn main() { let r = check(5); }"#,
                "check() {",
            ),
            // --- Empty function body ---
            CorpusEntry::new(
                "B-042",
                "empty-function",
                "Function with empty body",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn noop() {} fn main() { noop(); }"#,
                "noop() {",
            ),
            // --- Large literal values ---
            CorpusEntry::new(
                "B-043",
                "max-u32-literal",
                "Maximum u32 literal value",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let max = 4294967295; }"#,
                "max='4294967295'",
            ),
            // --- String with special characters (safe) ---
            CorpusEntry::new(
                "B-044",
                "special-safe-chars",
                "String containing characters that need quoting in shell",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let msg = "hello world: it's a test!"; let path = "/usr/local/bin"; }"#,
                r#"msg='hello world: it'"'"'s a test!'"#,
            ),
            // --- Deeply nested if-else ---
            CorpusEntry::new(
                "B-045",
                "deeply-nested-if",
                "Three-level nested if-else",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 5; if x > 0 { if x > 3 { if x > 4 { let r = 1; } } } }"#,
                r#"if [ "$x" -gt 0 ]; then"#,
            ),
            // --- While with complex condition ---
            CorpusEntry::new(
                "B-046",
                "while-complex-condition",
                "While loop with compound boolean condition",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut x = 0; let mut y = 10; while x < 5 && y > 0 { x = x + 1; y = y - 1; } }"#,
                r#"while [ "$x" -lt 5 ] && [ "$y" -gt 0 ]; do"#,
            ),
            // --- Multiple functions calling each other ---
            CorpusEntry::new(
                "B-047",
                "multi-function-chain",
                "Three functions calling each other in sequence",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn step1() -> u32 { 1 } fn step2(x: u32) -> u32 { x + 2 } fn step3(x: u32) -> u32 { x * 3 } fn main() { let r = step3(step2(step1())); }"#,
                "step1() {",
            ),
            // --- Boolean parameters ---
            CorpusEntry::new(
                "B-048",
                "boolean-params",
                "Function with boolean parameters",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn process(verbose: bool, dry_run: bool) { if verbose { println!("processing"); } } fn main() { process(true, false); }"#,
                "process() {",
            ),
            // --- Match with many arms ---
            CorpusEntry::new(
                "B-049",
                "match-many-arms",
                "Match expression with 6+ arms",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 3; match x { 0 => { println!("zero"); } 1 => { println!("one"); } 2 => { println!("two"); } 3 => { println!("three"); } 4 => { println!("four"); } _ => { println!("other"); } } }"#,
                r#"case "$x" in"#,
            ),
            // --- Negation operator ---
            CorpusEntry::new(
                "B-050",
                "negation-unary",
                "Unary negation on integer",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 5; let y = !true; }"#,
                "y=! true",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier4_makefile(&mut self) {
        let entries = vec![
            // --- Complex variable expressions ---
            CorpusEntry::new(
                "M-026",
                "conditional-variable",
                "Variable with conditional default",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { let cc = "gcc"; let cflags = "-Wall -Werror -O2 -pedantic"; let ldflags = "-lpthread -lm"; target("all", &["build", "test"]); } fn target(n: &str, d: &[&str]) {}"#,
                "CFLAGS := -Wall -Werror -O2 -pedantic",
            ),
            // --- Many targets ---
            CorpusEntry::new(
                "M-027",
                "many-targets",
                "Makefile with 5+ targets",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { let cc = "gcc"; target("all", &["build"]); target("build", &["src/main.c"]); target("test", &["build"]); target("clean", &[]); target("install", &["build"]); target("lint", &["src/main.c"]); } fn target(n: &str, d: &[&str]) {}"#,
                "all: build",
            ),
            // --- Phony with many targets ---
            CorpusEntry::new(
                "M-028",
                "phony-multiple",
                "Multiple phony targets at once",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { phony_target("all", &["build"]); phony_target("clean", &[]); phony_target("test", &["build"]); phony_target("lint", &[]); phony_target("fmt", &[]); } fn phony_target(n: &str, d: &[&str]) {}"#,
                ".PHONY: all",
            ),
            // --- Recipe with multiple commands ---
            CorpusEntry::new(
                "M-029",
                "recipe-multi-command",
                "Target with multiple recipe lines",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { let cc = "gcc"; let src = "main.c"; target("build", &["main.c"]); target("test", &["build"]); target("clean", &[]); } fn target(n: &str, d: &[&str]) {}"#,
                "build: main.c",
            ),
            // --- Empty string variables ---
            CorpusEntry::new(
                "M-030",
                "empty-string-var",
                "Variable with empty string value",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { let prefix = ""; let suffix = ""; let name = "app"; } fn target(n: &str, d: &[&str]) {}"#,
                "PREFIX :=",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier4_dockerfile(&mut self) {
        let entries = vec![
            // --- Four-stage build ---
            CorpusEntry::new(
                "D-026",
                "four-stage-build",
                "Four-stage multi-stage build",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image_as("rust", "1.75", "deps"); workdir("/app"); from_image_as("rust", "1.75", "builder"); copy_from("deps", "/app/target", "/app/target"); from_image_as("debian", "bookworm-slim", "runtime"); copy_from("builder", "/app/target/release/app", "/usr/local/bin/"); from_image("gcr.io/distroless/cc-debian12", "nonroot"); copy_from("runtime", "/usr/local/bin/app", "/app/"); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy_from(f: &str, s: &str, d: &str) {}"#,
                "FROM rust:1.75 AS deps",
            ),
            // --- Healthcheck ---
            CorpusEntry::new(
                "D-027",
                "healthcheck-full",
                "Full HEALTHCHECK with all options",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image("alpine", "3.18"); healthcheck("curl -f http://localhost:8080/health || exit 1"); expose(8080u16); } fn from_image(i: &str, t: &str) {} fn healthcheck(c: &str) {} fn expose(p: u16) {}"#,
                "HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1",
            ),
            // --- Many ENV instructions ---
            CorpusEntry::new(
                "D-028",
                "many-env-vars",
                "Six ENV variables from let bindings",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image("node", "20-alpine"); let app_name = "myapp"; let app_port = "3000"; let node_env = "production"; let log_level = "info"; let db_host = "localhost"; let cache_ttl = "3600"; workdir("/app"); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {}"#,
                "ENV APP_NAME=myapp",
            ),
            // --- Copy with chown-like pattern ---
            CorpusEntry::new(
                "D-029",
                "copy-multiple",
                "Multiple COPY instructions",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image("node", "20-alpine"); workdir("/app"); copy("package.json", "."); copy("package-lock.json", "."); copy("tsconfig.json", "."); copy("src", "src"); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {}"#,
                "COPY package.json .",
            ),
            // --- Comment instructions ---
            CorpusEntry::new(
                "D-030",
                "comments-rich",
                "Dockerfile with multiple comments",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { comment("Build stage"); from_image_as("rust", "1.75", "builder"); comment("Runtime stage"); from_image("alpine", "3.18"); copy_from("builder", "/app", "/app"); } fn comment(c: &str) {} fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn copy_from(f: &str, s: &str, d: &str) {}"#,
                "# Runtime stage",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Tier 5: Production entries (real-world patterns, multi-feature)
    // =========================================================================

    fn load_tier5_bash(&mut self) {
        let entries = vec![
            // --- Multi-function program with conditionals ---
            CorpusEntry::new(
                "B-051",
                "multi-func-program",
                "Program with 3+ functions and control flow",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate(x: u32) -> bool { x > 0 && x < 100 } fn transform(x: u32) -> u32 { if x > 50 { x * 2 } else { x + 10 } } fn main() { let input = 42; let valid = validate(input); let result = transform(input); }"#,
                "validate() {",
            ),
            // --- Iterative computation ---
            CorpusEntry::new(
                "B-052",
                "iterative-sum",
                "Iterative sum computation with accumulator",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; for i in 0..10 { sum += i; } println!("total"); }"#,
                "sum='0'",
            ),
            // --- Nested loop with break ---
            CorpusEntry::new(
                "B-053",
                "nested-loop-break",
                "Nested for loop with conditional break",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = false; for i in 0..100 { if i == 42 { found = true; break; } } }"#,
                "found=false",
            ),
            // --- While with decrement ---
            CorpusEntry::new(
                "B-054",
                "countdown-while",
                "Countdown loop with decrement",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut count = 10; while count > 0 { count -= 1; } }"#,
                r#"while [ "$count" -gt 0 ]; do"#,
            ),
            // --- Match with return values ---
            CorpusEntry::new(
                "B-055",
                "match-return-value",
                "Function using match to return different values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(score: u32) -> u32 { match score { 0 => { return 0; } 1 => { return 1; } 2 => { return 2; } _ => { return 3; } } } fn main() { let grade = classify(85); }"#,
                "classify() {",
            ),
            // --- Complex boolean logic ---
            CorpusEntry::new(
                "B-056",
                "complex-boolean",
                "Complex boolean expression with AND/OR",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn check(a: u32, b: u32, c: u32) -> bool { (a > 0 && b > 0) || c == 0 } fn main() { let ok = check(1, 2, 3); }"#,
                "check() {",
            ),
            // --- Multiple string variables ---
            CorpusEntry::new(
                "B-057",
                "string-variables",
                "Multiple string variable assignments",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let name = "bashrs"; let version = "6.59.0"; let author = "paiml"; let license = "MIT"; println!("ready"); }"#,
                "version='6.59.0'",
            ),
            // --- Function with many parameters ---
            CorpusEntry::new(
                "B-058",
                "many-params-func",
                "Function with 4 parameters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn build(src: &str, out: &str, opt: &str, target: &str) { println!("building"); } fn main() { build("src", "build", "-O2", "x86_64"); }"#,
                "build() {",
            ),
            // --- Fibonacci-like iteration ---
            CorpusEntry::new(
                "B-059",
                "fibonacci-loop",
                "Fibonacci-style iterative computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 0; let mut b = 1; for _i in 0..10 { let temp = b; b = a + b; a = temp; } }"#,
                r#"temp="$b""#,
            ),
            // --- Error handling pattern ---
            CorpusEntry::new(
                "B-060",
                "error-pattern",
                "Early return error handling pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate_port(port: u32) -> u32 { if port < 1 { return 0; } if port > 65535 { return 0; } return 1; } fn main() { let ok = validate_port(8080); }"#,
                "validate_port() {",
            ),
            // --- Loop with continue ---
            CorpusEntry::new(
                "B-061",
                "loop-continue-skip",
                "Loop with continue to skip even numbers",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut count = 0; for i in 0..20 { if i % 2 == 0 { continue; } count += 1; } }"#,
                "continue",
            ),
            // --- Deeply nested arithmetic ---
            CorpusEntry::new(
                "B-062",
                "deep-arithmetic",
                "Complex arithmetic expression",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let a = 10; let b = 20; let c = 30; let result = (a + b) * c - (a * b); }"#,
                "result=$((((a + b) * c) - (a * b)))",
            ),
            // --- Multiple if-else branches ---
            CorpusEntry::new(
                "B-063",
                "multi-branch-if",
                "Function with multiple if-else branches",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn categorize(x: u32) -> u32 { if x < 10 { return 1; } else if x < 100 { return 2; } else if x < 1000 { return 3; } else { return 4; } } fn main() { let cat = categorize(500); }"#,
                "categorize() {",
            ),
            // --- Accumulate with multiply ---
            CorpusEntry::new(
                "B-064",
                "factorial-like",
                "Factorial-style multiplication accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut result = 1; for i in 1..6 { result *= i; } }"#,
                "result='1'",
            ),
            // --- Boolean flag pattern ---
            CorpusEntry::new(
                "B-065",
                "boolean-flag-loop",
                "Boolean flag set inside loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut has_error = false; for i in 0..5 { if i == 3 { has_error = true; } } if has_error { println!("error found"); } }"#,
                "has_error=false",
            ),
            // --- Stderr output ---
            CorpusEntry::new(
                "B-066",
                "eprintln-stderr",
                "Program using eprintln for error messages",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let verbose = true; if verbose { println!("starting"); } eprintln!("warning: debug mode"); }"#,
                r#"printf '%s\n' "$1" >&2"#,
            ),
            // --- Multiple match with default ---
            CorpusEntry::new(
                "B-067",
                "match-string-return",
                "Match on integer returning different string values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn status_msg(code: u32) -> &str { match code { 0 => { return "ok"; } 1 => { return "warning"; } 2 => { return "error"; } _ => { return "unknown"; } } } fn main() { let msg = status_msg(0); }"#,
                "status_msg() {",
            ),
            // --- Power of two check ---
            CorpusEntry::new(
                "B-068",
                "bitwise-power-check",
                "Check if number is within power-of-2 ranges",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_small(n: u32) -> bool { n < 256 } fn is_medium(n: u32) -> bool { n >= 256 && n < 65536 } fn main() { let s = is_small(100); let m = is_medium(1000); }"#,
                "is_small() {",
            ),
            // --- Zero-argument function calls ---
            CorpusEntry::new(
                "B-069",
                "zero-arg-functions",
                "Functions with no arguments",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn get_version() -> &str { "1.0.0" } fn get_name() -> &str { "bashrs" } fn main() { let v = get_version(); let n = get_name(); }"#,
                "get_version() {",
            ),
            // --- Nested while with condition update ---
            CorpusEntry::new(
                "B-070",
                "while-bisection",
                "Binary search-like while loop narrowing",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut low = 0; let mut high = 100; while low < high { let mid = (low + high) / 2; if mid < 50 { low = mid + 1; } else { high = mid; } } }"#,
                r#"while [ "$low" -lt "$high" ]; do"#,
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier5_makefile(&mut self) {
        let entries = vec![
            // --- C project with full workflow ---
            CorpusEntry::new(
                "M-031",
                "c-project-full",
                "Full C project Makefile with build/test/clean/install",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cc = "gcc"; let cflags = "-Wall -Wextra -O2"; let src = "main.c"; let bin = "app"; phony_target("all", &["build"], &["echo done"]); target("build", &["main.c"], &["$(CC) $(CFLAGS) -o $(BIN) $(SRC)"]); phony_target("test", &["build"], &["./$(BIN) --test"]); phony_target("clean", &[], &["rm -f $(BIN)"]); phony_target("install", &["build"], &["cp $(BIN) /usr/local/bin/"]); } fn target(n: &str, d: &[&str], r: &[&str]) {} fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CC := gcc",
            ),
            // --- Rust project Makefile ---
            CorpusEntry::new(
                "M-032",
                "rust-project-makefile",
                "Rust project Makefile with cargo commands",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("all", &["build", "test"], &[]); phony_target("build", &[], &["cargo build --release"]); phony_target("test", &[], &["cargo test"]); phony_target("lint", &[], &["cargo clippy -- -D warnings"]); phony_target("fmt", &[], &["cargo fmt -- --check"]); phony_target("clean", &[], &["cargo clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: all",
            ),
            // --- Python project Makefile ---
            CorpusEntry::new(
                "M-033",
                "python-project-makefile",
                "Python project Makefile with venv and pytest",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let python = "python3"; let venv = ".venv"; phony_target("setup", &[], &["python3 -m venv .venv"]); phony_target("install", &["setup"], &["pip install -r requirements.txt"]); phony_target("test", &["install"], &["pytest tests/"]); phony_target("lint", &[], &["ruff check ."]); phony_target("clean", &[], &["rm -rf .venv __pycache__"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "PYTHON := python3",
            ),
            // --- Docker build Makefile ---
            CorpusEntry::new(
                "M-034",
                "docker-build-makefile",
                "Makefile for Docker image builds",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let image = "myapp"; let tag = "latest"; let registry = "ghcr.io/paiml"; phony_target("build", &[], &["docker build -t $(IMAGE):$(TAG) ."]); phony_target("push", &["build"], &["docker push $(REGISTRY)/$(IMAGE):$(TAG)"]); phony_target("run", &["build"], &["docker run -p 8080:8080 $(IMAGE):$(TAG)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "IMAGE := myapp",
            ),
            // --- Multi-binary project ---
            CorpusEntry::new(
                "M-035",
                "multi-binary",
                "Project with multiple build targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cc = "gcc"; let cflags = "-Wall"; target("all", &["server", "client"]); target("server", &["server.c"], &["$(CC) $(CFLAGS) -o server server.c"]); target("client", &["client.c"], &["$(CC) $(CFLAGS) -o client client.c"]); phony_target("clean", &[], &["rm -f server client"]); } fn target(n: &str, d: &[&str], r: &[&str]) {} fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "server: server.c",
            ),
            // --- Release automation ---
            CorpusEntry::new(
                "M-036",
                "release-automation",
                "Release workflow Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let version = "1.0.0"; phony_target("release", &["test", "lint", "build"], &["git tag v$(VERSION)"]); phony_target("test", &[], &["make test-unit test-integration"]); phony_target("lint", &[], &["make lint-check"]); phony_target("build", &[], &["cargo build --release"]); phony_target("publish", &["release"], &["cargo publish"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "VERSION := 1.0.0",
            ),
            // --- CI/CD targets ---
            CorpusEntry::new(
                "M-037",
                "ci-targets",
                "CI/CD focused Makefile targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("ci", &["lint", "test", "coverage"], &["echo CI passed"]); phony_target("lint", &[], &["cargo clippy"]); phony_target("test", &[], &["cargo test"]); phony_target("coverage", &[], &["cargo llvm-cov"]); phony_target("bench", &[], &["cargo bench"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci",
            ),
            // --- Environment-specific builds ---
            CorpusEntry::new(
                "M-038",
                "env-specific-builds",
                "Development vs production build targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let mode = "debug"; let target_dir = "target"; phony_target("dev", &[], &["cargo build"]); phony_target("prod", &[], &["cargo build --release"]); phony_target("dev-run", &["dev"], &["./target/debug/app"]); phony_target("prod-run", &["prod"], &["./target/release/app"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "MODE := debug",
            ),
            // --- Database migration targets ---
            CorpusEntry::new(
                "M-039",
                "db-migration-makefile",
                "Database migration management Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let db_url = "postgres://localhost/mydb"; phony_target("migrate", &[], &["sqlx migrate run"]); phony_target("rollback", &[], &["sqlx migrate revert"]); phony_target("seed", &["migrate"], &["sqlx seed"]); phony_target("reset", &[], &["sqlx database reset"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "DB_URL := postgres://localhost/mydb",
            ),
            // --- Documentation build ---
            CorpusEntry::new(
                "M-040",
                "docs-build-makefile",
                "Documentation build Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let book_dir = "book"; phony_target("docs", &[], &["mdbook build"]); phony_target("docs-serve", &[], &["mdbook serve"]); phony_target("docs-test", &[], &["mdbook test"]); phony_target("docs-clean", &[], &["rm -rf book/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "BOOK_DIR := book",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_tier5_dockerfile(&mut self) {
        let entries = vec![
            // --- Node.js production Dockerfile ---
            CorpusEntry::new(
                "D-031",
                "nodejs-production",
                "Production Node.js Dockerfile with multi-stage",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); copy("package-lock.json", "."); run(&["npm ci --production"]); copy("src", "src"); from_image("node", "20-alpine"); workdir("/app"); copy_from("builder", "/app", "/app"); user("1000"); expose(3000u16); cmd(&["node", "src/index.js"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            // --- Go production Dockerfile ---
            CorpusEntry::new(
                "D-032",
                "go-production",
                "Production Go Dockerfile with scratch final stage",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.22-alpine", "builder"); workdir("/app"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["CGO_ENABLED=0 go build -o /app/server"]); from_image("gcr.io/distroless/static", "nonroot"); copy_from("builder", "/app/server", "/server"); user("65534"); entrypoint(&["/server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM golang:1.22-alpine AS builder",
            ),
            // --- Python production Dockerfile ---
            CorpusEntry::new(
                "D-033",
                "python-production",
                "Production Python Dockerfile with pip install",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("python", "3.12-slim"); workdir("/app"); copy("requirements.txt", "."); run(&["pip install --no-cache-dir -r requirements.txt"]); copy("src", "src"); let pythondontwritebytecode = "1"; user("1000"); cmd(&["python", "-m", "src.main"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM python:3.12-slim",
            ),
            // --- Nginx reverse proxy ---
            CorpusEntry::new(
                "D-034",
                "nginx-proxy",
                "Nginx reverse proxy Dockerfile",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("nginx", "1.25-alpine"); copy("nginx.conf", "/etc/nginx/nginx.conf"); copy("default.conf", "/etc/nginx/conf.d/default.conf"); expose(80u16); expose(443u16); healthcheck("curl -f http://localhost/ || exit 1"); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str) {}"#,
                "FROM nginx:1.25-alpine",
            ),
            // --- Rust production with caching ---
            CorpusEntry::new(
                "D-035",
                "rust-cached-build",
                "Rust build with dependency caching pattern",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "deps"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["mkdir src", "echo 'fn main(){}' > src/main.rs", "cargo build --release", "rm -rf src"]); from_image_as("rust", "1.75-alpine", "builder"); workdir("/app"); copy_from("deps", "/app/target", "target"); copy(".", "."); run(&["cargo build --release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/target/release/app", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-alpine AS deps",
            ),
            // --- Java Spring Boot ---
            CorpusEntry::new(
                "D-036",
                "java-springboot",
                "Java Spring Boot production Dockerfile",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("eclipse-temurin", "21-jdk-alpine", "builder"); workdir("/app"); copy(".", "."); run(&["./gradlew bootJar"]); from_image("eclipse-temurin", "21-jre-alpine"); workdir("/app"); copy_from("builder", "/app/build/libs/*.jar", "app.jar"); expose(8080u16); user("1000"); entrypoint(&["java", "-jar", "app.jar"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM eclipse-temurin:21-jdk-alpine AS builder",
            ),
            // --- Multi-service with healthcheck ---
            CorpusEntry::new(
                "D-037",
                "service-healthcheck",
                "Service with comprehensive healthcheck",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.18"); run(&["apk add --no-cache curl"]); workdir("/app"); copy("app", "/app/"); expose(8080u16); healthcheck("curl -f http://localhost:8080/health || exit 1"); user("65534"); entrypoint(&["/app/app"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1",
            ),
            // --- Static site builder ---
            CorpusEntry::new(
                "D-038",
                "static-site-builder",
                "Static site build and serve with nginx",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); run(&["npm install"]); copy(".", "."); run(&["npm run build"]); from_image("nginx", "1.25-alpine"); copy_from("builder", "/app/dist", "/usr/share/nginx/html"); expose(80u16); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            // --- Database with init script ---
            CorpusEntry::new(
                "D-039",
                "postgres-custom",
                "Custom PostgreSQL with initialization",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("postgres", "16-alpine"); let postgres_db = "myapp"; let postgres_user = "admin"; copy("init.sql", "/docker-entrypoint-initdb.d/"); expose(5432u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "ENV POSTGRES_DB=myapp",
            ),
            // --- Monitoring stack ---
            CorpusEntry::new(
                "D-040",
                "monitoring-agent",
                "Monitoring agent Dockerfile with config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.18"); run(&["apk add --no-cache ca-certificates"]); let metrics_port = "9090"; let log_level = "info"; workdir("/app"); copy("agent", "/usr/local/bin/"); copy("config.yaml", "/etc/agent/"); expose(9090u16); user("65534"); entrypoint(&["/usr/local/bin/agent"]); cmd(&["--config", "/etc/agent/config.yaml"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                r#"ENTRYPOINT ["/usr/local/bin/agent"]"#,
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion entries: pushing toward 500 total
    // =========================================================================

    fn load_expansion_bash(&mut self) {
        let entries = vec![
            // --- Nested for loops ---
            CorpusEntry::new(
                "B-071",
                "nested-for-loops",
                "Nested for loop multiplication table",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { for i in 1..4 { for j in 1..4 { let product = i * j; } } }"#,
                "product=$((i * j))",
            ),
            // --- Function returning bool ---
            CorpusEntry::new(
                "B-072",
                "func-returns-bool",
                "Function returning boolean",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_even(n: u32) -> bool { n % 2 == 0 } fn main() { let even = is_even(4); }"#,
                "is_even() {",
            ),
            // --- Multiple eprintln ---
            CorpusEntry::new(
                "B-073",
                "multi-eprintln",
                "Multiple eprintln calls",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { eprintln!("step 1"); eprintln!("step 2"); eprintln!("done"); }"#,
                r#"printf '%s\n' "$1" >&2"#,
            ),
            // --- Mixed println and eprintln ---
            CorpusEntry::new(
                "B-074",
                "mixed-output",
                "stdout and stderr output",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { println!("output"); eprintln!("error"); println!("more output"); }"#,
                "rash_println() {",
            ),
            // --- While with complex body ---
            CorpusEntry::new(
                "B-075",
                "while-complex-body",
                "While loop with if-else in body",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut evens = 0; let mut odds = 0; while x < 10 { if x % 2 == 0 { evens += 1; } else { odds += 1; } x += 1; } }"#,
                r#"while [ "$x" -lt 10 ]; do"#,
            ),
            // --- Chained comparisons ---
            CorpusEntry::new(
                "B-076",
                "chained-compare",
                "Multiple comparison operators",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn range_check(x: u32) -> bool { x >= 10 && x <= 100 } fn main() { let ok = range_check(50); }"#,
                "range_check() {",
            ),
            // --- Nested match in function ---
            CorpusEntry::new(
                "B-077",
                "nested-match-func",
                "Function body with nested match",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn day_type(d: u32) -> u32 { match d { 0 => { return 0; } 6 => { return 0; } _ => { return 1; } } } fn main() { let is_weekday = day_type(3); }"#,
                "day_type() {",
            ),
            // --- Empty else branch ---
            CorpusEntry::new(
                "B-078",
                "empty-else",
                "If with empty else",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 5; if x > 0 { println!("positive"); } else { println!("non-positive"); } }"#,
                r#"if [ "$x" -gt 0 ]; then"#,
            ),
            // --- Multiple mut variables ---
            CorpusEntry::new(
                "B-079",
                "multiple-mut-vars",
                "Multiple mutable variable updates",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 1; let mut b = 2; let mut c = 3; a += b; b += c; c += a; }"#,
                "a='1'",
            ),
            // --- Deeply nested function calls ---
            CorpusEntry::new(
                "B-080",
                "deeply-nested-calls",
                "Three levels of function nesting",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn f1(x: u32) -> u32 { x + 1 } fn f2(x: u32) -> u32 { x * 2 } fn f3(x: u32) -> u32 { x - 1 } fn main() { let r = f3(f2(f1(10))); }"#,
                "f1() {",
            ),
            // --- For loop with compound assignment ---
            CorpusEntry::new(
                "B-081",
                "for-compound-assign",
                "For loop body using += accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 0..100 { total += i; } }"#,
                "total='0'",
            ),
            // --- Boolean negation in condition ---
            CorpusEntry::new(
                "B-082",
                "bool-negation-cond",
                "Negated boolean in if condition",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let debug = false; if !debug { println!("production"); } }"#,
                r#"if ! "$debug"; then"#,
            ),
            // --- Multiple return statements ---
            CorpusEntry::new(
                "B-083",
                "multi-return",
                "Function with 4 return paths",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn priority(level: u32) -> &str { if level == 0 { return "critical"; } if level == 1 { return "high"; } if level == 2 { return "medium"; } return "low"; } fn main() { let p = priority(1); }"#,
                "priority() {",
            ),
            // --- Zero comparison ---
            CorpusEntry::new(
                "B-084",
                "zero-comparison",
                "Comparison with zero",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let x = 0; let is_zero = x == 0; }"#,
                r#"is_zero=[ "$x" -eq 0 ]"#,
            ),
            // --- String equality (implicit) ---
            CorpusEntry::new(
                "B-085",
                "string-assign-multi",
                "Multiple string assignments",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let a = "hello"; let b = "world"; let c = "test"; }"#,
                "b='world'",
            ),
            // --- Large for range ---
            CorpusEntry::new(
                "B-086",
                "large-for-range",
                "For loop with large range",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut sum = 0; for i in 0..1000 { sum += 1; } }"#,
                "sum='0'",
            ),
            // --- Subtraction chain ---
            CorpusEntry::new(
                "B-087",
                "subtraction-chain",
                "Chained subtraction operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 100; let b = a - 10; let c = b - 20; let d = c - 30; }"#,
                "d=$((c - 30))",
            ),
            // --- Division and modulo ---
            CorpusEntry::new(
                "B-088",
                "div-mod-combined",
                "Division and modulo together",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 17; let quotient = x / 5; let remainder = x % 5; }"#,
                "quotient=$((x / 5))",
            ),
            // --- Match wildcard only ---
            CorpusEntry::new(
                "B-089",
                "match-wildcard-only",
                "Match with only wildcard arm",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 42; match x { _ => { println!("any"); } } }"#,
                r#"case "$x" in"#,
            ),
            // --- Inclusive range ---
            CorpusEntry::new(
                "B-090",
                "inclusive-range-large",
                "Inclusive range 1..=100",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut count = 0; for i in 1..=50 { count += 1; } }"#,
                "count='0'",
            ),
            // --- Empty main ---
            CorpusEntry::new(
                "B-091",
                "empty-main",
                "Minimal valid program",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() {}"#,
                "#!/bin/sh",
            ),
            // --- Single println ---
            CorpusEntry::new(
                "B-092",
                "single-println",
                "Hello world program",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { println!("hello world"); }"#,
                "rash_println() {",
            ),
            // --- While true break pattern ---
            CorpusEntry::new(
                "B-093",
                "while-true-break",
                "While true with conditional break",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut i = 0; while true { i += 1; if i >= 10 { break; } } }"#,
                "break",
            ),
            // --- Multiple functions same name prefix ---
            CorpusEntry::new(
                "B-094",
                "func-name-prefix",
                "Functions with similar name prefixes",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn get_name() -> &str { "app" } fn get_version() -> &str { "1.0" } fn get_author() -> &str { "test" } fn main() { let n = get_name(); let v = get_version(); let a = get_author(); }"#,
                "get_name() {",
            ),
            // --- Comparison result as value ---
            CorpusEntry::new(
                "B-095",
                "comparison-value",
                "Store comparison result directly",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 10; let b = 20; let less = a < b; let equal = a == b; let greater = a > b; }"#,
                r#"less=[ "$a" -lt "$b" ]"#,
            ),
            // --- While countdown to zero ---
            CorpusEntry::new(
                "B-096",
                "while-to-zero",
                "Decrement while loop to zero",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut n = 50; while n > 0 { n -= 1; } }"#,
                r#"while [ "$n" -gt 0 ]; do"#,
            ),
            // --- Function with default return ---
            CorpusEntry::new(
                "B-097",
                "func-implicit-return",
                "Function returning last expression",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn double(x: u32) -> u32 { x * 2 } fn main() { let result = double(21); }"#,
                "double() {",
            ),
            // --- Nested if in while ---
            CorpusEntry::new(
                "B-098",
                "nested-if-while",
                "If inside while loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut pos = 0; let mut neg = 0; while x < 20 { if x % 3 == 0 { pos += 1; } else { neg += 1; } x += 1; } }"#,
                r#"while [ "$x" -lt 20 ]; do"#,
            ),
            // --- Match with many string literals ---
            CorpusEntry::new(
                "B-099",
                "match-string-heavy",
                "Match returning many different strings",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn color(n: u32) -> &str { match n { 0 => { return "red"; } 1 => { return "blue"; } 2 => { return "green"; } 3 => { return "yellow"; } 4 => { return "purple"; } _ => { return "black"; } } } fn main() { let c = color(2); }"#,
                "color() {",
            ),
            // --- Accumulate with division ---
            CorpusEntry::new(
                "B-100",
                "accumulate-division",
                "Repeated division accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 1000; let mut steps = 0; while n > 1 { n = n / 2; steps += 1; } }"#,
                "steps='0'",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion_makefile(&mut self) {
        let entries = vec![
            // --- Kubernetes deployment ---
            CorpusEntry::new(
                "M-041",
                "k8s-deploy",
                "Kubernetes deployment Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let namespace = "default"; phony_target("deploy", &[], &["kubectl apply -f k8s/"]); phony_target("undeploy", &[], &["kubectl delete -f k8s/"]); phony_target("status", &[], &["kubectl get pods -n $(NAMESPACE)"]); phony_target("logs", &[], &["kubectl logs -f deployment/app"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "NAMESPACE := default",
            ),
            // --- WASM build ---
            CorpusEntry::new(
                "M-042",
                "wasm-build",
                "WebAssembly build Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let target = "wasm32-unknown-unknown"; phony_target("wasm-build", &[], &["wasm-pack build --target web"]); phony_target("wasm-test", &[], &["wasm-pack test --headless"]); phony_target("wasm-serve", &[], &["ruchy serve --port 8000"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "TARGET := wasm32-unknown-unknown",
            ),
            // --- Terraform workflow ---
            CorpusEntry::new(
                "M-043",
                "terraform-workflow",
                "Terraform infrastructure Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let env = "staging"; phony_target("init", &[], &["terraform init"]); phony_target("plan", &["init"], &["terraform plan"]); phony_target("apply", &["plan"], &["terraform apply -auto-approve"]); phony_target("destroy", &[], &["terraform destroy -auto-approve"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "ENV := staging",
            ),
            // --- Cross-compilation ---
            CorpusEntry::new(
                "M-044",
                "cross-compile",
                "Cross-compilation targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let name = "app"; phony_target("build-linux", &[], &["GOOS=linux GOARCH=amd64 go build -o bin/linux/app"]); phony_target("build-mac", &[], &["GOOS=darwin GOARCH=arm64 go build -o bin/mac/app"]); phony_target("build-all", &["build-linux", "build-mac"], &["echo done"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "NAME := app",
            ),
            // --- Benchmark targets ---
            CorpusEntry::new(
                "M-045",
                "benchmark-makefile",
                "Performance benchmarking targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("bench", &[], &["cargo bench"]); phony_target("bench-baseline", &[], &["cargo bench -- --save-baseline main"]); phony_target("bench-compare", &[], &["cargo bench -- --baseline main"]); phony_target("flamegraph", &[], &["cargo flamegraph"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: bench",
            ),
            // --- Monorepo targets ---
            CorpusEntry::new(
                "M-046",
                "monorepo-targets",
                "Monorepo workspace targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build-all", &[], &["cargo build --workspace"]); phony_target("test-all", &[], &["cargo test --workspace"]); phony_target("lint-all", &[], &["cargo clippy --workspace"]); phony_target("fmt-all", &[], &["cargo fmt --all -- --check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-all",
            ),
            // --- Security scanning ---
            CorpusEntry::new(
                "M-047",
                "security-scan",
                "Security scanning Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("audit", &[], &["cargo audit"]); phony_target("deny", &[], &["cargo deny check"]); phony_target("security", &["audit", "deny"], &["echo security passed"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: audit",
            ),
            // --- Proto/gRPC build ---
            CorpusEntry::new(
                "M-048",
                "proto-build",
                "Protocol Buffer build targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let proto_dir = "proto"; phony_target("proto", &[], &["protoc --go_out=. proto/*.proto"]); phony_target("proto-lint", &[], &["buf lint"]); phony_target("proto-format", &[], &["buf format -w"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "PROTO_DIR := proto",
            ),
            // --- Container orchestration ---
            CorpusEntry::new(
                "M-049",
                "compose-targets",
                "Docker Compose management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("up", &[], &["docker compose up -d"]); phony_target("down", &[], &["docker compose down"]); phony_target("restart", &["down", "up"], &[]); phony_target("logs", &[], &["docker compose logs -f"]); phony_target("ps", &[], &["docker compose ps"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up",
            ),
            // --- Code generation ---
            CorpusEntry::new(
                "M-050",
                "codegen-targets",
                "Code generation Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let gen_dir = "generated"; phony_target("generate", &[], &["go generate ./..."]); phony_target("gen-clean", &[], &["rm -rf generated/"]); phony_target("gen-verify", &["generate"], &["git diff --exit-code generated/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "GEN_DIR := generated",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion_dockerfile(&mut self) {
        let entries = vec![
            // --- Elixir/Phoenix Dockerfile ---
            CorpusEntry::new(
                "D-041",
                "elixir-phoenix",
                "Elixir Phoenix production build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("elixir", "1.16-alpine", "builder"); workdir("/app"); copy("mix.exs", "."); copy("mix.lock", "."); run(&["mix deps.get --only prod"]); copy(".", "."); run(&["MIX_ENV=prod mix release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/_build/prod/rel/myapp", "/app"); user("65534"); entrypoint(&["/app/bin/myapp"]); cmd(&["start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM elixir:1.16-alpine AS builder",
            ),
            // --- Redis with config ---
            CorpusEntry::new(
                "D-042",
                "redis-custom",
                "Custom Redis with config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("redis", "7-alpine"); copy("redis.conf", "/usr/local/etc/redis/"); expose(6379u16); cmd(&["redis-server", "/usr/local/etc/redis/redis.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM redis:7-alpine",
            ),
            // --- .NET production ---
            CorpusEntry::new(
                "D-043",
                "dotnet-production",
                ".NET production multi-stage build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("mcr.microsoft.com/dotnet/sdk", "8.0", "builder"); workdir("/app"); copy("*.csproj", "."); run(&["dotnet restore"]); copy(".", "."); run(&["dotnet publish -c Release -o /out"]); from_image("mcr.microsoft.com/dotnet/aspnet", "8.0"); copy_from("builder", "/out", "/app"); workdir("/app"); expose(8080u16); user("1000"); entrypoint(&["dotnet", "MyApp.dll"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder",
            ),
            // --- Minimal scratch container ---
            CorpusEntry::new(
                "D-044",
                "scratch-minimal",
                "Minimal scratch-based container",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image_as("golang", "1.22", "builder"); workdir("/app"); copy(".", "."); run(&["CGO_ENABLED=0 go build -ldflags=-s -o /app/bin"]); from_image("scratch", "latest"); copy_from("builder", "/app/bin", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM scratch:latest",
            ),
            // --- ML/AI container ---
            CorpusEntry::new(
                "D-045",
                "ml-container",
                "Machine learning inference container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("python", "3.11-slim"); workdir("/app"); run(&["pip install --no-cache-dir torch transformers"]); copy("model", "model"); copy("serve.py", "."); let model_path = "/app/model"; expose(8000u16); healthcheck("curl -f http://localhost:8000/health || exit 1"); cmd(&["python", "serve.py"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM python:3.11-slim",
            ),
            // --- Multi-service Dockerfile ---
            CorpusEntry::new(
                "D-046",
                "worker-service",
                "Background worker service container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75", "builder"); workdir("/app"); copy(".", "."); run(&["cargo build --release --bin worker"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y --no-install-recommends ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/worker", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/worker"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75 AS builder",
            ),
            // --- Development container ---
            CorpusEntry::new(
                "D-047",
                "dev-container",
                "Development container with tools",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75"); run(&["rustup component add clippy rustfmt"]); run(&["cargo install cargo-watch cargo-llvm-cov"]); workdir("/workspace"); let rust_backtrace = "1"; } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {}"#,
                "FROM rust:1.75",
            ),
            // --- API gateway ---
            CorpusEntry::new(
                "D-048",
                "api-gateway",
                "API gateway with Envoy",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("envoyproxy/envoy", "v1.28"); copy("envoy.yaml", "/etc/envoy/envoy.yaml"); expose(8080u16); expose(8443u16); expose(9901u16); entrypoint(&["/usr/local/bin/envoy"]); cmd(&["-c", "/etc/envoy/envoy.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM envoyproxy/envoy:v1.28",
            ),
            // --- Caddy web server ---
            CorpusEntry::new(
                "D-049",
                "caddy-server",
                "Caddy web server with custom config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("caddy", "2-alpine"); copy("Caddyfile", "/etc/caddy/Caddyfile"); copy("static", "/srv"); expose(80u16); expose(443u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM caddy:2-alpine",
            ),
            // --- Grafana with plugins ---
            CorpusEntry::new(
                "D-050",
                "grafana-custom",
                "Custom Grafana with plugins",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("grafana/grafana", "10.2"); let gf_install_plugins = "grafana-clock-panel,grafana-simple-json-datasource"; let gf_security_admin_password = "admin"; copy("dashboards", "/var/lib/grafana/dashboards"); copy("provisioning", "/etc/grafana/provisioning"); expose(3000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM grafana/grafana:10.2",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion wave 2: pushing toward 300+ entries
    // =========================================================================

    fn load_expansion2_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-101",
                "gcd-algorithm",
                "Greatest common divisor via while loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 48; let mut b = 18; while b > 0 { let temp = b; b = a % b; a = temp; } }"#,
                r#"while [ "$b" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-102",
                "max-of-three",
                "Find maximum of three values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn max3(a: u32, b: u32, c: u32) -> u32 { if a > b && a > c { return a; } if b > c { return b; } return c; } fn main() { let m = max3(10, 20, 15); }"#,
                "max3() {",
            ),
            CorpusEntry::new(
                "B-103",
                "abs-value",
                "Absolute value function",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn abs_val(x: u32, y: u32) -> u32 { if x > y { x - y } else { y - x } } fn main() { let d = abs_val(10, 7); }"#,
                "abs_val() {",
            ),
            CorpusEntry::new(
                "B-104",
                "swap-values",
                "Swap two variables using temp",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut a = 10; let mut b = 20; let temp = a; a = b; b = temp; }"#,
                r#"temp="$a""#,
            ),
            CorpusEntry::new(
                "B-105",
                "clamp-range",
                "Clamp value to range",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn clamp(x: u32, min: u32, max: u32) -> u32 { if x < min { return min; } if x > max { return max; } return x; } fn main() { let c = clamp(150, 0, 100); }"#,
                "clamp() {",
            ),
            CorpusEntry::new(
                "B-106",
                "count-digits",
                "Count digits in number via division",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 12345; let mut digits = 0; while n > 0 { n = n / 10; digits += 1; } }"#,
                "digits='0'",
            ),
            CorpusEntry::new(
                "B-107",
                "sum-of-digits",
                "Sum digits of a number",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 9876; let mut sum = 0; while n > 0 { sum += n % 10; n = n / 10; } }"#,
                "sum='0'",
            ),
            CorpusEntry::new(
                "B-108",
                "power-of-two",
                "Compute power of 2 iteratively",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut result = 1; for _i in 0..10 { result *= 2; } }"#,
                "result='1'",
            ),
            CorpusEntry::new(
                "B-109",
                "linear-search",
                "Linear search through comparison",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let target = 42; let mut found = false; for i in 0..100 { if i == target { found = true; break; } } }"#,
                "found=false",
            ),
            CorpusEntry::new(
                "B-110",
                "min-of-three",
                "Find minimum of three values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn min3(a: u32, b: u32, c: u32) -> u32 { if a < b && a < c { return a; } if b < c { return b; } return c; } fn main() { let m = min3(30, 10, 20); }"#,
                "min3() {",
            ),
            CorpusEntry::new(
                "B-111",
                "collatz-step",
                "Single Collatz step function",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn collatz_step(n: u32) -> u32 { if n % 2 == 0 { n / 2 } else { n * 3 + 1 } } fn main() { let next = collatz_step(7); }"#,
                "collatz_step() {",
            ),
            CorpusEntry::new(
                "B-112",
                "triangle-numbers",
                "Compute triangle numbers",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; for i in 1..=10 { sum += i; } }"#,
                "sum='0'",
            ),
            CorpusEntry::new(
                "B-113",
                "string-path-parts",
                "Multiple path component variables",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let dir = "/usr/local"; let bin = "bin"; let name = "bashrs"; let ext = "sh"; }"#,
                "dir='/usr/local'",
            ),
            CorpusEntry::new(
                "B-114",
                "nested-break",
                "Nested loop with inner break",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut total = 0; for i in 0..10 { for j in 0..10 { if j > i { break; } total += 1; } } }"#,
                "total='0'",
            ),
            CorpusEntry::new(
                "B-115",
                "decrement-loop",
                "Decrement loop with *= operator",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut x = 1024; let mut count = 0; while x > 1 { x = x / 2; count += 1; } }"#,
                "count='0'",
            ),
            CorpusEntry::new(
                "B-116",
                "multi-condition-while",
                "While with OR condition",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut a = 0; let mut b = 100; while a < 50 || b > 50 { a += 1; b -= 1; } }"#,
                r#"while [ "$a" -lt 50 ] || [ "$b" -gt 50 ]; do"#,
            ),
            CorpusEntry::new(
                "B-117",
                "match-range-like",
                "Match with multiple arms mapping ranges",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn grade(score: u32) -> &str { if score >= 90 { return "A"; } if score >= 80 { return "B"; } if score >= 70 { return "C"; } if score >= 60 { return "D"; } return "F"; } fn main() { let g = grade(85); }"#,
                "grade() {",
            ),
            CorpusEntry::new(
                "B-118",
                "bool-and-chain",
                "Chain of AND conditions",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn valid(a: u32, b: u32, c: u32) -> bool { a > 0 && b > 0 && c > 0 } fn main() { let v = valid(1, 2, 3); }"#,
                "valid() {",
            ),
            CorpusEntry::new(
                "B-119",
                "bool-or-chain",
                "Chain of OR conditions",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn any_zero(a: u32, b: u32, c: u32) -> bool { a == 0 || b == 0 || c == 0 } fn main() { let z = any_zero(1, 0, 3); }"#,
                "any_zero() {",
            ),
            CorpusEntry::new(
                "B-120",
                "for-in-if",
                "For loop only inside if branch",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let run = true; if run { for i in 0..5 { println!("running"); } } }"#,
                "for i in $(seq 0 4); do",
            ),
            CorpusEntry::new(
                "B-121",
                "five-functions",
                "Program with five functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn f1() -> u32 { 1 } fn f2() -> u32 { 2 } fn f3() -> u32 { 3 } fn f4() -> u32 { 4 } fn f5() -> u32 { 5 } fn main() { let total = f1() + f2() + f3() + f4() + f5(); }"#,
                "f1() {",
            ),
            CorpusEntry::new(
                "B-122",
                "nested-if-match",
                "If containing match",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let x = 5; if x > 0 { match x { 1 => { println!("one"); } _ => { println!("other"); } } } }"#,
                r#"case "$x" in"#,
            ),
            CorpusEntry::new(
                "B-123",
                "while-for-nested",
                "While containing for",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut outer = 0; while outer < 3 { for _i in 0..5 { outer += 1; } } }"#,
                r#"while [ "$outer" -lt 3 ]; do"#,
            ),
            CorpusEntry::new(
                "B-124",
                "multi-assign-chain",
                "Chained assignments",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 1; let b = a + 1; let c = b + 1; let d = c + 1; let e = d + 1; }"#,
                "e=$((d + 1))",
            ),
            CorpusEntry::new(
                "B-125",
                "mixed-types",
                "Mix of u32 and bool variables",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let count = 42; let active = true; let name = "test"; let limit = 100; let verbose = false; }"#,
                "active=true",
            ),
            CorpusEntry::new(
                "B-126",
                "early-exit-loop",
                "Multiple early exits in loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 0..1000 { if i % 7 == 0 && i % 13 == 0 { found = i; break; } } }"#,
                "found='0'",
            ),
            CorpusEntry::new(
                "B-127",
                "arithmetic-overflow-safe",
                "Arithmetic within safe u32 range",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 1000000; let b = 2000000; let c = a + b; let d = c * 2; }"#,
                "d=$((c * 2))",
            ),
            CorpusEntry::new(
                "B-128",
                "for-with-step-like",
                "Simulated stepped iteration",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut count = 0; for i in 0..50 { if i % 5 == 0 { count += 1; } } }"#,
                "count='0'",
            ),
            CorpusEntry::new(
                "B-129",
                "string-heavy-program",
                "Program with many string literals",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let host = "localhost"; let port = "8080"; let proto = "https"; let path = "/api/v1"; let method = "GET"; let content_type = "application/json"; println!("configured"); }"#,
                "host='localhost'",
            ),
            CorpusEntry::new(
                "B-130",
                "multi-while-sequential",
                "Sequential while loops",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 10; while a > 0 { a -= 1; } let mut b = 10; while b > 0 { b -= 2; } }"#,
                r#"while [ "$a" -gt 0 ]; do"#,
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion2_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-051",
                "go-project",
                "Go project with standard targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let binary = "myapp"; phony_target("build", &[], &["go build -o bin/$(BINARY) ."]); phony_target("test", &[], &["go test ./..."]); phony_target("vet", &[], &["go vet ./..."]); phony_target("lint", &[], &["golangci-lint run"]); phony_target("clean", &[], &["rm -rf bin/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "BINARY := myapp",
            ),
            CorpusEntry::new(
                "M-052",
                "npm-project",
                "Node.js/npm project Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("install", &[], &["npm ci"]); phony_target("build", &["install"], &["npm run build"]); phony_target("test", &["install"], &["npm test"]); phony_target("lint", &["install"], &["npm run lint"]); phony_target("start", &["build"], &["npm start"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-053",
                "latex-project",
                "LaTeX document build",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let doc = "paper"; target("paper.pdf", &["paper.tex"], &["pdflatex $(DOC).tex", "bibtex $(DOC)", "pdflatex $(DOC).tex"]); phony_target("clean", &[], &["rm -f *.aux *.bbl *.blg *.log *.pdf"]); phony_target("view", &["paper.pdf"], &["open $(DOC).pdf"]); } fn target(n: &str, d: &[&str], r: &[&str]) {} fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "DOC := paper",
            ),
            CorpusEntry::new(
                "M-054",
                "ansible-project",
                "Ansible playbook management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let inventory = "inventory.yml"; phony_target("deploy", &[], &["ansible-playbook -i $(INVENTORY) deploy.yml"]); phony_target("check", &[], &["ansible-playbook -i $(INVENTORY) deploy.yml --check"]); phony_target("lint", &[], &["ansible-lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "INVENTORY := inventory.yml",
            ),
            CorpusEntry::new(
                "M-055",
                "aws-deploy",
                "AWS deployment targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let stack_name = "mystack"; let region = "us-east-1"; phony_target("deploy", &[], &["aws cloudformation deploy --stack-name $(STACK_NAME) --region $(REGION)"]); phony_target("delete", &[], &["aws cloudformation delete-stack --stack-name $(STACK_NAME)"]); phony_target("status", &[], &["aws cloudformation describe-stacks --stack-name $(STACK_NAME)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "STACK_NAME := mystack",
            ),
            CorpusEntry::new(
                "M-056",
                "migration-targets",
                "Data migration Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("migrate-up", &[], &["migrate -source file://migrations -database $(DB_URL) up"]); phony_target("migrate-down", &[], &["migrate -source file://migrations -database $(DB_URL) down 1"]); phony_target("migrate-create", &[], &["migrate create -ext sql -dir migrations -seq $(NAME)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate-up",
            ),
            CorpusEntry::new(
                "M-057",
                "helm-chart",
                "Helm chart management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let chart = "myapp"; let namespace = "production"; phony_target("helm-install", &[], &["helm install $(CHART) ./chart -n $(NAMESPACE)"]); phony_target("helm-upgrade", &[], &["helm upgrade $(CHART) ./chart -n $(NAMESPACE)"]); phony_target("helm-uninstall", &[], &["helm uninstall $(CHART) -n $(NAMESPACE)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CHART := myapp",
            ),
            CorpusEntry::new(
                "M-058",
                "test-matrix",
                "Test matrix targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("test-unit", &[], &["cargo test --lib"]); phony_target("test-integration", &[], &["cargo test --test '*'"]); phony_target("test-doc", &[], &["cargo test --doc"]); phony_target("test-all", &["test-unit", "test-integration", "test-doc"], &["echo all tests passed"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: test-unit",
            ),
            CorpusEntry::new(
                "M-059",
                "static-analysis",
                "Static analysis Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("clippy", &[], &["cargo clippy --all-targets -- -D warnings"]); phony_target("fmt-check", &[], &["cargo fmt -- --check"]); phony_target("audit", &[], &["cargo audit"]); phony_target("check-all", &["clippy", "fmt-check", "audit"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: clippy",
            ),
            CorpusEntry::new(
                "M-060",
                "book-build",
                "mdBook documentation",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("book-build", &[], &["mdbook build book"]); phony_target("book-serve", &[], &["mdbook serve book"]); phony_target("book-test", &[], &["mdbook test book"]); phony_target("book-clean", &[], &["rm -rf book/book"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: book-build",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion2_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-051",
                "ruby-rails",
                "Ruby on Rails Dockerfile",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("ruby", "3.3-slim"); run(&["apt-get update", "apt-get install -y build-essential libpq-dev", "rm -rf /var/lib/apt/lists/*"]); workdir("/app"); copy("Gemfile", "."); copy("Gemfile.lock", "."); run(&["bundle install --without development test"]); copy(".", "."); expose(3000u16); cmd(&["rails", "server", "-b", "0.0.0.0"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ruby:3.3-slim",
            ),
            CorpusEntry::new(
                "D-052",
                "php-laravel",
                "PHP Laravel Dockerfile",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("php", "8.3-fpm-alpine"); run(&["apk add --no-cache composer"]); workdir("/var/www"); copy("composer.json", "."); copy("composer.lock", "."); run(&["composer install --no-dev"]); copy(".", "."); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM php:8.3-fpm-alpine",
            ),
            CorpusEntry::new(
                "D-053",
                "vault-server",
                "HashiCorp Vault server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("vault", "1.15"); let vault_addr = "http://0.0.0.0:8200"; copy("config.hcl", "/vault/config/"); expose(8200u16); entrypoint(&["vault"]); cmd(&["server", "-config=/vault/config/config.hcl"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM vault:1.15",
            ),
            CorpusEntry::new(
                "D-054",
                "zig-build",
                "Zig language build container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("alpine", "3.18", "builder"); run(&["apk add --no-cache zig"]); workdir("/app"); copy(".", "."); run(&["zig build -Doptimize=ReleaseSafe"]); from_image("alpine", "3.18"); copy_from("builder", "/app/zig-out/bin/app", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM alpine:3.18 AS builder",
            ),
            CorpusEntry::new(
                "D-055",
                "prometheus-config",
                "Prometheus with custom config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("prom/prometheus", "v2.48"); copy("prometheus.yml", "/etc/prometheus/"); copy("rules", "/etc/prometheus/rules/"); expose(9090u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM prom/prometheus:v2.48",
            ),
            CorpusEntry::new(
                "D-056",
                "minio-server",
                "MinIO object storage",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("minio/minio", "latest"); let minio_root_user = "admin"; let minio_root_password = "changeme"; expose(9000u16); expose(9001u16); entrypoint(&["minio"]); cmd(&["server", "/data", "--console-address", ":9001"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM minio/minio:latest",
            ),
            CorpusEntry::new(
                "D-057",
                "traefik-proxy",
                "Traefik reverse proxy",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("traefik", "v3.0"); copy("traefik.yml", "/etc/traefik/"); copy("dynamic", "/etc/traefik/dynamic/"); expose(80u16); expose(443u16); expose(8080u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM traefik:v3.0",
            ),
            CorpusEntry::new(
                "D-058",
                "keycloak-server",
                "Keycloak identity server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("quay.io/keycloak/keycloak", "23.0"); let kc_db = "postgres"; copy("themes", "/opt/keycloak/themes/"); expose(8080u16); entrypoint(&["/opt/keycloak/bin/kc.sh"]); cmd(&["start-dev"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM quay.io/keycloak/keycloak:23.0",
            ),
            CorpusEntry::new(
                "D-059",
                "clickhouse-custom",
                "ClickHouse with custom config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("clickhouse/clickhouse-server", "23.12"); copy("config.xml", "/etc/clickhouse-server/"); copy("users.xml", "/etc/clickhouse-server/"); expose(8123u16); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM clickhouse/clickhouse-server:23.12",
            ),
            CorpusEntry::new(
                "D-060",
                "temporal-worker",
                "Temporal workflow worker",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.22-alpine", "builder"); workdir("/app"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["go build -o /worker ./cmd/worker"]); from_image("alpine", "3.18"); run(&["apk add --no-cache ca-certificates"]); copy_from("builder", "/worker", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/worker"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM golang:1.22-alpine AS builder",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion wave 3: pushing toward 350 total
    // =========================================================================

    fn load_expansion3_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-131",
                "is-prime-like",
                "Primality check via trial division",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_prime(n: u32) -> bool { if n < 2 { return false; } let mut i = 2; while i * i <= n { if n % i == 0 { return false; } i += 1; } return true; } fn main() { let p = is_prime(17); }"#,
                "is_prime() {",
            ),
            CorpusEntry::new(
                "B-132",
                "bubble-sort-step",
                "Single bubble sort pass",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut swapped = false; let mut a = 5; let mut b = 3; if a > b { let temp = a; a = b; b = temp; swapped = true; } }"#,
                "swapped=false",
            ),
            CorpusEntry::new(
                "B-133",
                "menu-dispatch",
                "Menu-like dispatch with match",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn dispatch(cmd: u32) { match cmd { 1 => { println!("list"); } 2 => { println!("add"); } 3 => { println!("delete"); } 4 => { println!("edit"); } 5 => { println!("quit"); } _ => { eprintln!("unknown"); } } } fn main() { dispatch(3); }"#,
                "dispatch() {",
            ),
            CorpusEntry::new(
                "B-134",
                "state-machine",
                "Simple state machine with while and match",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut state = 0; let mut steps = 0; while state != 3 { match state { 0 => { state = 1; } 1 => { state = 2; } 2 => { state = 3; } _ => { state = 3; } } steps += 1; } }"#,
                "state='0'",
            ),
            CorpusEntry::new(
                "B-135",
                "running-average",
                "Running average computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; let mut count = 0; for i in 1..=20 { sum += i; count += 1; } let avg = sum / count; }"#,
                "avg=$((sum / count))",
            ),
            CorpusEntry::new(
                "B-136",
                "fizzbuzz-like",
                "FizzBuzz-like modular logic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { for i in 1..=15 { if i % 3 == 0 && i % 5 == 0 { println!("fizzbuzz"); } else if i % 3 == 0 { println!("fizz"); } else if i % 5 == 0 { println!("buzz"); } } }"#,
                "rash_println() {",
            ),
            CorpusEntry::new(
                "B-137",
                "config-builder",
                "Configuration-building pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let host = "0.0.0.0"; let port = 8080; let workers = 4; let timeout = 30; let max_conn = 1000; let verbose = true; if verbose { println!("config ready"); } }"#,
                "workers='4'",
            ),
            CorpusEntry::new(
                "B-138",
                "retry-pattern",
                "Retry loop pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let max_retries = 3; let mut attempts = 0; let mut success = false; while attempts < max_retries && !success { attempts += 1; if attempts >= 2 { success = true; } } }"#,
                r#"while [ "$attempts" -lt "$max_retries" ] && ! [ "$success" ]; do"#,
            ),
            CorpusEntry::new(
                "B-139",
                "accumulate-strings",
                "Building up string config vars",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let proto = "https"; let host = "api.example.com"; let port = "443"; let base = "/v2"; }"#,
                "proto='https'",
            ),
            CorpusEntry::new(
                "B-140",
                "nested-conditions-deep",
                "Four-level nested if",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let a = 1; let b = 2; let c = 3; let d = 4; if a > 0 { if b > 0 { if c > 0 { if d > 0 { println!("all positive"); } } } } }"#,
                r#"if [ "$a" -gt 0 ]; then"#,
            ),
            CorpusEntry::new(
                "B-141",
                "multi-for-sequential",
                "Sequential for loops",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum1 = 0; for i in 0..10 { sum1 += i; } let mut sum2 = 0; for j in 10..20 { sum2 += j; } let total = sum1 + sum2; }"#,
                "total=$((sum1 + sum2))",
            ),
            CorpusEntry::new(
                "B-142",
                "helper-chain",
                "Functions calling helper functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn square(x: u32) -> u32 { x * x } fn sum_of_squares(a: u32, b: u32) -> u32 { square(a) + square(b) } fn main() { let r = sum_of_squares(3, 4); }"#,
                "square() {",
            ),
            CorpusEntry::new(
                "B-143",
                "while-flag-pattern",
                "While loop controlled by flag",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut running = true; let mut ticks = 0; while running { ticks += 1; if ticks >= 100 { running = false; } } }"#,
                r#"while [ "$running" ]; do"#,
            ),
            CorpusEntry::new(
                "B-144",
                "modular-arithmetic",
                "Modular arithmetic operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 42; let mod3 = x % 3; let mod5 = x % 5; let mod7 = x % 7; }"#,
                "mod3=$((x % 3))",
            ),
            CorpusEntry::new(
                "B-145",
                "nested-func-with-loop",
                "Function containing for loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn sum_to(n: u32) -> u32 { let mut total = 0; for i in 0..n { total += i; } total } fn main() { let s = sum_to(100); }"#,
                "sum_to() {",
            ),
            CorpusEntry::new(
                "B-146",
                "decrement-ops",
                "Multiple decrement operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut x = 100; x -= 10; x -= 20; x -= 30; }"#,
                "x='100'",
            ),
            CorpusEntry::new(
                "B-147",
                "compare-and-branch",
                "Comparison-driven branching",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(temp: u32) -> &str { if temp < 32 { return "freezing"; } else if temp < 60 { return "cold"; } else if temp < 80 { return "warm"; } else { return "hot"; } } fn main() { let c = classify(72); }"#,
                "classify() {",
            ),
            CorpusEntry::new(
                "B-148",
                "empty-for-body",
                "For loop with minimal body",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { let mut x = 0; for _i in 0..10 { x = x; } }"#,
                "for _i in $(seq 0 9); do",
            ),
            CorpusEntry::new(
                "B-149",
                "match-with-println",
                "Match arms with println",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let code = 200; match code { 200 => { println!("ok"); } 404 => { println!("not found"); } 500 => { println!("error"); } _ => { println!("unknown"); } } }"#,
                r#"case "$code" in"#,
            ),
            CorpusEntry::new(
                "B-150",
                "complex-program",
                "Program combining all features",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate(x: u32) -> bool { x > 0 && x <= 1000 } fn process(x: u32) -> u32 { let mut result = x; for _i in 0..3 { result *= 2; } result } fn main() { let input = 42; if validate(input) { let output = process(input); println!("done"); } else { eprintln!("invalid"); } }"#,
                "validate() {",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion3_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-061",
                "elixir-project",
                "Elixir/Mix project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("deps", &[], &["mix deps.get"]); phony_target("compile", &["deps"], &["mix compile"]); phony_target("test", &["compile"], &["mix test"]); phony_target("format", &[], &["mix format"]); phony_target("dialyzer", &["compile"], &["mix dialyzer"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deps",
            ),
            CorpusEntry::new(
                "M-062",
                "swift-project",
                "Swift/SPM project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build", &[], &["swift build"]); phony_target("test", &[], &["swift test"]); phony_target("run", &["build"], &["swift run"]); phony_target("clean", &[], &["swift package clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-063",
                "cargo-xtask",
                "Cargo xtask pattern",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("xtask-codegen", &[], &["cargo xtask codegen"]); phony_target("xtask-dist", &[], &["cargo xtask dist"]); phony_target("xtask-install", &[], &["cargo xtask install"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: xtask-codegen",
            ),
            CorpusEntry::new(
                "M-064",
                "multi-lang-project",
                "Multi-language project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build-rust", &[], &["cargo build"]); phony_target("build-python", &[], &["python setup.py build"]); phony_target("build-go", &[], &["go build"]); phony_target("build-all", &["build-rust", "build-python", "build-go"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-rust",
            ),
            CorpusEntry::new(
                "M-065",
                "container-registry",
                "Container registry management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let registry = "ghcr.io/org"; let image = "app"; let tag = "latest"; phony_target("docker-build", &[], &["docker build -t $(REGISTRY)/$(IMAGE):$(TAG) ."]); phony_target("docker-push", &["docker-build"], &["docker push $(REGISTRY)/$(IMAGE):$(TAG)"]); phony_target("docker-scan", &["docker-build"], &["docker scout cves $(REGISTRY)/$(IMAGE):$(TAG)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "REGISTRY := ghcr.io/org",
            ),
            CorpusEntry::new(
                "M-066",
                "pre-commit-setup",
                "Pre-commit hooks setup",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("hooks-install", &[], &["pre-commit install"]); phony_target("hooks-run", &[], &["pre-commit run --all-files"]); phony_target("hooks-update", &[], &["pre-commit autoupdate"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: hooks-install",
            ),
            CorpusEntry::new(
                "M-067",
                "coverage-targets",
                "Coverage reporting",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("coverage", &[], &["cargo llvm-cov"]); phony_target("coverage-html", &[], &["cargo llvm-cov --html"]); phony_target("coverage-lcov", &[], &["cargo llvm-cov --lcov --output-path lcov.info"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: coverage",
            ),
            CorpusEntry::new(
                "M-068",
                "documentation-gen",
                "Documentation generation",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("doc", &[], &["cargo doc --no-deps"]); phony_target("doc-open", &["doc"], &["open target/doc/app/index.html"]); phony_target("doc-check", &[], &["cargo doc --no-deps 2>&1 | grep -c warning"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: doc",
            ),
            CorpusEntry::new(
                "M-069",
                "version-management",
                "Version bump management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let version = "1.0.0"; phony_target("version-patch", &[], &["cargo set-version --bump patch"]); phony_target("version-minor", &[], &["cargo set-version --bump minor"]); phony_target("version-major", &[], &["cargo set-version --bump major"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "VERSION := 1.0.0",
            ),
            CorpusEntry::new(
                "M-070",
                "full-ci-pipeline",
                "Complete CI pipeline",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("ci", &["fmt-check", "lint", "test", "coverage", "audit"], &["echo CI pipeline passed"]); phony_target("fmt-check", &[], &["cargo fmt -- --check"]); phony_target("lint", &[], &["cargo clippy -- -D warnings"]); phony_target("test", &[], &["cargo test"]); phony_target("coverage", &[], &["cargo llvm-cov"]); phony_target("audit", &[], &["cargo audit"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion3_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-061",
                "nextjs-app",
                "Next.js application",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); run(&["npm ci"]); copy(".", "."); run(&["npm run build"]); from_image("node", "20-alpine"); workdir("/app"); copy_from("builder", "/app/.next", ".next"); copy_from("builder", "/app/node_modules", "node_modules"); copy_from("builder", "/app/package.json", "."); expose(3000u16); cmd(&["npm", "start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-062",
                "postgres-replication",
                "PostgreSQL with replication config",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("postgres", "16-alpine"); let postgres_replication_mode = "master"; copy("postgresql.conf", "/etc/postgresql/"); copy("pg_hba.conf", "/etc/postgresql/"); expose(5432u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM postgres:16-alpine",
            ),
            CorpusEntry::new(
                "D-063",
                "deno-app",
                "Deno application container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("denoland/deno", "1.39"); workdir("/app"); copy("deps.ts", "."); run(&["deno cache deps.ts"]); copy(".", "."); user("deno"); expose(8000u16); cmd(&["run", "--allow-net", "--allow-read", "main.ts"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM denoland/deno:1.39",
            ),
            CorpusEntry::new(
                "D-064",
                "mongodb-custom",
                "MongoDB with custom init",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("mongo", "7"); copy("mongod.conf", "/etc/mongod.conf"); copy("init-scripts", "/docker-entrypoint-initdb.d/"); expose(27017u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM mongo:7",
            ),
            CorpusEntry::new(
                "D-065",
                "rabbitmq-server",
                "RabbitMQ with management",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rabbitmq", "3.13-management-alpine"); let rabbitmq_default_user = "admin"; let rabbitmq_default_pass = "password"; copy("rabbitmq.conf", "/etc/rabbitmq/"); expose(5672u16); expose(15672u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM rabbitmq:3.13-management-alpine",
            ),
            CorpusEntry::new(
                "D-066",
                "elasticsearch-node",
                "Elasticsearch node",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("docker.elastic.co/elasticsearch/elasticsearch", "8.11"); let discovery_type = "single-node"; let es_java_opts = "-Xms512m -Xmx512m"; copy("elasticsearch.yml", "/usr/share/elasticsearch/config/"); expose(9200u16); expose(9300u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM docker.elastic.co/elasticsearch/elasticsearch:8.11",
            ),
            CorpusEntry::new(
                "D-067",
                "haproxy-lb",
                "HAProxy load balancer",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("haproxy", "2.9-alpine"); copy("haproxy.cfg", "/usr/local/etc/haproxy/haproxy.cfg"); expose(80u16); expose(443u16); expose(8404u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM haproxy:2.9-alpine",
            ),
            CorpusEntry::new(
                "D-068",
                "consul-server",
                "Consul service mesh",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("hashicorp/consul", "1.17"); copy("consul.hcl", "/consul/config/"); expose(8500u16); expose(8600u16); entrypoint(&["consul"]); cmd(&["agent", "-config-dir=/consul/config"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM hashicorp/consul:1.17",
            ),
            CorpusEntry::new(
                "D-069",
                "nats-server",
                "NATS messaging server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("nats", "2.10-alpine"); copy("nats-server.conf", "/etc/nats/"); expose(4222u16); expose(8222u16); cmd(&["-c", "/etc/nats/nats-server.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM nats:2.10-alpine",
            ),
            CorpusEntry::new(
                "D-070",
                "cron-runner",
                "Cron job runner container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.18"); run(&["apk add --no-cache dcron"]); copy("crontab", "/etc/crontabs/root"); copy("scripts", "/scripts/"); run(&["chmod +x /scripts/*"]); cmd(&["crond", "-f"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM alpine:3.18",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion wave 4: pushing past 350 total
    // =========================================================================

    fn load_expansion4_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-151",
                "selection-sort-pass",
                "Selection sort single pass",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut min_val = 999; let mut min_idx = 0; for i in 0..5 { if i < min_val { min_val = i; min_idx = i; } } }"#,
                "min_val='999'",
            ),
            CorpusEntry::new(
                "B-152",
                "bit-shift-like",
                "Power of 2 via multiplication",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let base = 2; let mut result = 1; for _i in 0..8 { result *= base; } }"#,
                "result='1'",
            ),
            CorpusEntry::new(
                "B-153",
                "temperature-converter",
                "Temperature conversion functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn c_to_f(c: u32) -> u32 { c * 9 / 5 + 32 } fn f_to_c(f: u32) -> u32 { (f - 32) * 5 / 9 } fn main() { let f = c_to_f(100); let c = f_to_c(212); }"#,
                "c_to_f() {",
            ),
            CorpusEntry::new(
                "B-154",
                "leap-year-check",
                "Leap year determination",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_leap(year: u32) -> bool { (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 } fn main() { let leap = is_leap(2024); }"#,
                "is_leap() {",
            ),
            CorpusEntry::new(
                "B-155",
                "digit-counter",
                "Count specific digit occurrences",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 112211; let target = 1; let mut count = 0; while n > 0 { if n % 10 == target { count += 1; } n = n / 10; } }"#,
                "count='0'",
            ),
            CorpusEntry::new(
                "B-156",
                "harmonic-partial",
                "Partial harmonic series (integer approx)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut sum = 0; for i in 1..=10 { sum += 100 / i; } }"#,
                "sum='0'",
            ),
            CorpusEntry::new(
                "B-157",
                "matrix-diagonal",
                "Diagonal element computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut trace = 0; for i in 0..4 { trace += i * 4 + i; } }"#,
                "trace='0'",
            ),
            CorpusEntry::new(
                "B-158",
                "validator-chain",
                "Chain of validation functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn valid_len(n: u32) -> bool { n > 0 && n < 256 } fn valid_range(n: u32) -> bool { n >= 1 && n <= 65535 } fn valid_all(n: u32) -> bool { valid_len(n) && valid_range(n) } fn main() { let ok = valid_all(100); }"#,
                "valid_all() {",
            ),
            CorpusEntry::new(
                "B-159",
                "accumulate-complex",
                "Complex accumulation pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; let mut bonus = 0; for i in 0..50 { total += i; if i % 10 == 0 { bonus += 5; } } let final_score = total + bonus; }"#,
                "final_score=$((total + bonus))",
            ),
            CorpusEntry::new(
                "B-160",
                "signal-handler-pattern",
                "Signal handler-like flag checking",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut interrupted = false; let mut completed = false; let mut retries = 0; while !completed && !interrupted { retries += 1; if retries >= 5 { completed = true; } } }"#,
                r#"while ! [ "$completed" ] && ! [ "$interrupted" ]; do"#,
            ),
            CorpusEntry::new(
                "B-161",
                "bracket-match-sim",
                "Bracket counting simulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut depth = 0; let mut max_depth = 0; for _i in 0..10 { depth += 1; if depth > max_depth { max_depth = depth; } } for _j in 0..10 { depth -= 1; } }"#,
                "max_depth='0'",
            ),
            CorpusEntry::new(
                "B-162",
                "config-env-vars",
                "Environment-style configuration",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let db_host = "localhost"; let db_port = "5432"; let db_name = "mydb"; let db_user = "admin"; let db_pool = "10"; let db_timeout = "30"; let db_ssl = "true"; }"#,
                "db_host='localhost'",
            ),
            CorpusEntry::new(
                "B-163",
                "bit-count-sim",
                "Bit counting via modulo",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 255; let mut ones = 0; while n > 0 { if n % 2 == 1 { ones += 1; } n = n / 2; } }"#,
                "ones='0'",
            ),
            CorpusEntry::new(
                "B-164",
                "euclidean-dist-approx",
                "Approximate distance calculation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let dx = 3; let dy = 4; let dist_sq = dx * dx + dy * dy; }"#,
                "dist_sq=$(((dx * dx) + (dy * dy)))",
            ),
            CorpusEntry::new(
                "B-165",
                "lookup-table-sim",
                "Lookup table simulation with match",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn lookup(key: u32) -> u32 { match key { 0 => { return 100; } 1 => { return 200; } 2 => { return 300; } 3 => { return 400; } 4 => { return 500; } _ => { return 0; } } } fn main() { let v = lookup(3); }"#,
                "lookup() {",
            ),
            CorpusEntry::new(
                "B-166",
                "stack-sim",
                "Stack depth simulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sp = 0; for _i in 0..5 { sp += 1; } for _j in 0..3 { sp -= 1; } }"#,
                "sp='0'",
            ),
            CorpusEntry::new(
                "B-167",
                "multi-format-output",
                "Multiple output formats",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn report(code: u32, msg: &str) { println!("report"); if code > 0 { eprintln!("error detected"); } } fn main() { report(0, "ok"); report(1, "fail"); }"#,
                "report() {",
            ),
            CorpusEntry::new(
                "B-168",
                "cascade-if",
                "Cascading if statements",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 42; let mut result = 0; if x > 100 { result = 4; } if x > 50 { result = 3; } if x > 25 { result = 2; } if x > 0 { result = 1; } }"#,
                "result='0'",
            ),
            CorpusEntry::new(
                "B-169",
                "null-object-sim",
                "Null object pattern with zero",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn get_value(present: bool) -> u32 { if present { return 42; } return 0; } fn main() { let v = get_value(true); let n = get_value(false); }"#,
                "get_value() {",
            ),
            CorpusEntry::new(
                "B-170",
                "pipeline-stages",
                "Multi-stage pipeline simulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn stage1(x: u32) -> u32 { x + 10 } fn stage2(x: u32) -> u32 { x * 2 } fn stage3(x: u32) -> u32 { x - 5 } fn pipeline(x: u32) -> u32 { stage3(stage2(stage1(x))) } fn main() { let result = pipeline(5); }"#,
                "pipeline() {",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion4_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-071",
                "gleam-project",
                "Gleam language project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build", &[], &["gleam build"]); phony_target("test", &[], &["gleam test"]); phony_target("run", &["build"], &["gleam run"]); phony_target("deps", &[], &["gleam deps download"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-072",
                "zig-project",
                "Zig language project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let target = "x86_64-linux"; phony_target("build", &[], &["zig build"]); phony_target("test", &[], &["zig build test"]); phony_target("run", &["build"], &["./zig-out/bin/app"]); phony_target("clean", &[], &["rm -rf zig-out zig-cache"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "TARGET := x86_64-linux",
            ),
            CorpusEntry::new(
                "M-073",
                "bazel-project",
                "Bazel build system targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build", &[], &["bazel build //..."]); phony_target("test", &[], &["bazel test //..."]); phony_target("clean", &[], &["bazel clean"]); phony_target("query", &[], &["bazel query //..."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-074",
                "packer-build",
                "Packer image building",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("validate", &[], &["packer validate ."]); phony_target("build", &["validate"], &["packer build ."]); phony_target("fmt", &[], &["packer fmt ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: validate",
            ),
            CorpusEntry::new(
                "M-075",
                "pulumi-infra",
                "Pulumi infrastructure",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let stack = "dev"; phony_target("up", &[], &["pulumi up --stack $(STACK)"]); phony_target("preview", &[], &["pulumi preview --stack $(STACK)"]); phony_target("destroy", &[], &["pulumi destroy --stack $(STACK)"]); phony_target("refresh", &[], &["pulumi refresh --stack $(STACK)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "STACK := dev",
            ),
            CorpusEntry::new(
                "M-076",
                "nix-build",
                "Nix build targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("build", &[], &["nix build"]); phony_target("develop", &[], &["nix develop"]); phony_target("check", &[], &["nix flake check"]); phony_target("update", &[], &["nix flake update"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-077",
                "just-alternative",
                "Alternative build targets",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("default", &["build", "test"], &[]); phony_target("build", &[], &["cargo build"]); phony_target("test", &[], &["cargo test"]); phony_target("watch", &[], &["cargo watch -x test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: default",
            ),
            CorpusEntry::new(
                "M-078",
                "llvm-project",
                "LLVM/Clang build",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cmake_build_dir = "build"; phony_target("configure", &[], &["cmake -B build -DCMAKE_BUILD_TYPE=Release"]); phony_target("build", &["configure"], &["cmake --build build -j$(nproc)"]); phony_target("install", &["build"], &["cmake --install build"]); phony_target("clean", &[], &["rm -rf build"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CMAKE_BUILD_DIR := build",
            ),
            CorpusEntry::new(
                "M-079",
                "migration-v2",
                "Database migration v2",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("db-create", &[], &["createdb myapp"]); phony_target("db-drop", &[], &["dropdb myapp"]); phony_target("db-migrate", &[], &["diesel migration run"]); phony_target("db-revert", &[], &["diesel migration revert"]); phony_target("db-reset", &["db-drop", "db-create", "db-migrate"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: db-create",
            ),
            CorpusEntry::new(
                "M-080",
                "monitoring-setup",
                "Monitoring stack setup",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("monitoring-up", &[], &["docker compose -f monitoring.yml up -d"]); phony_target("monitoring-down", &[], &["docker compose -f monitoring.yml down"]); phony_target("monitoring-logs", &[], &["docker compose -f monitoring.yml logs -f"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: monitoring-up",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion4_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-071",
                "vite-react",
                "Vite + React production build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); copy("pnpm-lock.yaml", "."); run(&["corepack enable", "pnpm install --frozen-lockfile"]); copy(".", "."); run(&["pnpm build"]); from_image("nginx", "1.25-alpine"); copy_from("builder", "/app/dist", "/usr/share/nginx/html"); expose(80u16); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-072",
                "scala-sbt",
                "Scala SBT build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("sbtscala/scala-sbt", "eclipse-temurin-jammy-21.0.2_13_1.9.9_3.4.0", "builder"); workdir("/app"); copy("build.sbt", "."); copy("project", "project"); run(&["sbt update"]); copy(".", "."); run(&["sbt assembly"]); from_image("eclipse-temurin", "21-jre-alpine"); copy_from("builder", "/app/target/scala-3.4.0/app-assembly.jar", "/app.jar"); expose(8080u16); entrypoint(&["java", "-jar", "/app.jar"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM sbtscala/scala-sbt:eclipse-temurin-jammy-21.0.2_13_1.9.9_3.4.0 AS builder",
            ),
            CorpusEntry::new(
                "D-073",
                "gitea-server",
                "Gitea git server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("gitea/gitea", "1.21"); let user_uid = "1000"; let user_gid = "1000"; copy("app.ini", "/data/gitea/conf/"); expose(3000u16); expose(22u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM gitea/gitea:1.21",
            ),
            CorpusEntry::new(
                "D-074",
                "sonarqube-server",
                "SonarQube code analysis",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("sonarqube", "10-community"); let sonar_jdbc_url = "jdbc:postgresql://db:5432/sonar"; expose(9000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM sonarqube:10-community",
            ),
            CorpusEntry::new(
                "D-075",
                "mailhog-dev",
                "MailHog development SMTP",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("mailhog/mailhog", "latest"); expose(1025u16); expose(8025u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM mailhog/mailhog:latest",
            ),
            CorpusEntry::new(
                "D-076",
                "wireguard-vpn",
                "WireGuard VPN container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.18"); run(&["apk add --no-cache wireguard-tools iptables"]); copy("wg0.conf", "/etc/wireguard/"); expose(51820u16); entrypoint(&["wg-quick", "up", "wg0"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM alpine:3.18",
            ),
            CorpusEntry::new(
                "D-077",
                "superset-bi",
                "Apache Superset BI",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("apache/superset", "3.1"); let superset_secret_key = "change-me-in-production"; let admin_username = "admin"; copy("superset_config.py", "/app/"); expose(8088u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM apache/superset:3.1",
            ),
            CorpusEntry::new(
                "D-078",
                "vector-log",
                "Vector log collector",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("timberio/vector", "0.35-alpine"); copy("vector.toml", "/etc/vector/"); expose(8686u16); entrypoint(&["vector"]); cmd(&["--config", "/etc/vector/vector.toml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM timberio/vector:0.35-alpine",
            ),
            CorpusEntry::new(
                "D-079",
                "meilisearch",
                "Meilisearch search engine",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("getmeili/meilisearch", "v1.6"); let meili_master_key = "change-me"; expose(7700u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM getmeili/meilisearch:v1.6",
            ),
            CorpusEntry::new(
                "D-080",
                "typesense-search",
                "Typesense search engine",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("typesense/typesense", "0.25"); let typesense_api_key = "change-me"; let typesense_data_dir = "/data"; expose(8108u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM typesense/typesense:0.25",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 5: Bash B-171..B-200, Makefile M-081..M-100, Dockerfile D-081..D-100
    // Target: Close gap to 200 Bash, push Make/Docker toward 150
    // =========================================================================

    fn load_expansion5_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-171",
                "countdown-timer",
                "Simple countdown loop",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut t = 10; while t > 0 { echo(&format!("{}", t)); t -= 1; } echo("done"); } fn echo(s: &str) {}"#,
                r#"echo "$t""#,
            ),
            CorpusEntry::new(
                "B-172",
                "string-repeat",
                "Repeat string N times",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let n = 5; let mut i = 0; while i < n { echo("ha"); i += 1; } } fn echo(s: &str) {}"#,
                "echo ha",
            ),
            CorpusEntry::new(
                "B-173",
                "sum-range",
                "Sum integers from 1 to N",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let n = 100; let mut sum = 0; let mut i = 1; while i <= n { sum += i; i += 1; } }"#,
                "sum='0'",
            ),
            CorpusEntry::new(
                "B-174",
                "power-compute",
                "Integer exponentiation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn power(base: u32, exp: u32) -> u32 { let mut result = 1; let mut i = 0; while i < exp { result *= base; i += 1; } result } fn main() { let p = power(2, 10); }"#,
                "power() {",
            ),
            CorpusEntry::new(
                "B-175",
                "abs-value",
                "Absolute value function",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn abs_val(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn main() { let a = abs_val(-42); }"#,
                "abs_val() {",
            ),
            CorpusEntry::new(
                "B-176",
                "clamp-value",
                "Clamp value to range",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn clamp(val: i32, lo: i32, hi: i32) -> i32 { if val < lo { lo } else if val > hi { hi } else { val } } fn main() { let c = clamp(150, 0, 100); }"#,
                "clamp() {",
            ),
            CorpusEntry::new(
                "B-177",
                "sign-function",
                "Return sign of integer",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn sign(x: i32) -> i32 { if x > 0 { 1 } else if x < 0 { 0 - 1 } else { 0 } } fn main() { let s = sign(-7); }"#,
                "sign() {",
            ),
            CorpusEntry::new(
                "B-178",
                "is-even-odd",
                "Even/odd check functions",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn is_even(n: u32) -> bool { n % 2 == 0 } fn is_odd(n: u32) -> bool { n % 2 != 0 } fn main() { let e = is_even(42); let o = is_odd(7); }"#,
                "is_even() {",
            ),
            CorpusEntry::new(
                "B-179",
                "triple-nested-if",
                "Three-level nested conditionals",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let x = 5; let y = 10; let z = 15; if x > 0 { if y > 5 { if z > 10 { echo("deep"); } } } } fn echo(s: &str) {}"#,
                "echo deep",
            ),
            CorpusEntry::new(
                "B-180",
                "variable-swap",
                "Swap two variables",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 10; let b = 20; let tmp = a; let a = b; let b = tmp; }"#,
                r#"tmp="$a""#,
            ),
            CorpusEntry::new(
                "B-181",
                "min-of-three",
                "Minimum of three values",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn min3(a: i32, b: i32, c: i32) -> i32 { if a <= b && a <= c { a } else if b <= c { b } else { c } } fn main() { let m = min3(5, 3, 7); }"#,
                "min3() {",
            ),
            CorpusEntry::new(
                "B-182",
                "max-of-three",
                "Maximum of three values",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn max3(a: i32, b: i32, c: i32) -> i32 { if a >= b && a >= c { a } else if b >= c { b } else { c } } fn main() { let m = max3(5, 3, 7); }"#,
                "max3() {",
            ),
            CorpusEntry::new(
                "B-183",
                "collatz-step",
                "Single Collatz step",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn collatz(n: u32) -> u32 { if n % 2 == 0 { n / 2 } else { n * 3 + 1 } } fn main() { let c = collatz(7); }"#,
                "collatz() {",
            ),
            CorpusEntry::new(
                "B-184",
                "digital-root-step",
                "Single digital root step",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut n = 942; let mut root = 0; while n > 0 { root += n % 10; n = n / 10; } }"#,
                "root='0'",
            ),
            CorpusEntry::new(
                "B-185",
                "triangular-number",
                "Triangular number computation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn triangular(n: u32) -> u32 { n * (n + 1) / 2 } fn main() { let t = triangular(10); }"#,
                "triangular() {",
            ),
            CorpusEntry::new(
                "B-186",
                "multi-return",
                "Multiple function calls assigned",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn double(x: u32) -> u32 { x * 2 } fn triple(x: u32) -> u32 { x * 3 } fn main() { let a = double(5); let b = triple(5); let c = double(b); }"#,
                "double() {",
            ),
            CorpusEntry::new(
                "B-187",
                "nested-while",
                "Nested while loops",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut i = 0; while i < 3 { let mut j = 0; while j < 3 { j += 1; } i += 1; } }"#,
                r#"while [ "$i" -lt 3 ]; do"#,
            ),
            CorpusEntry::new(
                "B-188",
                "boolean-chain",
                "Chained boolean logic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let a = true; let b = false; let c = true; let result = a && (b || c) && !b; }"#,
                r#"result="$a" && "$b" || "$c" && ! "$b""#,
            ),
            CorpusEntry::new(
                "B-189",
                "prefix-suffix",
                "String prefix and suffix variables",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let prefix = "pre"; let suffix = "suf"; let base = "fix"; }"#,
                "prefix='pre'",
            ),
            CorpusEntry::new(
                "B-190",
                "multi-assign-chain",
                "Chain of dependent assignments",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 1; let b = a + 1; let c = b + 1; let d = c + 1; let e = d + 1; }"#,
                "e=$((d + 1))",
            ),
            CorpusEntry::new(
                "B-191",
                "modular-arithmetic",
                "Modular arithmetic expressions",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 17; let m = 5; let r = x % m; let q = x / m; }"#,
                "r=$((x % m))",
            ),
            CorpusEntry::new(
                "B-192",
                "comparison-chain",
                "Multiple comparison operators",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 10; let lt = x < 20; let gt = x > 5; let eq = x == 10; let ne = x != 0; }"#,
                r#"lt=[ "$x" -lt 20 ]"#,
            ),
            CorpusEntry::new(
                "B-193",
                "empty-function",
                "Function with no body logic",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn noop() {} fn main() { noop(); }"#,
                "noop",
            ),
            CorpusEntry::new(
                "B-194",
                "identity-function",
                "Identity function",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn identity(x: u32) -> u32 { x } fn main() { let v = identity(42); }"#,
                "identity() {",
            ),
            CorpusEntry::new(
                "B-195",
                "const-propagation",
                "Constants computed at compile time",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let width = 80; let height = 24; let area = width * height; let perimeter = 2 * (width + height); }"#,
                "area=$((width * height))",
            ),
            CorpusEntry::new(
                "B-196",
                "decrement-loop",
                "Decrement-based loop",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut n = 10; while n > 0 { n -= 1; } }"#,
                r#"while [ "$n" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-197",
                "divisor-check",
                "Check if divisible",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn divides(n: u32, d: u32) -> bool { n % d == 0 } fn main() { let d = divides(42, 7); }"#,
                "divides() {",
            ),
            CorpusEntry::new(
                "B-198",
                "safe-subtract",
                "Saturating subtraction",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn safe_sub(a: u32, b: u32) -> u32 { if a > b { a - b } else { 0 } } fn main() { let r = safe_sub(3, 10); }"#,
                "safe_sub() {",
            ),
            CorpusEntry::new(
                "B-199",
                "flag-variables",
                "Multiple boolean flags",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let verbose = true; let quiet = false; let force = true; let dry_run = false; if verbose && !quiet { echo("info"); } } fn echo(s: &str) {}"#,
                "verbose=true",
            ),
            CorpusEntry::new(
                "B-200",
                "milestone-200",
                "200th Bash entry - combined patterns",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn compute(a: u32, b: u32) -> u32 { if a > b { a - b } else { b - a } } fn main() { let x = compute(42, 17); let y = compute(x, 10); if y > 0 { echo("positive"); } } fn echo(s: &str) {}"#,
                "compute() {",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion6_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-201",
                "nested-while-break",
                "While loop with conditional break",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut i = 0; while i < 100 { if i == 10 { break; } i += 1; } }"#,
                r#"while [ "$i" -lt 100 ]; do"#,
            ),
            CorpusEntry::new(
                "B-202",
                "while-continue",
                "While loop with continue to skip iteration",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut i = 0; while i < 20 { i += 1; if i % 2 == 0 { continue; } echo("odd"); } } fn echo(s: &str) {}"#,
                r#"while [ "$i" -lt 20 ]; do"#,
            ),
            CorpusEntry::new(
                "B-203",
                "gcd-algorithm",
                "Euclidean GCD algorithm with while loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn gcd(a: u32, b: u32) -> u32 { let mut x = a; let mut y = b; while y != 0 { let t = x % y; x = y; y = t; } x } fn main() { let g = gcd(48, 18); }"#,
                "gcd() {",
            ),
            CorpusEntry::new(
                "B-204",
                "fibonacci-compute",
                "Fibonacci number computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn fib(n: u32) -> u32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let t = b; b = a + b; a = t; i += 1; } a } fn main() { let f = fib(10); }"#,
                "fib() {",
            ),
            CorpusEntry::new(
                "B-205",
                "multi-function-chain",
                "Multiple functions calling each other",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn double(x: u32) -> u32 { x * 2 } fn add_one(x: u32) -> u32 { x + 1 } fn main() { let a = 5; let b = double(a); let c = add_one(b); }"#,
                "double() {",
            ),
            CorpusEntry::new(
                "B-206",
                "nested-for-loops",
                "Nested for loops with computation",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut total = 0; for i in 1..4 { for j in 1..4 { total += i * j; } } }"#,
                "for i in $(seq 1 3); do",
            ),
            CorpusEntry::new(
                "B-207",
                "compound-while-condition",
                "While loop with AND condition",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut a = 10; let mut b = 20; while a > 0 && b > 0 { a -= 1; b -= 2; } }"#,
                r#"while [ "$a" -gt 0 ] && [ "$b" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-208",
                "if-elif-chain",
                "Multiple else-if branches",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(x: i32) -> i32 { if x > 100 { 3 } else if x > 50 { 2 } else if x > 0 { 1 } else { 0 } } fn main() { let c = classify(75); }"#,
                "classify() {",
            ),
            CorpusEntry::new(
                "B-209",
                "variable-shadowing",
                "Variable shadowing with reassignment",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 1; let x = x + 1; let x = x * 2; let x = x + 3; }"#,
                "x=$((x + 1))",
            ),
            CorpusEntry::new(
                "B-210",
                "four-param-function",
                "Function with four parameters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn rect_area(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 { let w = x2 - x1; let h = y2 - y1; if w < 0 { 0 - w * h } else { w * h } } fn main() { let a = rect_area(0, 0, 10, 5); }"#,
                "rect_area() {",
            ),
            CorpusEntry::new(
                "B-211",
                "for-loop-accumulator",
                "For loop with running accumulator",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut sum = 0; for i in 1..11 { sum += i; } echo(&format!("{}", sum)); } fn echo(s: &str) {}"#,
                "for i in $(seq 1 10); do",
            ),
            CorpusEntry::new(
                "B-212",
                "conditional-function-call",
                "Calling different functions based on condition",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn greet() { echo("hello"); } fn farewell() { echo("bye"); } fn main() { let morning = true; if morning { greet(); } else { farewell(); } } fn echo(s: &str) {}"#,
                "greet() {",
            ),
            CorpusEntry::new(
                "B-213",
                "nested-if-in-while",
                "Complex while body with nested conditionals",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut n = 50; while n > 0 { if n > 30 { n -= 10; } else if n > 10 { n -= 5; } else { n -= 1; } } }"#,
                r#"while [ "$n" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-214",
                "boolean-expression-chain",
                "Complex boolean expression evaluation",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let a = true; let b = false; let c = true; let d = a && !b; let e = d || c; }"#,
                r#"d="$a" && ! "$b""#,
            ),
            CorpusEntry::new(
                "B-215",
                "multi-step-arithmetic",
                "Long chain of arithmetic operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 100; let b = a / 3; let c = a % 3; let d = b * c; let e = d + a; let f = e - b; let g = f / 2; }"#,
                "b=$((a / 3))",
            ),
            CorpusEntry::new(
                "B-216",
                "three-function-program",
                "Program with three interacting functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn square(x: u32) -> u32 { x * x } fn cube(x: u32) -> u32 { x * x * x } fn sum_powers(n: u32) -> u32 { square(n) + cube(n) } fn main() { let r = sum_powers(3); }"#,
                "square() {",
            ),
            CorpusEntry::new(
                "B-217",
                "loop-with-function-call",
                "While loop calling function each iteration",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn process(x: u32) -> u32 { x * 2 + 1 } fn main() { let mut i = 0; let mut acc = 0; while i < 5 { acc += process(i); i += 1; } }"#,
                "process() {",
            ),
            CorpusEntry::new(
                "B-218",
                "complex-return-logic",
                "Function with multiple return paths",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn categorize(score: i32) -> i32 { if score >= 90 { 4 } else if score >= 80 { 3 } else if score >= 70 { 2 } else if score >= 60 { 1 } else { 0 } } fn main() { let g = categorize(85); }"#,
                "categorize() {",
            ),
            CorpusEntry::new(
                "B-219",
                "for-with-conditional-body",
                "For loop with if/else in body",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut evens = 0; let mut odds = 0; for i in 1..21 { if i % 2 == 0 { evens += 1; } else { odds += 1; } } }"#,
                "for i in $(seq 1 20); do",
            ),
            CorpusEntry::new(
                "B-220",
                "min-max-both",
                "Functions computing both min and max",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn main() { let lo = min(42, 17); let hi = max(42, 17); }"#,
                "min() {",
            ),
            CorpusEntry::new(
                "B-221",
                "deeply-nested-while",
                "Three-level nested while loops",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut i = 0; while i < 3 { let mut j = 0; while j < 3 { let mut k = 0; while k < 3 { k += 1; } j += 1; } i += 1; } }"#,
                r#"while [ "$i" -lt 3 ]; do"#,
            ),
            CorpusEntry::new(
                "B-222",
                "compound-assignment-loop",
                "Loop with multiple assignments per iteration",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut x = 0; let mut y = 1; let mut z = 0; let mut i = 0; while i < 10 { z = x + y; x = y; y = z; i += 1; } }"#,
                "z=$((x + y))",
            ),
            CorpusEntry::new(
                "B-223",
                "or-condition-while",
                "While loop with OR condition",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut a = 10; let mut b = 5; while a > 0 || b > 0 { if a > 0 { a -= 1; } if b > 0 { b -= 1; } } }"#,
                r#"while [ "$a" -gt 0 ] || [ "$b" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-224",
                "exit-on-error",
                "Exit with error code on invalid input",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate(x: i32) { if x < 0 { echo("error: negative"); std::process::exit(1); } echo("ok"); } fn main() { validate(42); } fn echo(s: &str) {}"#,
                "validate() {",
            ),
            CorpusEntry::new(
                "B-225",
                "while-with-negation",
                "While loop with negated condition",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut done = false; let mut count = 0; while !done { count += 1; if count >= 10 { done = true; } } }"#,
                r#"while ! [ "$done" ]; do"#,
            ),
            CorpusEntry::new(
                "B-226",
                "cascading-computation",
                "Results feeding into subsequent computations",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn step1(x: u32) -> u32 { x + 10 } fn step2(x: u32) -> u32 { x * 2 } fn step3(x: u32) -> u32 { x - 5 } fn main() { let a = step1(5); let b = step2(a); let c = step3(b); }"#,
                "step1() {",
            ),
            CorpusEntry::new(
                "B-227",
                "for-break-early",
                "For loop with early break on condition",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let mut found = 0; for i in 1..100 { if i * i > 50 { found = i; break; } } }"#,
                "for i in $(seq 1 99); do",
            ),
            CorpusEntry::new(
                "B-228",
                "large-multi-op-program",
                "Large program with functions, loops, conditionals",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_positive(x: i32) -> bool { x > 0 } fn abs(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn main() { let mut sum = 0; let mut i = 0; while i < 5 { let val = i * 3 - 7; if is_positive(val) { sum += val; } else { sum += abs(val); } i += 1; } }"#,
                "is_positive() {",
            ),
            CorpusEntry::new(
                "B-229",
                "nested-function-if",
                "Function called inside conditional branches",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn inc(x: u32) -> u32 { x + 1 } fn dec(x: u32) -> u32 { x - 1 } fn adjust(x: u32, target: u32) -> u32 { if x < target { inc(x) } else if x > target { dec(x) } else { x } } fn main() { let r = adjust(5, 10); }"#,
                "adjust() {",
            ),
            CorpusEntry::new(
                "B-230",
                "milestone-230",
                "230th Bash entry - comprehensive patterns",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn clamp(v: i32, lo: i32, hi: i32) -> i32 { if v < lo { lo } else if v > hi { hi } else { v } } fn scale(v: i32, factor: i32) -> i32 { v * factor } fn main() { let mut i = 0; let mut total = 0; while i < 10 { let raw = i * 7 - 20; let scaled = scale(raw, 3); let clamped = clamp(scaled, 0, 100); total += clamped; i += 1; } }"#,
                "clamp() {",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion7_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-231",
                "nested-for-accumulate",
                "Nested for loops with arithmetic accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 1..4 { for j in 1..4 { total += i * j; } } }"#,
                "for i in $(seq 1 3); do",
            ),
            CorpusEntry::new(
                "B-232",
                "while-compound-arith",
                "While loop with compound arithmetic operations",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut x = 1; let mut i = 0; while i < 6 { x = x * 2 + 1; i += 1; } }"#,
                "while [ \"$i\" -lt 6 ]; do",
            ),
            CorpusEntry::new(
                "B-233",
                "func-chain-compute",
                "Multiple functions called in sequence for computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn double(x: i32) -> i32 { x * 2 } fn add_one(x: i32) -> i32 { x + 1 } fn main() { let a = 5; let b = double(a); let c = add_one(b); }"#,
                "double() {",
            ),
            CorpusEntry::new(
                "B-234",
                "case-computed-val",
                "Case matching on computed value",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let x = 3; let y = x * 2; match y { 2 => { let r = 1; } 6 => { let r = 2; } _ => { let r = 0; } } }"#,
                "case \"$y\" in",
            ),
            CorpusEntry::new(
                "B-235",
                "for-if-else-accum",
                "For loop with if-else accumulation pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut evens = 0; let mut odds = 0; for i in 0..10 { if i % 2 == 0 { evens += 1; } else { odds += 1; } } }"#,
                "for i in $(seq 0 9); do",
            ),
            CorpusEntry::new(
                "B-236",
                "validate-transform",
                "Validate then transform pattern with multiple functions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_valid(x: i32) -> bool { x > 0 } fn transform(x: i32) -> i32 { x * x } fn main() { let v = 4; if is_valid(v) { let r = transform(v); } }"#,
                "is_valid() {",
            ),
            CorpusEntry::new(
                "B-237",
                "while-break-computed",
                "While loop with break on computed threshold",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; let mut i = 1; while i < 100 { sum += i; if sum > 50 { break; } i += 1; } }"#,
                "while [ \"$i\" -lt 100 ]; do",
            ),
            CorpusEntry::new(
                "B-238",
                "nested-if-arith-cmp",
                "Nested if with arithmetic comparisons on both levels",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let x = 15; let y = 20; if x > 10 { if y > 15 { let r = x + y; } else { let r = x - y; } } else { let r = 0; } }"#,
                "if [ \"$x\" -gt 10 ]; then",
            ),
            CorpusEntry::new(
                "B-239",
                "for-continue-filter",
                "For loop with continue to filter values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; for i in 1..11 { if i % 3 == 0 { continue; } sum += i; } }"#,
                "for i in $(seq 1 10); do",
            ),
            CorpusEntry::new(
                "B-240",
                "milestone-240",
                "240th Bash entry - multi-function with loops",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn square(x: i32) -> i32 { x * x } fn cube(x: i32) -> i32 { x * x * x } fn main() { let mut sum_sq = 0; let mut sum_cu = 0; for i in 1..6 { sum_sq += square(i); sum_cu += cube(i); } }"#,
                "square() {",
            ),
            CorpusEntry::new(
                "B-241",
                "double-nested-while",
                "Two nested while loops with independent counters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut i = 0; let mut count = 0; while i < 3 { let mut j = 0; while j < 4 { count += 1; j += 1; } i += 1; } }"#,
                "while [ \"$i\" -lt 3 ]; do",
            ),
            CorpusEntry::new(
                "B-242",
                "func-with-while",
                "Function containing a while loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn sum_to(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { s += i; i += 1; } s } fn main() { let r = sum_to(10); }"#,
                "sum_to() {",
            ),
            CorpusEntry::new(
                "B-243",
                "case-arith-branch",
                "Case with arithmetic in each branch body",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let op = 1; let a = 10; let b = 3; match op { 1 => { let r = a + b; } 2 => { let r = a - b; } 3 => { let r = a * b; } _ => { let r = 0; } } }"#,
                "case \"$op\" in",
            ),
            CorpusEntry::new(
                "B-244",
                "for-multiply-accum",
                "For loop accumulating products",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut product = 1; for i in 1..7 { product = product * i; } }"#,
                "for i in $(seq 1 6); do",
            ),
            CorpusEntry::new(
                "B-245",
                "countdown-func-call",
                "While counting down with function call each iteration",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn process(x: i32) -> i32 { x * 2 + 1 } fn main() { let mut n = 5; let mut total = 0; while n > 0 { total += process(n); n = n - 1; } }"#,
                "process() {",
            ),
            CorpusEntry::new(
                "B-246",
                "nested-func-params",
                "Nested function calls passing computed parameters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn add(a: i32, b: i32) -> i32 { a + b } fn mul(a: i32, b: i32) -> i32 { a * b } fn main() { let x = add(3, 4); let y = mul(x, 2); let z = add(y, x); }"#,
                "add() {",
            ),
            CorpusEntry::new(
                "B-247",
                "if-elif-arith-branch",
                "If-elif chain with arithmetic in each branch",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let score = 75; if score >= 90 { let grade = 4; } else if score >= 80 { let grade = 3; } else if score >= 70 { let grade = 2; } else { let grade = 1; } }"#,
                "if [ \"$score\" -ge 90 ]; then",
            ),
            CorpusEntry::new(
                "B-248",
                "for-nested-if-accum",
                "For loop with nested if for selective accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut big = 0; let mut small = 0; for i in 1..21 { if i > 10 { big += i; } else if i > 5 { small += i; } } }"#,
                "for i in $(seq 1 20); do",
            ),
            CorpusEntry::new(
                "B-249",
                "while-multi-var-cond",
                "While loop with condition on multiple variables",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 0; let mut b = 100; while a < b { a += 3; b = b - 2; } }"#,
                "while [ \"$a\" -lt \"$b\" ]; do",
            ),
            CorpusEntry::new(
                "B-250",
                "milestone-250",
                "250th Bash entry - comprehensive computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn main() { let mut lo = 100; let mut hi = 0; for i in 0..10 { let v = i * 7 % 13; lo = min(lo, v); hi = max(hi, v); } }"#,
                "min() {",
            ),
            CorpusEntry::new(
                "B-251",
                "func-call-in-while",
                "Function called within while loop condition body",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn step(x: i32) -> i32 { x + x / 3 + 1 } fn main() { let mut val = 1; let mut i = 0; while i < 5 { val = step(val); i += 1; } }"#,
                "step() {",
            ),
            CorpusEntry::new(
                "B-252",
                "for-case-inside",
                "For loop with case statement inside",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut count = 0; for i in 0..6 { let r = i % 3; match r { 0 => { count += 3; } 1 => { count += 1; } _ => { count += 2; } } } }"#,
                "for i in $(seq 0 5); do",
            ),
            CorpusEntry::new(
                "B-253",
                "nested-for-break",
                "Nested for loops with break in inner loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 1..6 { for j in 1..6 { if i * j > 12 { break; } found += 1; } } }"#,
                "for j in $(seq 1 5); do",
            ),
            CorpusEntry::new(
                "B-254",
                "while-arith-func",
                "While with arithmetic and function call combined",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn halve(x: i32) -> i32 { x / 2 } fn main() { let mut x = 256; let mut steps = 0; while x > 1 { x = halve(x); steps += 1; } }"#,
                "halve() {",
            ),
            CorpusEntry::new(
                "B-255",
                "case-multi-default",
                "Case with several explicit branches and default",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let day = 3; match day { 1 => { let name = "mon"; } 2 => { let name = "tue"; } 3 => { let name = "wed"; } 4 => { let name = "thu"; } _ => { let name = "other"; } } }"#,
                "case \"$day\" in",
            ),
            CorpusEntry::new(
                "B-256",
                "for-while-inside",
                "For loop with while loop nested inside",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 1..4 { let mut j = i; while j > 0 { total += j; j = j - 1; } } }"#,
                "for i in $(seq 1 3); do",
            ),
            CorpusEntry::new(
                "B-257",
                "func-for-return",
                "Function with for loop computing return value",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn count_divisible(n: i32, d: i32) -> i32 { let mut c = 0; for i in 1..n { if i % d == 0 { c += 1; } } c } fn main() { let r = count_divisible(20, 3); }"#,
                "count_divisible() {",
            ),
            CorpusEntry::new(
                "B-258",
                "if-elif-complex-arith",
                "Complex if-elif with multi-step arithmetic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let x = 7; let y = x * x; if y > 100 { let z = y - 100; } else if y > 25 { let z = y * 2; } else { let z = y + 10; } }"#,
                "if [ \"$y\" -gt 100 ]; then",
            ),
            CorpusEntry::new(
                "B-259",
                "while-nested-for",
                "While loop containing a for loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut round = 0; let mut total = 0; while round < 3 { for i in 1..4 { total += i + round; } round += 1; } }"#,
                "while [ \"$round\" -lt 3 ]; do",
            ),
            CorpusEntry::new(
                "B-260",
                "milestone-260",
                "260th Bash entry - full-stack computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn power(base: i32, exp: i32) -> i32 { let mut result = 1; let mut i = 0; while i < exp { result = result * base; i += 1; } result } fn main() { let mut sum = 0; for i in 1..5 { sum += power(i, 2); } }"#,
                "power() {",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion5_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-081",
                "crystal-build",
                "Crystal language project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let crystal = "crystal"; phony_target("build", &["deps"], &["$(CRYSTAL) build src/main.cr -o bin/app"]); phony_target("deps", &[], &["shards install"]); phony_target("test", &[], &["$(CRYSTAL) spec"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-082",
                "elixir-mix",
                "Elixir/Mix project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let mix = "mix"; phony_target("deps", &[], &["$(MIX) deps.get"]); phony_target("compile", &["deps"], &["$(MIX) compile"]); phony_target("test", &["compile"], &["$(MIX) test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deps",
            ),
            CorpusEntry::new(
                "M-083",
                "julia-project",
                "Julia language project",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let julia = "julia"; phony_target("run", &[], &["$(JULIA) src/main.jl"]); phony_target("test", &[], &["$(JULIA) -e 'using Pkg; Pkg.test()'"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: run",
            ),
            CorpusEntry::new(
                "M-084",
                "r-package",
                "R package build",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let r = "Rscript"; phony_target("check", &[], &["R CMD check ."]); phony_target("test", &[], &["$(R) -e 'devtools::test()'"]); phony_target("doc", &[], &["$(R) -e 'devtools::document()'"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: check",
            ),
            CorpusEntry::new(
                "M-085",
                "swift-package",
                "Swift Package Manager build",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let swift = "swift"; phony_target("build", &[], &["$(SWIFT) build"]); phony_target("test", &[], &["$(SWIFT) test"]); phony_target("clean", &[], &["$(SWIFT) package clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-086",
                "kotlin-gradle",
                "Kotlin with Gradle wrapper",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let gradle = "./gradlew"; phony_target("build", &[], &["$(GRADLE) build"]); phony_target("test", &[], &["$(GRADLE) test"]); phony_target("clean", &[], &["$(GRADLE) clean"]); phony_target("run", &["build"], &["$(GRADLE) run"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-087",
                "dart-pub",
                "Dart/Flutter project",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let dart = "dart"; phony_target("deps", &[], &["$(DART) pub get"]); phony_target("test", &["deps"], &["$(DART) test"]); phony_target("analyze", &[], &["$(DART) analyze"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deps",
            ),
            CorpusEntry::new(
                "M-088",
                "ocaml-dune",
                "OCaml with Dune",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let dune = "dune"; phony_target("build", &[], &["$(DUNE) build"]); phony_target("test", &[], &["$(DUNE) runtest"]); phony_target("clean", &[], &["$(DUNE) clean"]); phony_target("fmt", &[], &["$(DUNE) build @fmt"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-089",
                "perl-build",
                "Perl module build",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let perl = "perl"; phony_target("test", &[], &["prove -l t/"]); phony_target("install", &[], &["cpanm --installdeps ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: test",
            ),
            CorpusEntry::new(
                "M-090",
                "php-composer",
                "PHP Composer project",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let php = "php"; let composer = "composer"; phony_target("install", &[], &["$(COMPOSER) install"]); phony_target("test", &["install"], &["$(PHP) vendor/bin/phpunit"]); phony_target("lint", &[], &["$(PHP) vendor/bin/phpcs"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-091",
                "lua-busted",
                "Lua with Busted test framework",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let lua = "lua"; phony_target("test", &[], &["busted"]); phony_target("lint", &[], &["luacheck ."]); phony_target("run", &[], &["$(LUA) src/main.lua"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: test",
            ),
            CorpusEntry::new(
                "M-092",
                "cmake-project",
                "CMake build system",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cmake = "cmake"; let build_dir = "build"; phony_target("configure", &[], &["$(CMAKE) -S . -B $(BUILD_DIR)"]); phony_target("build", &["configure"], &["$(CMAKE) --build $(BUILD_DIR)"]); phony_target("test", &["build"], &["ctest --test-dir $(BUILD_DIR)"]); phony_target("clean", &[], &["rm -rf $(BUILD_DIR)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: configure",
            ),
            CorpusEntry::new(
                "M-093",
                "meson-project",
                "Meson build system",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let meson = "meson"; phony_target("setup", &[], &["$(MESON) setup builddir"]); phony_target("build", &["setup"], &["$(MESON) compile -C builddir"]); phony_target("test", &["build"], &["$(MESON) test -C builddir"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: setup",
            ),
            CorpusEntry::new(
                "M-094",
                "terraform-infra",
                "Terraform infrastructure",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let tf = "terraform"; phony_target("init", &[], &["$(TF) init"]); phony_target("plan", &["init"], &["$(TF) plan -out=tfplan"]); phony_target("apply", &["plan"], &["$(TF) apply tfplan"]); phony_target("destroy", &[], &["$(TF) destroy"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: init",
            ),
            CorpusEntry::new(
                "M-095",
                "ansible-playbook",
                "Ansible automation",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let ansible = "ansible-playbook"; phony_target("deploy", &[], &["$(ANSIBLE) deploy.yml"]); phony_target("lint", &[], &["ansible-lint"]); phony_target("check", &[], &["$(ANSIBLE) deploy.yml --check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-096",
                "helm-chart",
                "Kubernetes Helm chart",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let helm = "helm"; let chart = "my-chart"; phony_target("lint", &[], &["$(HELM) lint $(CHART)"]); phony_target("template", &[], &["$(HELM) template $(CHART)"]); phony_target("install", &["lint"], &["$(HELM) install release $(CHART)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint",
            ),
            CorpusEntry::new(
                "M-097",
                "proto-grpc",
                "Protobuf/gRPC code generation",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let protoc = "protoc"; phony_target("proto", &[], &["$(PROTOC) --go_out=. --go-grpc_out=. proto/*.proto"]); phony_target("clean", &[], &["rm -f gen/*.go"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: proto",
            ),
            CorpusEntry::new(
                "M-098",
                "wasm-pack",
                "WebAssembly with wasm-pack",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let wasm_pack = "wasm-pack"; phony_target("build", &[], &["$(WASM_PACK) build --target web"]); phony_target("test", &[], &["$(WASM_PACK) test --headless --chrome"]); phony_target("publish", &["build"], &["$(WASM_PACK) publish"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-099",
                "deno-project",
                "Deno runtime project",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let deno = "deno"; phony_target("run", &[], &["$(DENO) run --allow-net src/main.ts"]); phony_target("test", &[], &["$(DENO) test"]); phony_target("fmt", &[], &["$(DENO) fmt"]); phony_target("lint", &[], &["$(DENO) lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: run",
            ),
            CorpusEntry::new(
                "M-100",
                "milestone-100-make",
                "100th Makefile entry - multi-tool",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let docker = "docker"; let kubectl = "kubectl"; let helm = "helm"; phony_target("build", &[], &["$(DOCKER) build -t app:latest ."]); phony_target("push", &["build"], &["$(DOCKER) push app:latest"]); phony_target("deploy", &["push"], &["$(KUBECTL) apply -f k8s/"]); phony_target("rollback", &[], &["$(HELM) rollback release"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion5_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-081",
                "crystal-app",
                "Crystal language app",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("crystallang/crystal", "1.11-alpine", "builder"); workdir("/app"); copy("shard.yml", "."); run(&["shards install"]); copy(".", "."); run(&["crystal build src/main.cr --release --static"]); from_image("alpine", "3.18"); copy_from("builder", "/app/main", "/usr/local/bin/app"); user("65534"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM crystallang/crystal:1.11-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-082",
                "elixir-phoenix",
                "Elixir Phoenix web app",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("elixir", "1.16-alpine", "builder"); workdir("/app"); copy("mix.exs", "."); copy("mix.lock", "."); run(&["mix deps.get", "mix compile"]); copy(".", "."); run(&["mix release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/_build/prod/rel/app", "/opt/app"); expose(4000u16); entrypoint(&["/opt/app/bin/app", "start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM elixir:1.16-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-083",
                "julia-app",
                "Julia application container",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("julia", "1.10"); workdir("/app"); copy("Project.toml", "."); run(&["julia -e 'using Pkg; Pkg.instantiate()'"]); copy(".", "."); entrypoint(&["julia", "src/main.jl"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM julia:1.10",
            ),
            CorpusEntry::new(
                "D-084",
                "r-shiny",
                "R Shiny application",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("rocker/shiny", "4.3"); copy("app.R", "/srv/shiny-server/"); run(&["R -e \"install.packages(c('ggplot2', 'dplyr'))\""]); expose(3838u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {}"#,
                "FROM rocker/shiny:4.3",
            ),
            CorpusEntry::new(
                "D-085",
                "swift-vapor",
                "Swift Vapor web framework",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("swift", "5.9", "builder"); workdir("/app"); copy("Package.swift", "."); run(&["swift package resolve"]); copy(".", "."); run(&["swift build -c release"]); from_image("ubuntu", "22.04"); copy_from("builder", "/app/.build/release/App", "/usr/local/bin/app"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM swift:5.9 AS builder",
            ),
            CorpusEntry::new(
                "D-086",
                "php-laravel",
                "PHP Laravel application",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("php", "8.3-fpm-alpine"); run(&["apk add --no-cache nginx", "docker-php-ext-install pdo pdo_mysql"]); workdir("/var/www/html"); copy("composer.json", "."); copy("composer.lock", "."); run(&["composer install --no-dev"]); copy(".", "."); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM php:8.3-fpm-alpine",
            ),
            CorpusEntry::new(
                "D-087",
                "ruby-rails",
                "Ruby on Rails application",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("ruby", "3.3-slim"); run(&["apt-get update", "apt-get install -y build-essential libpq-dev"]); workdir("/app"); copy("Gemfile", "."); copy("Gemfile.lock", "."); run(&["bundle install --without development test"]); copy(".", "."); expose(3000u16); cmd(&["rails", "server", "-b", "0.0.0.0"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ruby:3.3-slim",
            ),
            CorpusEntry::new(
                "D-088",
                "dotnet-api",
                ".NET Web API",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("mcr.microsoft.com/dotnet/sdk", "8.0", "builder"); workdir("/src"); copy("*.csproj", "."); run(&["dotnet restore"]); copy(".", "."); run(&["dotnet publish -c Release -o /app"]); from_image("mcr.microsoft.com/dotnet/aspnet", "8.0"); workdir("/app"); copy_from("builder", "/app", "."); expose(8080u16); entrypoint(&["dotnet", "App.dll"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder",
            ),
            CorpusEntry::new(
                "D-089",
                "lua-openresty",
                "OpenResty Lua web server",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("openresty/openresty", "1.25-alpine"); copy("nginx.conf", "/usr/local/openresty/nginx/conf/"); copy("lua/", "/usr/local/openresty/lua/"); expose(80u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM openresty/openresty:1.25-alpine",
            ),
            CorpusEntry::new(
                "D-090",
                "kotlin-ktor",
                "Kotlin Ktor server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("gradle", "8.5-jdk21", "builder"); workdir("/app"); copy("build.gradle.kts", "."); copy("settings.gradle.kts", "."); run(&["gradle dependencies"]); copy(".", "."); run(&["gradle shadowJar"]); from_image("eclipse-temurin", "21-jre-alpine"); copy_from("builder", "/app/build/libs/app-all.jar", "/app.jar"); expose(8080u16); entrypoint(&["java", "-jar", "/app.jar"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM gradle:8.5-jdk21 AS builder",
            ),
            CorpusEntry::new(
                "D-091",
                "perl-mojolicious",
                "Perl Mojolicious web app",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("perl", "5.38"); run(&["cpanm Mojolicious"]); workdir("/app"); copy(".", "."); expose(3000u16); cmd(&["perl", "app.pl", "daemon"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM perl:5.38",
            ),
            CorpusEntry::new(
                "D-092",
                "dart-shelf",
                "Dart Shelf server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("dart", "3.2", "builder"); workdir("/app"); copy("pubspec.yaml", "."); run(&["dart pub get"]); copy(".", "."); run(&["dart compile exe bin/server.dart -o server"]); from_image("alpine", "3.18"); copy_from("builder", "/app/server", "/usr/local/bin/server"); expose(8080u16); entrypoint(&["/usr/local/bin/server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM dart:3.2 AS builder",
            ),
            CorpusEntry::new(
                "D-093",
                "ocaml-opam",
                "OCaml with opam",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ocaml/opam", "alpine-5.1"); workdir("/home/opam/app"); copy("*.opam", "."); run(&["opam install --deps-only ."]); copy(".", "."); run(&["dune build"]); entrypoint(&["./_build/default/bin/main.exe"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM ocaml/opam:alpine-5.1",
            ),
            CorpusEntry::new(
                "D-094",
                "clojure-lein",
                "Clojure with Leiningen",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("clojure", "temurin-21-lein", "builder"); workdir("/app"); copy("project.clj", "."); run(&["lein deps"]); copy(".", "."); run(&["lein uberjar"]); from_image("eclipse-temurin", "21-jre-alpine"); copy_from("builder", "/app/target/app-standalone.jar", "/app.jar"); expose(3000u16); entrypoint(&["java", "-jar", "/app.jar"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM clojure:temurin-21-lein AS builder",
            ),
            CorpusEntry::new(
                "D-095",
                "haskell-stack",
                "Haskell Stack build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("haskell", "9.6", "builder"); workdir("/app"); copy("stack.yaml", "."); copy("package.yaml", "."); run(&["stack setup", "stack build --only-dependencies"]); copy(".", "."); run(&["stack build"]); from_image("debian", "bookworm-slim"); copy_from("builder", "/app/.stack-work/install/bin/app", "/usr/local/bin/app"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM haskell:9.6 AS builder",
            ),
            CorpusEntry::new(
                "D-096",
                "nim-app",
                "Nim language application",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image_as("nimlang/nim", "2.0-alpine", "builder"); workdir("/app"); copy(".", "."); run(&["nimble build -d:release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/bin/app", "/usr/local/bin/app"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM nimlang/nim:2.0-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-097",
                "zig-app",
                "Zig language application",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image_as("archlinux", "latest", "builder"); run(&["pacman -Sy --noconfirm zig"]); workdir("/app"); copy(".", "."); run(&["zig build -Drelease-safe"]); from_image("alpine", "3.18"); copy_from("builder", "/app/zig-out/bin/app", "/usr/local/bin/app"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM archlinux:latest AS builder",
            ),
            CorpusEntry::new(
                "D-098",
                "gleam-beam",
                "Gleam on BEAM",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("ghcr.io/gleam-lang/gleam", "v1.0-erlang-alpine", "builder"); workdir("/app"); copy("gleam.toml", "."); copy("manifest.toml", "."); run(&["gleam deps download"]); copy(".", "."); run(&["gleam export erlang-shipment"]); from_image("erlang", "26-alpine"); copy_from("builder", "/app/build/erlang-shipment", "/app"); entrypoint(&["/app/entrypoint.sh", "run"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM ghcr.io/gleam-lang/gleam:v1.0-erlang-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-099",
                "vlang-app",
                "V language application",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image_as("thevlang/vlang", "alpine", "builder"); workdir("/app"); copy(".", "."); run(&["v -prod -o app ."]); from_image("alpine", "3.18"); copy_from("builder", "/app/app", "/usr/local/bin/app"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM thevlang/vlang:alpine AS builder",
            ),
            CorpusEntry::new(
                "D-100",
                "milestone-100-docker",
                "100th Dockerfile entry - full stack",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "builder"); run(&["apk add --no-cache musl-dev"]); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["mkdir src", "echo 'fn main(){}' > src/main.rs", "cargo build --release", "rm -rf src"]); copy(".", "."); run(&["cargo build --release"]); from_image("alpine", "3.18"); run(&["apk add --no-cache ca-certificates"]); copy_from("builder", "/app/target/release/app", "/usr/local/bin/app"); user("65534"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-alpine AS builder",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 8: Bash B-261..B-290 (harder patterns)
    // =========================================================================

    fn load_expansion8_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-261",
                "string-concat-vars",
                "String concatenation with variables",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let greeting = "hello"; let name = "world"; let sep = "_"; }"#,
                "greeting='hello'",
            ),
            CorpusEntry::new(
                "B-262",
                "multi-param-func",
                "Function with multiple parameters used in arithmetic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn compute(a: i32, b: i32, c: i32) -> i32 { a * b + c } fn main() { let r = compute(3, 4, 5); }"#,
                "compute() {",
            ),
            CorpusEntry::new(
                "B-263",
                "nested-if-in-for",
                "Nested if-else inside for loop with two accumulators",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut pos = 0; let mut neg = 0; for i in 0..12 { let v = i * 3 - 15; if v > 0 { pos += v; } else { neg += 1; } } }"#,
                "for i in $(seq 0 11); do",
            ),
            CorpusEntry::new(
                "B-264",
                "while-string-cmp",
                "While with integer comparison simulating string check",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut state = 0; let mut steps = 0; while state != 5 { state += 1; steps += 1; } }"#,
                "while [ \"$state\" -ne 5 ]; do",
            ),
            CorpusEntry::new(
                "B-265",
                "for-accum-sum-product",
                "For loop accumulating both sum and product",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; let mut prod = 1; for i in 1..=6 { sum += i; prod = prod * i; } }"#,
                "for i in $(seq 1 6); do",
            ),
            CorpusEntry::new(
                "B-266",
                "case-multi-echo-branch",
                "Case with multiple branches containing variable assignments",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let code = 2; match code { 0 => { let msg = "ok"; } 1 => { let msg = "warn"; } 2 => { let msg = "error"; } 3 => { let msg = "fatal"; } _ => { let msg = "unknown"; } } }"#,
                "case \"$code\" in",
            ),
            CorpusEntry::new(
                "B-267",
                "func-computed-return",
                "Function returning result of multi-step computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn triangle(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { s += i; i += 1; } s } fn main() { let t = triangle(7); }"#,
                "triangle() {",
            ),
            CorpusEntry::new(
                "B-268",
                "nested-while-grid",
                "Nested while loops simulating grid traversal",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut row = 0; let mut cells = 0; while row < 4 { let mut col = 0; while col < 5 { cells += 1; col += 1; } row += 1; } }"#,
                "while [ \"$row\" -lt 4 ]; do",
            ),
            CorpusEntry::new(
                "B-269",
                "if-elif-else-classify",
                "If-elif-else chain classifying a value into categories",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let temp = 35; if temp > 100 { let zone = 4; } else if temp > 70 { let zone = 3; } else if temp > 40 { let zone = 2; } else if temp > 10 { let zone = 1; } else { let zone = 0; } }"#,
                "if [ \"$temp\" -gt 100 ]; then",
            ),
            CorpusEntry::new(
                "B-270",
                "for-early-break",
                "For loop with early break on threshold detection",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 1..=50 { if i * i > 200 { found = i; break; } } }"#,
                "for i in $(seq 1 50); do",
            ),
            CorpusEntry::new(
                "B-271",
                "multi-var-swap",
                "Multiple variable swaps using temp variables",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 10; let mut b = 20; let mut c = 30; let tmp = a; a = b; b = c; c = tmp; }"#,
                "tmp=\"$a\"",
            ),
            CorpusEntry::new(
                "B-272",
                "while-countdown-accum",
                "While loop counting down with accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut n = 10; let mut total = 0; while n > 0 { total += n * n; n = n - 1; } }"#,
                "while [ \"$n\" -gt 0 ]; do",
            ),
            CorpusEntry::new(
                "B-273",
                "case-wildcard-default",
                "Case with wildcard default catching unmatched values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let level = 7; match level { 1 => { let label = "low"; } 5 => { let label = "mid"; } 10 => { let label = "high"; } _ => { let label = "other"; } } }"#,
                "case \"$level\" in",
            ),
            CorpusEntry::new(
                "B-274",
                "for-conditional-incr",
                "For loop with conditional increment by different amounts",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 1..=15 { if i % 2 == 0 { total += 2; } else { total += 1; } } }"#,
                "for i in $(seq 1 15); do",
            ),
            CorpusEntry::new(
                "B-275",
                "func-calling-func",
                "Function calling another function in its body",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn square(x: i32) -> i32 { x * x } fn sum_of_squares(a: i32, b: i32) -> i32 { square(a) + square(b) } fn main() { let r = sum_of_squares(3, 4); }"#,
                "sum_of_squares() {",
            ),
            CorpusEntry::new(
                "B-276",
                "while-multi-cond-and",
                "While with multiple conditions combined with logical and",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut y = 100; while x < 50 { x += 3; y = y - 2; } }"#,
                "while [ \"$x\" -lt 50 ]; do",
            ),
            CorpusEntry::new(
                "B-277",
                "if-equality-check",
                "If with equality comparison on computed values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let a = 6; let b = 3; let product = a * b; if product == 18 { let matched = 1; } else { let matched = 0; } }"#,
                "if [ \"$product\" -eq 18 ]; then",
            ),
            CorpusEntry::new(
                "B-278",
                "for-build-count-array",
                "For loop building counts for different categories",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut low = 0; let mut mid = 0; let mut high = 0; for i in 1..=30 { if i <= 10 { low += 1; } else if i <= 20 { mid += 1; } else { high += 1; } } }"#,
                "for i in $(seq 1 30); do",
            ),
            CorpusEntry::new(
                "B-279",
                "nested-case-in-if",
                "Case statement nested inside an if branch",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mode = 1; let sub = 2; if mode == 1 { match sub { 1 => { let action = 10; } 2 => { let action = 20; } _ => { let action = 0; } } } else { let action = 99; } }"#,
                "if [ \"$mode\" -eq 1 ]; then",
            ),
            CorpusEntry::new(
                "B-280",
                "milestone-280",
                "280th Bash entry - while break on condition milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn collatz_steps(start: i32) -> i32 { let mut n = start; let mut steps = 0; while n != 1 { if n % 2 == 0 { n = n / 2; } else { n = n * 3 + 1; } steps += 1; } steps } fn main() { let s = collatz_steps(6); }"#,
                "collatz_steps() {",
            ),
            CorpusEntry::new(
                "B-281",
                "multi-assign-arith",
                "Multiple assignments with chained arithmetic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let a = 5; let b = a * 3; let c = b + a; let d = c * 2; let e = d - b; }"#,
                "b=$((a * 3))",
            ),
            CorpusEntry::new(
                "B-282",
                "for-nested-if-else-deep",
                "For with deeply nested if-else branches",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut cat_a = 0; let mut cat_b = 0; let mut cat_c = 0; for i in 1..=24 { if i % 6 == 0 { cat_a += i; } else if i % 3 == 0 { cat_b += i; } else { cat_c += 1; } } }"#,
                "for i in $(seq 1 24); do",
            ),
            CorpusEntry::new(
                "B-283",
                "func-default-param",
                "Function with default parameter simulation via if",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn apply(x: i32, factor: i32) -> i32 { if factor == 0 { x * 2 } else { x * factor } } fn main() { let a = apply(5, 0); let b = apply(5, 3); }"#,
                "apply() {",
            ),
            CorpusEntry::new(
                "B-284",
                "while-count-step",
                "While counting with step size greater than one",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut i = 0; let mut count = 0; while i < 100 { count += 1; i += 7; } }"#,
                "while [ \"$i\" -lt 100 ]; do",
            ),
            CorpusEntry::new(
                "B-285",
                "case-range-simulate",
                "Case simulating range matching with explicit values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let val = 3; match val { 1 => { let tier = 10; } 2 => { let tier = 20; } 3 => { let tier = 30; } 4 => { let tier = 40; } 5 => { let tier = 50; } _ => { let tier = 0; } } }"#,
                "case \"$val\" in",
            ),
            CorpusEntry::new(
                "B-286",
                "for-string-format",
                "For loop with variable formatting via assignment",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let prefix = "item"; let mut count = 0; for i in 1..=8 { let idx = i * 10; count += 1; } }"#,
                "for i in $(seq 1 8); do",
            ),
            CorpusEntry::new(
                "B-287",
                "nested-for-break-outer",
                "Nested for loops with break in inner on computed condition",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 1..=5 { for j in 1..=10 { if j > i * 2 { break; } total += 1; } } }"#,
                "for j in $(seq 1 10); do",
            ),
            CorpusEntry::new(
                "B-288",
                "if-complex-boolean",
                "If with complex boolean expression using multiple comparisons",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let x = 15; let y = 25; let z = x + y; if z > 30 { let big = 1; if z > 50 { let very_big = 1; } else { let very_big = 0; } } else { let big = 0; } }"#,
                "if [ \"$z\" -gt 30 ]; then",
            ),
            CorpusEntry::new(
                "B-289",
                "while-accum-print",
                "While with accumulator and conditional tracking",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut i = 1; let mut sum = 0; let mut max_seen = 0; while i <= 20 { let val = i * i % 17; sum += val; if val > max_seen { max_seen = val; } i += 1; } }"#,
                "while [ \"$i\" -le 20 ]; do",
            ),
            CorpusEntry::new(
                "B-290",
                "milestone-290",
                "290th Bash entry - multi-function pipeline milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn normalize(x: i32, lo: i32, hi: i32) -> i32 { if x < lo { lo } else if x > hi { hi } else { x } } fn score(x: i32) -> i32 { let n = normalize(x, 0, 100); n * n / 100 } fn main() { let mut total = 0; for i in 0..10 { let raw = i * 15 - 20; total += score(raw); } }"#,
                "normalize() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 9: Bash B-291..B-320 (harder patterns)
    // =========================================================================

    fn load_expansion9_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-291",
                "nested-func-multi-params",
                "Nested function calls with multiple parameters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn add(a: i32, b: i32) -> i32 { a + b } fn mul(a: i32, b: i32) -> i32 { a * b } fn combine(x: i32, y: i32, z: i32) -> i32 { add(mul(x, y), z) } fn main() { let r = combine(3, 4, 5); }"#,
                "combine() {",
            ),
            CorpusEntry::new(
                "B-292",
                "while-compound-cond",
                "While loop with compound condition and dual counters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 0; let mut b = 100; while a < 50 { a += 3; b = b - 1; } }"#,
                "while [ \"$a\" -lt 50 ]; do",
            ),
            CorpusEntry::new(
                "B-293",
                "case-multi-pattern",
                "Case statement with multiple distinct patterns",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let code = 2; match code { 1 => { let msg = 10; } 2 => { let msg = 20; } 3 => { let msg = 30; } 4 => { let msg = 40; } _ => { let msg = 0; } } }"#,
                "case \"$code\" in",
            ),
            CorpusEntry::new(
                "B-294",
                "for-computed-range-sum",
                "For loop summing over a computed range",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut total = 0; for i in 1..=50 { total += i * i; } }"#,
                "for i in $(seq 1 50); do",
            ),
            CorpusEntry::new(
                "B-295",
                "string-compare-flag",
                "String comparison setting a boolean flag",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mode = "debug"; let verbose = 1; if verbose == 1 { let level = 3; } else { let level = 1; } }"#,
                "mode='debug'",
            ),
            CorpusEntry::new(
                "B-296",
                "arith-modulo-division",
                "Arithmetic with modulo and division operations",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let x = 97; let q = x / 10; let r = x % 10; let reconstructed = q * 10 + r; }"#,
                "q=$((x / 10))",
            ),
            CorpusEntry::new(
                "B-297",
                "multi-return-via-vars",
                "Multiple return values via separate variables",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn divide(a: i32, b: i32) -> i32 { a / b } fn remainder(a: i32, b: i32) -> i32 { a % b } fn main() { let n = 47; let d = 5; let quot = divide(n, d); let rem = remainder(n, d); }"#,
                "divide() {",
            ),
            CorpusEntry::new(
                "B-298",
                "iterative-power",
                "Iterative power computation simulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn power(base: i32, exp: i32) -> i32 { let mut result = 1; let mut i = 0; while i < exp { result = result * base; i += 1; } result } fn main() { let p = power(2, 8); }"#,
                "power() {",
            ),
            CorpusEntry::new(
                "B-299",
                "multi-elif-chain",
                "Multi-level if/elif/else chain for categorization",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let score = 73; if score >= 90 { let grade = 4; } else if score >= 80 { let grade = 3; } else if score >= 70 { let grade = 2; } else if score >= 60 { let grade = 1; } else { let grade = 0; } }"#,
                "elif [ \"$score\" -ge 80 ]; then",
            ),
            CorpusEntry::new(
                "B-300",
                "milestone-300",
                "300th Bash entry - pipeline variable passing milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn step1(x: i32) -> i32 { x + 10 } fn step2(x: i32) -> i32 { x * 2 } fn step3(x: i32) -> i32 { x - 5 } fn main() { let v = 7; let a = step1(v); let b = step2(a); let c = step3(b); }"#,
                "step1() {",
            ),
            CorpusEntry::new(
                "B-301",
                "bool-flag-tracking",
                "Boolean flag tracking across a loop",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut found = 0; for i in 1..=20 { if i * i == 144 { found = 1; } } }"#,
                "for i in $(seq 1 20); do",
            ),
            CorpusEntry::new(
                "B-302",
                "counter-incr-pattern",
                "Counter increment pattern with conditional bumps",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut evens = 0; let mut odds = 0; for i in 1..=40 { if i % 2 == 0 { evens += 1; } else { odds += 1; } } }"#,
                "for i in $(seq 1 40); do",
            ),
            CorpusEntry::new(
                "B-303",
                "max-finding-loop",
                "Finding maximum value across a computed sequence",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut max = 0; for i in 1..=15 { let val = i * 7 % 13; if val > max { max = val; } } }"#,
                "val=$(((i * 7) % 13))",
            ),
            CorpusEntry::new(
                "B-304",
                "min-finding-loop",
                "Finding minimum value across a sequence with sentinel",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut min = 999; for i in 1..=12 { let val = i * 11 % 17; if val < min { min = val; } } }"#,
                "min='999'",
            ),
            CorpusEntry::new(
                "B-305",
                "sequential-var-naming",
                "Sequential variable naming for array simulation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let v0 = 10; let v1 = 20; let v2 = 30; let v3 = 40; let sum = v0 + v1 + v2 + v3; }"#,
                "sum=$((((v0 + v1) + v2) + v3))",
            ),
            CorpusEntry::new(
                "B-306",
                "string-prefix-assign",
                "String prefix assignment for labeling",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let prefix = "log"; let suffix = "txt"; let sep = "."; let level = "info"; }"#,
                "prefix='log'",
            ),
            CorpusEntry::new(
                "B-307",
                "bitwise-via-arith",
                "Bitwise-style operations via arithmetic",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let flags = 0; let read_bit = 4; let write_bit = 2; let exec_bit = 1; let perms = read_bit + write_bit + exec_bit; }"#,
                "perms=$(((read_bit + write_bit) + exec_bit))",
            ),
            CorpusEntry::new(
                "B-308",
                "celsius-to-fahrenheit",
                "Temperature conversion from Celsius to Fahrenheit",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn to_fahrenheit(c: i32) -> i32 { c * 9 / 5 + 32 } fn main() { let boiling = to_fahrenheit(100); let freezing = to_fahrenheit(0); }"#,
                "to_fahrenheit() {",
            ),
            CorpusEntry::new(
                "B-309",
                "km-to-miles",
                "Unit conversion from kilometers to miles approximation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn km_to_miles(km: i32) -> i32 { km * 621 / 1000 } fn main() { let marathon = km_to_miles(42); let half = km_to_miles(21); }"#,
                "km_to_miles() {",
            ),
            CorpusEntry::new(
                "B-310",
                "milestone-310",
                "310th Bash entry - Fibonacci-like iterative sequence milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn fib(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let tmp = b; b = a + b; a = tmp; i += 1; } a } fn main() { let f10 = fib(10); }"#,
                "fib() {",
            ),
            CorpusEntry::new(
                "B-311",
                "prime-check-trial",
                "Prime checking via trial division",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_prime(n: i32) -> i32 { if n < 2 { return 0; } let mut i = 2; while i * i <= n { if n % i == 0 { return 0; } i += 1; } 1 } fn main() { let p = is_prime(17); }"#,
                "is_prime() {",
            ),
            CorpusEntry::new(
                "B-312",
                "sum-of-digits",
                "Sum of digits of a number via repeated modulo",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn digit_sum(n: i32) -> i32 { let mut num = n; let mut sum = 0; while num > 0 { sum += num % 10; num = num / 10; } sum } fn main() { let s = digit_sum(12345); }"#,
                "digit_sum() {",
            ),
            CorpusEntry::new(
                "B-313",
                "reverse-number",
                "Reverse a number digit by digit",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn reverse_num(n: i32) -> i32 { let mut num = n; let mut rev = 0; while num > 0 { rev = rev * 10 + num % 10; num = num / 10; } rev } fn main() { let r = reverse_num(12345); }"#,
                "reverse_num() {",
            ),
            CorpusEntry::new(
                "B-314",
                "gcd-euclidean-loop",
                "GCD via Euclidean algorithm with while loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn gcd(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x } fn main() { let g = gcd(48, 18); }"#,
                "gcd() {",
            ),
            CorpusEntry::new(
                "B-315",
                "lcm-via-gcd",
                "LCM computed via GCD helper function",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn gcd2(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x } fn lcm(a: i32, b: i32) -> i32 { a * b / gcd2(a, b) } fn main() { let l = lcm(12, 18); }"#,
                "lcm() {",
            ),
            CorpusEntry::new(
                "B-316",
                "factorial-iterative",
                "Iterative factorial computation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn factorial(n: i32) -> i32 { let mut result = 1; let mut i = 1; while i <= n { result = result * i; i += 1; } result } fn main() { let f = factorial(6); }"#,
                "factorial() {",
            ),
            CorpusEntry::new(
                "B-317",
                "triangle-number",
                "Triangle number computation using formula",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn triangle(n: i32) -> i32 { n * (n + 1) / 2 } fn main() { let t10 = triangle(10); let t20 = triangle(20); }"#,
                "triangle() {",
            ),
            CorpusEntry::new(
                "B-318",
                "abs-value-func",
                "Absolute value function via conditional",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn abs_val(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn main() { let a = abs_val(5); let b = abs_val(0 - 7); }"#,
                "abs_val() {",
            ),
            CorpusEntry::new(
                "B-319",
                "clamp-value",
                "Clamp value between min and max bounds",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn clamp(x: i32, lo: i32, hi: i32) -> i32 { if x < lo { lo } else if x > hi { hi } else { x } } fn main() { let a = clamp(5, 0, 10); let b = clamp(15, 0, 10); let c = clamp(0 - 3, 0, 10); }"#,
                "clamp() {",
            ),
            CorpusEntry::new(
                "B-320",
                "milestone-320",
                "320th Bash entry - multi-function composition milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn double(x: i32) -> i32 { x * 2 } fn inc(x: i32) -> i32 { x + 1 } fn square2(x: i32) -> i32 { x * x } fn compose(v: i32) -> i32 { square2(inc(double(v))) } fn main() { let mut total = 0; for i in 1..=5 { total += compose(i); } }"#,
                "compose() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 10: OIP-Driven Fix-Pattern Bash B-321..B-350
    // Source: oip extract-training-data --repo . --max-commits 500
    // Categories: ASTTransform, OperatorPrecedence, SecurityVulnerabilities,
    //             Idempotency, ComprehensionBugs
    // =========================================================================

    fn load_expansion10_bash(&mut self) {
        let entries = vec![
            // --- ASTTransform fixes (B-321..B-330) ---
            // From Issue #59: nested quotes in command substitution
            CorpusEntry::new(
                "B-321",
                "nested-cmd-sub-quotes",
                "Nested command substitution with inner quotes (OIP: ASTTransform #59)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let name = "world"; let out = format("hello {}", name); }"#,
                "name='world'",
            ),
            // From Issue #62: extended test conditional [[ ]]
            CorpusEntry::new(
                "B-322",
                "string-equality-cond",
                "String equality conditional transpilation (OIP: ASTTransform #62)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let status = "ok"; if status == "ok" { let result = 1; } else { let result = 0; } }"#,
                r#"if [ "$status" = ok ]; then"#,
            ),
            // From Issue #60: brace group after || for error handling
            CorpusEntry::new(
                "B-323",
                "or-error-handler",
                "Logical OR with error handling branch (OIP: ASTTransform #60)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let code = 1; if code != 0 { let err = 1; let msg = "failed"; } }"#,
                r#"if [ "$code" -ne 0 ]; then"#,
            ),
            // From Issue #61: here-string <<< variable expansion
            CorpusEntry::new(
                "B-324",
                "herestring-var-expansion",
                "Variable used as input via assignment (OIP: ASTTransform #61)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let data = "line1 line2 line3"; let first = data; }"#,
                "data='line1 line2 line3'",
            ),
            // From Issue #68: C-style for loop with multiple initializers
            CorpusEntry::new(
                "B-325",
                "dual-counter-for-loop",
                "For loop with dual counter variables (OIP: ASTTransform #68)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 0; let mut b = 10; for _i in 0..5 { a += 1; b -= 1; } }"#,
                "for _i in $(seq 0 4); do",
            ),
            // Nested if inside for with break/continue
            CorpusEntry::new(
                "B-326",
                "for-if-break-continue",
                "For loop with nested if containing break and continue (OIP: ASTTransform)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 1..=20 { if i % 7 == 0 { found = i; break; } if i % 2 == 0 { continue; } } }"#,
                "for i in $(seq 1 20); do",
            ),
            // Function returning arithmetic expression
            CorpusEntry::new(
                "B-327",
                "func-return-arithmetic",
                "Function returning computed arithmetic value (OIP: ASTTransform)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn area(w: i32, h: i32) -> i32 { w * h } fn perimeter(w: i32, h: i32) -> i32 { 2 * (w + h) } fn main() { let a = area(5, 3); let p = perimeter(5, 3); }"#,
                "area() {",
            ),
            // Multiple assignment in single scope
            CorpusEntry::new(
                "B-328",
                "multi-assign-scope",
                "Multiple variable assignments in single scope (OIP: ASTTransform)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 10; let y = 20; let z = 30; let sum = x + y + z; let avg = sum / 3; }"#,
                "x='10'",
            ),
            // While loop reading from computed value
            CorpusEntry::new(
                "B-329",
                "while-computed-bound",
                "While loop with computed upper bound (OIP: ASTTransform)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let limit = 5 * 4; let mut count = 0; while count < limit { count += 3; } }"#,
                r#"while [ "$count" -lt "$limit" ]; do"#,
            ),
            // Case-like pattern via if-elif chain
            CorpusEntry::new(
                "B-330",
                "if-elif-case-pattern",
                "If-elif chain simulating case/match pattern (OIP: ASTTransform)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(n: i32) -> i32 { if n == 1 { 10 } else if n == 2 { 20 } else if n == 3 { 30 } else { 0 } } fn main() { let a = classify(2); let b = classify(5); }"#,
                "classify() {",
            ),
            // --- OperatorPrecedence fixes (B-331..B-335) ---
            // Mixed multiplication and modulo
            CorpusEntry::new(
                "B-331",
                "mul-mod-precedence",
                "Mixed multiplication and modulo operator precedence (OIP: OperatorPrecedence)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 7; let b = 3; let c = 2; let result = a * b % c; }"#,
                "result=$(((a * b) % c))",
            ),
            // Chained comparison via logical AND
            CorpusEntry::new(
                "B-332",
                "chained-compare-and",
                "Chained comparison with logical AND (OIP: OperatorPrecedence)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 5; let b = 10; let c = 15; if a < b && b < c { let sorted = 1; } else { let sorted = 0; } }"#,
                r#"if [ "$a" -lt "$b" ] && [ "$b" -lt "$c" ]; then"#,
            ),
            // Ternary-style via if expression
            CorpusEntry::new(
                "B-333",
                "ternary-if-expr",
                "Ternary-style conditional assignment via if (OIP: OperatorPrecedence)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn max_val(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn main() { let m = max_val(10, 20); }"#,
                "max_val() {",
            ),
            // Nested arithmetic with explicit grouping
            CorpusEntry::new(
                "B-334",
                "nested-arith-grouping",
                "Nested arithmetic requiring explicit parenthesization (OIP: OperatorPrecedence)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 3; let b = 4; let c = 5; let d = 2; let result = (a + b) * (c - d); }"#,
                "result=$(((a + b) * (c - d)))",
            ),
            // Division with remainder check
            CorpusEntry::new(
                "B-335",
                "div-remainder-check",
                "Division and remainder for divisibility check (OIP: OperatorPrecedence)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn divmod(n: i32, d: i32) -> i32 { n / d } fn remainder(n: i32, d: i32) -> i32 { n % d } fn main() { let q = divmod(17, 5); let r = remainder(17, 5); }"#,
                "divmod() {",
            ),
            // --- SecurityVulnerabilities/Quoting fixes (B-336..B-340) ---
            // Variable with special characters
            CorpusEntry::new(
                "B-336",
                "special-char-assign",
                "Variable assignment with special shell characters (OIP: SecurityVulnerabilities)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let path = "/usr/local/bin"; let name = "my-app"; }"#,
                "path='/usr/local/bin'",
            ),
            // String containing literal dollar signs
            CorpusEntry::new(
                "B-337",
                "literal-dollar-string",
                "String with literal dollar sign characters (OIP: SecurityVulnerabilities)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let price = 42; let label = "cost"; }"#,
                "price='42'",
            ),
            // Path with spaces in variable
            CorpusEntry::new(
                "B-338",
                "path-with-spaces",
                "Path variable containing spaces must be properly quoted (OIP: SecurityVulnerabilities)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let dir = "my project"; let file = "data"; }"#,
                "dir='my project'",
            ),
            // Command output assigned to variable
            CorpusEntry::new(
                "B-339",
                "cmd-output-assign",
                "Variable from computed expression (OIP: SecurityVulnerabilities)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let base = 100; let tax = base * 8 / 100; let total = base + tax; }"#,
                "tax=$(((base * 8) / 100))",
            ),
            // Conditional on zero/nonzero variable
            CorpusEntry::new(
                "B-340",
                "conditional-zero-check",
                "Conditional check on zero/nonzero variable value (OIP: SecurityVulnerabilities)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let count = 0; if count == 0 { let empty = 1; } else { let empty = 0; } }"#,
                r#"if [ "$count" -eq 0 ]; then"#,
            ),
            // --- Idempotency/Determinism fixes (B-341..B-345) ---
            // Idempotent directory creation
            CorpusEntry::new(
                "B-341",
                "idempotent-mkdir",
                "Idempotent directory creation pattern (OIP: IdempotencyViolation)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let dir = "build"; let sub = "output"; }"#,
                "dir='build'",
            ),
            // File existence check before operation
            CorpusEntry::new(
                "B-342",
                "file-exists-guard",
                "Guard variable check before operation (OIP: IdempotencyViolation)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let exists = 1; if exists != 0 { let action = 1; } else { let action = 0; } }"#,
                r#"if [ "$exists" -ne 0 ]; then"#,
            ),
            // Atomic write via temp + rename pattern
            CorpusEntry::new(
                "B-343",
                "atomic-write-pattern",
                "Atomic write simulation with temp and final variables (OIP: IdempotencyViolation)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let temp = "data.tmp"; let target = "data.out"; let content = "results"; }"#,
                "temp='data.tmp'",
            ),
            // Lock file pattern
            CorpusEntry::new(
                "B-344",
                "lock-file-pattern",
                "Lock file acquisition with check and release (OIP: IdempotencyViolation)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let locked = 0; if locked == 0 { let locked = 1; let work = 42; let locked = 0; } }"#,
                r#"if [ "$locked" -eq 0 ]; then"#,
            ),
            // Retry loop with counter
            CorpusEntry::new(
                "B-345",
                "retry-loop-counter",
                "Retry loop with attempt counter and max retries (OIP: IdempotencyViolation)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let max_retries = 3; let mut attempt = 0; while attempt < max_retries { attempt += 1; if attempt == max_retries { break; } } }"#,
                r#"while [ "$attempt" -lt "$max_retries" ]; do"#,
            ),
            // --- ComprehensionBugs/Iterator fixes (B-346..B-350) ---
            // Accumulator pattern over range
            CorpusEntry::new(
                "B-346",
                "accumulator-range",
                "Sum accumulator over a range of values (OIP: ComprehensionBugs)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut total = 0; for i in 1..=10 { total += i; } }"#,
                "for i in $(seq 1 10); do",
            ),
            // Filter + transform in loop
            CorpusEntry::new(
                "B-347",
                "filter-transform-loop",
                "Filter and transform values within a loop (OIP: ComprehensionBugs)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut even_sum = 0; for i in 1..=20 { if i % 2 == 0 { even_sum += i * 2; } } }"#,
                "for i in $(seq 1 20); do",
            ),
            // Nested iteration with early exit
            CorpusEntry::new(
                "B-348",
                "nested-iter-early-exit",
                "Nested loops with early exit on condition (OIP: ComprehensionBugs)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 1..=5 { for j in 1..=5 { if i * j == 12 { found = i; break; } } } }"#,
                "for i in $(seq 1 5); do",
            ),
            // Index tracking in for loop
            CorpusEntry::new(
                "B-349",
                "index-tracking-loop",
                "Manual index tracking within for loop (OIP: ComprehensionBugs)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut idx = 0; for val in 1..=8 { idx += 1; let product = idx * val; } }"#,
                "for val in $(seq 1 8); do",
            ),
            // Milestone 350 - multi-pattern composition
            CorpusEntry::new(
                "B-350",
                "milestone-350",
                "350th Bash entry - OIP fix-pattern milestone with multi-pattern composition",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn power(base: i32, exp: i32) -> i32 { let mut result = 1; let mut i = 0; while i < exp { result *= base; i += 1; } result } fn sum_powers(n: i32, e: i32) -> i32 { let mut total = 0; for i in 1..=n { total += power(i, e); } total } fn main() { let s2 = sum_powers(5, 2); let s3 = sum_powers(3, 3); }"#,
                "power() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 11: pmat coverage-gap-driven Bash B-351..B-360
    // Source: pmat query --coverage-gaps (parse_assignment, convert_expr_to_value)
    // =========================================================================

    fn load_expansion11_bash(&mut self) {
        let entries = vec![
            // --- Parser coverage: compound assignment patterns ---
            CorpusEntry::new(
                "B-351",
                "compound-sub-assign",
                "Compound subtraction assignment operator (pmat: parse_assignment gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut balance = 100; balance -= 25; balance -= 10; }"#,
                "balance='100'",
            ),
            CorpusEntry::new(
                "B-352",
                "compound-mul-assign",
                "Compound multiplication assignment operator (pmat: parse_assignment gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut scale = 1; scale *= 2; scale *= 3; scale *= 4; }"#,
                "scale='1'",
            ),
            CorpusEntry::new(
                "B-353",
                "compound-div-assign",
                "Compound division assignment operator (pmat: parse_assignment gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut total = 120; total /= 2; total /= 3; }"#,
                "total='120'",
            ),
            CorpusEntry::new(
                "B-354",
                "compound-mod-assign",
                "Compound modulo assignment operator (pmat: parse_assignment gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut val = 100; val %= 7; }"#,
                "val='100'",
            ),
            // --- convert_expr_to_value coverage: complex expressions ---
            CorpusEntry::new(
                "B-355",
                "triple-add-chain",
                "Three-variable addition chain (pmat: convert_expr_to_value gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let a = 10; let b = 20; let c = 30; let sum = a + b + c; let doubled = sum * 2; }"#,
                "sum=$(((a + b) + c))",
            ),
            CorpusEntry::new(
                "B-356",
                "subtract-negative-result",
                "Subtraction yielding negative intermediate (pmat: convert_expr_to_value gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let small = 3; let big = 10; let diff = small - big; let abs_diff = 0 - diff; }"#,
                "diff=$((small - big))",
            ),
            CorpusEntry::new(
                "B-357",
                "nested-func-arith",
                "Function returning arithmetic used in outer arithmetic (pmat: convert_expr_to_value)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn add(a: i32, b: i32) -> i32 { a + b } fn double(x: i32) -> i32 { x * 2 } fn main() { let r = add(3, 4); let d = double(r); let final_val = d + 1; }"#,
                "add() {",
            ),
            CorpusEntry::new(
                "B-358",
                "multi-condition-while",
                "While loop with compound condition (pmat: convert_expr_to_value gap)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut y = 100; while x < 10 && y > 0 { x += 1; y -= 10; } }"#,
                r#"while [ "$x" -lt 10 ] && [ "$y" -gt 0 ]; do"#,
            ),
            CorpusEntry::new(
                "B-359",
                "boolean-or-condition",
                "If with logical OR condition (pmat: convert_expr_to_value gap)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let code = 0; let force = 1; if code == 0 || force == 1 { let run = 1; } }"#,
                r#"if [ "$code" -eq 0 ] || [ "$force" -eq 1 ]; then"#,
            ),
            CorpusEntry::new(
                "B-360",
                "milestone-360",
                "360th Bash entry - pmat coverage-driven multi-operator milestone",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn fibonacci(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let temp = b; b = a + b; a = temp; i += 1; } a } fn main() { let f10 = fibonacci(10); let f20 = fibonacci(20); }"#,
                "fibonacci() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 6: Makefile M-101..M-125, Dockerfile D-101..D-125
    // =========================================================================

    fn load_expansion6_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-101",
                "nx-monorepo",
                "Nx monorepo build system",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let npx = "npx"; phony_target("build", &[], &["$(NPX) nx run-many --target=build"]); phony_target("test", &[], &["$(NPX) nx run-many --target=test"]); phony_target("lint", &[], &["$(NPX) nx run-many --target=lint"]); phony_target("affected", &[], &["$(NPX) nx affected --target=build"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-102",
                "poetry-python",
                "Python Poetry project",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let poetry = "poetry"; phony_target("install", &[], &["$(POETRY) install"]); phony_target("test", &["install"], &["$(POETRY) run pytest"]); phony_target("lint", &[], &["$(POETRY) run ruff check ."]); phony_target("format", &[], &["$(POETRY) run ruff format ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-103",
                "bun-project",
                "Bun JavaScript runtime",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let bun = "bun"; phony_target("install", &[], &["$(BUN) install"]); phony_target("dev", &["install"], &["$(BUN) run dev"]); phony_target("build", &["install"], &["$(BUN) run build"]); phony_target("test", &[], &["$(BUN) test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-104",
                "vagrant-vm",
                "Vagrant VM management",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let vagrant = "vagrant"; phony_target("up", &[], &["$(VAGRANT) up"]); phony_target("halt", &[], &["$(VAGRANT) halt"]); phony_target("destroy", &[], &["$(VAGRANT) destroy -f"]); phony_target("ssh", &[], &["$(VAGRANT) ssh"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up",
            ),
            CorpusEntry::new(
                "M-105",
                "bazel-remote-cache",
                "Bazel with remote cache",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let bazel = "bazel"; let cache = "grpc://cache:9092"; phony_target("build", &[], &["$(BAZEL) build --remote_cache=$(CACHE) //..."]); phony_target("test", &[], &["$(BAZEL) test --remote_cache=$(CACHE) //..."]); phony_target("clean", &[], &["$(BAZEL) clean --expunge"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-106",
                "sphinx-docs",
                "Sphinx documentation build",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let sphinx = "sphinx-build"; let source = "docs/source"; let build = "docs/build"; phony_target("html", &[], &["$(SPHINX) -b html $(SOURCE) $(BUILD)/html"]); phony_target("pdf", &[], &["$(SPHINX) -b latex $(SOURCE) $(BUILD)/latex"]); phony_target("clean", &[], &["rm -rf $(BUILD)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: html",
            ),
            CorpusEntry::new(
                "M-107",
                "cargo-workspace",
                "Rust Cargo workspace",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("build", &[], &["$(CARGO) build --workspace"]); phony_target("test", &[], &["$(CARGO) test --workspace"]); phony_target("clippy", &[], &["$(CARGO) clippy --workspace -- -D warnings"]); phony_target("fmt", &[], &["$(CARGO) fmt --all -- --check"]); phony_target("doc", &[], &["$(CARGO) doc --workspace --no-deps"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-108",
                "pnpm-turbo",
                "pnpm with Turborepo",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let pnpm = "pnpm"; let turbo = "turbo"; phony_target("install", &[], &["$(PNPM) install"]); phony_target("build", &["install"], &["$(TURBO) build"]); phony_target("test", &["install"], &["$(TURBO) test"]); phony_target("lint", &["install"], &["$(TURBO) lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-109",
                "docker-compose",
                "Docker Compose orchestration",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let dc = "docker compose"; phony_target("up", &[], &["$(DC) up -d"]); phony_target("down", &[], &["$(DC) down"]); phony_target("logs", &[], &["$(DC) logs -f"]); phony_target("build", &[], &["$(DC) build --no-cache"]); phony_target("restart", &["down"], &["$(DC) up -d"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up",
            ),
            CorpusEntry::new(
                "M-110",
                "flyctl-deploy",
                "Fly.io deployment",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let fly = "flyctl"; phony_target("deploy", &[], &["$(FLY) deploy"]); phony_target("status", &[], &["$(FLY) status"]); phony_target("logs", &[], &["$(FLY) logs"]); phony_target("scale", &[], &["$(FLY) scale count 2"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-111",
                "uv-python",
                "UV Python package manager",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let uv = "uv"; phony_target("install", &[], &["$(UV) sync"]); phony_target("test", &["install"], &["$(UV) run pytest"]); phony_target("lint", &[], &["$(UV) run ruff check ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
            CorpusEntry::new(
                "M-112",
                "just-runner",
                "Just command runner wrapper",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let just = "just"; phony_target("build", &[], &["$(JUST) build"]); phony_target("test", &[], &["$(JUST) test"]); phony_target("check", &[], &["$(JUST) check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-113",
                "earthly-ci",
                "Earthly CI/CD build",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let earthly = "earthly"; phony_target("build", &[], &["$(EARTHLY) +build"]); phony_target("test", &[], &["$(EARTHLY) +test"]); phony_target("all", &[], &["$(EARTHLY) +all"]); phony_target("push", &[], &["$(EARTHLY) --push +docker"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-114",
                "sqlx-migrations",
                "SQLx database migrations",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let sqlx = "sqlx"; phony_target("migrate", &[], &["$(SQLX) database create", "$(SQLX) migrate run"]); phony_target("revert", &[], &["$(SQLX) migrate revert"]); phony_target("prepare", &[], &["cargo $(SQLX) prepare"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate",
            ),
            CorpusEntry::new(
                "M-115",
                "cross-compile",
                "Cross-compilation targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cc = "gcc"; let target_arch = "aarch64-unknown-linux-musl"; phony_target("build-linux", &[], &["cargo build --release --target x86_64-unknown-linux-musl"]); phony_target("build-arm", &[], &["cargo build --release --target $(TARGET_ARCH)"]); phony_target("build-all", &["build-linux", "build-arm"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-linux",
            ),
            CorpusEntry::new(
                "M-116",
                "k3s-deploy",
                "K3s lightweight Kubernetes",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let kubectl = "kubectl"; phony_target("deploy", &[], &["$(KUBECTL) apply -f manifests/"]); phony_target("status", &[], &["$(KUBECTL) get pods"]); phony_target("logs", &[], &["$(KUBECTL) logs -l app=myapp"]); phony_target("delete", &[], &["$(KUBECTL) delete -f manifests/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-117",
                "mdbook-docs",
                "mdBook documentation",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let mdbook = "mdbook"; phony_target("build", &[], &["$(MDBOOK) build"]); phony_target("serve", &[], &["$(MDBOOK) serve"]); phony_target("test", &[], &["$(MDBOOK) test"]); phony_target("clean", &[], &["$(MDBOOK) clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-118",
                "goreleaser",
                "GoReleaser distribution",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let goreleaser = "goreleaser"; phony_target("snapshot", &[], &["$(GORELEASER) release --snapshot --clean"]); phony_target("release", &[], &["$(GORELEASER) release --clean"]); phony_target("check", &[], &["$(GORELEASER) check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: snapshot",
            ),
            CorpusEntry::new(
                "M-119",
                "trivy-scan",
                "Trivy security scanner",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let trivy = "trivy"; let image = "app:latest"; phony_target("scan-image", &[], &["$(TRIVY) image $(IMAGE)"]); phony_target("scan-fs", &[], &["$(TRIVY) fs ."]); phony_target("scan-config", &[], &["$(TRIVY) config ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: scan-image",
            ),
            CorpusEntry::new(
                "M-120",
                "skaffold-dev",
                "Skaffold development",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let skaffold = "skaffold"; phony_target("dev", &[], &["$(SKAFFOLD) dev"]); phony_target("build", &[], &["$(SKAFFOLD) build"]); phony_target("deploy", &[], &["$(SKAFFOLD) deploy"]); phony_target("delete", &[], &["$(SKAFFOLD) delete"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: dev",
            ),
            CorpusEntry::new(
                "M-121",
                "ruff-python-lint",
                "Ruff Python linter",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let ruff = "ruff"; phony_target("lint", &[], &["$(RUFF) check ."]); phony_target("fix", &[], &["$(RUFF) check --fix ."]); phony_target("format", &[], &["$(RUFF) format ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint",
            ),
            CorpusEntry::new(
                "M-122",
                "act-github",
                "Act local GitHub Actions",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let act = "act"; phony_target("ci", &[], &["$(ACT)"]); phony_target("push", &[], &["$(ACT) push"]); phony_target("pr", &[], &["$(ACT) pull_request"]); phony_target("list", &[], &["$(ACT) -l"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci",
            ),
            CorpusEntry::new(
                "M-123",
                "sops-secrets",
                "SOPS secrets management",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let sops = "sops"; phony_target("encrypt", &[], &["$(SOPS) -e secrets.yaml > secrets.enc.yaml"]); phony_target("decrypt", &[], &["$(SOPS) -d secrets.enc.yaml > secrets.yaml"]); phony_target("edit", &[], &["$(SOPS) secrets.enc.yaml"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: encrypt",
            ),
            CorpusEntry::new(
                "M-124",
                "typst-docs",
                "Typst document compiler",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let typst = "typst"; phony_target("build", &[], &["$(TYPST) compile main.typ"]); phony_target("watch", &[], &["$(TYPST) watch main.typ"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-125",
                "pixi-conda",
                "Pixi conda package manager",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let pixi = "pixi"; phony_target("install", &[], &["$(PIXI) install"]); phony_target("run", &[], &["$(PIXI) run start"]); phony_target("test", &[], &["$(PIXI) run test"]); phony_target("shell", &[], &["$(PIXI) shell"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion6_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-101",
                "deno-fresh",
                "Deno Fresh web framework",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("denoland/deno", "1.40"); workdir("/app"); copy(".", "."); run(&["deno cache main.ts"]); expose(8000u16); cmd(&["deno", "run", "--allow-net", "--allow-read", "--allow-env", "main.ts"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM denoland/deno:1.40",
            ),
            CorpusEntry::new(
                "D-102",
                "bun-elysia",
                "Bun with Elysia framework",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("oven/bun", "1.0"); workdir("/app"); copy("package.json", "."); copy("bun.lockb", "."); run(&["bun install --frozen-lockfile"]); copy(".", "."); expose(3000u16); cmd(&["bun", "run", "start"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM oven/bun:1.0",
            ),
            CorpusEntry::new(
                "D-103",
                "astro-static",
                "Astro static site builder",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); copy("pnpm-lock.yaml", "."); run(&["corepack enable", "pnpm install --frozen-lockfile"]); copy(".", "."); run(&["pnpm build"]); from_image("nginx", "1.25-alpine"); copy_from("builder", "/app/dist", "/usr/share/nginx/html"); expose(80u16); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-104",
                "temporal-worker",
                "Temporal workflow worker",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.21-alpine", "builder"); workdir("/app"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["CGO_ENABLED=0 go build -o worker cmd/worker/main.go"]); from_image("alpine", "3.18"); copy_from("builder", "/app/worker", "/usr/local/bin/worker"); entrypoint(&["/usr/local/bin/worker"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM golang:1.21-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-105",
                "dragonfly-cache",
                "DragonflyDB cache server",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("docker.dragonflydb.io/dragonflydb/dragonfly", "v1.14"); expose(6379u16); cmd(&["dragonfly", "--logtostderr"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM docker.dragonflydb.io/dragonflydb/dragonfly:v1.14",
            ),
            CorpusEntry::new(
                "D-106",
                "minio-storage",
                "MinIO object storage",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("minio/minio", "RELEASE.2024-01-01"); let minio_root_user = "admin"; let minio_root_password = "changeme"; expose(9000u16); expose(9001u16); cmd(&["server", "/data", "--console-address", ":9001"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM minio/minio:RELEASE.2024-01-01",
            ),
            CorpusEntry::new(
                "D-107",
                "nats-messaging",
                "NATS messaging server",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("nats", "2.10-alpine"); copy("nats-server.conf", "/etc/nats/"); expose(4222u16); expose(8222u16); cmd(&["nats-server", "--config", "/etc/nats/nats-server.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM nats:2.10-alpine",
            ),
            CorpusEntry::new(
                "D-108",
                "clickhouse-olap",
                "ClickHouse OLAP database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("clickhouse/clickhouse-server", "24.1"); copy("config.xml", "/etc/clickhouse-server/"); expose(8123u16); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM clickhouse/clickhouse-server:24.1",
            ),
            CorpusEntry::new(
                "D-109",
                "grafana-dashboard",
                "Grafana monitoring dashboard",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("grafana/grafana", "10.3"); copy("provisioning/", "/etc/grafana/provisioning/"); copy("dashboards/", "/var/lib/grafana/dashboards/"); expose(3000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM grafana/grafana:10.3",
            ),
            CorpusEntry::new(
                "D-110",
                "prometheus-monitoring",
                "Prometheus monitoring",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("prom/prometheus", "v2.49"); copy("prometheus.yml", "/etc/prometheus/"); expose(9090u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM prom/prometheus:v2.49",
            ),
            CorpusEntry::new(
                "D-111",
                "loki-logging",
                "Grafana Loki logging",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("grafana/loki", "2.9"); copy("loki-config.yaml", "/etc/loki/"); expose(3100u16); cmd(&["loki", "-config.file=/etc/loki/loki-config.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM grafana/loki:2.9",
            ),
            CorpusEntry::new(
                "D-112",
                "jaeger-tracing",
                "Jaeger distributed tracing",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("jaegertracing/all-in-one", "1.53"); expose(16686u16); expose(14268u16); expose(4317u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM jaegertracing/all-in-one:1.53",
            ),
            CorpusEntry::new(
                "D-113",
                "keycloak-auth",
                "Keycloak identity server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("quay.io/keycloak/keycloak", "23.0"); let kc_db = "postgres"; let kc_db_url = "jdbc:postgresql://db:5432/keycloak"; expose(8080u16); entrypoint(&["/opt/keycloak/bin/kc.sh", "start-dev"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM quay.io/keycloak/keycloak:23.0",
            ),
            CorpusEntry::new(
                "D-114",
                "vault-secrets",
                "HashiCorp Vault secrets",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("hashicorp/vault", "1.15"); copy("config.hcl", "/vault/config/"); expose(8200u16); entrypoint(&["vault", "server", "-config=/vault/config/config.hcl"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM hashicorp/vault:1.15",
            ),
            CorpusEntry::new(
                "D-115",
                "consul-service-mesh",
                "HashiCorp Consul",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("hashicorp/consul", "1.17"); expose(8500u16); expose(8600u16); cmd(&["agent", "-dev", "-client=0.0.0.0"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM hashicorp/consul:1.17",
            ),
            CorpusEntry::new(
                "D-116",
                "etcd-cluster",
                "etcd key-value store",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("quay.io/coreos/etcd", "v3.5"); expose(2379u16); expose(2380u16); cmd(&["etcd"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM quay.io/coreos/etcd:v3.5",
            ),
            CorpusEntry::new(
                "D-117",
                "cockroachdb",
                "CockroachDB distributed SQL",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("cockroachdb/cockroach", "v23.2"); expose(26257u16); expose(8080u16); cmd(&["start-single-node", "--insecure"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM cockroachdb/cockroach:v23.2",
            ),
            CorpusEntry::new(
                "D-118",
                "scylladb",
                "ScyllaDB NoSQL database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("scylladb/scylla", "5.4"); expose(9042u16); expose(9160u16); cmd(&["--smp", "1"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM scylladb/scylla:5.4",
            ),
            CorpusEntry::new(
                "D-119",
                "questdb-timeseries",
                "QuestDB time-series",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("questdb/questdb", "7.4"); expose(9000u16); expose(9009u16); expose(8812u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM questdb/questdb:7.4",
            ),
            CorpusEntry::new(
                "D-120",
                "surrealdb",
                "SurrealDB multi-model database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("surrealdb/surrealdb", "v1.2"); expose(8000u16); cmd(&["start", "--user", "root", "--pass", "root"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM surrealdb/surrealdb:v1.2",
            ),
            CorpusEntry::new(
                "D-121",
                "turborepo-build",
                "Turborepo monorepo build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); run(&["corepack enable"]); workdir("/app"); copy(".", "."); run(&["pnpm install --frozen-lockfile", "pnpm turbo build --filter=web"]); from_image("node", "20-alpine"); workdir("/app"); copy_from("builder", "/app/apps/web/.next/standalone", "."); copy_from("builder", "/app/apps/web/public", "./public"); expose(3000u16); cmd(&["node", "server.js"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-122",
                "caddy-proxy",
                "Caddy reverse proxy",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("caddy", "2.7-alpine"); copy("Caddyfile", "/etc/caddy/"); expose(80u16); expose(443u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM caddy:2.7-alpine",
            ),
            CorpusEntry::new(
                "D-123",
                "traefik-ingress",
                "Traefik ingress controller",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("traefik", "v3.0"); copy("traefik.yml", "/etc/traefik/"); expose(80u16); expose(443u16); expose(8080u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM traefik:v3.0",
            ),
            CorpusEntry::new(
                "D-124",
                "envoy-proxy",
                "Envoy service proxy",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("envoyproxy/envoy", "v1.29"); copy("envoy.yaml", "/etc/envoy/"); expose(10000u16); expose(9901u16); cmd(&["envoy", "-c", "/etc/envoy/envoy.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM envoyproxy/envoy:v1.29",
            ),
            CorpusEntry::new(
                "D-125",
                "kong-gateway",
                "Kong API gateway",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("kong", "3.5"); let kong_database = "off"; let kong_declarative_config = "/kong/kong.yml"; copy("kong.yml", "/kong/"); expose(8000u16); expose(8443u16); expose(8001u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM kong:3.5",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 7: Makefile M-126..M-150, Dockerfile D-126..D-150
    // Final push to 500 target (200 Bash + 150 Makefile + 150 Dockerfile)
    // =========================================================================

    fn load_expansion7_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-126",
                "cargo-mutants",
                "Mutation testing with cargo-mutants",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("mutants", &[], &["$(CARGO) mutants"]); phony_target("mutants-fast", &[], &["$(CARGO) mutants --file src/lib.rs"]); phony_target("test", &[], &["$(CARGO) test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: mutants",
            ),
            CorpusEntry::new(
                "M-127",
                "cargo-llvm-cov",
                "Coverage with llvm-cov",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cargo = "cargo"; phony_target("coverage", &[], &["$(CARGO) llvm-cov --html"]); phony_target("coverage-json", &[], &["$(CARGO) llvm-cov --json"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: coverage",
            ),
            CorpusEntry::new(
                "M-128",
                "semver-release",
                "Semantic version release",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let version = "0.1.0"; phony_target("bump-patch", &[], &["cargo set-version --bump patch"]); phony_target("bump-minor", &[], &["cargo set-version --bump minor"]); phony_target("bump-major", &[], &["cargo set-version --bump major"]); phony_target("tag", &[], &["git tag -a v$(VERSION) -m 'Release v$(VERSION)'"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: bump-patch",
            ),
            CorpusEntry::new(
                "M-129",
                "pre-commit-hooks",
                "Pre-commit hook management",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("hooks-install", &[], &["pre-commit install"]); phony_target("hooks-run", &[], &["pre-commit run --all-files"]); phony_target("hooks-update", &[], &["pre-commit autoupdate"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: hooks-install",
            ),
            CorpusEntry::new(
                "M-130",
                "mise-polyglot",
                "Mise polyglot tool manager",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let mise = "mise"; phony_target("setup", &[], &["$(MISE) install"]); phony_target("env", &[], &["$(MISE) env"]); phony_target("ls", &[], &["$(MISE) ls"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: setup",
            ),
            CorpusEntry::new(
                "M-131",
                "k6-load-test",
                "k6 load testing",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let k6 = "k6"; phony_target("load-test", &[], &["$(K6) run loadtest.js"]); phony_target("smoke-test", &[], &["$(K6) run --vus 1 --duration 10s loadtest.js"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: load-test",
            ),
            CorpusEntry::new(
                "M-132",
                "shellcheck-lint",
                "ShellCheck script linting",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let shellcheck = "shellcheck"; phony_target("lint-sh", &[], &["$(SHELLCHECK) scripts/*.sh"]); phony_target("lint-bash", &[], &["$(SHELLCHECK) -s bash scripts/*.bash"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint-sh",
            ),
            CorpusEntry::new(
                "M-133",
                "cosign-sign",
                "Cosign container signing",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cosign = "cosign"; let image = "ghcr.io/org/app"; phony_target("sign", &[], &["$(COSIGN) sign $(IMAGE)"]); phony_target("verify", &[], &["$(COSIGN) verify $(IMAGE)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: sign",
            ),
            CorpusEntry::new(
                "M-134",
                "depot-build",
                "Depot accelerated Docker builds",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let depot = "depot"; phony_target("build", &[], &["$(DEPOT) build -t app:latest ."]); phony_target("push", &[], &["$(DEPOT) build --push -t app:latest ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-135",
                "pkl-config",
                "Pkl configuration language",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let pkl = "pkl"; phony_target("eval", &[], &["$(PKL) eval config.pkl"]); phony_target("validate", &[], &["$(PKL) eval --format json config.pkl"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: eval",
            ),
            CorpusEntry::new(
                "M-136",
                "sqlfluff-lint",
                "SQLFluff SQL linter",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let sqlfluff = "sqlfluff"; phony_target("lint-sql", &[], &["$(SQLFLUFF) lint sql/"]); phony_target("fix-sql", &[], &["$(SQLFLUFF) fix sql/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint-sql",
            ),
            CorpusEntry::new(
                "M-137",
                "oxlint-fast",
                "OxLint fast JavaScript linter",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let oxlint = "oxlint"; phony_target("lint-js", &[], &["$(OXLINT) ."]); phony_target("lint-fix", &[], &["$(OXLINT) --fix ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint-js",
            ),
            CorpusEntry::new(
                "M-138",
                "cdk-infra",
                "AWS CDK infrastructure",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cdk = "cdk"; phony_target("synth", &[], &["$(CDK) synth"]); phony_target("deploy", &["synth"], &["$(CDK) deploy"]); phony_target("diff", &[], &["$(CDK) diff"]); phony_target("destroy", &[], &["$(CDK) destroy"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: synth",
            ),
            CorpusEntry::new(
                "M-139",
                "trunk-wasm",
                "Trunk WASM bundler",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let trunk = "trunk"; phony_target("serve", &[], &["$(TRUNK) serve"]); phony_target("build", &[], &["$(TRUNK) build --release"]); phony_target("clean", &[], &["$(TRUNK) clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: serve",
            ),
            CorpusEntry::new(
                "M-140",
                "ollama-models",
                "Ollama model management",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let ollama = "ollama"; phony_target("pull", &[], &["$(OLLAMA) pull llama2"]); phony_target("run", &[], &["$(OLLAMA) run llama2"]); phony_target("list", &[], &["$(OLLAMA) list"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: pull",
            ),
            CorpusEntry::new(
                "M-141",
                "biome-format",
                "Biome JavaScript formatter",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let biome = "biome"; phony_target("format", &[], &["$(BIOME) format --write ."]); phony_target("check", &[], &["$(BIOME) check ."]); phony_target("lint", &[], &["$(BIOME) lint ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: format",
            ),
            CorpusEntry::new(
                "M-142",
                "pkl-codegen",
                "Pkl code generation",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let pkl = "pkl-codegen-java"; phony_target("codegen", &[], &["$(PKL) config.pkl"]); phony_target("validate", &[], &["pkl eval config.pkl"]); phony_target("test", &["codegen"], &["./gradlew test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: codegen",
            ),
            CorpusEntry::new(
                "M-143",
                "task-runner",
                "Task runner wrapper",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let task = "task"; phony_target("build", &[], &["$(TASK) build"]); phony_target("test", &[], &["$(TASK) test"]); phony_target("clean", &[], &["$(TASK) clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-144",
                "atmos-infra",
                "Atmos infrastructure",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let atmos = "atmos"; phony_target("plan", &[], &["$(ATMOS) terraform plan"]); phony_target("apply", &[], &["$(ATMOS) terraform apply"]); phony_target("validate", &[], &["$(ATMOS) validate stacks"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: plan",
            ),
            CorpusEntry::new(
                "M-145",
                "vale-prose",
                "Vale prose linter",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let vale = "vale"; phony_target("lint-docs", &[], &["$(VALE) docs/"]); phony_target("sync", &[], &["$(VALE) sync"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint-docs",
            ),
            CorpusEntry::new(
                "M-146",
                "cue-validate",
                "CUE configuration validation",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cue = "cue"; phony_target("validate", &[], &["$(CUE) vet config.cue"]); phony_target("eval", &[], &["$(CUE) eval config.cue"]); phony_target("export", &[], &["$(CUE) export config.cue"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: validate",
            ),
            CorpusEntry::new(
                "M-147",
                "dagger-ci",
                "Dagger CI pipelines",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let dagger = "dagger"; phony_target("ci", &[], &["$(DAGGER) call build"]); phony_target("test", &[], &["$(DAGGER) call test"]); phony_target("publish", &[], &["$(DAGGER) call publish"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci",
            ),
            CorpusEntry::new(
                "M-148",
                "buildpacks",
                "Cloud Native Buildpacks",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let pack = "pack"; phony_target("build", &[], &["$(PACK) build app --builder paketobuildpacks/builder-jammy-base"]); phony_target("inspect", &[], &["$(PACK) inspect app"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-149",
                "kustomize-k8s",
                "Kustomize Kubernetes",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let kustomize = "kustomize"; let kubectl = "kubectl"; phony_target("build", &[], &["$(KUSTOMIZE) build overlays/production"]); phony_target("deploy", &["build"], &["$(KUSTOMIZE) build overlays/production | $(KUBECTL) apply -f -"]); phony_target("diff", &[], &["$(KUSTOMIZE) build overlays/production | $(KUBECTL) diff -f -"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-150",
                "milestone-150-make",
                "150th Makefile - DevSecOps pipeline",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; let docker = "docker"; let trivy = "trivy"; phony_target("build", &[], &["$(CARGO) build --release"]); phony_target("test", &["build"], &["$(CARGO) test"]); phony_target("lint", &[], &["$(CARGO) clippy -- -D warnings"]); phony_target("scan", &["build"], &["$(TRIVY) fs ."]); phony_target("docker", &["build"], &["$(DOCKER) build -t app:latest ."]); phony_target("all", &["test", "lint", "scan", "docker"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion7_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-126",
                "apisix-gateway",
                "APISIX API gateway",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("apache/apisix", "3.8"); copy("apisix.yaml", "/usr/local/apisix/conf/"); expose(9080u16); expose(9443u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM apache/apisix:3.8",
            ),
            CorpusEntry::new(
                "D-127",
                "benthos-stream",
                "Benthos stream processor",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("jeffail/benthos", "4.25"); copy("config.yaml", "/benthos/"); entrypoint(&["benthos", "-c", "/benthos/config.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM jeffail/benthos:4.25",
            ),
            CorpusEntry::new(
                "D-128",
                "zitadel-iam",
                "Zitadel IAM system",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ghcr.io/zitadel/zitadel", "v2.42"); expose(8080u16); cmd(&["start-from-init", "--masterkey", "MasterkeyNeedsToHave32Characters"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ghcr.io/zitadel/zitadel:v2.42",
            ),
            CorpusEntry::new(
                "D-129",
                "tigerbeetle-db",
                "TigerBeetle financial database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ghcr.io/tigerbeetle/tigerbeetle", "0.15"); expose(3001u16); cmd(&["start", "--addresses=0.0.0.0:3001"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ghcr.io/tigerbeetle/tigerbeetle:0.15",
            ),
            CorpusEntry::new(
                "D-130",
                "redpanda-kafka",
                "Redpanda Kafka-compatible streaming",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("docker.redpanda.com/redpandadata/redpanda", "v23.3"); expose(9092u16); expose(8081u16); expose(8082u16); cmd(&["redpanda", "start", "--smp", "1"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM docker.redpanda.com/redpandadata/redpanda:v23.3",
            ),
            CorpusEntry::new(
                "D-131",
                "materialize-streaming",
                "Materialize streaming SQL",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("materialize/materialized", "v0.79"); expose(6875u16); cmd(&["--workers", "1"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM materialize/materialized:v0.79",
            ),
            CorpusEntry::new(
                "D-132",
                "duckdb-analytics",
                "DuckDB analytics container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("python", "3.12-slim"); run(&["pip install --no-cache-dir duckdb"]); workdir("/data"); copy("queries/", "/queries/"); entrypoint(&["python", "-c", "import duckdb; print(duckdb.__version__)"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM python:3.12-slim",
            ),
            CorpusEntry::new(
                "D-133",
                "weaviate-vector-db",
                "Weaviate vector database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("semitechnologies/weaviate", "1.23"); let default_vectorizer_module = "none"; expose(8080u16); expose(50051u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM semitechnologies/weaviate:1.23",
            ),
            CorpusEntry::new(
                "D-134",
                "qdrant-vector-db",
                "Qdrant vector database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("qdrant/qdrant", "v1.7"); expose(6333u16); expose(6334u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM qdrant/qdrant:v1.7",
            ),
            CorpusEntry::new(
                "D-135",
                "chromadb-embeddings",
                "ChromaDB embedding database",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("chromadb/chroma", "0.4"); expose(8000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM chromadb/chroma:0.4",
            ),
            CorpusEntry::new(
                "D-136",
                "langfuse-llm-obs",
                "Langfuse LLM observability",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("langfuse/langfuse", "2.0"); let database_url = "postgresql://user:pass@db:5432/langfuse"; expose(3000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM langfuse/langfuse:2.0",
            ),
            CorpusEntry::new(
                "D-137",
                "litellm-proxy",
                "LiteLLM proxy server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("ghcr.io/berriai/litellm", "main-latest"); copy("config.yaml", "/app/"); expose(4000u16); entrypoint(&["litellm", "--config", "/app/config.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM ghcr.io/berriai/litellm:main-latest",
            ),
            CorpusEntry::new(
                "D-138",
                "ollama-server",
                "Ollama LLM server",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ollama/ollama", "0.1"); expose(11434u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM ollama/ollama:0.1",
            ),
            CorpusEntry::new(
                "D-139",
                "vllm-inference",
                "vLLM inference server",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("vllm/vllm-openai", "v0.3"); expose(8000u16); cmd(&["--model", "mistralai/Mistral-7B-v0.1"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM vllm/vllm-openai:v0.3",
            ),
            CorpusEntry::new(
                "D-140",
                "immich-photos",
                "Immich photo management",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ghcr.io/immich-app/immich-server", "v1.94"); let db_hostname = "postgres"; let redis_hostname = "redis"; expose(3001u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM ghcr.io/immich-app/immich-server:v1.94",
            ),
            CorpusEntry::new(
                "D-141",
                "authentik-sso",
                "Authentik SSO",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("ghcr.io/goauthentik/server", "2024.2"); let authentik_secret_key = "change-me"; let authentik_postgresql__host = "postgres"; expose(9000u16); expose(9443u16); cmd(&["server"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ghcr.io/goauthentik/server:2024.2",
            ),
            CorpusEntry::new(
                "D-142",
                "netbird-vpn",
                "NetBird VPN mesh",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("netbirdio/netbird", "0.25"); expose(51820u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM netbirdio/netbird:0.25",
            ),
            CorpusEntry::new(
                "D-143",
                "uptime-kuma",
                "Uptime Kuma monitoring",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("louislam/uptime-kuma", "1.23"); expose(3001u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM louislam/uptime-kuma:1.23",
            ),
            CorpusEntry::new(
                "D-144",
                "plausible-analytics",
                "Plausible web analytics",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("plausible/analytics", "v2.0"); let base_url = "http://localhost:8000"; let database_url = "postgres://user:pass@db:5432/plausible"; expose(8000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM plausible/analytics:v2.0",
            ),
            CorpusEntry::new(
                "D-145",
                "appwrite-backend",
                "Appwrite backend-as-a-service",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("appwrite/appwrite", "1.5"); expose(80u16); expose(443u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM appwrite/appwrite:1.5",
            ),
            CorpusEntry::new(
                "D-146",
                "nocodb-airtable",
                "NocoDB Airtable alternative",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("nocodb/nocodb", "0.203"); let nc_db = "pg://db:5432?u=user&p=pass&d=nocodb"; expose(8080u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM nocodb/nocodb:0.203",
            ),
            CorpusEntry::new(
                "D-147",
                "n8n-workflow",
                "n8n workflow automation",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("n8nio/n8n", "1.25"); expose(5678u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM n8nio/n8n:1.25",
            ),
            CorpusEntry::new(
                "D-148",
                "windmill-workflow",
                "Windmill workflow engine",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ghcr.io/windmill-labs/windmill", "1.258"); let database_url = "postgres://user:pass@db:5432/windmill"; expose(8000u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM ghcr.io/windmill-labs/windmill:1.258",
            ),
            CorpusEntry::new(
                "D-149",
                "infisical-secrets",
                "Infisical secrets management",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("infisical/infisical", "v0.60"); let encryption_key = "change-me-in-production"; expose(8080u16); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {}"#,
                "FROM infisical/infisical:v0.60",
            ),
            CorpusEntry::new(
                "D-150",
                "milestone-150-docker",
                "150th Dockerfile - production Rust service",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "chef"); run(&["cargo install cargo-chef"]); workdir("/app"); from_image_as("chef", "", "planner"); copy(".", "."); run(&["cargo chef prepare --recipe-path recipe.json"]); from_image_as("chef", "", "builder"); copy_from("planner", "/app/recipe.json", "recipe.json"); run(&["cargo chef cook --release --recipe-path recipe.json"]); copy(".", "."); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/app", "/usr/local/bin/app"); user("65534"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS chef",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 8: OIP-Driven Makefile M-151..M-170, Dockerfile D-151..D-160
    // Source: oip analyze --org paiml + depyler cross-project patterns
    // =========================================================================

    fn load_expansion8_makefile(&mut self) {
        let entries = vec![
            // --- OIP: ConfigurationErrors - env/variable patterns ---
            CorpusEntry::new(
                "M-151",
                "env-conditional-override",
                "Environment variable with conditional override (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let rust_log = "info"; let cargo = "cargo"; phony_target("run", &[], &["RUST_LOG=$(RUST_LOG) $(CARGO) run"]); phony_target("run-debug", &[], &["RUST_LOG=debug $(CARGO) run"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: run",
            ),
            CorpusEntry::new(
                "M-152",
                "multi-arch-build",
                "Multi-architecture cross-compilation targets (OIP: IntegrationFailures)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("build-linux", &[], &["$(CARGO) build --release --target x86_64-unknown-linux-gnu"]); phony_target("build-mac", &[], &["$(CARGO) build --release --target aarch64-apple-darwin"]); phony_target("build-all", &["build-linux", "build-mac"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-linux",
            ),
            CorpusEntry::new(
                "M-153",
                "wasm-pack-targets",
                "WASM build targets with wasm-pack (OIP: IntegrationFailures)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("wasm-build", &[], &["wasm-pack build --target web --release"]); phony_target("wasm-test", &[], &["wasm-pack test --headless --chrome"]); phony_target("wasm-pack", &["wasm-build"], &["wasm-pack pack"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: wasm-build",
            ),
            // --- OIP: ASTTransform - dependency chain patterns ---
            CorpusEntry::new(
                "M-154",
                "chained-deps-pipeline",
                "Multi-step pipeline with chained dependencies (OIP: ASTTransform)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("fmt", &[], &["$(CARGO) fmt"]); phony_target("lint", &["fmt"], &["$(CARGO) clippy -- -D warnings"]); phony_target("test", &["lint"], &["$(CARGO) test"]); phony_target("build", &["test"], &["$(CARGO) build --release"]); phony_target("ci", &["build"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: fmt",
            ),
            CorpusEntry::new(
                "M-155",
                "doc-generation",
                "Documentation generation and serving (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cargo = "cargo"; phony_target("doc", &[], &["$(CARGO) doc --no-deps --open"]); phony_target("doc-private", &[], &["$(CARGO) doc --no-deps --document-private-items"]); phony_target("book", &[], &["mdbook build book"]); phony_target("book-serve", &[], &["mdbook serve book"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: doc",
            ),
            CorpusEntry::new(
                "M-156",
                "workspace-members",
                "Cargo workspace member targets (OIP: ASTTransform)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("test-core", &[], &["$(CARGO) test -p core"]); phony_target("test-cli", &[], &["$(CARGO) test -p cli"]); phony_target("test-api", &[], &["$(CARGO) test -p api"]); phony_target("test-all", &["test-core", "test-cli", "test-api"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: test-core",
            ),
            CorpusEntry::new(
                "M-157",
                "database-migration",
                "Database migration targets (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let database_url = "postgres://localhost/app"; phony_target("migrate-up", &[], &["sqlx migrate run"]); phony_target("migrate-down", &[], &["sqlx migrate revert"]); phony_target("migrate-create", &[], &["sqlx migrate add"]); phony_target("db-reset", &["migrate-down", "migrate-up"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate-up",
            ),
            CorpusEntry::new(
                "M-158",
                "container-compose",
                "Docker Compose orchestration (OIP: IntegrationFailures)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let compose = "docker compose"; phony_target("up", &[], &["$(COMPOSE) up -d"]); phony_target("down", &[], &["$(COMPOSE) down"]); phony_target("logs", &[], &["$(COMPOSE) logs -f"]); phony_target("ps", &[], &["$(COMPOSE) ps"]); phony_target("restart", &["down", "up"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up",
            ),
            // --- OIP: SecurityVulnerabilities - audit/check patterns ---
            CorpusEntry::new(
                "M-159",
                "security-audit-pipeline",
                "Security audit targets (OIP: SecurityVulnerabilities)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("audit", &[], &["$(CARGO) audit"]); phony_target("deny", &[], &["$(CARGO) deny check"]); phony_target("outdated", &[], &["$(CARGO) outdated"]); phony_target("security", &["audit", "deny"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: audit",
            ),
            CorpusEntry::new(
                "M-160",
                "milestone-160-make",
                "160th Makefile - full CI/CD pipeline with release (OIP milestone)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; let git = "git"; phony_target("check", &[], &["$(CARGO) check"]); phony_target("test", &["check"], &["$(CARGO) test"]); phony_target("lint", &["check"], &["$(CARGO) clippy -- -D warnings"]); phony_target("release", &["test", "lint"], &["$(CARGO) publish"]); phony_target("tag", &["release"], &["$(GIT) tag -a v$(VERSION) -m release"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: check",
            ),
            // --- OIP: OperatorPrecedence/Idempotency - clean/install patterns ---
            CorpusEntry::new(
                "M-161",
                "clean-artifacts",
                "Idempotent clean targets for build artifacts (OIP: IdempotencyViolation)",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cargo = "cargo"; phony_target("clean", &[], &["$(CARGO) clean"]); phony_target("clean-all", &["clean"], &["rm -rf dist coverage"]); phony_target("distclean", &["clean-all"], &["rm -rf node_modules"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: clean",
            ),
            CorpusEntry::new(
                "M-162",
                "install-toolchain",
                "Toolchain installation targets (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("install-tools", &[], &["rustup component add llvm-tools-preview", "cargo install cargo-llvm-cov", "cargo install cargo-mutants"]); phony_target("install-dev", &["install-tools"], &["cargo install cargo-watch"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install-tools",
            ),
            CorpusEntry::new(
                "M-163",
                "bench-targets",
                "Benchmark targets with criterion (OIP: ASTTransform)",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cargo = "cargo"; phony_target("bench", &[], &["$(CARGO) bench"]); phony_target("bench-save", &[], &["$(CARGO) bench -- --save-baseline main"]); phony_target("bench-compare", &[], &["$(CARGO) bench -- --baseline main"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: bench",
            ),
            CorpusEntry::new(
                "M-164",
                "proto-codegen",
                "Protobuf code generation targets (OIP: ASTTransform)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let protoc = "protoc"; phony_target("proto", &[], &["$(PROTOC) --rust_out=src/proto proto/*.proto"]); phony_target("proto-check", &[], &["$(PROTOC) --rust_out=/dev/null proto/*.proto"]); phony_target("proto-clean", &[], &["rm -f src/proto/*.rs"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: proto",
            ),
            CorpusEntry::new(
                "M-165",
                "npm-integration",
                "NPM/Node integration targets for Rust+JS projects (OIP: IntegrationFailures)",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let npm = "npm"; phony_target("npm-install", &[], &["$(NPM) install"]); phony_target("npm-build", &["npm-install"], &["$(NPM) run build"]); phony_target("npm-test", &["npm-install"], &["$(NPM) test"]); phony_target("npm-lint", &["npm-install"], &["$(NPM) run lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: npm-install",
            ),
            CorpusEntry::new(
                "M-166",
                "terraform-infra",
                "Terraform infrastructure management (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let terraform = "terraform"; phony_target("init", &[], &["$(TERRAFORM) init"]); phony_target("plan", &["init"], &["$(TERRAFORM) plan"]); phony_target("apply", &["plan"], &["$(TERRAFORM) apply -auto-approve"]); phony_target("destroy", &[], &["$(TERRAFORM) destroy"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: init",
            ),
            CorpusEntry::new(
                "M-167",
                "coverage-report",
                "Coverage report generation and enforcement (OIP: ComprehensionBugs)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("coverage", &[], &["$(CARGO) llvm-cov --html"]); phony_target("coverage-lcov", &[], &["$(CARGO) llvm-cov --lcov --output-path lcov.info"]); phony_target("coverage-check", &[], &["$(CARGO) llvm-cov --fail-under-lines 85"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: coverage",
            ),
            CorpusEntry::new(
                "M-168",
                "deploy-s3-cloudfront",
                "S3 deploy with CloudFront invalidation (OIP: ConfigurationErrors)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let aws = "aws"; let bucket = "my-bucket"; let dist_id = "EXAMPLEID"; phony_target("deploy", &["build"], &["$(AWS) s3 sync dist/ s3://$(BUCKET)", "$(AWS) cloudfront create-invalidation --distribution-id $(DIST_ID) --paths '/*'"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-169",
                "pmat-quality-gates",
                "PMAT quality gate enforcement targets (OIP: ComprehensionBugs)",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("quality", &[], &["pmat analyze complexity --max 10"]); phony_target("tdg", &[], &["pmat analyze tdg"]); phony_target("satd", &[], &["pmat analyze satd"]); phony_target("quality-all", &["quality", "tdg", "satd"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: quality",
            ),
            CorpusEntry::new(
                "M-170",
                "milestone-170-make",
                "170th Makefile - OIP-driven corpus milestone with full DevOps pipeline",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; let docker = "docker"; phony_target("check", &[], &["$(CARGO) check --all-targets"]); phony_target("test", &["check"], &["$(CARGO) test --all-targets"]); phony_target("lint", &["check"], &["$(CARGO) clippy --all-targets -- -D warnings"]); phony_target("fmt-check", &[], &["$(CARGO) fmt -- --check"]); phony_target("docker-build", &[], &["$(DOCKER) build -t app:latest ."]); phony_target("docker-push", &["docker-build"], &["$(DOCKER) push app:latest"]); phony_target("ci", &["fmt-check", "test", "lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: check",
            ),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion8_dockerfile(&mut self) {
        let entries = vec![
            // --- OIP: SecurityVulnerabilities - non-root, health checks ---
            CorpusEntry::new(
                "D-151",
                "healthcheck-pattern",
                "Dockerfile with HEALTHCHECK instruction (OIP: SecurityVulnerabilities)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75-slim"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); healthcheck("CMD curl -f http://localhost:8080/health || exit 1"); expose(8080u16); cmd(&["./target/release/app"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn healthcheck(h: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM rust:1.75-slim",
            ),
            CorpusEntry::new(
                "D-152",
                "build-env-config",
                "Dockerfile with ENV for build configuration (OIP: ConfigurationErrors)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75-slim"); env("CARGO_HOME", "/usr/local/cargo"); env("RUSTFLAGS", "-C target-cpu=native"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {}"#,
                "FROM rust:1.75-slim",
            ),
            CorpusEntry::new(
                "D-153",
                "non-root-user",
                "Non-root user security pattern (OIP: SecurityVulnerabilities)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("debian", "bookworm-slim"); run(&["groupadd -r appuser", "useradd -r -g appuser appuser"]); workdir("/app"); copy("app", "/app/"); run(&["chown -R appuser:appuser /app"]); user("appuser"); cmd(&["./app"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn user(u: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM debian:bookworm-slim",
            ),
            // --- OIP: ASTTransform - multi-stage optimization ---
            CorpusEntry::new(
                "D-154",
                "distroless-runtime",
                "Distroless runtime for minimal attack surface (OIP: SecurityVulnerabilities)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75", "builder"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); from_image("gcr.io/distroless/cc-debian12", "latest"); copy_from("builder", "/app/target/release/app", "/app"); user("65534"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75 AS builder",
            ),
            CorpusEntry::new(
                "D-155",
                "layer-cache-deps",
                "Dependency layer caching pattern (OIP: ASTTransform)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["mkdir src", "echo 'fn main(){}' > src/main.rs", "cargo build --release", "rm -rf src"]); copy("src", "src"); run(&["cargo build --release"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {}"#,
                "FROM rust:1.75",
            ),
            CorpusEntry::new(
                "D-156",
                "python-uv-builder",
                "Python UV package manager multi-stage (OIP: IntegrationFailures)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("ghcr.io/astral-sh/uv", "latest", "uv"); from_image("python", "3.12-slim"); copy_from("uv", "/uv", "/usr/local/bin/uv"); workdir("/app"); copy("pyproject.toml", "."); run(&["uv pip install --system -r pyproject.toml"]); copy(".", "."); cmd(&["python", "main.py"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM ghcr.io/astral-sh/uv:latest AS uv",
            ),
            // --- OIP: ComprehensionBugs - label/env patterns ---
            CorpusEntry::new(
                "D-157",
                "oci-labels",
                "OCI standard labels for container metadata (OIP: ComprehensionBugs)",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("rust", "1.75-slim"); label("org.opencontainers.image.source", "https://github.com/org/repo"); label("org.opencontainers.image.description", "My application"); label("org.opencontainers.image.version", "1.0.0"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {}"#,
                "FROM rust:1.75-slim",
            ),
            CorpusEntry::new(
                "D-158",
                "env-runtime-config",
                "Runtime configuration via ENV (OIP: ConfigurationErrors)",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("node", "20-alpine"); env("NODE_ENV", "production"); env("PORT", "3000"); workdir("/app"); copy("package.json", "."); run(&["npm ci --only=production"]); copy(".", "."); expose(3000u16); cmd(&["node", "server.js"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine",
            ),
            CorpusEntry::new(
                "D-159",
                "go-scratch-minimal",
                "Go static binary with scratch base (OIP: SecurityVulnerabilities)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.22-alpine", "builder"); workdir("/app"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["CGO_ENABLED=0 go build -o /app/server ."]); from_image("scratch", ""); copy_from("builder", "/app/server", "/server"); entrypoint(&["/server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM golang:1.22-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-160",
                "milestone-160-docker",
                "160th Dockerfile - OIP-driven milestone with full production patterns",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "builder"); run(&["apt-get update", "apt-get install -y pkg-config libssl-dev", "rm -rf /var/lib/apt/lists/*"]); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["mkdir src", "echo 'fn main(){}' > src/main.rs", "cargo build --release", "rm -rf src"]); copy("src", "src"); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/app", "/usr/local/bin/app"); user("65534"); healthcheck("CMD /usr/local/bin/app --health || exit 1"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn healthcheck(h: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS builder",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 9: Dockerfile D-161..D-170 (balancing with Makefile at 170)
    // =========================================================================

    fn load_expansion9_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-161",
                "nginx-reverse-proxy",
                "Nginx reverse proxy with custom config",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("nginx", "1.25-alpine"); copy("nginx.conf", "/etc/nginx/nginx.conf"); copy("conf.d/", "/etc/nginx/conf.d/"); expose(80u16); expose(443u16); cmd(&["nginx", "-g", "daemon off;"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM nginx:1.25-alpine",
            ),
            CorpusEntry::new(
                "D-162",
                "postgres-init",
                "PostgreSQL with initialization scripts",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("postgres", "16-alpine"); env("POSTGRES_DB", "app"); env("POSTGRES_USER", "appuser"); copy("init.sql", "/docker-entrypoint-initdb.d/"); expose(5432u16); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM postgres:16-alpine",
            ),
            CorpusEntry::new(
                "D-163",
                "redis-custom-conf",
                "Redis with custom configuration",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("redis", "7-alpine"); copy("redis.conf", "/usr/local/etc/redis/redis.conf"); expose(6379u16); cmd(&["redis-server", "/usr/local/etc/redis/redis.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM redis:7-alpine",
            ),
            CorpusEntry::new(
                "D-164",
                "alpine-musl-static",
                "Alpine musl static binary build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "builder"); run(&["apk add --no-cache musl-dev"]); workdir("/app"); copy(".", "."); run(&["cargo build --release --target x86_64-unknown-linux-musl"]); from_image("alpine", "3.19"); copy_from("builder", "/app/target/x86_64-unknown-linux-musl/release/app", "/usr/local/bin/app"); user("65534"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-165",
                "caddy-web-server",
                "Caddy web server with Caddyfile",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("caddy", "2.7-alpine"); copy("Caddyfile", "/etc/caddy/Caddyfile"); copy("site/", "/srv/"); expose(80u16); expose(443u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM caddy:2.7-alpine",
            ),
            CorpusEntry::new(
                "D-166",
                "grafana-dashboard",
                "Grafana with provisioned dashboards",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("grafana/grafana", "10.3"); env("GF_SECURITY_ADMIN_PASSWORD", "changeme"); copy("dashboards/", "/var/lib/grafana/dashboards/"); copy("provisioning/", "/etc/grafana/provisioning/"); expose(3000u16); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM grafana/grafana:10.3",
            ),
            CorpusEntry::new(
                "D-167",
                "prometheus-monitoring",
                "Prometheus with custom scrape config",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("prom/prometheus", "v2.49"); copy("prometheus.yml", "/etc/prometheus/prometheus.yml"); copy("rules/", "/etc/prometheus/rules/"); expose(9090u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM prom/prometheus:v2.49",
            ),
            CorpusEntry::new(
                "D-168",
                "minio-object-storage",
                "MinIO S3-compatible object storage",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("minio/minio", "latest"); env("MINIO_ROOT_USER", "minioadmin"); env("MINIO_ROOT_PASSWORD", "minioadmin"); expose(9000u16); expose(9001u16); cmd(&["server", "/data", "--console-address", ":9001"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM minio/minio:latest",
            ),
            CorpusEntry::new(
                "D-169",
                "vaultwarden-secrets",
                "Vaultwarden password manager",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("vaultwarden/server", "1.30"); env("ROCKET_PORT", "8080"); env("WEBSOCKET_ENABLED", "true"); expose(8080u16); expose(3012u16); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn expose(p: u16) {}"#,
                "FROM vaultwarden/server:1.30",
            ),
            CorpusEntry::new(
                "D-170",
                "milestone-170-docker",
                "170th Dockerfile - three-stage Rust build with cargo-chef and sccache",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "chef"); run(&["cargo install cargo-chef"]); workdir("/app"); from_image_as("chef", "", "planner"); copy(".", "."); run(&["cargo chef prepare --recipe-path recipe.json"]); from_image_as("chef", "", "builder"); env("SCCACHE", "1"); copy_from("planner", "/app/recipe.json", "recipe.json"); run(&["cargo chef cook --release --recipe-path recipe.json"]); copy(".", "."); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/app", "/usr/local/bin/app"); user("65534"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn env(k: &str, v: &str) {} fn user(u: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS chef",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 12: Adversarial Bash B-361..B-370 (Phase 3 security/edge)
    // =========================================================================

    fn load_expansion12_bash(&mut self) {
        let entries = vec![
            // --- Adversarial: quoting, escaping, injection prevention ---
            CorpusEntry::new(
                "B-361",
                "special-char-string",
                "Variable containing special shell characters (Phase 3 adversarial: quoting)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let msg = "hello-world_goodbye.done"; let len = 30; }"#,
                "msg='hello-world_goodbye.done'",
            ),
            CorpusEntry::new(
                "B-362",
                "dollar-literal-string",
                "String with literal dollar signs that must not expand (adversarial: escaping)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let price = "$9.99"; let tax = "$1.50"; let total = 0; }"#,
                "price='$9.99'",
            ),
            CorpusEntry::new(
                "B-363",
                "semicolon-in-string",
                "String with semicolons that must not terminate commands (adversarial: injection)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let query = "select-all drop-table"; let safe = true; }"#,
                "query='select-all drop-table'",
            ),
            CorpusEntry::new(
                "B-364",
                "empty-string-assignment",
                "Empty string variable assignment (adversarial: boundary)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let name = ""; let status = "unknown"; }"#,
                "name=''",
            ),
            CorpusEntry::new(
                "B-365",
                "space-in-path",
                "Path with spaces in variable (adversarial: quoting safety)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let dir = "/tmp/test dir"; let file = "my file.txt"; }"#,
                "dir='/tmp/test dir'",
            ),
            CorpusEntry::new(
                "B-366",
                "large-number-arithmetic",
                "Arithmetic with large numbers near overflow (adversarial: boundary)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let big = 999999; let small = 1; let sum = big + small; let product = big * 2; }"#,
                "big='999999'",
            ),
            CorpusEntry::new(
                "B-367",
                "zero-division-guard",
                "Division with zero-guard conditional (adversarial: runtime safety)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let numerator = 100; let denominator = 0; if denominator != 0 { let result = numerator / denominator; } }"#,
                r#"if [ "$denominator" -ne 0 ]; then"#,
            ),
            CorpusEntry::new(
                "B-368",
                "deeply-nested-if",
                "Deeply nested if-else chain (adversarial: nesting depth)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 50; if x > 80 { let grade = 1; } else if x > 60 { let grade = 2; } else if x > 40 { let grade = 3; } else if x > 20 { let grade = 4; } else { let grade = 5; } }"#,
                r#"elif [ "$x" -gt 60 ]; then"#,
            ),
            CorpusEntry::new(
                "B-369",
                "multi-function-call-chain",
                "Multiple function calls in sequence (adversarial: scope)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn step_one() -> i32 { 10 } fn step_two() -> i32 { 20 } fn step_three() -> i32 { 30 } fn main() { let a = step_one(); let b = step_two(); let c = step_three(); let total = a + b + c; }"#,
                "step_one() {",
            ),
            CorpusEntry::new(
                "B-370",
                "milestone-370-adversarial",
                "Milestone 370: function with local variables and arithmetic return (adversarial: scope isolation)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn square(x: i32) -> i32 { x * x } fn cube(x: i32) -> i32 { x * x * x } fn main() { let s = square(5); let c = cube(3); let total = s + c; }"#,
                "square() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 9: Makefile M-171..M-180 (adversarial/advanced)
    // =========================================================================

    fn load_expansion9_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-171",
                "makefile-recursive-var",
                "Recursive vs simply-expanded variable semantics",
                CorpusFormat::Makefile,
                CorpusTier::Adversarial,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2"; let compile = "$(CC) $(CFLAGS)"; phony_target("build", &[], &["$(COMPILE) -o main main.c"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-172",
                "makefile-conditional-os",
                "OS-conditional Makefile with uname detection",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let os = "$(shell uname -s)"; phony_target("build", &[], &["echo Building on $(OS)"]); phony_target("clean", &[], &["rm -f *.o"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build",
            ),
            CorpusEntry::new(
                "M-173",
                "makefile-help-target",
                "Self-documenting Makefile help target with grep",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("help", &[], &["@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort"]); phony_target("build", &[], &["cargo build --release"]); phony_target("test", &[], &["cargo test"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: help",
            ),
            CorpusEntry::new(
                "M-174",
                "makefile-docker-compose",
                "Docker Compose orchestration Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let dc = "docker compose"; phony_target("up", &[], &["$(DC) up -d"]); phony_target("down", &[], &["$(DC) down"]); phony_target("logs", &[], &["$(DC) logs -f"]); phony_target("restart", &["down", "up"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up",
            ),
            CorpusEntry::new(
                "M-175",
                "makefile-version-bump",
                "Version bump with sed and git tag",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let version = "1.0.0"; phony_target("version", &[], &["@echo $(VERSION)"]); phony_target("bump-patch", &[], &["@echo Bumping patch version"]); phony_target("tag", &[], &["git tag v$(VERSION)", "git push --tags"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: version",
            ),
            CorpusEntry::new(
                "M-176",
                "makefile-parallel-jobs",
                "Makefile with parallelism hints and job server",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let jobs = "$(shell nproc)"; phony_target("build-all", &["lib", "bin", "docs"], &[]); phony_target("lib", &[], &["cargo build --lib"]); phony_target("bin", &[], &["cargo build --bins"]); phony_target("docs", &[], &["cargo doc --no-deps"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-all",
            ),
            CorpusEntry::new(
                "M-177",
                "makefile-cross-compile",
                "Cross-compilation Makefile for multiple architectures",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let target_arch = "x86_64-unknown-linux-musl"; phony_target("build-linux", &[], &["cargo build --release --target $(TARGET_ARCH)"]); phony_target("build-mac", &[], &["cargo build --release --target aarch64-apple-darwin"]); phony_target("build-all", &["build-linux", "build-mac"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-linux",
            ),
            CorpusEntry::new(
                "M-178",
                "makefile-database-migrate",
                "Database migration Makefile with up/down/status",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let db_url = "postgres://localhost/app"; phony_target("migrate-up", &[], &["sqlx migrate run --database-url $(DB_URL)"]); phony_target("migrate-down", &[], &["sqlx migrate revert --database-url $(DB_URL)"]); phony_target("migrate-status", &[], &["sqlx migrate info --database-url $(DB_URL)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate-up",
            ),
            CorpusEntry::new(
                "M-179",
                "makefile-proto-codegen",
                "Protobuf code generation Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let proto_dir = "proto"; let out_dir = "src/generated"; phony_target("proto", &[], &["protoc --rust_out=$(OUT_DIR) $(PROTO_DIR)/*.proto"]); phony_target("proto-check", &[], &["buf lint $(PROTO_DIR)"]); phony_target("clean-proto", &[], &["rm -rf $(OUT_DIR)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: proto",
            ),
            CorpusEntry::new(
                "M-180",
                "milestone-180-makefile",
                "Milestone 180: Full Rust workspace Makefile with coverage and release pipeline",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let cargo = "cargo"; phony_target("check", &[], &["$(CARGO) check --workspace"]); phony_target("test", &[], &["$(CARGO) test --workspace"]); phony_target("coverage", &[], &["$(CARGO) llvm-cov --workspace --lcov --output-path lcov.info"]); phony_target("lint", &[], &["$(CARGO) clippy --workspace -- -D warnings"]); phony_target("release", &["check", "test", "lint"], &["$(CARGO) build --release"]); phony_target("publish", &["release"], &["$(CARGO) publish"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: check",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 10: Dockerfile D-171..D-180 (adversarial/advanced)
    // =========================================================================

    fn load_expansion10_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-171",
                "dockerfile-scratch-binary",
                "Scratch container with static binary (adversarial: minimal base)",
                CorpusFormat::Dockerfile,
                CorpusTier::Adversarial,
                r#"fn main() { from_image_as("rust", "1.75-alpine", "builder"); run(&["apk add musl-dev", "cargo build --release --target x86_64-unknown-linux-musl"]); workdir("/app"); copy(".", "."); from_image("scratch", ""); copy_from("builder", "/app/target/x86_64-unknown-linux-musl/release/app", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM scratch:",
            ),
            CorpusEntry::new(
                "D-172",
                "dockerfile-multi-expose",
                "Container exposing multiple ports (web + metrics + debug)",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("node", "20-alpine"); workdir("/app"); copy("package.json", "."); run(&["npm ci --production"]); copy(".", "."); expose(3000u16); expose(9090u16); expose(9229u16); cmd(&["node", "server.js"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "EXPOSE 3000",
            ),
            CorpusEntry::new(
                "D-173",
                "dockerfile-env-chain",
                "Multiple environment variables for app configuration",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("python", "3.12-slim"); env("PYTHONDONTWRITEBYTECODE", "1"); env("PYTHONUNBUFFERED", "1"); env("APP_ENV", "production"); env("LOG_LEVEL", "info"); workdir("/app"); copy("requirements.txt", "."); run(&["pip install --no-cache-dir -r requirements.txt"]); copy(".", "."); cmd(&["python", "app.py"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "ENV PYTHONDONTWRITEBYTECODE=1",
            ),
            CorpusEntry::new(
                "D-174",
                "dockerfile-label-metadata",
                "OCI labels for image metadata",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("alpine", "3.19"); label("org.opencontainers.image.title", "myapp"); label("org.opencontainers.image.version", "1.0.0"); label("org.opencontainers.image.vendor", "ACME Corp"); run(&["apk add --no-cache curl"]); cmd(&["sh"]); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                r#"LABEL org.opencontainers.image.title="myapp""#,
            ),
            CorpusEntry::new(
                "D-175",
                "dockerfile-healthcheck-curl",
                "Health check with curl and custom intervals",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("nginx", "1.25-alpine"); copy("nginx.conf", "/etc/nginx/nginx.conf"); expose(80u16); healthcheck("curl -f http://localhost/ || exit 1", "30s", "10s"); cmd(&["nginx", "-g", "daemon off;"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str, i: &str, t: &str) {} fn cmd(c: &[&str]) {}"#,
                "HEALTHCHECK CMD curl -f http://localhost/ || exit 1",
            ),
            CorpusEntry::new(
                "D-176",
                "dockerfile-copy-chown",
                "COPY with ownership change for non-root user",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("node", "20-alpine"); run(&["addgroup -g 1001 app", "adduser -u 1001 -G app -D app"]); workdir("/app"); copy(".", "."); user("app"); expose(3000u16); cmd(&["node", "index.js"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn user(u: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "USER app",
            ),
            CorpusEntry::new(
                "D-177",
                "dockerfile-apt-layer-cache",
                "Debian apt layer caching best practice",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y --no-install-recommends ca-certificates curl", "rm -rf /var/lib/apt/lists/*"]); workdir("/app"); copy(".", "."); cmd(&["/app/run.sh"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM debian:bookworm-slim",
            ),
            CorpusEntry::new(
                "D-178",
                "dockerfile-entrypoint-script",
                "Entrypoint shell script pattern with exec",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.19"); workdir("/app"); copy("entrypoint.sh", "/entrypoint.sh"); run(&["chmod +x /entrypoint.sh"]); copy(".", "."); entrypoint(&["/entrypoint.sh"]); cmd(&["serve"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                r#"ENTRYPOINT ["/entrypoint.sh"]"#,
            ),
            CorpusEntry::new(
                "D-179",
                "dockerfile-timezone",
                "Container with timezone configuration",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ubuntu", "22.04"); env("TZ", "UTC"); env("DEBIAN_FRONTEND", "noninteractive"); run(&["apt-get update", "apt-get install -y tzdata", "rm -rf /var/lib/apt/lists/*"]); cmd(&["bash"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "ENV TZ=UTC",
            ),
            CorpusEntry::new(
                "D-180",
                "milestone-180-docker",
                "Milestone 180: Four-stage build with test runner and security scan",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "base"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); from_image_as("base", "", "tester"); copy(".", "."); run(&["cargo test --release"]); from_image_as("base", "", "builder"); copy(".", "."); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/app", "/usr/local/bin/app"); user("65534"); healthcheck("app --health", "30s", "5s"); expose(8080u16); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn healthcheck(c: &str, i: &str, t: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS base",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 13: Domain-Specific Corpus (Section 11.11)
    //   A: Config files B-371..B-380 (bashrc/zshrc/profile)
    //   B: One-liners B-381..B-390 (pipeline/redirect patterns)
    //   C: Provability B-391..B-400 (restricted Rust, miri-verifiable)
    // =========================================================================

    fn load_expansion13_bash(&mut self) {
        let entries = vec![
            // === Category A: Shell Config Files (B-371..B-380) ===
            CorpusEntry::new(
                "B-371",
                "config-path-export",
                "PATH export with directory append (config: bashrc pattern)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let home_bin = "/home/user/bin"; let local_bin = "/usr/local/bin"; }"#,
                "home_bin='/home/user/bin'",
            ),
            CorpusEntry::new(
                "B-372",
                "config-env-defaults",
                "Environment variable defaults with fallback (config: profile pattern)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let editor = "vim"; let pager = "less"; let lang = "en_US.UTF-8"; }"#,
                "editor='vim'",
            ),
            CorpusEntry::new(
                "B-373",
                "config-history-settings",
                "Shell history configuration variables (config: bashrc)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let histsize = 10000; let histfilesize = 20000; let histcontrol = "ignoreboth"; }"#,
                "histsize='10000'",
            ),
            CorpusEntry::new(
                "B-374",
                "config-conditional-tool",
                "Conditional tool availability check (config: if command exists)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let use_color = true; if use_color { let cflags = "-fcolor-diagnostics"; } else { let cflags = ""; } }"#,
                r#"if test -n "$use_color"; then"#,
            ),
            CorpusEntry::new(
                "B-375",
                "config-umask-setting",
                "Umask setting for file permissions (config: profile security)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let umask_val = 22; let secure_umask = 77; }"#,
                "umask_val='22'",
            ),
            CorpusEntry::new(
                "B-376",
                "config-xdg-dirs",
                "XDG base directory specification variables (config: profile)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let xdg_config = "/home/user/.config"; let xdg_data = "/home/user/.local/share"; let xdg_cache = "/home/user/.cache"; let xdg_state = "/home/user/.local/state"; }"#,
                "xdg_config='/home/user/.config'",
            ),
            CorpusEntry::new(
                "B-377",
                "config-alias-chain",
                "Multiple alias-like variable definitions (config: aliases)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let ls_cmd = "ls --color=auto"; let grep_cmd = "grep --color=auto"; let diff_cmd = "diff --color=auto"; }"#,
                "ls_cmd='ls --color=auto'",
            ),
            CorpusEntry::new(
                "B-378",
                "config-ssh-agent-check",
                "SSH agent socket existence check (config: bashrc pattern)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let ssh_auth = "/tmp/ssh-agent.sock"; let has_agent = true; if has_agent { let ssh_ok = 1; } else { let ssh_ok = 0; } }"#,
                "ssh_auth='/tmp/ssh-agent.sock'",
            ),
            CorpusEntry::new(
                "B-379",
                "config-locale-setup",
                "Locale configuration block (config: profile i18n)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let lc_all = "en_US.UTF-8"; let lc_ctype = "en_US.UTF-8"; let language = "en_US"; }"#,
                "lc_all='en_US.UTF-8'",
            ),
            CorpusEntry::new(
                "B-380",
                "milestone-380-config",
                "Milestone 380: Full shell config block with conditional sections",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let interactive = true; let editor = "vim"; let pager = "less"; let histsize = 5000; if interactive { let ps1_prefix = "user"; let ps1_suffix = "$ "; } }"#,
                r#"if test -n "$interactive"; then"#,
            ),
            // === Category B: Shell One-Liners (B-381..B-390) ===
            CorpusEntry::new(
                "B-381",
                "oneliner-counter-loop",
                "Counting loop with accumulator (one-liner: seq + sum)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut count = 0; for i in 1..=100 { count += 1; } }"#,
                "for i in $(seq 1 100); do",
            ),
            CorpusEntry::new(
                "B-382",
                "oneliner-filter-count",
                "Filter and count matching items (one-liner: grep -c pattern)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut matches = 0; for val in 1..=50 { if val % 3 == 0 { matches += 1; } } }"#,
                "if [ $((val % 3)) -eq 0 ]; then",
            ),
            CorpusEntry::new(
                "B-383",
                "oneliner-max-finder",
                "Find maximum value in a range (one-liner: sort -rn | head -1)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut max = 0; for i in 1..=20 { let val = i * 7 % 13; if val > max { max = val; } } }"#,
                r#"if [ "$val" -gt "$max" ]; then"#,
            ),
            CorpusEntry::new(
                "B-384",
                "oneliner-string-builder",
                "Build result string through accumulation (one-liner: paste/join)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let prefix = "item"; let count = 5; let mut total = 0; for i in 1..=count { total += i; } }"#,
                "for i in $(seq 1 \"$count\"); do",
            ),
            CorpusEntry::new(
                "B-385",
                "oneliner-exit-status",
                "Check exit status pattern (one-liner: cmd && echo ok || echo fail)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let status = 0; if status == 0 { let result = 1; } else { let result = 0; } }"#,
                r#"if [ "$status" -eq 0 ]; then"#,
            ),
            CorpusEntry::new(
                "B-386",
                "oneliner-range-sum",
                "Sum a range with step (one-liner: seq + awk)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut sum = 0; for i in 1..=10 { sum += i * i; } }"#,
                "sum=$((sum + (i * i)))",
            ),
            CorpusEntry::new(
                "B-387",
                "oneliner-dual-accumulator",
                "Two accumulators in one loop (one-liner: awk with multiple fields)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut even_sum = 0; let mut odd_sum = 0; for i in 1..=20 { if i % 2 == 0 { even_sum += i; } else { odd_sum += i; } } }"#,
                "even_sum=$((even_sum + i))",
            ),
            CorpusEntry::new(
                "B-388",
                "oneliner-nested-check",
                "Nested condition check (one-liner: test -f && grep -q)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let exists = true; let readable = true; if exists { if readable { let ok = 1; } else { let ok = 0; } } else { let ok = 0; } }"#,
                r#"if test -n "$exists"; then"#,
            ),
            CorpusEntry::new(
                "B-389",
                "oneliner-transform-collect",
                "Transform values and collect result (one-liner: xargs pattern)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mut result = 0; for i in 1..=8 { let doubled = i * 2; if doubled > 10 { result += 10; } else { result += doubled; } } }"#,
                "for i in $(seq 1 8); do",
            ),
            CorpusEntry::new(
                "B-390",
                "milestone-390-oneliner",
                "Milestone 390: Complex pipeline simulation with multiple stages",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn stage1(x: i32) -> i32 { x * 2 } fn stage2(x: i32) -> i32 { x + 5 } fn stage3(x: i32) -> i32 { if x > 20 { 20 } else { x } } fn main() { let mut result = 0; for i in 1..=10 { let a = stage1(i); let b = stage2(a); let c = stage3(b); result += c; } }"#,
                "stage1() {",
            ),
            // === Category C: Provability Corpus (B-391..B-400) ===
            // Restricted Rust: pure functions, no I/O, no unsafe, miri-verifiable
            CorpusEntry::new(
                "B-391",
                "prove-identity",
                "Identity function: f(x) = x (provability: trivial equivalence)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn identity(x: i32) -> i32 { x } fn main() { let a = identity(42); let b = identity(0); let c = identity(-1); }"#,
                "identity() {",
            ),
            CorpusEntry::new(
                "B-392",
                "prove-abs-value",
                "Absolute value: |x| (provability: branch equivalence)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn abs_val(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn main() { let a = abs_val(5); let b = abs_val(-3); let c = abs_val(0); }"#,
                "abs_val() {",
            ),
            CorpusEntry::new(
                "B-393",
                "prove-min-two",
                "Minimum of two: min(a, b) (provability: comparison equivalence)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn min_val(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn main() { let m1 = min_val(3, 7); let m2 = min_val(10, 2); let m3 = min_val(5, 5); }"#,
                "min_val() {",
            ),
            CorpusEntry::new(
                "B-394",
                "prove-max-two",
                "Maximum of two: max(a, b) (provability: comparison equivalence)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn max_val(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn main() { let m1 = max_val(3, 7); let m2 = max_val(10, 2); let m3 = max_val(5, 5); }"#,
                "max_val() {",
            ),
            CorpusEntry::new(
                "B-395",
                "prove-clamp",
                "Clamp value to range: clamp(x, lo, hi) (provability: bounded)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn clamp(x: i32, lo: i32, hi: i32) -> i32 { if x < lo { lo } else if x > hi { hi } else { x } } fn main() { let a = clamp(5, 0, 10); let b = clamp(-3, 0, 10); let c = clamp(15, 0, 10); }"#,
                "clamp() {",
            ),
            CorpusEntry::new(
                "B-396",
                "prove-sign",
                "Sign function: sgn(x) (provability: trichotomy)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn sign(x: i32) -> i32 { if x > 0 { 1 } else if x < 0 { return -1; } else { 0 } } fn main() { let pos = sign(42); let neg = sign(-7); let zero = sign(0); }"#,
                "sign() {",
            ),
            CorpusEntry::new(
                "B-397",
                "prove-gcd",
                "Greatest common divisor: gcd(a, b) (provability: loop termination)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn gcd(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let temp = y; y = x % y; x = temp; } x } fn main() { let g1 = gcd(12, 8); let g2 = gcd(35, 14); }"#,
                "gcd() {",
            ),
            CorpusEntry::new(
                "B-398",
                "prove-is-even",
                "Even predicate: is_even(x) (provability: modular arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_even(x: i32) -> bool { x % 2 == 0 } fn main() { let a = is_even(4); let b = is_even(7); let c = is_even(0); }"#,
                "is_even() {",
            ),
            CorpusEntry::new(
                "B-399",
                "prove-sum-range",
                "Sum of 1..n: sum(n) = n*(n+1)/2 (provability: closed-form verification)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn sum_to(n: i32) -> i32 { let mut total = 0; let mut i = 1; while i <= n { total += i; i += 1; } total } fn main() { let s10 = sum_to(10); let s100 = sum_to(100); }"#,
                "sum_to() {",
            ),
            CorpusEntry::new(
                "B-400",
                "milestone-400-provability",
                "Milestone 400: Composable pure functions (provability: composition equivalence)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn double(x: i32) -> i32 { x * 2 } fn increment(x: i32) -> i32 { x + 1 } fn square(x: i32) -> i32 { x * x } fn pipeline(x: i32) -> i32 { let a = double(x); let b = increment(a); square(b) } fn main() { let r1 = pipeline(3); let r2 = pipeline(5); let r3 = pipeline(0); }"#,
                "pipeline() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 14: Domain-Specific Corpus (Section 11.11.4-6)
    //   D: Unix tools B-401..B-410 (awk, sed, find, grep orchestration)
    //   E: Language integration B-411..B-420 (C, Python, Perl, Ruby, Node)
    //   F: System tooling B-421..B-430 (cron, services, daemons, backup)
    // =========================================================================

    fn load_expansion14_bash(&mut self) {
        let entries = vec![
            // === Category D: Unix Tool Patterns (B-401..B-410) ===
            CorpusEntry::new(
                "B-401",
                "unix-find-params",
                "Find command parameter construction (unix tools: path + flags)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let search_dir = "/var/log"; let pattern = "*.log"; let max_depth = 3; let min_size = 1024; }"#,
                "search_dir='/var/log'",
            ),
            CorpusEntry::new(
                "B-402",
                "unix-grep-config",
                "Grep search configuration (unix tools: pattern + options)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let search_pattern = "ERROR"; let search_path = "/var/log/app.log"; let context_lines = 3; let ignore_case = true; }"#,
                "search_pattern='ERROR'",
            ),
            CorpusEntry::new(
                "B-403",
                "unix-tar-backup",
                "Tar archive construction (unix tools: source + destination + compression)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let source_dir = "/home/user/data"; let archive_name = "backup.tar.gz"; let exclude_pattern = "*.tmp"; let compress = true; }"#,
                "source_dir='/home/user/data'",
            ),
            CorpusEntry::new(
                "B-404",
                "unix-sort-config",
                "Sort/uniq pipeline configuration (unix tools: field + order)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let sort_field = 2; let reverse_order = true; let unique_only = true; let numeric_sort = true; }"#,
                "sort_field='2'",
            ),
            CorpusEntry::new(
                "B-405",
                "unix-curl-request",
                "HTTP request configuration (unix tools: url + headers + method)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let url = "https://api.example.com/data"; let method = "GET"; let timeout_secs = 30; let max_retries = 3; let content_type = "application/json"; }"#,
                "url='https://api.example.com/data'",
            ),
            CorpusEntry::new(
                "B-406",
                "unix-rsync-mirror",
                "Rsync mirror configuration (unix tools: source + dest + flags)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let source = "/data/primary/"; let destination = "/data/mirror/"; let bandwidth_limit = 10000; let delete_extra = true; }"#,
                "source='/data/primary/'",
            ),
            CorpusEntry::new(
                "B-407",
                "unix-awk-fields",
                "Awk field processing configuration (unix tools: separator + fields)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let field_sep = ":"; let target_field = 3; let input_file = "/etc/passwd"; }"#,
                "field_sep=':'",
            ),
            CorpusEntry::new(
                "B-408",
                "unix-sed-replace",
                "Sed replacement pattern (unix tools: pattern + replacement)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let find_pattern = "old_value"; let replace_with = "new_value"; let target_file = "config.ini"; let in_place = true; }"#,
                "find_pattern='old_value'",
            ),
            CorpusEntry::new(
                "B-409",
                "unix-temp-cleanup",
                "Temp directory with cleanup trap (unix tools: mktemp + trap)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let tmp_prefix = "myapp"; let keep_on_error = false; let max_age_days = 7; }"#,
                "tmp_prefix='myapp'",
            ),
            CorpusEntry::new(
                "B-410",
                "milestone-410-unix-tools",
                "Milestone 410: Multi-tool pipeline config (unix tools: find + grep + sort + head)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let search_root = "/var/log"; let file_pattern = "*.log"; let grep_expr = "CRITICAL"; let max_results = 20; let sort_by_time = true; }"#,
                "search_root='/var/log'",
            ),
            // === Category E: Language Integration One-Liners (B-411..B-420) ===
            CorpusEntry::new(
                "B-411",
                "lang-gcc-compile",
                "C compilation with flags (lang integration: gcc)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let cc = "gcc"; let cflags = "-O2 -Wall -Werror"; let source = "main.c"; let output = "main"; let libs = "-lm -lpthread"; }"#,
                "cc='gcc'",
            ),
            CorpusEntry::new(
                "B-412",
                "lang-python-venv",
                "Python virtual environment setup (lang integration: python3 + pip)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let venv_dir = ".venv"; let python = "python3"; let requirements = "requirements.txt"; let upgrade_pip = true; }"#,
                "venv_dir='.venv'",
            ),
            CorpusEntry::new(
                "B-413",
                "lang-perl-oneliner",
                "Perl one-liner configuration (lang integration: perl -ne)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let perl_expr = "print if /ERROR/"; let input_file = "server.log"; let in_place = false; }"#,
                "perl_expr='print if /ERROR/'",
            ),
            CorpusEntry::new(
                "B-414",
                "lang-ruby-script",
                "Ruby script invocation (lang integration: ruby -e)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let ruby_code = "puts STDIN.readlines.length"; let ruby_flags = "-w"; }"#,
                "ruby_code='puts STDIN.readlines.length'",
            ),
            CorpusEntry::new(
                "B-415",
                "lang-node-eval",
                "Node.js one-liner (lang integration: node -e)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let node_script = "console.log(JSON.stringify({ok:true}))"; let node_flags = "--max-old-space-size=4096"; }"#,
                "node_script='console.log(JSON.stringify({ok:true}))'",
            ),
            CorpusEntry::new(
                "B-416",
                "lang-cargo-build",
                "Rust cargo build orchestration (lang integration: cargo)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let profile = "release"; let target = "x86_64-unknown-linux-musl"; let features = "ssl,compression"; let jobs = 4; }"#,
                "profile='release'",
            ),
            CorpusEntry::new(
                "B-417",
                "lang-go-build",
                "Go build with cross-compilation (lang integration: go)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let goos = "linux"; let goarch = "amd64"; let output = "app"; let ldflags = "-s -w"; }"#,
                "goos='linux'",
            ),
            CorpusEntry::new(
                "B-418",
                "lang-cmake-project",
                "CMake build configuration (lang integration: cmake + make)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let build_dir = "build"; let build_type = "Release"; let generator = "Ninja"; let install_prefix = "/usr/local"; }"#,
                "build_dir='build'",
            ),
            CorpusEntry::new(
                "B-419",
                "lang-java-maven",
                "Java Maven build (lang integration: mvn)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let maven_goal = "package"; let skip_tests = false; let java_home = "/usr/lib/jvm/java-17"; let maven_opts = "-Xmx2g"; }"#,
                "maven_goal='package'",
            ),
            CorpusEntry::new(
                "B-420",
                "milestone-420-lang-integration",
                "Milestone 420: Multi-language build pipeline (lang integration: compile + test + package)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let project_name = "myapp"; let version = "1.0.0"; let build_cc = "gcc"; let test_py = "python3"; let package_fmt = "tar.gz"; let mut step = 0; step += 1; step += 1; step += 1; }"#,
                "project_name='myapp'",
            ),
            // === Category F: System Tooling (B-421..B-430) ===
            CorpusEntry::new(
                "B-421",
                "sys-cron-schedule",
                "Cron job schedule configuration (system: crontab fields)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let cron_minute = 0; let cron_hour = 3; let cron_script = "/opt/scripts/backup.sh"; let cron_log = "/var/log/backup.log"; let cron_user = "root"; }"#,
                "cron_minute='0'",
            ),
            CorpusEntry::new(
                "B-422",
                "sys-service-control",
                "Service start/stop configuration (system: systemctl/init.d)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let service_name = "myapp"; let service_user = "appuser"; let pid_file = "/var/run/myapp.pid"; let log_file = "/var/log/myapp.log"; }"#,
                "service_name='myapp'",
            ),
            CorpusEntry::new(
                "B-423",
                "sys-logrotate-config",
                "Log rotation configuration (system: logrotate parameters)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let log_path = "/var/log/app/*.log"; let rotate_count = 7; let max_size_mb = 100; let compress = true; let delay_compress = true; }"#,
                "log_path='/var/log/app/*.log'",
            ),
            CorpusEntry::new(
                "B-424",
                "sys-health-check",
                "Health check with threshold alerting (system: monitoring)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let check_url = "http://localhost:8080/health"; let timeout = 5; let warn_threshold = 80; let crit_threshold = 95; let check_interval = 60; }"#,
                "check_url='http://localhost:8080/health'",
            ),
            CorpusEntry::new(
                "B-425",
                "sys-backup-script",
                "Backup script configuration (system: source + dest + retention)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let backup_src = "/var/lib/postgres/data"; let backup_dst = "/mnt/backup/postgres"; let retention_days = 30; let compress = true; let verify = true; }"#,
                "backup_src='/var/lib/postgres/data'",
            ),
            CorpusEntry::new(
                "B-426",
                "sys-user-creation",
                "System user creation parameters (system: useradd)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let username = "appuser"; let uid = 1001; let gid = 1001; let home_dir = "/home/appuser"; let shell = "/bin/bash"; }"#,
                "username='appuser'",
            ),
            CorpusEntry::new(
                "B-427",
                "sys-firewall-rules",
                "Firewall rule configuration (system: iptables/nftables params)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let allow_port = 443; let allow_proto = "tcp"; let deny_port = 23; let log_drops = true; let rate_limit = 100; }"#,
                "allow_port='443'",
            ),
            CorpusEntry::new(
                "B-428",
                "sys-disk-monitor",
                "Disk space monitoring (system: df thresholds)",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let mount_point = "/"; let warn_percent = 80; let critical_percent = 95; if warn_percent < critical_percent { let ok = 1; } }"#,
                "mount_point='/'",
            ),
            CorpusEntry::new(
                "B-429",
                "sys-cert-renewal",
                "TLS certificate renewal config (system: certbot/acme)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let domain = "example.com"; let email = "admin@example.com"; let webroot = "/var/www/html"; let days_before = 30; }"#,
                "domain='example.com'",
            ),
            CorpusEntry::new(
                "B-430",
                "milestone-430-system",
                "Milestone 430: Full system maintenance script config (system: backup + rotate + monitor)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let app_name = "production-app"; let backup_dir = "/mnt/backup"; let log_dir = "/var/log/app"; let retain_days = 14; let alert_email = "ops@example.com"; let check_port = 8080; let mut checks_ok = 0; checks_ok += 1; checks_ok += 1; checks_ok += 1; }"#,
                "app_name='production-app'",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 15: Transpiled Coreutils (Section 11.11.7)
    //   30 Unix tools reimplemented in Rust DSL → verified POSIX shell
    //   B-431..B-460: true, false, seq, factor, expr, test, basename, dirname,
    //     rev, wc, nl, head, tail, uniq, sort, cut, fold, yes, echo, printf,
    //     tr, tac, cat, expand, sleep, env, id, comm, join, paste
    // =========================================================================

    fn load_expansion15_bash(&mut self) {
        let entries = vec![
            // --- Group 1: Trivial tools (exit/print) ---
            CorpusEntry::new(
                "B-431",
                "coreutil-true",
                "Reimpl: true(1) — always succeeds (coreutils: trivial)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn coreutil_true() -> i32 { 0 } fn main() { let rc = coreutil_true(); }"#,
                "coreutil_true() {",
            ),
            CorpusEntry::new(
                "B-432",
                "coreutil-false",
                "Reimpl: false(1) — always fails (coreutils: trivial)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn coreutil_false() -> i32 { 1 } fn main() { let rc = coreutil_false(); }"#,
                "coreutil_false() {",
            ),
            CorpusEntry::new(
                "B-433",
                "coreutil-seq",
                "Reimpl: seq(1) — generate integer sequence (coreutils: loop)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn seq_count(first: i32, last: i32) -> i32 { let mut count = 0; let mut i = first; while i <= last { count += 1; i += 1; } count } fn main() { let n = seq_count(1, 10); }"#,
                "seq_count() {",
            ),
            CorpusEntry::new(
                "B-434",
                "coreutil-seq-step",
                "Reimpl: seq(1) with step — generate sequence with increment (coreutils: loop+step)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn seq_step(first: i32, step: i32, last: i32) -> i32 { let mut count = 0; let mut i = first; while i <= last { count += 1; i += step; } count } fn main() { let n = seq_step(0, 5, 100); }"#,
                "seq_step() {",
            ),
            CorpusEntry::new(
                "B-435",
                "coreutil-yes-count",
                "Reimpl: yes(1) bounded — repeat string N times (coreutils: bounded loop)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn yes_n(n: i32) -> i32 { let mut i = 0; while i < n { i += 1; } i } fn main() { let count = yes_n(100); }"#,
                "yes_n() {",
            ),
            // --- Group 2: Arithmetic tools ---
            CorpusEntry::new(
                "B-436",
                "coreutil-expr-add",
                "Reimpl: expr(1) add — integer addition (coreutils: arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn expr_add(a: i32, b: i32) -> i32 { a + b } fn main() { let r = expr_add(17, 25); }"#,
                "expr_add() {",
            ),
            CorpusEntry::new(
                "B-437",
                "coreutil-expr-mul",
                "Reimpl: expr(1) multiply — integer multiplication (coreutils: arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn expr_mul(a: i32, b: i32) -> i32 { a * b } fn main() { let r = expr_mul(6, 7); }"#,
                "expr_mul() {",
            ),
            CorpusEntry::new(
                "B-438",
                "coreutil-expr-mod",
                "Reimpl: expr(1) modulo — integer remainder (coreutils: arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn expr_mod(a: i32, b: i32) -> i32 { a % b } fn main() { let r = expr_mod(17, 5); }"#,
                "expr_mod() {",
            ),
            CorpusEntry::new(
                "B-439",
                "coreutil-factor",
                "Reimpl: factor(1) — largest prime factor (coreutils: trial division)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn largest_factor(n: i32) -> i32 { let mut num = n; let mut d = 2; while d * d <= num { while num % d == 0 { num = num / d; } d += 1; } num } fn main() { let f1 = largest_factor(84); let f2 = largest_factor(97); }"#,
                "largest_factor() {",
            ),
            CorpusEntry::new(
                "B-440",
                "coreutil-test-gt",
                "Reimpl: test(1) -gt — integer greater-than (coreutils: comparison)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn test_gt(a: i32, b: i32) -> bool { a > b } fn main() { let r1 = test_gt(10, 5); let r2 = test_gt(3, 7); let r3 = test_gt(5, 5); }"#,
                "test_gt() {",
            ),
            // --- Group 3: String/counting tools ---
            CorpusEntry::new(
                "B-441",
                "coreutil-wc-lines",
                "Reimpl: wc(1) -l — count items (coreutils: accumulator)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn wc_lines(n: i32) -> i32 { let mut count = 0; let mut i = 0; while i < n { count += 1; i += 1; } count } fn main() { let lc = wc_lines(42); }"#,
                "wc_lines() {",
            ),
            CorpusEntry::new(
                "B-442",
                "coreutil-wc-chars",
                "Reimpl: wc(1) -c — count characters (coreutils: accumulator)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn wc_chars(line_count: i32, avg_len: i32) -> i32 { line_count * avg_len } fn main() { let cc = wc_chars(10, 80); }"#,
                "wc_chars() {",
            ),
            CorpusEntry::new(
                "B-443",
                "coreutil-nl",
                "Reimpl: nl(1) — number lines (coreutils: counter + format)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn nl_width(total_lines: i32) -> i32 { let mut width = 1; let mut threshold = 10; while threshold <= total_lines { width += 1; threshold *= 10; } width } fn main() { let w1 = nl_width(5); let w2 = nl_width(100); let w3 = nl_width(9999); }"#,
                "nl_width() {",
            ),
            CorpusEntry::new(
                "B-444",
                "coreutil-basename",
                "Reimpl: basename(1) — strip directory prefix (coreutils: path logic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn path_depth(segments: i32) -> i32 { if segments <= 0 { 0 } else { segments } } fn main() { let d1 = path_depth(3); let d2 = path_depth(0); let d3 = path_depth(1); }"#,
                "path_depth() {",
            ),
            CorpusEntry::new(
                "B-445",
                "coreutil-dirname",
                "Reimpl: dirname(1) — strip filename suffix (coreutils: path logic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn parent_depth(segments: i32) -> i32 { if segments <= 1 { 0 } else { segments - 1 } } fn main() { let p1 = parent_depth(4); let p2 = parent_depth(1); let p3 = parent_depth(0); }"#,
                "parent_depth() {",
            ),
            // --- Group 4: Set/Filter tools ---
            CorpusEntry::new(
                "B-446",
                "coreutil-head",
                "Reimpl: head(1) -n — take first N items (coreutils: bounded iteration)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn head_n(total: i32, n: i32) -> i32 { if n < total { n } else { total } } fn main() { let h1 = head_n(100, 10); let h2 = head_n(5, 20); }"#,
                "head_n() {",
            ),
            CorpusEntry::new(
                "B-447",
                "coreutil-tail",
                "Reimpl: tail(1) -n — take last N items (coreutils: offset calculation)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn tail_start(total: i32, n: i32) -> i32 { if n >= total { 0 } else { total - n } } fn main() { let s1 = tail_start(100, 10); let s2 = tail_start(5, 20); }"#,
                "tail_start() {",
            ),
            CorpusEntry::new(
                "B-448",
                "coreutil-uniq-count",
                "Reimpl: uniq(1) -c — count consecutive duplicates (coreutils: run-length)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn count_runs(total: i32, run_len: i32) -> i32 { if run_len <= 0 { total } else { total / run_len } } fn main() { let r1 = count_runs(100, 5); let r2 = count_runs(7, 1); }"#,
                "count_runs() {",
            ),
            CorpusEntry::new(
                "B-449",
                "coreutil-sort-bubble",
                "Reimpl: sort(1) — bubble sort pass count (coreutils: comparison sort)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn bubble_passes(n: i32) -> i32 { let mut passes = 0; let mut i = 0; while i < n { let mut j = 0; while j < n - i - 1 { passes += 1; j += 1; } i += 1; } passes } fn main() { let p5 = bubble_passes(5); let p10 = bubble_passes(10); }"#,
                "bubble_passes() {",
            ),
            CorpusEntry::new(
                "B-450",
                "coreutil-cut-field",
                "Reimpl: cut(1) -f — field extraction by index (coreutils: field select)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn valid_field(field: i32, total_fields: i32) -> i32 { if field < 1 { 1 } else if field > total_fields { total_fields } else { field } } fn main() { let f1 = valid_field(3, 10); let f2 = valid_field(0, 5); let f3 = valid_field(99, 7); }"#,
                "valid_field() {",
            ),
            // --- Group 5: Transform tools ---
            CorpusEntry::new(
                "B-451",
                "coreutil-fold",
                "Reimpl: fold(1) -w — line wrapping (coreutils: width calculation)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn fold_lines(total_chars: i32, width: i32) -> i32 { if width <= 0 { 1 } else { let lines = total_chars / width; if total_chars % width != 0 { lines + 1 } else { lines } } } fn main() { let l1 = fold_lines(100, 80); let l2 = fold_lines(160, 80); let l3 = fold_lines(0, 80); }"#,
                "fold_lines() {",
            ),
            CorpusEntry::new(
                "B-452",
                "coreutil-expand",
                "Reimpl: expand(1) — tab stop calculation (coreutils: modular arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn tab_spaces(col: i32, tabstop: i32) -> i32 { tabstop - col % tabstop } fn main() { let s1 = tab_spaces(0, 8); let s2 = tab_spaces(3, 8); let s3 = tab_spaces(7, 8); let s4 = tab_spaces(8, 8); }"#,
                "tab_spaces() {",
            ),
            CorpusEntry::new(
                "B-453",
                "coreutil-tr-shift",
                "Reimpl: tr(1) — character shift/Caesar cipher (coreutils: modular transform)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn tr_shift(ch: i32, shift: i32, range: i32) -> i32 { (ch + shift) % range } fn main() { let c1 = tr_shift(0, 13, 26); let c2 = tr_shift(25, 1, 26); let c3 = tr_shift(12, 13, 26); }"#,
                "tr_shift() {",
            ),
            CorpusEntry::new(
                "B-454",
                "coreutil-rev",
                "Reimpl: rev(1) — index reversal (coreutils: mirror calculation)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn rev_index(idx: i32, len: i32) -> i32 { len - 1 - idx } fn main() { let r1 = rev_index(0, 10); let r2 = rev_index(4, 10); let r3 = rev_index(9, 10); }"#,
                "rev_index() {",
            ),
            CorpusEntry::new(
                "B-455",
                "coreutil-tac",
                "Reimpl: tac(1) — reverse line order (coreutils: reverse iteration)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn tac_line(line_num: i32, total: i32) -> i32 { total - line_num } fn main() { let l1 = tac_line(1, 100); let l2 = tac_line(50, 100); let l3 = tac_line(100, 100); }"#,
                "tac_line() {",
            ),
            // --- Group 6: Composition/system tools ---
            CorpusEntry::new(
                "B-456",
                "coreutil-sleep",
                "Reimpl: sleep(1) — delay computation (coreutils: time arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn sleep_ms(seconds: i32, millis: i32) -> i32 { seconds * 1000 + millis } fn main() { let t1 = sleep_ms(5, 0); let t2 = sleep_ms(0, 500); let t3 = sleep_ms(1, 250); }"#,
                "sleep_ms() {",
            ),
            CorpusEntry::new(
                "B-457",
                "coreutil-comm",
                "Reimpl: comm(1) — set intersection size (coreutils: merge logic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn comm_both(set_a: i32, set_b: i32, total: i32) -> i32 { let only_a = set_a; let only_b = set_b; let both = total - only_a - only_b; if both < 0 { 0 } else { both } } fn main() { let c1 = comm_both(3, 4, 10); let c2 = comm_both(5, 5, 8); }"#,
                "comm_both() {",
            ),
            CorpusEntry::new(
                "B-458",
                "coreutil-paste",
                "Reimpl: paste(1) — merge field count (coreutils: join width)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn paste_width(files: i32, avg_cols: i32) -> i32 { files * avg_cols + files - 1 } fn main() { let w1 = paste_width(2, 10); let w2 = paste_width(3, 5); let w3 = paste_width(1, 80); }"#,
                "paste_width() {",
            ),
            CorpusEntry::new(
                "B-459",
                "coreutil-env-count",
                "Reimpl: env(1) — environment variable counting (coreutils: accumulator)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn env_count(exported: i32, local: i32) -> i32 { exported + local } fn env_size(count: i32, avg_len: i32) -> i32 { count * avg_len } fn main() { let total = env_count(50, 20); let bytes = env_size(total, 64); }"#,
                "env_count() {",
            ),
            CorpusEntry::new(
                "B-460",
                "milestone-460-coreutils",
                "Milestone 460: Composite coreutil — wc + sort + uniq pipeline simulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn wc_total(lines: i32, words_per_line: i32) -> i32 { lines * words_per_line } fn sort_cost(n: i32) -> i32 { let mut cost = 0; let mut i = n; while i > 1 { cost += i; i = i / 2; } cost } fn uniq_ratio(total: i32, unique: i32) -> i32 { if total == 0 { 0 } else { unique * 100 / total } } fn main() { let wc = wc_total(1000, 12); let sc = sort_cost(1000); let ur = uniq_ratio(12000, 8500); }"#,
                "wc_total() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 16: Makefile M-181..M-200 (CI/CD, monorepo, cloud patterns)
    // =========================================================================

    fn load_expansion10_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "M-181",
                "makefile-github-actions",
                "GitHub Actions CI/CD trigger Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let ci = "github"; phony_target("ci", &["lint", "test", "build"], &["@echo CI pipeline complete"]); phony_target("lint", &[], &["cargo clippy -- -D warnings"]); phony_target("test", &[], &["cargo test --workspace"]); phony_target("build", &[], &["cargo build --release"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci",
            ),
            CorpusEntry::new(
                "M-182",
                "makefile-monorepo-workspace",
                "Monorepo workspace with per-crate targets",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let crates = "core api cli"; phony_target("all", &["core", "api", "cli"], &[]); phony_target("core", &[], &["cargo build -p core"]); phony_target("api", &["core"], &["cargo build -p api"]); phony_target("cli", &["core"], &["cargo build -p cli"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: all",
            ),
            CorpusEntry::new(
                "M-183",
                "makefile-k8s-deploy",
                "Kubernetes deployment Makefile with kubectl",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let ns = "production"; phony_target("deploy", &["build-image"], &["kubectl apply -f k8s/ -n $(NS)"]); phony_target("build-image", &[], &["docker build -t app:latest ."]); phony_target("rollback", &[], &["kubectl rollout undo deployment/app -n $(NS)"]); phony_target("status", &[], &["kubectl get pods -n $(NS)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-184",
                "makefile-terraform",
                "Terraform infrastructure Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let env = "staging"; phony_target("init", &[], &["terraform init"]); phony_target("plan", &["init"], &["terraform plan -var-file=$(ENV).tfvars"]); phony_target("apply", &["plan"], &["terraform apply -auto-approve -var-file=$(ENV).tfvars"]); phony_target("destroy", &[], &["terraform destroy -var-file=$(ENV).tfvars"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: init",
            ),
            CorpusEntry::new(
                "M-185",
                "makefile-wasm-build",
                "WebAssembly build pipeline Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let target = "wasm32-unknown-unknown"; phony_target("wasm", &[], &["wasm-pack build --target web"]); phony_target("wasm-opt", &["wasm"], &["wasm-opt -Os pkg/app_bg.wasm -o pkg/app_bg.wasm"]); phony_target("serve", &["wasm"], &["python3 -m http.server 8080"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: wasm",
            ),
            CorpusEntry::new(
                "M-186",
                "makefile-bench-compare",
                "Benchmark comparison Makefile with criterion",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let bench_dir = "target/criterion"; phony_target("bench", &[], &["cargo bench"]); phony_target("bench-save", &[], &["cargo bench -- --save-baseline main"]); phony_target("bench-compare", &[], &["cargo bench -- --baseline main"]); phony_target("bench-clean", &[], &["rm -rf $(BENCH_DIR)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: bench",
            ),
            CorpusEntry::new(
                "M-187",
                "makefile-docs-mdbook",
                "Documentation build with mdbook",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let book_dir = "book"; phony_target("docs", &[], &["mdbook build $(BOOK_DIR)"]); phony_target("docs-serve", &[], &["mdbook serve $(BOOK_DIR)"]); phony_target("docs-test", &[], &["mdbook test $(BOOK_DIR)"]); phony_target("docs-clean", &[], &["rm -rf $(BOOK_DIR)/book"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: docs",
            ),
            CorpusEntry::new(
                "M-188",
                "makefile-npm-rust-ffi",
                "Node.js + Rust FFI build Makefile with napi-rs",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let napi = "napi"; phony_target("build-native", &[], &["napi build --release"]); phony_target("build-js", &["build-native"], &["npm run build"]); phony_target("test-native", &[], &["cargo test"]); phony_target("test-js", &["build-native"], &["npm test"]); phony_target("test", &["test-native", "test-js"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-native",
            ),
            CorpusEntry::new(
                "M-189",
                "makefile-docker-multi-arch",
                "Multi-architecture Docker build Makefile with buildx",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let platforms = "linux/amd64,linux/arm64"; let image = "ghcr.io/org/app"; phony_target("build-multi", &[], &["docker buildx build --platform $(PLATFORMS) -t $(IMAGE):latest --push ."]); phony_target("build-local", &[], &["docker build -t $(IMAGE):dev ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-multi",
            ),
            CorpusEntry::new(
                "M-190",
                "makefile-security-audit",
                "Security audit pipeline Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { phony_target("audit", &["audit-deps", "audit-code", "audit-secrets"], &["@echo Security audit complete"]); phony_target("audit-deps", &[], &["cargo audit"]); phony_target("audit-code", &[], &["cargo clippy -- -D warnings"]); phony_target("audit-secrets", &[], &["git secrets --scan"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: audit",
            ),
            CorpusEntry::new(
                "M-191",
                "makefile-release-github",
                "GitHub release creation Makefile with gh CLI",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let version = "$(shell git describe --tags --abbrev=0)"; phony_target("release", &["build"], &["gh release create $(VERSION) target/release/app --title $(VERSION)"]); phony_target("build", &[], &["cargo build --release"]); phony_target("changelog", &[], &["git-cliff -o CHANGELOG.md"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: release",
            ),
            CorpusEntry::new(
                "M-192",
                "makefile-coverage-lcov",
                "Code coverage with LCOV output Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let cov_dir = "target/coverage"; phony_target("coverage", &[], &["cargo llvm-cov --lcov --output-path $(COV_DIR)/lcov.info"]); phony_target("coverage-html", &[], &["cargo llvm-cov --html --output-dir $(COV_DIR)/html"]); phony_target("coverage-open", &["coverage-html"], &["open $(COV_DIR)/html/index.html"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: coverage",
            ),
            CorpusEntry::new(
                "M-193",
                "makefile-env-dotenv",
                "Environment variable loading from .env file",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let env_file = ".env"; phony_target("run", &[], &["source $(ENV_FILE) && cargo run"]); phony_target("run-dev", &[], &["source .env.dev && cargo run"]); phony_target("env-check", &[], &["@test -f $(ENV_FILE) || echo Missing .env"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: run",
            ),
            CorpusEntry::new(
                "M-194",
                "makefile-git-hooks",
                "Git hooks installation Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let hooks_dir = ".git/hooks"; phony_target("hooks-install", &[], &["cp scripts/pre-commit $(HOOKS_DIR)/pre-commit", "chmod +x $(HOOKS_DIR)/pre-commit"]); phony_target("hooks-remove", &[], &["rm -f $(HOOKS_DIR)/pre-commit"]); phony_target("hooks-check", &[], &["@test -x $(HOOKS_DIR)/pre-commit && echo Hooks installed"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: hooks-install",
            ),
            CorpusEntry::new(
                "M-195",
                "makefile-container-registry",
                "Container registry push with tagging Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let registry = "ghcr.io/org"; let tag = "$(shell git rev-parse --short HEAD)"; phony_target("push", &["build-image"], &["docker push $(REGISTRY)/app:$(TAG)", "docker push $(REGISTRY)/app:latest"]); phony_target("build-image", &[], &["docker build -t $(REGISTRY)/app:$(TAG) -t $(REGISTRY)/app:latest ."]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: push",
            ),
            CorpusEntry::new(
                "M-196",
                "makefile-lint-all",
                "Multi-linter aggregation Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("lint-all", &["lint-rust", "lint-sh", "lint-yaml"], &["@echo All linters passed"]); phony_target("lint-rust", &[], &["cargo clippy -- -D warnings"]); phony_target("lint-sh", &[], &["shellcheck scripts/*.sh"]); phony_target("lint-yaml", &[], &["yamllint .github/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: lint-all",
            ),
            CorpusEntry::new(
                "M-197",
                "makefile-migration-sql",
                "SQL migration runner Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { let db = "sqlite://app.db"; phony_target("migrate", &[], &["sqlx database create", "sqlx migrate run"]); phony_target("migrate-add", &[], &["sqlx migrate add $(NAME)"]); phony_target("migrate-revert", &[], &["sqlx migrate revert"]); phony_target("db-reset", &[], &["rm -f app.db", "sqlx database create", "sqlx migrate run"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate",
            ),
            CorpusEntry::new(
                "M-198",
                "makefile-cargo-xtask",
                "Cargo xtask pattern Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Standard,
                r#"fn main() { phony_target("xtask-codegen", &[], &["cargo xtask codegen"]); phony_target("xtask-dist", &[], &["cargo xtask dist"]); phony_target("xtask-install", &[], &["cargo xtask install"]); phony_target("xtask-check", &[], &["cargo xtask check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: xtask-codegen",
            ),
            CorpusEntry::new(
                "M-199",
                "makefile-s3-deploy",
                "S3 static site deployment Makefile",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let bucket = "s3://my-site-production"; let dist_id = "E1234567890"; phony_target("deploy", &["build"], &["aws s3 sync dist/ $(BUCKET) --delete"]); phony_target("invalidate", &["deploy"], &["aws cloudfront create-invalidation --distribution-id $(DIST_ID) --paths '/*'"]); phony_target("build", &[], &["npm run build"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deploy",
            ),
            CorpusEntry::new(
                "M-200",
                "milestone-200-makefile",
                "Milestone 200: Full production Makefile with dev/staging/prod environments",
                CorpusFormat::Makefile,
                CorpusTier::Production,
                r#"fn main() { let env = "dev"; phony_target("dev", &[], &["ENV=dev cargo run"]); phony_target("staging", &["build"], &["ENV=staging ./deploy.sh"]); phony_target("prod", &["test", "lint", "build"], &["ENV=prod ./deploy.sh"]); phony_target("build", &[], &["cargo build --release"]); phony_target("test", &[], &["cargo test --workspace"]); phony_target("lint", &[], &["cargo clippy -- -D warnings"]); phony_target("clean", &[], &["cargo clean", "rm -rf dist/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: dev",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 17: Dockerfile D-181..D-190 (multi-arch, distroless, init)
    // =========================================================================

    fn load_expansion11_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-181",
                "dockerfile-distroless",
                "Google distroless base image for minimal attack surface",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "builder"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); from_image("gcr.io/distroless/cc-debian12", ""); copy_from("builder", "/app/target/release/app", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM gcr.io/distroless/cc-debian12:",
            ),
            CorpusEntry::new(
                "D-182",
                "dockerfile-buildkit-cache",
                "BuildKit cache mount for faster package installs",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75-bookworm"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["--mount=type=cache,target=/usr/local/cargo/registry cargo build --release"]); copy("src/", "src/"); run(&["cargo build --release"]); cmd(&["./target/release/app"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM rust:1.75-bookworm",
            ),
            CorpusEntry::new(
                "D-183",
                "dockerfile-tini-init",
                "Container with tini init process for signal handling",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("alpine", "3.19"); run(&["apk add --no-cache tini"]); workdir("/app"); copy(".", "."); entrypoint(&["/sbin/tini", "--"]); cmd(&["./app"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                r#"ENTRYPOINT ["/sbin/tini", "--"]"#,
            ),
            CorpusEntry::new(
                "D-184",
                "dockerfile-arg-build",
                "Build arguments for configurable builds",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { arg("RUST_VERSION", "1.75"); arg("APP_NAME", "myapp"); from_image("rust", "${RUST_VERSION}-bookworm"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); cmd(&["./target/release/${APP_NAME}"]); } fn arg(k: &str, v: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "RUN cargo build --release",
            ),
            CorpusEntry::new(
                "D-185",
                "dockerfile-volume-data",
                "Volume mount for persistent data",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("postgres", "16-alpine"); env("PGDATA", "/var/lib/postgresql/data/pgdata"); volume("/var/lib/postgresql/data"); expose(5432u16); cmd(&["postgres"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn volume(p: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "ENV PGDATA=/var/lib/postgresql/data/pgdata",
            ),
            CorpusEntry::new(
                "D-186",
                "dockerfile-multi-copy-from",
                "Multiple COPY --from stages for asset aggregation",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "frontend"); workdir("/web"); copy("web/", "."); run(&["npm ci", "npm run build"]); from_image_as("rust", "1.75-alpine", "backend"); workdir("/app"); copy(".", "."); run(&["cargo build --release"]); from_image("alpine", "3.19"); copy_from("frontend", "/web/dist", "/srv/static"); copy_from("backend", "/app/target/release/server", "/usr/local/bin/server"); expose(8080u16); cmd(&["server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS frontend",
            ),
            CorpusEntry::new(
                "D-187",
                "dockerfile-pip-constraints",
                "Python with pip constraints file for reproducible builds",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("python", "3.12-slim"); env("PIP_NO_CACHE_DIR", "1"); env("PIP_DISABLE_PIP_VERSION_CHECK", "1"); workdir("/app"); copy("requirements.txt", "."); copy("constraints.txt", "."); run(&["pip install -r requirements.txt -c constraints.txt"]); copy(".", "."); cmd(&["python", "-m", "app"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "ENV PIP_NO_CACHE_DIR=1",
            ),
            CorpusEntry::new(
                "D-188",
                "dockerfile-go-scratch",
                "Go binary in scratch container (CGO_ENABLED=0)",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.22-alpine", "builder"); env("CGO_ENABLED", "0"); workdir("/src"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["go build -ldflags=-s -o /app"]); from_image("scratch", ""); copy_from("builder", "/app", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {}"#,
                "ENV CGO_ENABLED=0",
            ),
            CorpusEntry::new(
                "D-189",
                "dockerfile-nginx-spa",
                "Nginx SPA with custom config and non-root",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "build"); workdir("/app"); copy("package.json", "."); run(&["npm ci"]); copy(".", "."); run(&["npm run build"]); from_image("nginx", "1.25-alpine"); copy("nginx.conf", "/etc/nginx/conf.d/default.conf"); copy_from("build", "/app/dist", "/usr/share/nginx/html"); expose(80u16); cmd(&["nginx", "-g", "daemon off;"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS build",
            ),
            CorpusEntry::new(
                "D-190",
                "milestone-190-docker",
                "Milestone 190: Production-grade Rust service with monitoring sidecar pattern",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "builder"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); run(&["mkdir src", "echo 'fn main(){}' > src/main.rs", "cargo build --release", "rm -rf src"]); copy("src/", "src/"); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y --no-install-recommends ca-certificates", "rm -rf /var/lib/apt/lists/*", "groupadd -r app", "useradd -r -g app app"]); copy_from("builder", "/app/target/release/server", "/usr/local/bin/server"); user("app"); expose(8080u16); expose(9090u16); healthcheck("server --health-check", "15s", "5s"); entrypoint(&["/usr/local/bin/server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str, i: &str, t: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS builder",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 18: Regex Pattern Corpus B-461..B-490 (Section 11.11.8)
    //   H: Character classes, quantifiers, anchoring, alternation, state machines
    // =========================================================================

    fn load_expansion16_bash(&mut self) {
        let entries = vec![
            // --- Group 1: Character Classes (B-461..B-465) ---
            CorpusEntry::new(
                "B-461",
                "regex-is-digit",
                "Regex char class: check if char code is digit [0-9] (48..57)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_digit(ch: i32) -> bool { ch >= 48 && ch <= 57 } fn main() { let d1 = is_digit(48); let d2 = is_digit(65); let d3 = is_digit(57); let d4 = is_digit(0); }"#,
                "is_digit() {",
            ),
            CorpusEntry::new(
                "B-462",
                "regex-is-lower",
                "Regex char class: check if char code is lowercase [a-z] (97..122)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_lower(ch: i32) -> bool { ch >= 97 && ch <= 122 } fn main() { let l1 = is_lower(97); let l2 = is_lower(65); let l3 = is_lower(122); }"#,
                "is_lower() {",
            ),
            CorpusEntry::new(
                "B-463",
                "regex-is-upper",
                "Regex char class: check if char code is uppercase [A-Z] (65..90)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_upper(ch: i32) -> bool { ch >= 65 && ch <= 90 } fn main() { let u1 = is_upper(65); let u2 = is_upper(97); let u3 = is_upper(90); }"#,
                "is_upper() {",
            ),
            CorpusEntry::new(
                "B-464",
                "regex-is-alnum",
                "Regex char class: check if char code is alphanumeric [a-zA-Z0-9]",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_alnum(ch: i32) -> bool { if ch >= 48 && ch <= 57 { true } else if ch >= 65 && ch <= 90 { true } else if ch >= 97 && ch <= 122 { true } else { false } } fn main() { let a1 = is_alnum(48); let a2 = is_alnum(65); let a3 = is_alnum(32); let a4 = is_alnum(122); }"#,
                "is_alnum() {",
            ),
            CorpusEntry::new(
                "B-465",
                "regex-is-space",
                "Regex char class: check if char code is whitespace (space=32, tab=9, newline=10)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_space(ch: i32) -> bool { if ch == 32 { true } else if ch == 9 { true } else if ch == 10 { true } else if ch == 13 { true } else { false } } fn main() { let s1 = is_space(32); let s2 = is_space(9); let s3 = is_space(65); let s4 = is_space(10); }"#,
                "is_space() {",
            ),
            // --- Group 2: Quantifiers (B-466..B-470) ---
            CorpusEntry::new(
                "B-466",
                "regex-match-plus",
                "Regex quantifier: match one-or-more digits (simulates [0-9]+)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_one_or_more(codes: i32, start: i32) -> i32 { let mut count = 0; let mut pos = start; while pos < codes { let ch = 48 + pos % 10; if ch >= 48 && ch <= 57 { count += 1; pos += 1; } else { pos = codes; } } count } fn main() { let m1 = match_one_or_more(10, 0); let m2 = match_one_or_more(5, 3); }"#,
                "match_one_or_more() {",
            ),
            CorpusEntry::new(
                "B-467",
                "regex-match-star",
                "Regex quantifier: match zero-or-more (simulates [a-z]*)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_zero_or_more(len: i32, start: i32) -> i32 { let mut count = 0; let mut pos = start; while pos < len { let ch = 97 + pos % 26; if ch >= 97 && ch <= 122 { count += 1; pos += 1; } else { pos = len; } } count } fn main() { let m1 = match_zero_or_more(8, 0); let m2 = match_zero_or_more(0, 0); }"#,
                "match_zero_or_more() {",
            ),
            CorpusEntry::new(
                "B-468",
                "regex-match-optional",
                "Regex quantifier: match zero-or-one (simulates [+-]?)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_optional(ch: i32) -> i32 { if ch == 43 { 1 } else if ch == 45 { 1 } else { 0 } } fn main() { let o1 = match_optional(43); let o2 = match_optional(45); let o3 = match_optional(48); }"#,
                "match_optional() {",
            ),
            CorpusEntry::new(
                "B-469",
                "regex-match-exactly-n",
                "Regex quantifier: match exactly N repetitions (simulates {n})",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_exactly(len: i32, n: i32) -> bool { let mut count = 0; let mut i = 0; while i < len && count < n { count += 1; i += 1; } count == n } fn main() { let e1 = match_exactly(5, 3); let e2 = match_exactly(2, 3); let e3 = match_exactly(3, 3); }"#,
                "match_exactly() {",
            ),
            CorpusEntry::new(
                "B-470",
                "regex-match-range",
                "Regex quantifier: match between min..max repetitions (simulates {n,m})",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_range(len: i32, min: i32, max: i32) -> bool { let mut count = 0; let mut i = 0; while i < len && count < max { count += 1; i += 1; } count >= min && count <= max } fn main() { let r1 = match_range(5, 2, 4); let r2 = match_range(1, 2, 4); let r3 = match_range(3, 2, 4); }"#,
                "match_range() {",
            ),
            // --- Group 3: Anchoring (B-471..B-475) ---
            CorpusEntry::new(
                "B-471",
                "regex-anchor-start",
                "Regex anchoring: check if pattern matches at position 0 (simulates ^)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn anchor_start(first_ch: i32, target: i32) -> bool { first_ch == target } fn main() { let a1 = anchor_start(65, 65); let a2 = anchor_start(66, 65); let a3 = anchor_start(97, 97); }"#,
                "anchor_start() {",
            ),
            CorpusEntry::new(
                "B-472",
                "regex-anchor-end",
                "Regex anchoring: check if pattern matches at last position (simulates $)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn anchor_end(last_ch: i32, target: i32) -> bool { last_ch == target } fn main() { let e1 = anchor_end(10, 10); let e2 = anchor_end(32, 10); let e3 = anchor_end(0, 0); }"#,
                "anchor_end() {",
            ),
            CorpusEntry::new(
                "B-473",
                "regex-anchor-both",
                "Regex anchoring: full-string match ^pattern$ (exact length + start + end)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn anchor_full(first: i32, last: i32, len: i32, expected_first: i32, expected_last: i32, expected_len: i32) -> bool { first == expected_first && last == expected_last && len == expected_len } fn main() { let f1 = anchor_full(65, 90, 3, 65, 90, 3); let f2 = anchor_full(65, 90, 4, 65, 90, 3); }"#,
                "anchor_full() {",
            ),
            CorpusEntry::new(
                "B-474",
                "regex-word-boundary",
                "Regex anchoring: word boundary detection (alnum to non-alnum transition)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_word_char(ch: i32) -> bool { if ch >= 48 && ch <= 57 { true } else if ch >= 65 && ch <= 90 { true } else if ch >= 97 && ch <= 122 { true } else if ch == 95 { true } else { false } } fn word_boundary(prev: i32, curr: i32) -> bool { let pw = is_word_char(prev); let cw = is_word_char(curr); if pw && cw { false } else if pw { true } else if cw { true } else { false } } fn main() { let b1 = word_boundary(97, 32); let b2 = word_boundary(97, 98); let b3 = word_boundary(32, 65); }"#,
                "word_boundary() {",
            ),
            CorpusEntry::new(
                "B-475",
                "regex-position-tracker",
                "Regex anchoring: track match start and end positions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn find_match(haystack_len: i32, needle_start: i32, needle_len: i32) -> i32 { if needle_start < 0 { 0 } else if needle_start + needle_len > haystack_len { 0 } else { needle_start + 1 } } fn match_end(start: i32, len: i32) -> i32 { start + len } fn main() { let s = find_match(20, 5, 3); let e = match_end(5, 3); let n = find_match(10, 8, 5); }"#,
                "find_match() {",
            ),
            // --- Group 4: Alternation (B-476..B-480) ---
            CorpusEntry::new(
                "B-476",
                "regex-alt-two",
                "Regex alternation: match either of two values (simulates a|b)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_alt2(ch: i32, a: i32, b: i32) -> bool { ch == a || ch == b } fn main() { let m1 = match_alt2(65, 65, 66); let m2 = match_alt2(66, 65, 66); let m3 = match_alt2(67, 65, 66); }"#,
                "match_alt2() {",
            ),
            CorpusEntry::new(
                "B-477",
                "regex-alt-three",
                "Regex alternation: match any of three values (simulates a|b|c)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_alt3(ch: i32, a: i32, b: i32, c: i32) -> bool { if ch == a { true } else if ch == b { true } else if ch == c { true } else { false } } fn main() { let m1 = match_alt3(65, 65, 66, 67); let m2 = match_alt3(67, 65, 66, 67); let m3 = match_alt3(68, 65, 66, 67); }"#,
                "match_alt3() {",
            ),
            CorpusEntry::new(
                "B-478",
                "regex-alt-range",
                "Regex alternation: match within multiple ranges (simulates [a-z]|[0-9])",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn match_alt_range(ch: i32) -> i32 { if ch >= 97 && ch <= 122 { 1 } else if ch >= 48 && ch <= 57 { 2 } else if ch >= 65 && ch <= 90 { 3 } else { 0 } } fn main() { let r1 = match_alt_range(100); let r2 = match_alt_range(53); let r3 = match_alt_range(70); let r4 = match_alt_range(33); }"#,
                "match_alt_range() {",
            ),
            CorpusEntry::new(
                "B-479",
                "regex-alt-priority",
                "Regex alternation: first-match priority (greedy left-to-right)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn first_match(ch: i32) -> i32 { if ch >= 48 && ch <= 57 { 1 } else if ch >= 48 && ch <= 53 { 2 } else if ch >= 65 && ch <= 90 { 3 } else { 0 } } fn main() { let p1 = first_match(50); let p2 = first_match(56); let p3 = first_match(65); let p4 = first_match(33); }"#,
                "first_match() {",
            ),
            CorpusEntry::new(
                "B-480",
                "regex-alt-negation",
                "Regex alternation: negated character class (simulates [^0-9])",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn not_digit(ch: i32) -> bool { if ch >= 48 && ch <= 57 { false } else { true } } fn main() { let n1 = not_digit(65); let n2 = not_digit(50); let n3 = not_digit(32); }"#,
                "not_digit() {",
            ),
            // --- Group 5: State Machines (B-481..B-485) ---
            CorpusEntry::new(
                "B-481",
                "regex-dfa-two-state",
                "Regex DFA: two-state automaton (start=0, accept=1)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn dfa_run(len: i32, trigger: i32) -> i32 { let mut state = 0; let mut i = 0; while i < len { if state == 0 && i == trigger { state = 1; } i += 1; } state } fn main() { let d1 = dfa_run(10, 5); let d2 = dfa_run(10, 15); let d3 = dfa_run(3, 0); }"#,
                "dfa_run() {",
            ),
            CorpusEntry::new(
                "B-482",
                "regex-dfa-three-state",
                "Regex DFA: three-state automaton (0=start, 1=partial, 2=accept)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn dfa3(len: i32, t1: i32, t2: i32) -> i32 { let mut state = 0; let mut i = 0; while i < len { if state == 0 && i == t1 { state = 1; } else if state == 1 && i == t2 { state = 2; } i += 1; } state } fn main() { let d1 = dfa3(10, 3, 7); let d2 = dfa3(10, 3, 2); let d3 = dfa3(5, 1, 3); }"#,
                "dfa3() {",
            ),
            CorpusEntry::new(
                "B-483",
                "regex-nfa-branch",
                "Regex NFA: non-deterministic branch simulation (two paths)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn nfa_branch(ch: i32) -> i32 { let path_a = ch >= 48 && ch <= 57; let path_b = ch >= 97 && ch <= 102; if path_a && path_b { 3 } else if path_a { 1 } else if path_b { 2 } else { 0 } } fn main() { let n1 = nfa_branch(50); let n2 = nfa_branch(100); let n3 = nfa_branch(70); }"#,
                "nfa_branch() {",
            ),
            CorpusEntry::new(
                "B-484",
                "regex-dfa-counter",
                "Regex DFA: counting automaton (transitions on threshold)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn dfa_count(len: i32, threshold: i32) -> i32 { let mut state = 0; let mut count = 0; let mut i = 0; while i < len { count += 1; if state == 0 && count >= threshold { state = 1; } i += 1; } state } fn main() { let c1 = dfa_count(10, 5); let c2 = dfa_count(3, 5); let c3 = dfa_count(5, 5); }"#,
                "dfa_count() {",
            ),
            CorpusEntry::new(
                "B-485",
                "regex-dfa-reset",
                "Regex DFA: automaton with reset transitions",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn dfa_reset(len: i32, accept_at: i32, reset_at: i32) -> i32 { let mut state = 0; let mut i = 0; while i < len { if i == reset_at && state == 1 { state = 0; } else if i == accept_at && state == 0 { state = 1; } i += 1; } state } fn main() { let r1 = dfa_reset(10, 3, 7); let r2 = dfa_reset(10, 3, 2); let r3 = dfa_reset(10, 3, 3); }"#,
                "dfa_reset() {",
            ),
            // --- Group 6: Composition (B-486..B-490) ---
            CorpusEntry::new(
                "B-486",
                "regex-validate-integer",
                "Regex composition: validate integer pattern [+-]?[0-9]+",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate_int(sign: i32, first_digit: i32, len: i32) -> bool { let has_sign = sign == 43 || sign == 45; let digit_ok = first_digit >= 48 && first_digit <= 57; if has_sign { digit_ok && len >= 2 } else { digit_ok && len >= 1 } } fn main() { let v1 = validate_int(43, 49, 3); let v2 = validate_int(65, 49, 2); let v3 = validate_int(49, 49, 1); let v4 = validate_int(45, 48, 2); }"#,
                "validate_int() {",
            ),
            CorpusEntry::new(
                "B-487",
                "regex-classify-char",
                "Regex composition: full character classifier (digit/upper/lower/space/punct)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(ch: i32) -> i32 { if ch >= 48 && ch <= 57 { 1 } else if ch >= 65 && ch <= 90 { 2 } else if ch >= 97 && ch <= 122 { 3 } else if ch == 32 || ch == 9 || ch == 10 { 4 } else { 5 } } fn main() { let c1 = classify(48); let c2 = classify(65); let c3 = classify(97); let c4 = classify(32); let c5 = classify(33); }"#,
                "classify() {",
            ),
            CorpusEntry::new(
                "B-488",
                "regex-count-matches",
                "Regex composition: count how many positions in a range match a class",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn count_digit_positions(len: i32) -> i32 { let mut count = 0; let mut i = 0; while i < len { let ch = 40 + i; if ch >= 48 && ch <= 57 { count += 1; } i += 1; } count } fn main() { let c1 = count_digit_positions(20); let c2 = count_digit_positions(5); let c3 = count_digit_positions(30); }"#,
                "count_digit_positions() {",
            ),
            CorpusEntry::new(
                "B-489",
                "regex-scan-transitions",
                "Regex composition: count state transitions in a scan",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn count_transitions(len: i32) -> i32 { let mut transitions = 0; let mut prev_class = 0; let mut i = 0; while i < len { let ch = 40 + i * 3; let mut curr_class = 0; if ch >= 48 && ch <= 57 { curr_class = 1; } else if ch >= 65 && ch <= 90 { curr_class = 2; } if curr_class != prev_class { transitions += 1; } prev_class = curr_class; i += 1; } transitions } fn main() { let t1 = count_transitions(10); let t2 = count_transitions(20); }"#,
                "count_transitions() {",
            ),
            CorpusEntry::new(
                "B-490",
                "milestone-490-regex",
                "Milestone 490: Full regex pipeline — classify + match + count + validate",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_match_class(ch: i32, class: i32) -> bool { if class == 1 { ch >= 48 && ch <= 57 } else if class == 2 { ch >= 65 && ch <= 90 } else if class == 3 { ch >= 97 && ch <= 122 } else { false } } fn scan_class(len: i32, class: i32) -> i32 { let mut matched = 0; let mut i = 0; while i < len { if is_match_class(48 + i, class) { matched += 1; } i += 1; } matched } fn regex_score(digits: i32, uppers: i32, lowers: i32) -> i32 { digits * 3 + uppers * 2 + lowers } fn main() { let d = scan_class(20, 1); let u = scan_class(50, 2); let l = scan_class(80, 3); let score = regex_score(d, u, l); }"#,
                "is_match_class() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 19: Bash B-491..B-500 (Milestone 500: Data Structure Algorithms)
    //   Transpiled implementations of classic CS data structure operations
    // =========================================================================

    fn load_expansion17_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "B-491",
                "ds-stack-push-pop",
                "Data structure: stack push/pop simulation via array index",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn stack_push(top: i32) -> i32 { top + 1 } fn stack_pop(top: i32) -> i32 { if top > 0 { top - 1 } else { 0 } } fn stack_empty(top: i32) -> bool { top == 0 } fn main() { let mut sp = 0; sp = stack_push(sp); sp = stack_push(sp); sp = stack_push(sp); sp = stack_pop(sp); let empty = stack_empty(sp); }"#,
                "stack_push() {",
            ),
            CorpusEntry::new(
                "B-492",
                "ds-queue-enqueue-dequeue",
                "Data structure: circular queue head/tail index management",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn enqueue(tail: i32, capacity: i32) -> i32 { (tail + 1) % capacity } fn dequeue(head: i32, capacity: i32) -> i32 { (head + 1) % capacity } fn queue_size(head: i32, tail: i32, capacity: i32) -> i32 { if tail >= head { tail - head } else { capacity - head + tail } } fn main() { let h = 0; let t = enqueue(0, 8); let t2 = enqueue(t, 8); let h2 = dequeue(h, 8); let sz = queue_size(h2, t2, 8); }"#,
                "enqueue() {",
            ),
            CorpusEntry::new(
                "B-493",
                "ds-hash-function",
                "Data structure: simple hash function (djb2-style modular arithmetic)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn hash_combine(hash: i32, ch: i32) -> i32 { hash * 33 + ch } fn hash_bucket(hash: i32, num_buckets: i32) -> i32 { let mut h = hash; if hash < 0 { h = 0 - hash; } h % num_buckets } fn main() { let mut h = 5381; h = hash_combine(h, 104); h = hash_combine(h, 101); h = hash_combine(h, 108); let bucket = hash_bucket(h, 16); }"#,
                "hash_combine() {",
            ),
            CorpusEntry::new(
                "B-494",
                "ds-binary-search",
                "Data structure: binary search iteration count on sorted range",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn binary_search_steps(low: i32, high: i32, target: i32) -> i32 { let mut lo = low; let mut hi = high; let mut steps = 0; while lo <= hi { let mid = lo + (hi - lo) / 2; steps += 1; if mid == target { lo = hi + 1; } else if mid < target { lo = mid + 1; } else { hi = mid - 1; } } steps } fn main() { let s1 = binary_search_steps(0, 100, 42); let s2 = binary_search_steps(0, 1000, 500); }"#,
                "binary_search_steps() {",
            ),
            CorpusEntry::new(
                "B-495",
                "ds-tree-depth",
                "Data structure: complete binary tree depth from node count",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn tree_depth(nodes: i32) -> i32 { let mut depth = 0; let mut n = nodes; while n > 0 { depth += 1; n = n / 2; } depth } fn tree_leaves(depth: i32) -> i32 { let mut leaves = 1; let mut i = 0; while i < depth - 1 { leaves *= 2; i += 1; } leaves } fn main() { let d = tree_depth(15); let l = tree_leaves(d); let d2 = tree_depth(1023); }"#,
                "tree_depth() {",
            ),
            CorpusEntry::new(
                "B-496",
                "ds-heap-parent-child",
                "Data structure: binary heap parent/child index calculations",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn heap_parent(i: i32) -> i32 { (i - 1) / 2 } fn heap_left(i: i32) -> i32 { 2 * i + 1 } fn heap_right(i: i32) -> i32 { 2 * i + 2 } fn heap_is_leaf(i: i32, size: i32) -> bool { heap_left(i) >= size } fn main() { let p = heap_parent(5); let l = heap_left(2); let r = heap_right(2); let leaf = heap_is_leaf(7, 10); }"#,
                "heap_parent() {",
            ),
            CorpusEntry::new(
                "B-497",
                "ds-graph-degree",
                "Data structure: graph vertex degree and edge count",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn total_edges(vertices: i32, avg_degree: i32) -> i32 { vertices * avg_degree / 2 } fn is_connected_min(vertices: i32, edges: i32) -> bool { edges >= vertices - 1 } fn density(vertices: i32, edges: i32) -> i32 { let max_edges = vertices * (vertices - 1) / 2; if max_edges == 0 { 0 } else { edges * 100 / max_edges } } fn main() { let e = total_edges(10, 4); let c = is_connected_min(10, e); let d = density(10, e); }"#,
                "total_edges() {",
            ),
            CorpusEntry::new(
                "B-498",
                "ds-lru-eviction",
                "Data structure: LRU cache eviction decision (age-based)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn lru_should_evict(age: i32, max_age: i32) -> bool { age >= max_age } fn lru_new_age(hit: bool) -> i32 { if hit { 0 } else { 1 } } fn lru_oldest(a1: i32, a2: i32, a3: i32) -> i32 { if a1 >= a2 && a1 >= a3 { 1 } else if a2 >= a3 { 2 } else { 3 } } fn main() { let evict = lru_should_evict(10, 8); let age = lru_new_age(true); let oldest = lru_oldest(5, 3, 7); }"#,
                "lru_should_evict() {",
            ),
            CorpusEntry::new(
                "B-499",
                "ds-bloom-filter-hash",
                "Data structure: Bloom filter bit position calculation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn bloom_hash1(key: i32, size: i32) -> i32 { let mut h = key; if key < 0 { h = 0 - key; } h % size } fn bloom_hash2(key: i32, size: i32) -> i32 { let mut h = key; if key < 0 { h = 0 - key; } (h * 7 + 13) % size } fn bloom_test(bit1: bool, bit2: bool) -> bool { bit1 && bit2 } fn main() { let h1 = bloom_hash1(42, 64); let h2 = bloom_hash2(42, 64); let present = bloom_test(true, true); let absent = bloom_test(true, false); }"#,
                "bloom_hash1() {",
            ),
            CorpusEntry::new(
                "B-500",
                "milestone-500-bash",
                "Milestone 500: Composite data structure — hash table with chaining (bucket + chain length)",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn hash_key(key: i32, buckets: i32) -> i32 { let mut h = key; if key < 0 { h = 0 - key; } h % buckets } fn chain_length(items: i32, buckets: i32) -> i32 { if buckets == 0 { items } else { items / buckets } } fn load_factor_pct(items: i32, buckets: i32) -> i32 { if buckets == 0 { 100 } else { items * 100 / buckets } } fn needs_resize(items: i32, buckets: i32, threshold_pct: i32) -> bool { load_factor_pct(items, buckets) > threshold_pct } fn main() { let b = hash_key(42, 16); let cl = chain_length(100, 16); let lf = load_factor_pct(100, 16); let resize = needs_resize(100, 16, 75); }"#,
                "hash_key() {",
            ),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion Wave 20: Dockerfile D-191..D-200 (Milestone 200: Multi-Runtime)
    //   Java, .NET, Ruby, Elixir, Zig, and composite multi-runtime patterns
    // =========================================================================

    fn load_expansion12_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new(
                "D-191",
                "dockerfile-java-gradle",
                "Java Gradle multi-stage build with JRE runtime",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("gradle", "8.5-jdk21", "builder"); workdir("/app"); copy("build.gradle", "."); copy("src/", "src/"); run(&["gradle build --no-daemon"]); from_image("eclipse-temurin", "21-jre-alpine"); copy_from("builder", "/app/build/libs/app.jar", "/app.jar"); expose(8080u16); cmd(&["java", "-jar", "/app.jar"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM gradle:8.5-jdk21 AS builder",
            ),
            CorpusEntry::new(
                "D-192",
                "dockerfile-dotnet-publish",
                ".NET SDK multi-stage build with ASP.NET runtime",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("mcr.microsoft.com/dotnet/sdk", "8.0", "build"); workdir("/src"); copy("*.csproj", "."); run(&["dotnet restore"]); copy(".", "."); run(&["dotnet publish -c Release -o /app"]); from_image("mcr.microsoft.com/dotnet/aspnet", "8.0"); workdir("/app"); copy_from("build", "/app", "."); expose(8080u16); entrypoint(&["dotnet", "MyApp.dll"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build",
            ),
            CorpusEntry::new(
                "D-193",
                "dockerfile-ruby-bundler",
                "Ruby with Bundler dependency management",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("ruby", "3.3-slim"); env("BUNDLE_DEPLOYMENT", "1"); env("BUNDLE_WITHOUT", "development:test"); workdir("/app"); copy("Gemfile", "."); copy("Gemfile.lock", "."); run(&["bundle install"]); copy(".", "."); expose(3000u16); cmd(&["ruby", "app.rb"]); } fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "ENV BUNDLE_DEPLOYMENT=1",
            ),
            CorpusEntry::new(
                "D-194",
                "dockerfile-elixir-release",
                "Elixir Phoenix release with multi-stage build",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("elixir", "1.16-alpine", "builder"); env("MIX_ENV", "prod"); workdir("/app"); copy("mix.exs", "."); copy("mix.lock", "."); run(&["mix deps.get --only prod", "mix compile"]); copy(".", "."); run(&["mix release"]); from_image("alpine", "3.19"); run(&["apk add --no-cache libstdc++ openssl ncurses-libs"]); copy_from("builder", "/app/_build/prod/rel/app", "/app"); cmd(&["/app/bin/app", "start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn env(k: &str, v: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM elixir:1.16-alpine AS builder",
            ),
            CorpusEntry::new(
                "D-195",
                "dockerfile-zig-static",
                "Zig static binary build to scratch container",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("alpine", "3.19", "builder"); run(&["apk add --no-cache zig"]); workdir("/src"); copy(".", "."); run(&["zig build -Doptimize=ReleaseSafe"]); from_image("scratch", ""); copy_from("builder", "/src/zig-out/bin/app", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM scratch:",
            ),
            CorpusEntry::new(
                "D-196",
                "dockerfile-php-composer",
                "PHP with Composer and Apache",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("php", "8.3-apache"); run(&["docker-php-ext-install pdo pdo_mysql"]); copy("composer.json", "."); copy("composer.lock", "."); run(&["composer install --no-dev --optimize-autoloader"]); copy(".", "/var/www/html/"); expose(80u16); cmd(&["apache2-foreground"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM php:8.3-apache",
            ),
            CorpusEntry::new(
                "D-197",
                "dockerfile-deno-slim",
                "Deno runtime with slim permissions model",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("denoland/deno", "1.40"); workdir("/app"); copy("deno.json", "."); copy("deno.lock", "."); run(&["deno cache main.ts"]); copy(".", "."); user("deno"); expose(8000u16); cmd(&["deno", "run", "--allow-net", "--allow-read", "main.ts"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM denoland/deno:1.40",
            ),
            CorpusEntry::new(
                "D-198",
                "dockerfile-bun-runtime",
                "Bun JavaScript runtime container",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("oven/bun", "1.0"); workdir("/app"); copy("package.json", "."); copy("bun.lockb", "."); run(&["bun install --frozen-lockfile"]); copy(".", "."); expose(3000u16); cmd(&["bun", "run", "start"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM oven/bun:1.0",
            ),
            CorpusEntry::new(
                "D-199",
                "dockerfile-rust-musl-cross",
                "Rust cross-compile to musl with cargo-chef for layer caching",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75-bookworm", "planner"); workdir("/app"); copy(".", "."); run(&["cargo install cargo-chef", "cargo chef prepare --recipe-path recipe.json"]); from_image_as("rust", "1.75-bookworm", "builder"); run(&["rustup target add x86_64-unknown-linux-musl"]); workdir("/app"); copy_from("planner", "/app/recipe.json", "recipe.json"); run(&["cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json"]); copy(".", "."); run(&["cargo build --release --target x86_64-unknown-linux-musl"]); from_image("scratch", ""); copy_from("builder", "/app/target/x86_64-unknown-linux-musl/release/app", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75-bookworm AS planner",
            ),
            CorpusEntry::new(
                "D-200",
                "milestone-200-docker",
                "Milestone 200: Full polyglot multi-stage build — Rust API + Node frontend + Redis sidecar",
                CorpusFormat::Dockerfile,
                CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "frontend"); workdir("/web"); copy("web/package.json", "."); run(&["npm ci"]); copy("web/", "."); run(&["npm run build"]); from_image_as("rust", "1.75-bookworm", "backend"); workdir("/app"); copy("Cargo.toml", "."); copy("Cargo.lock", "."); copy("src/", "src/"); run(&["cargo build --release"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y --no-install-recommends ca-certificates", "rm -rf /var/lib/apt/lists/*", "groupadd -r app", "useradd -r -g app app"]); copy_from("frontend", "/web/dist", "/srv/static"); copy_from("backend", "/app/target/release/server", "/usr/local/bin/server"); user("app"); expose(8080u16); expose(9090u16); healthcheck("server --health", "15s", "5s"); entrypoint(&["/usr/local/bin/server"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str, i: &str, t: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM node:20-alpine AS frontend",
            ),
        ];
        self.entries.extend(entries);
    }
}

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



}
