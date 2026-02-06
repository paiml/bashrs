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
