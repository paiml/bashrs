# Corpus Testing

Rash v6.61.0 includes a comprehensive transpilation corpus with 14,712 entries for validating the Rust-to-Shell transpiler across three formats (Bash, Makefile, Dockerfile).

## Overview

The corpus is a registry of known-good transpilation test cases. Each entry contains Rust source code, the expected shell output pattern, and metadata about format and difficulty tier.

```bash
# Run the full corpus
bashrs corpus

# Run corpus for a specific format
bashrs corpus --format bash
bashrs corpus --format makefile
bashrs corpus --format dockerfile
```

## Corpus Tiers

| Tier | Name | Purpose | Entry Range |
|------|------|---------|-------------|
| 1 | Core | Basic constructs (variables, echo, strings) | B-001 to B-010 |
| 2 | Standard | Control flow (if/else, loops, match) | B-011 to B-020 |
| 3 | Advanced | Functions, nesting, complex expressions | B-021 to B-050 |
| 4 | Adversarial | Edge cases designed to break the transpiler | B-051+ |
| 5 | Production | Real-world scale programs | B-171+ |

## Supported Formats

### Bash (B-codes)

Transpile Rust to POSIX shell:

```bash
# Example: B-011 (if-else)
# Input (Rust):
#   fn main() { let x = 5; if x > 3 { let msg = "big"; } }
# Output (Shell):
#   if [ "$x" -gt 3 ]; then msg="big"; fi
```

### Makefile (M-codes)

Transpile Rust to Makefile targets:

```bash
bashrs corpus --format makefile
```

### Dockerfile (D-codes)

Transpile Rust to Dockerfile instructions:

```bash
bashrs corpus --format dockerfile
```

## Scoring

The corpus uses Popperian falsification scoring:

- **Below 60% pass rate**: Score is capped (gateway barrier)
- **Above 60% pass rate**: Weighted average across all entries
- **Grade scale**: A+ (90-100), A (80-89), B (70-79), C (60-69), F (<60)

```text
Corpus Score: 152.5/159 (95.9%)
Grade: A+
Entries: 500+ total, 100% pass rate
```

## Adding Custom Corpus Entries

Corpus entries follow this structure:

```rust,ignore
CorpusEntry::new(
    "B-200",                        // ID
    "custom-feature",               // Name
    "Description of the test case", // Description
    CorpusFormat::Bash,             // Format
    CorpusTier::Standard,           // Tier
    r#"fn main() { /* Rust source */ }"#, // Input
    "expected_output_pattern",      // Output pattern
)
```

## Adversarial Testing (Tier 4)

Tier 4 entries are intentionally crafted to expose transpiler bugs. In v6.61.0, these found and fixed:

1. **format! macro bug**: `format!("{}", x)` was not transpiled correctly
2. **Assignment expression bug**: `x = x + 1` inside complex expressions failed
3. **Arithmetic command substitution**: `$(( ... ))` inside `$()` produced invalid output

## Best Practices

- Run the full corpus before any release: `bashrs corpus`
- Add Tier 4 adversarial entries when you find edge cases
- Target 100% pass rate across all tiers before shipping
- Use `--format` to validate specific transpilation targets

## See Also

- [Probar Testing](./probar-testing.md)
- [Property Testing](./property-testing.md)
- [CLI Commands Reference](../reference/cli.md)
