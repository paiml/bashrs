# Quality Gates Overview

bashrs implements **tiered quality gates** following Toyota Production System principles. Gates are configured via `.pmat-gates.toml` and enforced at three tiers.

## Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Jidoka** | Automation with human touch - ML classifies, human approves |
| **Kaizen** | Continuous improvement - learn from fix acceptance |
| **Mieruka** | Visual management - rich ASCII dashboards |
| **Genchi Genbutsu** | Go and see - SBFL locates actual faults |
| **Poka-yoke** | Error-proofing - confidence scores prevent bad fixes |

## Tier Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    QUALITY GATE TIERS                           │
├─────────────────────────────────────────────────────────────────┤
│  TIER 1 (ON-SAVE)     │ Sub-second checks                      │
│  ─────────────────────┼─────────────────────────────────────── │
│  • Clippy lints       │ Immediate feedback                      │
│  • Complexity checks  │ < 1 second                              │
│  • Basic security     │                                         │
├─────────────────────────────────────────────────────────────────┤
│  TIER 2 (ON-COMMIT)   │ 1-5 minute checks                      │
│  ─────────────────────┼─────────────────────────────────────── │
│  • Full test suite    │ Pre-commit validation                   │
│  • Coverage analysis  │ 85% minimum                             │
│  • SATD detection     │ No new tech debt                        │
├─────────────────────────────────────────────────────────────────┤
│  TIER 3 (NIGHTLY)     │ Hours-long checks                      │
│  ─────────────────────┼─────────────────────────────────────── │
│  • Mutation testing   │ 90% kill rate                           │
│  • Security audit     │ cargo-audit, cargo-deny                 │
│  • Fuzz testing       │ Edge case discovery                     │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration

Create `.pmat-gates.toml` in your project root:

```toml
[tier1]
enabled = true
complexity_threshold = 10
max_lint_warnings = 0

[tier2]
enabled = true
min_coverage = 85.0
satd_allowed = false

[tier3]
enabled = true
mutation_threshold = 90.0
security_audit = true
```

## Using Quality Gates

```rust,ignore
use bashrs::quality::{GateConfig, QualityGate, Tier};

fn main() {
    // Load configuration
    let config = GateConfig::default();

    // Create gate for specific tier
    let gate = QualityGate::new(Tier::OnCommit, config);

    // Run checks
    let result = gate.run_checks();

    if result.passed {
        println!("All quality gates passed!");
    } else {
        println!("Failed gates: {:?}", result.failures);
    }
}
```

## Grade System

Quality is measured using letter grades:

| Grade | Score | Status |
|-------|-------|--------|
| A+ | 97-100 | Excellent |
| A | 93-96 | Great |
| A- | 90-92 | Very Good |
| B+ | 87-89 | Good |
| B | 83-86 | Above Average |
| B- | 80-82 | Acceptable |
| C+ | 77-79 | Below Average |
| C | 73-76 | Needs Work |
| C- | 70-72 | Minimum Passing |
| D | 60-69 | Poor |
| F | 0-59 | Failing |

Grades C- and above are considered passing.
