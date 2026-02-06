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
                "x=42",
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
                "name=",
            ),
            CorpusEntry::new(
                "B-006",
                "function-call",
                "Simple function call",
                CorpusFormat::Bash,
                CorpusTier::Trivial,
                r#"fn main() { greet("World"); } fn greet(name: &str) {}"#,
                "greet",
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
                "x=",
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
                "exit 1",
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
                "ENTRYPOINT",
            ),
            CorpusEntry::new(
                "D-008",
                "label",
                "LABEL instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Trivial,
                r#"fn main() { from_image("alpine", "3.18"); label("maintainer", "team@example.com"); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {}"#,
                "LABEL maintainer=",
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
                "if [",
            ),
            CorpusEntry::new(
                "B-012",
                "for-loop-range",
                "For loop with integer range",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { for i in 0..5 { let x = i; } }",
                "for ",
            ),
            CorpusEntry::new(
                "B-013",
                "binary-ops",
                "Multiple binary operations",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let sum = 10 + 20; let product = 3 * 4; }",
                "sum=",
            ),
            CorpusEntry::new(
                "B-014",
                "nested-calls",
                "Nested function calls",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = "hello"; greet(x); } fn greet(name: &str) {} "#,
                "greet",
            ),
            CorpusEntry::new(
                "B-015",
                "negation",
                "Boolean negation",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let flag = !false; }",
                "flag=",
            ),
            // Harder entries - potential falsifiers
            CorpusEntry::new(
                "B-016",
                "while-loop",
                "While loop with condition",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let mut x = 0; while x < 10 { x = x + 1; } }",
                "while",
            ),
            CorpusEntry::new(
                "B-017",
                "match-statement",
                "Match/case statement",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                r#"fn main() { let x = 1; match x { 1 => { let a = "one"; }, 2 => { let b = "two"; }, _ => { let c = "other"; } } }"#,
                "case",
            ),
            CorpusEntry::new(
                "B-018",
                "negative-integer",
                "Negative integer literal",
                CorpusFormat::Bash,
                CorpusTier::Standard,
                "fn main() { let x = -42; }",
                "x=",
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
                "RUN apt-get update",
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
                "COPY --from=builder",
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
                "CMD",
            ),
            // Harder entries
            CorpusEntry::new(
                "D-016",
                "healthcheck",
                "HEALTHCHECK instruction",
                CorpusFormat::Dockerfile,
                CorpusTier::Standard,
                r#"fn main() { from_image("nginx", "1.25"); healthcheck("curl -f http://localhost/"); } fn from_image(i: &str, t: &str) {} fn healthcheck(c: &str) {}"#,
                "HEALTHCHECK CMD",
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
                "elif",
            ),
            CorpusEntry::new(
                "B-022",
                "inclusive-range",
                "For loop with inclusive range (0..=5)",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { for i in 0..=5 { let x = i; } }",
                "for ",
            ),
            CorpusEntry::new(
                "B-023",
                "println-macro",
                "println! macro transpilation",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { println!("hello world"); }"#,
                "echo",
            ),
            CorpusEntry::new(
                "B-024",
                "function-with-params",
                "Function with typed parameters",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { greet("World", 3); } fn greet(name: &str, count: u32) {}"#,
                "greet",
            ),
            CorpusEntry::new(
                "B-025",
                "nested-if-in-loop",
                "If inside for loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { for i in 0..10 { if i > 5 { let big = true; } } }",
                "for ",
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
                "remainder=",
            ),
            CorpusEntry::new(
                "B-029",
                "string-empty",
                "Empty string variable",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                r#"fn main() { let empty = ""; }"#,
                "empty=",
            ),
            CorpusEntry::new(
                "B-030",
                "multiple-assignments-in-loop",
                "Multiple assignments inside while loop",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let mut i = 0; let mut sum = 0; while i < 5 { sum = sum + i; i = i + 1; } }",
                "while",
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
                "quotient=",
            ),
            CorpusEntry::new(
                "B-035",
                "large-integer",
                "Large integer value",
                CorpusFormat::Bash,
                CorpusTier::Complex,
                "fn main() { let big = 4294967295; }",
                "big=",
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
                "ENTRYPOINT",
            ),
            CorpusEntry::new(
                "D-023",
                "label-multiple",
                "Multiple LABEL instructions",
                CorpusFormat::Dockerfile,
                CorpusTier::Complex,
                r#"fn main() { from_image("alpine", "3.18"); label("maintainer", "team@example.com"); label("version", "1.0"); label("description", "My application"); } fn from_image(i: &str, t: &str) {} fn label(k: &str, v: &str) {}"#,
                "LABEL maintainer=",
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
                "count=",
            ),
            CorpusEntry::new(
                "B-037",
                "compound-sub-assign",
                "Compound subtraction assignment (x -= 1)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut total = 100; total -= 25; }"#,
                "total=",
            ),
            CorpusEntry::new(
                "B-038",
                "compound-mul-assign",
                "Compound multiplication assignment (x *= 2)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut factor = 3; factor *= 4; }"#,
                "factor=",
            ),
            // --- eprintln! macro ---
            CorpusEntry::new(
                "B-039",
                "eprintln-macro",
                "eprintln! macro support (stderr output)",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { eprintln!("error: something went wrong"); }"#,
                ">&2",
            ),
            // --- Nested function calls ---
            CorpusEntry::new(
                "B-040",
                "nested-function-calls",
                "Function call as argument to another function",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn add(a: u32, b: u32) -> u32 { a + b } fn double(x: u32) -> u32 { x * 2 } fn main() { let result = double(add(3, 4)); }"#,
                "result=",
            ),
            // --- Multiple return paths ---
            CorpusEntry::new(
                "B-041",
                "early-return-conditional",
                "Early return based on condition",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn check(x: u32) -> u32 { if x > 10 { return 1; } return 0; } fn main() { let r = check(5); }"#,
                "check()",
            ),
            // --- Empty function body ---
            CorpusEntry::new(
                "B-042",
                "empty-function",
                "Function with empty body",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn noop() {} fn main() { noop(); }"#,
                "noop()",
            ),
            // --- Large literal values ---
            CorpusEntry::new(
                "B-043",
                "max-u32-literal",
                "Maximum u32 literal value",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let max = 4294967295; }"#,
                "max=4294967295",
            ),
            // --- String with special characters (safe) ---
            CorpusEntry::new(
                "B-044",
                "special-safe-chars",
                "String containing characters that need quoting in shell",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let msg = "hello world: it's a test!"; let path = "/usr/local/bin"; }"#,
                "msg=",
            ),
            // --- Deeply nested if-else ---
            CorpusEntry::new(
                "B-045",
                "deeply-nested-if",
                "Three-level nested if-else",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 5; if x > 0 { if x > 3 { if x > 4 { let r = 1; } } } }"#,
                "if",
            ),
            // --- While with complex condition ---
            CorpusEntry::new(
                "B-046",
                "while-complex-condition",
                "While loop with compound boolean condition",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let mut x = 0; let mut y = 10; while x < 5 && y > 0 { x = x + 1; y = y - 1; } }"#,
                "while",
            ),
            // --- Multiple functions calling each other ---
            CorpusEntry::new(
                "B-047",
                "multi-function-chain",
                "Three functions calling each other in sequence",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn step1() -> u32 { 1 } fn step2(x: u32) -> u32 { x + 2 } fn step3(x: u32) -> u32 { x * 3 } fn main() { let r = step3(step2(step1())); }"#,
                "step1()",
            ),
            // --- Boolean parameters ---
            CorpusEntry::new(
                "B-048",
                "boolean-params",
                "Function with boolean parameters",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn process(verbose: bool, dry_run: bool) { if verbose { println!("processing"); } } fn main() { process(true, false); }"#,
                "process()",
            ),
            // --- Match with many arms ---
            CorpusEntry::new(
                "B-049",
                "match-many-arms",
                "Match expression with 6+ arms",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 3; match x { 0 => { println!("zero"); } 1 => { println!("one"); } 2 => { println!("two"); } 3 => { println!("three"); } 4 => { println!("four"); } _ => { println!("other"); } } }"#,
                "case",
            ),
            // --- Negation operator ---
            CorpusEntry::new(
                "B-050",
                "negation-unary",
                "Unary negation on integer",
                CorpusFormat::Bash,
                CorpusTier::Adversarial,
                r#"fn main() { let x = 5; let y = !true; }"#,
                "y=",
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
                "CFLAGS :=",
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
                ".PHONY:",
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
                "HEALTHCHECK",
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
                "# Build stage",
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
                "validate()",
            ),
            // --- Iterative computation ---
            CorpusEntry::new(
                "B-052",
                "iterative-sum",
                "Iterative sum computation with accumulator",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut sum = 0; for i in 0..10 { sum += i; } println!("total"); }"#,
                "sum=",
            ),
            // --- Nested loop with break ---
            CorpusEntry::new(
                "B-053",
                "nested-loop-break",
                "Nested for loop with conditional break",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut found = false; for i in 0..100 { if i == 42 { found = true; break; } } }"#,
                "found=",
            ),
            // --- While with decrement ---
            CorpusEntry::new(
                "B-054",
                "countdown-while",
                "Countdown loop with decrement",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut count = 10; while count > 0 { count -= 1; } }"#,
                "while",
            ),
            // --- Match with return values ---
            CorpusEntry::new(
                "B-055",
                "match-return-value",
                "Function using match to return different values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn classify(score: u32) -> u32 { match score { 0 => { return 0; } 1 => { return 1; } 2 => { return 2; } _ => { return 3; } } } fn main() { let grade = classify(85); }"#,
                "classify()",
            ),
            // --- Complex boolean logic ---
            CorpusEntry::new(
                "B-056",
                "complex-boolean",
                "Complex boolean expression with AND/OR",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn check(a: u32, b: u32, c: u32) -> bool { (a > 0 && b > 0) || c == 0 } fn main() { let ok = check(1, 2, 3); }"#,
                "check()",
            ),
            // --- Multiple string variables ---
            CorpusEntry::new(
                "B-057",
                "string-variables",
                "Multiple string variable assignments",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let name = "bashrs"; let version = "6.59.0"; let author = "paiml"; let license = "MIT"; println!("ready"); }"#,
                "version=",
            ),
            // --- Function with many parameters ---
            CorpusEntry::new(
                "B-058",
                "many-params-func",
                "Function with 4 parameters",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn build(src: &str, out: &str, opt: &str, target: &str) { println!("building"); } fn main() { build("src", "build", "-O2", "x86_64"); }"#,
                "build()",
            ),
            // --- Fibonacci-like iteration ---
            CorpusEntry::new(
                "B-059",
                "fibonacci-loop",
                "Fibonacci-style iterative computation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut a = 0; let mut b = 1; for _i in 0..10 { let temp = b; b = a + b; a = temp; } }"#,
                "temp=",
            ),
            // --- Error handling pattern ---
            CorpusEntry::new(
                "B-060",
                "error-pattern",
                "Early return error handling pattern",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn validate_port(port: u32) -> u32 { if port < 1 { return 0; } if port > 65535 { return 0; } return 1; } fn main() { let ok = validate_port(8080); }"#,
                "validate_port()",
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
                "result=",
            ),
            // --- Multiple if-else branches ---
            CorpusEntry::new(
                "B-063",
                "multi-branch-if",
                "Function with multiple if-else branches",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn categorize(x: u32) -> u32 { if x < 10 { return 1; } else if x < 100 { return 2; } else if x < 1000 { return 3; } else { return 4; } } fn main() { let cat = categorize(500); }"#,
                "categorize()",
            ),
            // --- Accumulate with multiply ---
            CorpusEntry::new(
                "B-064",
                "factorial-like",
                "Factorial-style multiplication accumulation",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut result = 1; for i in 1..6 { result *= i; } }"#,
                "result=",
            ),
            // --- Boolean flag pattern ---
            CorpusEntry::new(
                "B-065",
                "boolean-flag-loop",
                "Boolean flag set inside loop",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut has_error = false; for i in 0..5 { if i == 3 { has_error = true; } } if has_error { println!("error found"); } }"#,
                "has_error=",
            ),
            // --- Stderr output ---
            CorpusEntry::new(
                "B-066",
                "eprintln-stderr",
                "Program using eprintln for error messages",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let verbose = true; if verbose { println!("starting"); } eprintln!("warning: debug mode"); }"#,
                ">&2",
            ),
            // --- Multiple match with default ---
            CorpusEntry::new(
                "B-067",
                "match-string-return",
                "Match on integer returning different string values",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn status_msg(code: u32) -> &str { match code { 0 => { return "ok"; } 1 => { return "warning"; } 2 => { return "error"; } _ => { return "unknown"; } } } fn main() { let msg = status_msg(0); }"#,
                "status_msg()",
            ),
            // --- Power of two check ---
            CorpusEntry::new(
                "B-068",
                "bitwise-power-check",
                "Check if number is within power-of-2 ranges",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn is_small(n: u32) -> bool { n < 256 } fn is_medium(n: u32) -> bool { n >= 256 && n < 65536 } fn main() { let s = is_small(100); let m = is_medium(1000); }"#,
                "is_small()",
            ),
            // --- Zero-argument function calls ---
            CorpusEntry::new(
                "B-069",
                "zero-arg-functions",
                "Functions with no arguments",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn get_version() -> &str { "1.0.0" } fn get_name() -> &str { "bashrs" } fn main() { let v = get_version(); let n = get_name(); }"#,
                "get_version()",
            ),
            // --- Nested while with condition update ---
            CorpusEntry::new(
                "B-070",
                "while-bisection",
                "Binary search-like while loop narrowing",
                CorpusFormat::Bash,
                CorpusTier::Production,
                r#"fn main() { let mut low = 0; let mut high = 100; while low < high { let mid = (low + high) / 2; if mid < 50 { low = mid + 1; } else { high = mid; } } }"#,
                "while",
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
                "HEALTHCHECK",
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
                "ENTRYPOINT",
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
            CorpusEntry::new("B-071", "nested-for-loops", "Nested for loop multiplication table", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { for i in 1..4 { for j in 1..4 { let product = i * j; } } }"#, "product="),
            // --- Function returning bool ---
            CorpusEntry::new("B-072", "func-returns-bool", "Function returning boolean", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn is_even(n: u32) -> bool { n % 2 == 0 } fn main() { let even = is_even(4); }"#, "is_even()"),
            // --- Multiple eprintln ---
            CorpusEntry::new("B-073", "multi-eprintln", "Multiple eprintln calls", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { eprintln!("step 1"); eprintln!("step 2"); eprintln!("done"); }"#, ">&2"),
            // --- Mixed println and eprintln ---
            CorpusEntry::new("B-074", "mixed-output", "stdout and stderr output", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { println!("output"); eprintln!("error"); println!("more output"); }"#, "rash_println"),
            // --- While with complex body ---
            CorpusEntry::new("B-075", "while-complex-body", "While loop with if-else in body", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut evens = 0; let mut odds = 0; while x < 10 { if x % 2 == 0 { evens += 1; } else { odds += 1; } x += 1; } }"#, "while"),
            // --- Chained comparisons ---
            CorpusEntry::new("B-076", "chained-compare", "Multiple comparison operators", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn range_check(x: u32) -> bool { x >= 10 && x <= 100 } fn main() { let ok = range_check(50); }"#, "range_check()"),
            // --- Nested match in function ---
            CorpusEntry::new("B-077", "nested-match-func", "Function body with nested match", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn day_type(d: u32) -> u32 { match d { 0 => { return 0; } 6 => { return 0; } _ => { return 1; } } } fn main() { let is_weekday = day_type(3); }"#, "day_type()"),
            // --- Empty else branch ---
            CorpusEntry::new("B-078", "empty-else", "If with empty else", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let x = 5; if x > 0 { println!("positive"); } else { println!("non-positive"); } }"#, "if"),
            // --- Multiple mut variables ---
            CorpusEntry::new("B-079", "multiple-mut-vars", "Multiple mutable variable updates", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut a = 1; let mut b = 2; let mut c = 3; a += b; b += c; c += a; }"#, "a="),
            // --- Deeply nested function calls ---
            CorpusEntry::new("B-080", "deeply-nested-calls", "Three levels of function nesting", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn f1(x: u32) -> u32 { x + 1 } fn f2(x: u32) -> u32 { x * 2 } fn f3(x: u32) -> u32 { x - 1 } fn main() { let r = f3(f2(f1(10))); }"#, "f1()"),
            // --- For loop with compound assignment ---
            CorpusEntry::new("B-081", "for-compound-assign", "For loop body using += accumulation", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut total = 0; for i in 0..100 { total += i; } }"#, "total="),
            // --- Boolean negation in condition ---
            CorpusEntry::new("B-082", "bool-negation-cond", "Negated boolean in if condition", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let debug = false; if !debug { println!("production"); } }"#, "if"),
            // --- Multiple return statements ---
            CorpusEntry::new("B-083", "multi-return", "Function with 4 return paths", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn priority(level: u32) -> &str { if level == 0 { return "critical"; } if level == 1 { return "high"; } if level == 2 { return "medium"; } return "low"; } fn main() { let p = priority(1); }"#, "priority()"),
            // --- Zero comparison ---
            CorpusEntry::new("B-084", "zero-comparison", "Comparison with zero", CorpusFormat::Bash, CorpusTier::Trivial,
                r#"fn main() { let x = 0; let is_zero = x == 0; }"#, "is_zero="),
            // --- String equality (implicit) ---
            CorpusEntry::new("B-085", "string-assign-multi", "Multiple string assignments", CorpusFormat::Bash, CorpusTier::Trivial,
                r#"fn main() { let a = "hello"; let b = "world"; let c = "test"; }"#, "b="),
            // --- Large for range ---
            CorpusEntry::new("B-086", "large-for-range", "For loop with large range", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let mut sum = 0; for i in 0..1000 { sum += 1; } }"#, "sum="),
            // --- Subtraction chain ---
            CorpusEntry::new("B-087", "subtraction-chain", "Chained subtraction operations", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let a = 100; let b = a - 10; let c = b - 20; let d = c - 30; }"#, "d="),
            // --- Division and modulo ---
            CorpusEntry::new("B-088", "div-mod-combined", "Division and modulo together", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let x = 17; let quotient = x / 5; let remainder = x % 5; }"#, "quotient="),
            // --- Match wildcard only ---
            CorpusEntry::new("B-089", "match-wildcard-only", "Match with only wildcard arm", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let x = 42; match x { _ => { println!("any"); } } }"#, "case"),
            // --- Inclusive range ---
            CorpusEntry::new("B-090", "inclusive-range-large", "Inclusive range 1..=100", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let mut count = 0; for i in 1..=50 { count += 1; } }"#, "count="),
            // --- Empty main ---
            CorpusEntry::new("B-091", "empty-main", "Minimal valid program", CorpusFormat::Bash, CorpusTier::Trivial,
                r#"fn main() {}"#, "#!/bin/sh"),
            // --- Single println ---
            CorpusEntry::new("B-092", "single-println", "Hello world program", CorpusFormat::Bash, CorpusTier::Trivial,
                r#"fn main() { println!("hello world"); }"#, "rash_println"),
            // --- While true break pattern ---
            CorpusEntry::new("B-093", "while-true-break", "While true with conditional break", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let mut i = 0; while true { i += 1; if i >= 10 { break; } } }"#, "break"),
            // --- Multiple functions same name prefix ---
            CorpusEntry::new("B-094", "func-name-prefix", "Functions with similar name prefixes", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn get_name() -> &str { "app" } fn get_version() -> &str { "1.0" } fn get_author() -> &str { "test" } fn main() { let n = get_name(); let v = get_version(); let a = get_author(); }"#, "get_name()"),
            // --- Comparison result as value ---
            CorpusEntry::new("B-095", "comparison-value", "Store comparison result directly", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let a = 10; let b = 20; let less = a < b; let equal = a == b; let greater = a > b; }"#, "less="),
            // --- While countdown to zero ---
            CorpusEntry::new("B-096", "while-to-zero", "Decrement while loop to zero", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let mut n = 50; while n > 0 { n -= 1; } }"#, "while"),
            // --- Function with default return ---
            CorpusEntry::new("B-097", "func-implicit-return", "Function returning last expression", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn double(x: u32) -> u32 { x * 2 } fn main() { let result = double(21); }"#, "double()"),
            // --- Nested if in while ---
            CorpusEntry::new("B-098", "nested-if-while", "If inside while loop", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut x = 0; let mut pos = 0; let mut neg = 0; while x < 20 { if x % 3 == 0 { pos += 1; } else { neg += 1; } x += 1; } }"#, "while"),
            // --- Match with many string literals ---
            CorpusEntry::new("B-099", "match-string-heavy", "Match returning many different strings", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn color(n: u32) -> &str { match n { 0 => { return "red"; } 1 => { return "blue"; } 2 => { return "green"; } 3 => { return "yellow"; } 4 => { return "purple"; } _ => { return "black"; } } } fn main() { let c = color(2); }"#, "color()"),
            // --- Accumulate with division ---
            CorpusEntry::new("B-100", "accumulate-division", "Repeated division accumulation", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut n = 1000; let mut steps = 0; while n > 1 { n = n / 2; steps += 1; } }"#, "steps="),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion_makefile(&mut self) {
        let entries = vec![
            // --- Kubernetes deployment ---
            CorpusEntry::new("M-041", "k8s-deploy", "Kubernetes deployment Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let namespace = "default"; phony_target("deploy", &[], &["kubectl apply -f k8s/"]); phony_target("undeploy", &[], &["kubectl delete -f k8s/"]); phony_target("status", &[], &["kubectl get pods -n $(NAMESPACE)"]); phony_target("logs", &[], &["kubectl logs -f deployment/app"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "NAMESPACE := default"),
            // --- WASM build ---
            CorpusEntry::new("M-042", "wasm-build", "WebAssembly build Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let target = "wasm32-unknown-unknown"; phony_target("wasm-build", &[], &["wasm-pack build --target web"]); phony_target("wasm-test", &[], &["wasm-pack test --headless"]); phony_target("wasm-serve", &[], &["ruchy serve --port 8000"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "TARGET := wasm32-unknown-unknown"),
            // --- Terraform workflow ---
            CorpusEntry::new("M-043", "terraform-workflow", "Terraform infrastructure Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let env = "staging"; phony_target("init", &[], &["terraform init"]); phony_target("plan", &["init"], &["terraform plan"]); phony_target("apply", &["plan"], &["terraform apply -auto-approve"]); phony_target("destroy", &[], &["terraform destroy -auto-approve"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "ENV := staging"),
            // --- Cross-compilation ---
            CorpusEntry::new("M-044", "cross-compile", "Cross-compilation targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let name = "app"; phony_target("build-linux", &[], &["GOOS=linux GOARCH=amd64 go build -o bin/linux/app"]); phony_target("build-mac", &[], &["GOOS=darwin GOARCH=arm64 go build -o bin/mac/app"]); phony_target("build-all", &["build-linux", "build-mac"], &["echo done"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "NAME := app"),
            // --- Benchmark targets ---
            CorpusEntry::new("M-045", "benchmark-makefile", "Performance benchmarking targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("bench", &[], &["cargo bench"]); phony_target("bench-baseline", &[], &["cargo bench -- --save-baseline main"]); phony_target("bench-compare", &[], &["cargo bench -- --baseline main"]); phony_target("flamegraph", &[], &["cargo flamegraph"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: bench"),
            // --- Monorepo targets ---
            CorpusEntry::new("M-046", "monorepo-targets", "Monorepo workspace targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("build-all", &[], &["cargo build --workspace"]); phony_target("test-all", &[], &["cargo test --workspace"]); phony_target("lint-all", &[], &["cargo clippy --workspace"]); phony_target("fmt-all", &[], &["cargo fmt --all -- --check"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-all"),
            // --- Security scanning ---
            CorpusEntry::new("M-047", "security-scan", "Security scanning Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("audit", &[], &["cargo audit"]); phony_target("deny", &[], &["cargo deny check"]); phony_target("security", &["audit", "deny"], &["echo security passed"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: audit"),
            // --- Proto/gRPC build ---
            CorpusEntry::new("M-048", "proto-build", "Protocol Buffer build targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let proto_dir = "proto"; phony_target("proto", &[], &["protoc --go_out=. proto/*.proto"]); phony_target("proto-lint", &[], &["buf lint"]); phony_target("proto-format", &[], &["buf format -w"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "PROTO_DIR := proto"),
            // --- Container orchestration ---
            CorpusEntry::new("M-049", "compose-targets", "Docker Compose management", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("up", &[], &["docker compose up -d"]); phony_target("down", &[], &["docker compose down"]); phony_target("restart", &["down", "up"], &[]); phony_target("logs", &[], &["docker compose logs -f"]); phony_target("ps", &[], &["docker compose ps"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: up"),
            // --- Code generation ---
            CorpusEntry::new("M-050", "codegen-targets", "Code generation Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let gen_dir = "generated"; phony_target("generate", &[], &["go generate ./..."]); phony_target("gen-clean", &[], &["rm -rf generated/"]); phony_target("gen-verify", &["generate"], &["git diff --exit-code generated/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "GEN_DIR := generated"),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion_dockerfile(&mut self) {
        let entries = vec![
            // --- Elixir/Phoenix Dockerfile ---
            CorpusEntry::new("D-041", "elixir-phoenix", "Elixir Phoenix production build", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("elixir", "1.16-alpine", "builder"); workdir("/app"); copy("mix.exs", "."); copy("mix.lock", "."); run(&["mix deps.get --only prod"]); copy(".", "."); run(&["MIX_ENV=prod mix release"]); from_image("alpine", "3.18"); copy_from("builder", "/app/_build/prod/rel/myapp", "/app"); user("65534"); entrypoint(&["/app/bin/myapp"]); cmd(&["start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM elixir:1.16-alpine AS builder"),
            // --- Redis with config ---
            CorpusEntry::new("D-042", "redis-custom", "Custom Redis with config", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("redis", "7-alpine"); copy("redis.conf", "/usr/local/etc/redis/"); expose(6379u16); cmd(&["redis-server", "/usr/local/etc/redis/redis.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM redis:7-alpine"),
            // --- .NET production ---
            CorpusEntry::new("D-043", "dotnet-production", ".NET production multi-stage build", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("mcr.microsoft.com/dotnet/sdk", "8.0", "builder"); workdir("/app"); copy("*.csproj", "."); run(&["dotnet restore"]); copy(".", "."); run(&["dotnet publish -c Release -o /out"]); from_image("mcr.microsoft.com/dotnet/aspnet", "8.0"); copy_from("builder", "/out", "/app"); workdir("/app"); expose(8080u16); user("1000"); entrypoint(&["dotnet", "MyApp.dll"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder"),
            // --- Minimal scratch container ---
            CorpusEntry::new("D-044", "scratch-minimal", "Minimal scratch-based container", CorpusFormat::Dockerfile, CorpusTier::Adversarial,
                r#"fn main() { from_image_as("golang", "1.22", "builder"); workdir("/app"); copy(".", "."); run(&["CGO_ENABLED=0 go build -ldflags=-s -o /app/bin"]); from_image("scratch", "latest"); copy_from("builder", "/app/bin", "/app"); entrypoint(&["/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM scratch"),
            // --- ML/AI container ---
            CorpusEntry::new("D-045", "ml-container", "Machine learning inference container", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("python", "3.11-slim"); workdir("/app"); run(&["pip install --no-cache-dir torch transformers"]); copy("model", "model"); copy("serve.py", "."); let model_path = "/app/model"; expose(8000u16); healthcheck("curl -f http://localhost:8000/health || exit 1"); cmd(&["python", "serve.py"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn healthcheck(c: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM python:3.11-slim"),
            // --- Multi-service Dockerfile ---
            CorpusEntry::new("D-046", "worker-service", "Background worker service container", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("rust", "1.75", "builder"); workdir("/app"); copy(".", "."); run(&["cargo build --release --bin worker"]); from_image("debian", "bookworm-slim"); run(&["apt-get update", "apt-get install -y --no-install-recommends ca-certificates", "rm -rf /var/lib/apt/lists/*"]); copy_from("builder", "/app/target/release/worker", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/worker"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM rust:1.75 AS builder"),
            // --- Development container ---
            CorpusEntry::new("D-047", "dev-container", "Development container with tools", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("rust", "1.75"); run(&["rustup component add clippy rustfmt"]); run(&["cargo install cargo-watch cargo-llvm-cov"]); workdir("/workspace"); let rust_backtrace = "1"; } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {}"#,
                "FROM rust:1.75"),
            // --- API gateway ---
            CorpusEntry::new("D-048", "api-gateway", "API gateway with Envoy", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("envoyproxy/envoy", "v1.28"); copy("envoy.yaml", "/etc/envoy/envoy.yaml"); expose(8080u16); expose(8443u16); expose(9901u16); entrypoint(&["/usr/local/bin/envoy"]); cmd(&["-c", "/etc/envoy/envoy.yaml"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM envoyproxy/envoy:v1.28"),
            // --- Caddy web server ---
            CorpusEntry::new("D-049", "caddy-server", "Caddy web server with custom config", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("caddy", "2-alpine"); copy("Caddyfile", "/etc/caddy/Caddyfile"); copy("static", "/srv"); expose(80u16); expose(443u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM caddy:2-alpine"),
            // --- Grafana with plugins ---
            CorpusEntry::new("D-050", "grafana-custom", "Custom Grafana with plugins", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("grafana/grafana", "10.2"); let gf_install_plugins = "grafana-clock-panel,grafana-simple-json-datasource"; let gf_security_admin_password = "admin"; copy("dashboards", "/var/lib/grafana/dashboards"); copy("provisioning", "/etc/grafana/provisioning"); expose(3000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM grafana/grafana:10.2"),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion wave 2: pushing toward 300+ entries
    // =========================================================================

    fn load_expansion2_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new("B-101", "gcd-algorithm", "Greatest common divisor via while loop", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut a = 48; let mut b = 18; while b > 0 { let temp = b; b = a % b; a = temp; } }"#, "while"),
            CorpusEntry::new("B-102", "max-of-three", "Find maximum of three values", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn max3(a: u32, b: u32, c: u32) -> u32 { if a > b && a > c { return a; } if b > c { return b; } return c; } fn main() { let m = max3(10, 20, 15); }"#, "max3()"),
            CorpusEntry::new("B-103", "abs-value", "Absolute value function", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn abs_val(x: u32, y: u32) -> u32 { if x > y { x - y } else { y - x } } fn main() { let d = abs_val(10, 7); }"#, "abs_val()"),
            CorpusEntry::new("B-104", "swap-values", "Swap two variables using temp", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let mut a = 10; let mut b = 20; let temp = a; a = b; b = temp; }"#, "temp="),
            CorpusEntry::new("B-105", "clamp-range", "Clamp value to range", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn clamp(x: u32, min: u32, max: u32) -> u32 { if x < min { return min; } if x > max { return max; } return x; } fn main() { let c = clamp(150, 0, 100); }"#, "clamp()"),
            CorpusEntry::new("B-106", "count-digits", "Count digits in number via division", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut n = 12345; let mut digits = 0; while n > 0 { n = n / 10; digits += 1; } }"#, "digits="),
            CorpusEntry::new("B-107", "sum-of-digits", "Sum digits of a number", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut n = 9876; let mut sum = 0; while n > 0 { sum += n % 10; n = n / 10; } }"#, "sum="),
            CorpusEntry::new("B-108", "power-of-two", "Compute power of 2 iteratively", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut result = 1; for _i in 0..10 { result *= 2; } }"#, "result="),
            CorpusEntry::new("B-109", "linear-search", "Linear search through comparison", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let target = 42; let mut found = false; for i in 0..100 { if i == target { found = true; break; } } }"#, "found="),
            CorpusEntry::new("B-110", "min-of-three", "Find minimum of three values", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn min3(a: u32, b: u32, c: u32) -> u32 { if a < b && a < c { return a; } if b < c { return b; } return c; } fn main() { let m = min3(30, 10, 20); }"#, "min3()"),
            CorpusEntry::new("B-111", "collatz-step", "Single Collatz step function", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn collatz_step(n: u32) -> u32 { if n % 2 == 0 { n / 2 } else { n * 3 + 1 } } fn main() { let next = collatz_step(7); }"#, "collatz_step()"),
            CorpusEntry::new("B-112", "triangle-numbers", "Compute triangle numbers", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut sum = 0; for i in 1..=10 { sum += i; } }"#, "sum="),
            CorpusEntry::new("B-113", "string-path-parts", "Multiple path component variables", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let dir = "/usr/local"; let bin = "bin"; let name = "bashrs"; let ext = "sh"; }"#, "dir="),
            CorpusEntry::new("B-114", "nested-break", "Nested loop with inner break", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let mut total = 0; for i in 0..10 { for j in 0..10 { if j > i { break; } total += 1; } } }"#, "total="),
            CorpusEntry::new("B-115", "decrement-loop", "Decrement loop with *= operator", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let mut x = 1024; let mut count = 0; while x > 1 { x = x / 2; count += 1; } }"#, "count="),
            CorpusEntry::new("B-116", "multi-condition-while", "While with OR condition", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let mut a = 0; let mut b = 100; while a < 50 || b > 50 { a += 1; b -= 1; } }"#, "while"),
            CorpusEntry::new("B-117", "match-range-like", "Match with multiple arms mapping ranges", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn grade(score: u32) -> &str { if score >= 90 { return "A"; } if score >= 80 { return "B"; } if score >= 70 { return "C"; } if score >= 60 { return "D"; } return "F"; } fn main() { let g = grade(85); }"#, "grade()"),
            CorpusEntry::new("B-118", "bool-and-chain", "Chain of AND conditions", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn valid(a: u32, b: u32, c: u32) -> bool { a > 0 && b > 0 && c > 0 } fn main() { let v = valid(1, 2, 3); }"#, "valid()"),
            CorpusEntry::new("B-119", "bool-or-chain", "Chain of OR conditions", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn any_zero(a: u32, b: u32, c: u32) -> bool { a == 0 || b == 0 || c == 0 } fn main() { let z = any_zero(1, 0, 3); }"#, "any_zero()"),
            CorpusEntry::new("B-120", "for-in-if", "For loop only inside if branch", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let run = true; if run { for i in 0..5 { println!("running"); } } }"#, "for"),
            CorpusEntry::new("B-121", "five-functions", "Program with five functions", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn f1() -> u32 { 1 } fn f2() -> u32 { 2 } fn f3() -> u32 { 3 } fn f4() -> u32 { 4 } fn f5() -> u32 { 5 } fn main() { let total = f1() + f2() + f3() + f4() + f5(); }"#, "f1()"),
            CorpusEntry::new("B-122", "nested-if-match", "If containing match", CorpusFormat::Bash, CorpusTier::Complex,
                r#"fn main() { let x = 5; if x > 0 { match x { 1 => { println!("one"); } _ => { println!("other"); } } } }"#, "case"),
            CorpusEntry::new("B-123", "while-for-nested", "While containing for", CorpusFormat::Bash, CorpusTier::Complex,
                r#"fn main() { let mut outer = 0; while outer < 3 { for _i in 0..5 { outer += 1; } } }"#, "while"),
            CorpusEntry::new("B-124", "multi-assign-chain", "Chained assignments", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let a = 1; let b = a + 1; let c = b + 1; let d = c + 1; let e = d + 1; }"#, "e="),
            CorpusEntry::new("B-125", "mixed-types", "Mix of u32 and bool variables", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let count = 42; let active = true; let name = "test"; let limit = 100; let verbose = false; }"#, "active="),
            CorpusEntry::new("B-126", "early-exit-loop", "Multiple early exits in loop", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut found = 0; for i in 0..1000 { if i % 7 == 0 && i % 13 == 0 { found = i; break; } } }"#, "found="),
            CorpusEntry::new("B-127", "arithmetic-overflow-safe", "Arithmetic within safe u32 range", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let a = 1000000; let b = 2000000; let c = a + b; let d = c * 2; }"#, "d="),
            CorpusEntry::new("B-128", "for-with-step-like", "Simulated stepped iteration", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut count = 0; for i in 0..50 { if i % 5 == 0 { count += 1; } } }"#, "count="),
            CorpusEntry::new("B-129", "string-heavy-program", "Program with many string literals", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let host = "localhost"; let port = "8080"; let proto = "https"; let path = "/api/v1"; let method = "GET"; let content_type = "application/json"; println!("configured"); }"#, "host="),
            CorpusEntry::new("B-130", "multi-while-sequential", "Sequential while loops", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut a = 10; while a > 0 { a -= 1; } let mut b = 10; while b > 0 { b -= 2; } }"#, "while"),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion2_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new("M-051", "go-project", "Go project with standard targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let binary = "myapp"; phony_target("build", &[], &["go build -o bin/$(BINARY) ."]); phony_target("test", &[], &["go test ./..."]); phony_target("vet", &[], &["go vet ./..."]); phony_target("lint", &[], &["golangci-lint run"]); phony_target("clean", &[], &["rm -rf bin/"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "BINARY := myapp"),
            CorpusEntry::new("M-052", "npm-project", "Node.js/npm project Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("install", &[], &["npm ci"]); phony_target("build", &["install"], &["npm run build"]); phony_target("test", &["install"], &["npm test"]); phony_target("lint", &["install"], &["npm run lint"]); phony_target("start", &["build"], &["npm start"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: install"),
            CorpusEntry::new("M-053", "latex-project", "LaTeX document build", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let doc = "paper"; target("paper.pdf", &["paper.tex"], &["pdflatex $(DOC).tex", "bibtex $(DOC)", "pdflatex $(DOC).tex"]); phony_target("clean", &[], &["rm -f *.aux *.bbl *.blg *.log *.pdf"]); phony_target("view", &["paper.pdf"], &["open $(DOC).pdf"]); } fn target(n: &str, d: &[&str], r: &[&str]) {} fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "DOC := paper"),
            CorpusEntry::new("M-054", "ansible-project", "Ansible playbook management", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let inventory = "inventory.yml"; phony_target("deploy", &[], &["ansible-playbook -i $(INVENTORY) deploy.yml"]); phony_target("check", &[], &["ansible-playbook -i $(INVENTORY) deploy.yml --check"]); phony_target("lint", &[], &["ansible-lint"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "INVENTORY := inventory.yml"),
            CorpusEntry::new("M-055", "aws-deploy", "AWS deployment targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let stack_name = "mystack"; let region = "us-east-1"; phony_target("deploy", &[], &["aws cloudformation deploy --stack-name $(STACK_NAME) --region $(REGION)"]); phony_target("delete", &[], &["aws cloudformation delete-stack --stack-name $(STACK_NAME)"]); phony_target("status", &[], &["aws cloudformation describe-stacks --stack-name $(STACK_NAME)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "STACK_NAME := mystack"),
            CorpusEntry::new("M-056", "migration-targets", "Data migration Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("migrate-up", &[], &["migrate -source file://migrations -database $(DB_URL) up"]); phony_target("migrate-down", &[], &["migrate -source file://migrations -database $(DB_URL) down 1"]); phony_target("migrate-create", &[], &["migrate create -ext sql -dir migrations -seq $(NAME)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: migrate-up"),
            CorpusEntry::new("M-057", "helm-chart", "Helm chart management", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let chart = "myapp"; let namespace = "production"; phony_target("helm-install", &[], &["helm install $(CHART) ./chart -n $(NAMESPACE)"]); phony_target("helm-upgrade", &[], &["helm upgrade $(CHART) ./chart -n $(NAMESPACE)"]); phony_target("helm-uninstall", &[], &["helm uninstall $(CHART) -n $(NAMESPACE)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "CHART := myapp"),
            CorpusEntry::new("M-058", "test-matrix", "Test matrix targets", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("test-unit", &[], &["cargo test --lib"]); phony_target("test-integration", &[], &["cargo test --test '*'"]); phony_target("test-doc", &[], &["cargo test --doc"]); phony_target("test-all", &["test-unit", "test-integration", "test-doc"], &["echo all tests passed"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: test-unit"),
            CorpusEntry::new("M-059", "static-analysis", "Static analysis Makefile", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("clippy", &[], &["cargo clippy --all-targets -- -D warnings"]); phony_target("fmt-check", &[], &["cargo fmt -- --check"]); phony_target("audit", &[], &["cargo audit"]); phony_target("check-all", &["clippy", "fmt-check", "audit"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: clippy"),
            CorpusEntry::new("M-060", "book-build", "mdBook documentation", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("book-build", &[], &["mdbook build book"]); phony_target("book-serve", &[], &["mdbook serve book"]); phony_target("book-test", &[], &["mdbook test book"]); phony_target("book-clean", &[], &["rm -rf book/book"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: book-build"),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion2_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new("D-051", "ruby-rails", "Ruby on Rails Dockerfile", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("ruby", "3.3-slim"); run(&["apt-get update", "apt-get install -y build-essential libpq-dev", "rm -rf /var/lib/apt/lists/*"]); workdir("/app"); copy("Gemfile", "."); copy("Gemfile.lock", "."); run(&["bundle install --without development test"]); copy(".", "."); expose(3000u16); cmd(&["rails", "server", "-b", "0.0.0.0"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM ruby:3.3-slim"),
            CorpusEntry::new("D-052", "php-laravel", "PHP Laravel Dockerfile", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("php", "8.3-fpm-alpine"); run(&["apk add --no-cache composer"]); workdir("/var/www"); copy("composer.json", "."); copy("composer.lock", "."); run(&["composer install --no-dev"]); copy(".", "."); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM php:8.3-fpm-alpine"),
            CorpusEntry::new("D-053", "vault-server", "HashiCorp Vault server", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("vault", "1.15"); let vault_addr = "http://0.0.0.0:8200"; copy("config.hcl", "/vault/config/"); expose(8200u16); entrypoint(&["vault"]); cmd(&["server", "-config=/vault/config/config.hcl"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM vault:1.15"),
            CorpusEntry::new("D-054", "zig-build", "Zig language build container", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("alpine", "3.18", "builder"); run(&["apk add --no-cache zig"]); workdir("/app"); copy(".", "."); run(&["zig build -Doptimize=ReleaseSafe"]); from_image("alpine", "3.18"); copy_from("builder", "/app/zig-out/bin/app", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/app"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM alpine:3.18 AS builder"),
            CorpusEntry::new("D-055", "prometheus-config", "Prometheus with custom config", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("prom/prometheus", "v2.48"); copy("prometheus.yml", "/etc/prometheus/"); copy("rules", "/etc/prometheus/rules/"); expose(9090u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM prom/prometheus:v2.48"),
            CorpusEntry::new("D-056", "minio-server", "MinIO object storage", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("minio/minio", "latest"); let minio_root_user = "admin"; let minio_root_password = "changeme"; expose(9000u16); expose(9001u16); entrypoint(&["minio"]); cmd(&["server", "/data", "--console-address", ":9001"]); } fn from_image(i: &str, t: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM minio/minio:latest"),
            CorpusEntry::new("D-057", "traefik-proxy", "Traefik reverse proxy", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("traefik", "v3.0"); copy("traefik.yml", "/etc/traefik/"); copy("dynamic", "/etc/traefik/dynamic/"); expose(80u16); expose(443u16); expose(8080u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM traefik:v3.0"),
            CorpusEntry::new("D-058", "keycloak-server", "Keycloak identity server", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("quay.io/keycloak/keycloak", "23.0"); let kc_db = "postgres"; copy("themes", "/opt/keycloak/themes/"); expose(8080u16); entrypoint(&["/opt/keycloak/bin/kc.sh"]); cmd(&["start-dev"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM quay.io/keycloak/keycloak:23.0"),
            CorpusEntry::new("D-059", "clickhouse-custom", "ClickHouse with custom config", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("clickhouse/clickhouse-server", "23.12"); copy("config.xml", "/etc/clickhouse-server/"); copy("users.xml", "/etc/clickhouse-server/"); expose(8123u16); expose(9000u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM clickhouse/clickhouse-server:23.12"),
            CorpusEntry::new("D-060", "temporal-worker", "Temporal workflow worker", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("golang", "1.22-alpine", "builder"); workdir("/app"); copy("go.mod", "."); copy("go.sum", "."); run(&["go mod download"]); copy(".", "."); run(&["go build -o /worker ./cmd/worker"]); from_image("alpine", "3.18"); run(&["apk add --no-cache ca-certificates"]); copy_from("builder", "/worker", "/usr/local/bin/"); user("65534"); entrypoint(&["/usr/local/bin/worker"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn user(u: &str) {} fn entrypoint(e: &[&str]) {}"#,
                "FROM golang:1.22-alpine AS builder"),
        ];
        self.entries.extend(entries);
    }

    // =========================================================================
    // Expansion wave 3: pushing toward 350 total
    // =========================================================================

    fn load_expansion3_bash(&mut self) {
        let entries = vec![
            CorpusEntry::new("B-131", "is-prime-like", "Primality check via trial division", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn is_prime(n: u32) -> bool { if n < 2 { return false; } let mut i = 2; while i * i <= n { if n % i == 0 { return false; } i += 1; } return true; } fn main() { let p = is_prime(17); }"#, "is_prime()"),
            CorpusEntry::new("B-132", "bubble-sort-step", "Single bubble sort pass", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut swapped = false; let mut a = 5; let mut b = 3; if a > b { let temp = a; a = b; b = temp; swapped = true; } }"#, "swapped="),
            CorpusEntry::new("B-133", "menu-dispatch", "Menu-like dispatch with match", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn dispatch(cmd: u32) { match cmd { 1 => { println!("list"); } 2 => { println!("add"); } 3 => { println!("delete"); } 4 => { println!("edit"); } 5 => { println!("quit"); } _ => { eprintln!("unknown"); } } } fn main() { dispatch(3); }"#, "dispatch()"),
            CorpusEntry::new("B-134", "state-machine", "Simple state machine with while and match", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut state = 0; let mut steps = 0; while state != 3 { match state { 0 => { state = 1; } 1 => { state = 2; } 2 => { state = 3; } _ => { state = 3; } } steps += 1; } }"#, "state="),
            CorpusEntry::new("B-135", "running-average", "Running average computation", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut sum = 0; let mut count = 0; for i in 1..=20 { sum += i; count += 1; } let avg = sum / count; }"#, "avg="),
            CorpusEntry::new("B-136", "fizzbuzz-like", "FizzBuzz-like modular logic", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { for i in 1..=15 { if i % 3 == 0 && i % 5 == 0 { println!("fizzbuzz"); } else if i % 3 == 0 { println!("fizz"); } else if i % 5 == 0 { println!("buzz"); } } }"#, "rash_println"),
            CorpusEntry::new("B-137", "config-builder", "Configuration-building pattern", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let host = "0.0.0.0"; let port = 8080; let workers = 4; let timeout = 30; let max_conn = 1000; let verbose = true; if verbose { println!("config ready"); } }"#, "workers="),
            CorpusEntry::new("B-138", "retry-pattern", "Retry loop pattern", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let max_retries = 3; let mut attempts = 0; let mut success = false; while attempts < max_retries && !success { attempts += 1; if attempts >= 2 { success = true; } } }"#, "while"),
            CorpusEntry::new("B-139", "accumulate-strings", "Building up string config vars", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let proto = "https"; let host = "api.example.com"; let port = "443"; let base = "/v2"; }"#, "proto="),
            CorpusEntry::new("B-140", "nested-conditions-deep", "Four-level nested if", CorpusFormat::Bash, CorpusTier::Adversarial,
                r#"fn main() { let a = 1; let b = 2; let c = 3; let d = 4; if a > 0 { if b > 0 { if c > 0 { if d > 0 { println!("all positive"); } } } } }"#, "if"),
            CorpusEntry::new("B-141", "multi-for-sequential", "Sequential for loops", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut sum1 = 0; for i in 0..10 { sum1 += i; } let mut sum2 = 0; for j in 10..20 { sum2 += j; } let total = sum1 + sum2; }"#, "total="),
            CorpusEntry::new("B-142", "helper-chain", "Functions calling helper functions", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn square(x: u32) -> u32 { x * x } fn sum_of_squares(a: u32, b: u32) -> u32 { square(a) + square(b) } fn main() { let r = sum_of_squares(3, 4); }"#, "square()"),
            CorpusEntry::new("B-143", "while-flag-pattern", "While loop controlled by flag", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn main() { let mut running = true; let mut ticks = 0; while running { ticks += 1; if ticks >= 100 { running = false; } } }"#, "while"),
            CorpusEntry::new("B-144", "modular-arithmetic", "Modular arithmetic operations", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let x = 42; let mod3 = x % 3; let mod5 = x % 5; let mod7 = x % 7; }"#, "mod3="),
            CorpusEntry::new("B-145", "nested-func-with-loop", "Function containing for loop", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn sum_to(n: u32) -> u32 { let mut total = 0; for i in 0..n { total += i; } total } fn main() { let s = sum_to(100); }"#, "sum_to()"),
            CorpusEntry::new("B-146", "decrement-ops", "Multiple decrement operations", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let mut x = 100; x -= 10; x -= 20; x -= 30; }"#, "x="),
            CorpusEntry::new("B-147", "compare-and-branch", "Comparison-driven branching", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn classify(temp: u32) -> &str { if temp < 32 { return "freezing"; } else if temp < 60 { return "cold"; } else if temp < 80 { return "warm"; } else { return "hot"; } } fn main() { let c = classify(72); }"#, "classify()"),
            CorpusEntry::new("B-148", "empty-for-body", "For loop with minimal body", CorpusFormat::Bash, CorpusTier::Trivial,
                r#"fn main() { let mut x = 0; for _i in 0..10 { x = x; } }"#, "for"),
            CorpusEntry::new("B-149", "match-with-println", "Match arms with println", CorpusFormat::Bash, CorpusTier::Standard,
                r#"fn main() { let code = 200; match code { 200 => { println!("ok"); } 404 => { println!("not found"); } 500 => { println!("error"); } _ => { println!("unknown"); } } }"#, "case"),
            CorpusEntry::new("B-150", "complex-program", "Program combining all features", CorpusFormat::Bash, CorpusTier::Production,
                r#"fn validate(x: u32) -> bool { x > 0 && x <= 1000 } fn process(x: u32) -> u32 { let mut result = x; for _i in 0..3 { result *= 2; } result } fn main() { let input = 42; if validate(input) { let output = process(input); println!("done"); } else { eprintln!("invalid"); } }"#, "validate()"),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion3_makefile(&mut self) {
        let entries = vec![
            CorpusEntry::new("M-061", "elixir-project", "Elixir/Mix project", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("deps", &[], &["mix deps.get"]); phony_target("compile", &["deps"], &["mix compile"]); phony_target("test", &["compile"], &["mix test"]); phony_target("format", &[], &["mix format"]); phony_target("dialyzer", &["compile"], &["mix dialyzer"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: deps"),
            CorpusEntry::new("M-062", "swift-project", "Swift/SPM project", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("build", &[], &["swift build"]); phony_target("test", &[], &["swift test"]); phony_target("run", &["build"], &["swift run"]); phony_target("clean", &[], &["swift package clean"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build"),
            CorpusEntry::new("M-063", "cargo-xtask", "Cargo xtask pattern", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("xtask-codegen", &[], &["cargo xtask codegen"]); phony_target("xtask-dist", &[], &["cargo xtask dist"]); phony_target("xtask-install", &[], &["cargo xtask install"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: xtask-codegen"),
            CorpusEntry::new("M-064", "multi-lang-project", "Multi-language project", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("build-rust", &[], &["cargo build"]); phony_target("build-python", &[], &["python setup.py build"]); phony_target("build-go", &[], &["go build"]); phony_target("build-all", &["build-rust", "build-python", "build-go"], &[]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: build-rust"),
            CorpusEntry::new("M-065", "container-registry", "Container registry management", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let registry = "ghcr.io/org"; let image = "app"; let tag = "latest"; phony_target("docker-build", &[], &["docker build -t $(REGISTRY)/$(IMAGE):$(TAG) ."]); phony_target("docker-push", &["docker-build"], &["docker push $(REGISTRY)/$(IMAGE):$(TAG)"]); phony_target("docker-scan", &["docker-build"], &["docker scout cves $(REGISTRY)/$(IMAGE):$(TAG)"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "REGISTRY := ghcr.io/org"),
            CorpusEntry::new("M-066", "pre-commit-setup", "Pre-commit hooks setup", CorpusFormat::Makefile, CorpusTier::Standard,
                r#"fn main() { phony_target("hooks-install", &[], &["pre-commit install"]); phony_target("hooks-run", &[], &["pre-commit run --all-files"]); phony_target("hooks-update", &[], &["pre-commit autoupdate"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: hooks-install"),
            CorpusEntry::new("M-067", "coverage-targets", "Coverage reporting", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("coverage", &[], &["cargo llvm-cov"]); phony_target("coverage-html", &[], &["cargo llvm-cov --html"]); phony_target("coverage-lcov", &[], &["cargo llvm-cov --lcov --output-path lcov.info"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: coverage"),
            CorpusEntry::new("M-068", "documentation-gen", "Documentation generation", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("doc", &[], &["cargo doc --no-deps"]); phony_target("doc-open", &["doc"], &["open target/doc/app/index.html"]); phony_target("doc-check", &[], &["cargo doc --no-deps 2>&1 | grep -c warning"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: doc"),
            CorpusEntry::new("M-069", "version-management", "Version bump management", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { let version = "1.0.0"; phony_target("version-patch", &[], &["cargo set-version --bump patch"]); phony_target("version-minor", &[], &["cargo set-version --bump minor"]); phony_target("version-major", &[], &["cargo set-version --bump major"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                "VERSION := 1.0.0"),
            CorpusEntry::new("M-070", "full-ci-pipeline", "Complete CI pipeline", CorpusFormat::Makefile, CorpusTier::Production,
                r#"fn main() { phony_target("ci", &["fmt-check", "lint", "test", "coverage", "audit"], &["echo CI pipeline passed"]); phony_target("fmt-check", &[], &["cargo fmt -- --check"]); phony_target("lint", &[], &["cargo clippy -- -D warnings"]); phony_target("test", &[], &["cargo test"]); phony_target("coverage", &[], &["cargo llvm-cov"]); phony_target("audit", &[], &["cargo audit"]); } fn phony_target(n: &str, d: &[&str], r: &[&str]) {}"#,
                ".PHONY: ci"),
        ];
        self.entries.extend(entries);
    }

    fn load_expansion3_dockerfile(&mut self) {
        let entries = vec![
            CorpusEntry::new("D-061", "nextjs-app", "Next.js application", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image_as("node", "20-alpine", "builder"); workdir("/app"); copy("package.json", "."); run(&["npm ci"]); copy(".", "."); run(&["npm run build"]); from_image("node", "20-alpine"); workdir("/app"); copy_from("builder", "/app/.next", ".next"); copy_from("builder", "/app/node_modules", "node_modules"); copy_from("builder", "/app/package.json", "."); expose(3000u16); cmd(&["npm", "start"]); } fn from_image_as(i: &str, t: &str, a: &str) {} fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn copy_from(f: &str, s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM node:20-alpine AS builder"),
            CorpusEntry::new("D-062", "postgres-replication", "PostgreSQL with replication config", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("postgres", "16-alpine"); let postgres_replication_mode = "master"; copy("postgresql.conf", "/etc/postgresql/"); copy("pg_hba.conf", "/etc/postgresql/"); expose(5432u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM postgres:16-alpine"),
            CorpusEntry::new("D-063", "deno-app", "Deno application container", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("denoland/deno", "1.39"); workdir("/app"); copy("deps.ts", "."); run(&["deno cache deps.ts"]); copy(".", "."); user("deno"); expose(8000u16); cmd(&["run", "--allow-net", "--allow-read", "main.ts"]); } fn from_image(i: &str, t: &str) {} fn workdir(p: &str) {} fn copy(s: &str, d: &str) {} fn run(c: &[&str]) {} fn user(u: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM denoland/deno:1.39"),
            CorpusEntry::new("D-064", "mongodb-custom", "MongoDB with custom init", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("mongo", "7"); copy("mongod.conf", "/etc/mongod.conf"); copy("init-scripts", "/docker-entrypoint-initdb.d/"); expose(27017u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM mongo:7"),
            CorpusEntry::new("D-065", "rabbitmq-server", "RabbitMQ with management", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("rabbitmq", "3.13-management-alpine"); let rabbitmq_default_user = "admin"; let rabbitmq_default_pass = "password"; copy("rabbitmq.conf", "/etc/rabbitmq/"); expose(5672u16); expose(15672u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM rabbitmq:3.13-management-alpine"),
            CorpusEntry::new("D-066", "elasticsearch-node", "Elasticsearch node", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("docker.elastic.co/elasticsearch/elasticsearch", "8.11"); let discovery_type = "single-node"; let es_java_opts = "-Xms512m -Xmx512m"; copy("elasticsearch.yml", "/usr/share/elasticsearch/config/"); expose(9200u16); expose(9300u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM docker.elastic.co/elasticsearch/elasticsearch:8.11"),
            CorpusEntry::new("D-067", "haproxy-lb", "HAProxy load balancer", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("haproxy", "2.9-alpine"); copy("haproxy.cfg", "/usr/local/etc/haproxy/haproxy.cfg"); expose(80u16); expose(443u16); expose(8404u16); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {}"#,
                "FROM haproxy:2.9-alpine"),
            CorpusEntry::new("D-068", "consul-server", "Consul service mesh", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("hashicorp/consul", "1.17"); copy("consul.hcl", "/consul/config/"); expose(8500u16); expose(8600u16); entrypoint(&["consul"]); cmd(&["agent", "-config-dir=/consul/config"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn entrypoint(e: &[&str]) {} fn cmd(c: &[&str]) {}"#,
                "FROM hashicorp/consul:1.17"),
            CorpusEntry::new("D-069", "nats-server", "NATS messaging server", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("nats", "2.10-alpine"); copy("nats-server.conf", "/etc/nats/"); expose(4222u16); expose(8222u16); cmd(&["-c", "/etc/nats/nats-server.conf"]); } fn from_image(i: &str, t: &str) {} fn copy(s: &str, d: &str) {} fn expose(p: u16) {} fn cmd(c: &[&str]) {}"#,
                "FROM nats:2.10-alpine"),
            CorpusEntry::new("D-070", "cron-runner", "Cron job runner container", CorpusFormat::Dockerfile, CorpusTier::Production,
                r#"fn main() { from_image("alpine", "3.18"); run(&["apk add --no-cache dcron"]); copy("crontab", "/etc/crontabs/root"); copy("scripts", "/scripts/"); run(&["chmod +x /scripts/*"]); cmd(&["crond", "-f"]); } fn from_image(i: &str, t: &str) {} fn run(c: &[&str]) {} fn copy(s: &str, d: &str) {} fn cmd(c: &[&str]) {}"#,
                "FROM alpine:3.18"),
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
