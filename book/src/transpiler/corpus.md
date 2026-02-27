# Corpus Testing

The transpiler is validated by a corpus of 14,712 entries across three formats: Bash, Makefile, and Dockerfile. Every entry specifies Rust input, expected output patterns, and behavioral equivalence checks.

## V2 Scoring System

The corpus uses a 100-point V2 scoring system with 9 dimensions:

| Dimension | Points | Description |
|-----------|--------|-------------|
| A: Transpilation | 30 | Does the Rust input parse and transpile without error? |
| B1: Containment | 10 | Does the output contain the expected substring? |
| B2: Exact Match | 8 | Does a full output line match the expected pattern? |
| B3: Behavioral | 7 | Does the generated script execute correctly in `sh`? |
| C: Coverage | 15 | LLVM line coverage ratio for the format's source files |
| D: Lint Clean | 10 | Does the output pass `shellcheck -s sh`? |
| E: Deterministic | 10 | Does the same input produce byte-identical output? |
| F: Metamorphic | 5 | Does whitespace-varied input produce equivalent output? |
| G: Cross-Shell | 5 | Does the output execute identically in `sh` and `dash`? |

### Grading Scale

| Grade | Score |
|-------|-------|
| A+ | >= 97.0 |
| A | >= 93.0 |
| B | >= 85.0 |
| C | >= 75.0 |
| D | >= 65.0 |
| F | < 65.0 |

## Running the Corpus

```bash
# Full corpus run with V2 scoring
bashrs corpus run

# Show specific entry details
bashrs corpus show B-001

# Show failure analysis
bashrs corpus failures

# Score history
bashrs corpus history
```

## Entry Format

Each corpus entry in `registry.rs` uses the `CorpusEntry::new` constructor:

```rust,ignore
CorpusEntry::new(
    "B-001",                                    // id
    "hello_world",                              // name
    "Basic println transpilation",              // description
    CorpusFormat::Bash,                         // format
    CorpusTier::Basic,                          // tier
    r#"fn main() { println!("Hello"); }"#,     // rust_source (input)
    "Hello",                                    // expected_contains
)
```

- `rust_source`: The Rust code to transpile
- `expected_contains`: A line that must appear in the generated shell output (used for B1 containment and B2 exact match)

## Current Status (v6.65.0)

- **17,882 entries** (16,411 Bash + 784 Makefile + 687 Dockerfile)
- **97.0/100 (A+)** overall score
- **100%** transpilation pass rate (A dimension: 30/30)
- **100%** determinism (E dimension: 10/10)
- **99.9%** lint clean (D dimension: 10/10)
- **99.6%** metamorphic (F dimension: 5/5)
- **98.6%** behavioral (B3 dimension: 6.9/7)
- **95.4%** containment (B1 dimension: 9.5/10)
- **96.0%** cross-shell (G dimension: 4.8/5)
- **84.7%** exact match (B2 dimension: 6.8/8)

### Per-Format Scores

| Format | Score | Grade | Entries |
|--------|-------|-------|---------|
| Bash | 97.0/100 | A+ | 16,411 |
| Makefile | 94.3/100 | A | 784 |
| Dockerfile | 99.3/100 | A+ | 687 |
