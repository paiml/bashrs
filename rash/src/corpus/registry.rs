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
