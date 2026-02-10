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

Each corpus entry in `registry.rs` specifies:

```rust
CorpusEntry {
    id: "B-001",
    name: "hello_world",
    rust_source: r#"fn main() { println!("Hello"); }"#,
    expected_output: "Hello",
    expected_contains: "printf '%s\\n'",
    category: "basic",
    format: CorpusFormat::Bash,
}
```

- `rust_source`: The Rust code to transpile
- `expected_output`: What `sh output.sh` should print (B1 containment check)
- `expected_contains`: A line that must appear in the generated shell (B2 exact match)

## Current Status

- **14,712 entries** (13,397 Bash + 695 Makefile + 620 Dockerfile)
- **97.5/100 (A+)** overall score
- **0 failures** across all entries
- **100%** transpilation pass rate
