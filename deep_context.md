# Deep Context Analysis

## Executive Summary

Generated: 2025-06-04 16:52:58.671219155 UTC
Version: 0.21.0
Analysis Time: 0.00s
Cache Hit Rate: 0.0%

## Quality Scorecard

- **Overall Health**: ⚠️ (75.0/100)
- **Maintainability Index**: 70.0
- **Technical Debt**: 40.0 hours estimated

## Project Structure

```
└── /
    ├── install.sh
    ├── README.md
    ├── CLAUDE.md
    ├── coverage-html/
    │   └── html/
    │       ├── control.js
    │       ├── coverage/
    │       │   └── home/
    │       │       └── noah/
    │       │           └── src/
    │       │               └── rash/
    │       │                   ├── rash/
    │       │                   │   └── src/
    │       │                   │       ├── bin/
    │       │                   │       │   ├── rash-metrics.rs.html
    │       │                   │       │   ├── quality-dashboard.rs.html
    │       │                   │       │   ├── rash.rs.html
    │       │                   │       │   └── quality-gate.rs.html
    │       │                   │       ├── testing/
    │       │                   │       │   ├── boundary.rs.html
    │       │                   │       │   ├── cross_validation.rs.html
    │       │                   │       │   ├── mod.rs.html
    │       │                   │       │   ├── regression.rs.html
    │       │                   │       │   ├── fuzz.rs.html
    │       │                   │       │   ├── error_injection.rs.html
    │       │                   │       │   ├── stress.rs.html
    │       │                   │       │   ├── mutation.rs.html
    │       │                   │       │   └── coverage.rs.html
    │       │                   │       ├── ir/
    │       │                   │       │   ├── mod.rs.html
    │       │                   │       │   ├── tests.rs.html
    │       │                   │       │   ├── shell_ir.rs.html
    │       │                   │       │   └── effects.rs.html
    │       │                   │       ├── lib.rs.html
    │       │                   │       ├── cli/
    │       │                   │       │   ├── args.rs.html
    │       │                   │       │   └── commands.rs.html
    │       │                   │       ├── verifier/
    │       │                   │       │   ├── mod.rs.html
    │       │                   │       │   └── properties.rs.html
    │       │                   │       ├── ast/
    │       │                   │       │   ├── mod.rs.html
    │       │                   │       │   ├── tests.rs.html
    │       │                   │       │   ├── visitor.rs.html
    │       │                   │       │   └── restricted.rs.html
    │       │                   │       ├── models/
    │       │                   │       │   └── config.rs.html
    │       │                   │       ├── emitter/
    │       │                   │       │   ├── mod.rs.html
    │       │                   │       │   ├── tests.rs.html
    │       │                   │       │   ├── escape.rs.html
    │       │                   │       │   └── posix.rs.html
    │       │                   │       └── services/
    │       │                   │           ├── tests.rs.html
    │       │                   │           └── parser.rs.html
    │       │                   └── rash-tests/
    │       │                       └── src/
    │       │                           └── sandbox.rs.html
    │       ├── style.css
    │       └── index.html
    ├── quality-report.json
    ├── install-rash.rs
    ├── RELEASE_SUMMARY.md
    ├── paiml-analysis-summary.md
    ├── LICENSE
    ├── Cargo.toml
    ├── .gitignore
    ├── test_output.sh
    ├── src/
    │   └── install.rs
    ├── coverage.json
    ├── coverage-report/
    │   └── html/
    │       ├── control.js
    │       ├── coverage/
    │       │   └── home/
    │       │       └── noah/
    │       │           └── src/
    │       │               └── rash/
    │       │                   └── rash/
    │       │                       └── src/
    │       │                           ├── bin/
    │       │                           │   ├── rash-metrics.rs.html
    │       │                           │   ├── quality-dashboard.rs.html
    │       │                           │   ├── rash.rs.html
    │       │                           │   └── quality-gate.rs.html
    │       │                           ├── testing/
    │       │                           │   ├── boundary.rs.html
    │       │                           │   ├── cross_validation.rs.html
    │       │                           │   ├── mod.rs.html
    │       │                           │   ├── regression.rs.html
    │       │                           │   ├── fuzz.rs.html
    │       │                           │   ├── error_injection.rs.html
    │       │                           │   ├── stress.rs.html
    │       │                           │   ├── mutation.rs.html
    │       │                           │   └── coverage.rs.html
    │       │                           ├── ir/
    │       │                           │   ├── mod.rs.html
    │       │                           │   ├── tests.rs.html
    │       │                           │   ├── shell_ir.rs.html
    │       │                           │   └── effects.rs.html
    │       │                           ├── lib.rs.html
    │       │                           ├── cli/
    │       │                           │   ├── args.rs.html
    │       │                           │   ├── tests.rs.html
    │       │                           │   ├── commands.rs.html
    │       │                           │   └── command_tests.rs.html
    │       │                           ├── verifier/
    │       │                           │   ├── mod.rs.html
    │       │                           │   ├── tests.rs.html
    │       │                           │   └── properties.rs.html
    │       │                           ├── ast/
    │       │                           │   ├── mod.rs.html
    │       │                           │   ├── tests.rs.html
    │       │                           │   ├── visitor.rs.html
    │       │                           │   └── restricted.rs.html
    │       │                           ├── models/
    │       │                           │   └── config.rs.html
    │       │                           ├── emitter/
    │       │                           │   ├── mod.rs.html
    │       │                           │   ├── tests.rs.html
    │       │                           │   ├── escape.rs.html
    │       │                           │   └── posix.rs.html
    │       │                           └── services/
    │       │                               ├── tests.rs.html
    │       │                               └── parser.rs.html
    │       ├── style.css
    │       └── index.html
    ├── TESTING_REPORT.md
    ├── .quality/
    ├── coverage-summary.txt
    ├── bootstrap-report.json
    ├── complexity-analysis.json
    ├── QUALITY_SUMMARY.md
    ├── tarpaulin-report.json
    ├── dependency-graph.mmd
    ├── rash-analysis.json
    ├── PROJECT_CONTEXT.md
    ├── test-project/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── main.rs
    │   └── rash.toml
    ├── RELEASE_NOTES.md
    ├── docs/
    │   ├── rash-spec.md
    │   ├── formatter-spec.md
    │   ├── zkp-spec.md
    │   ├── spellcheck-compatability-spec.md
    │   ├── quality-dashboard.md
    │   ├── rash-rigid-verification-spec.md
    │   ├── developer-focus-spec.md
    │   ├── enhanced-build-spec.md
    │   ├── formal-verification-implementation.md
    │   ├── dependency-management.md
    │   ├── continued-spec.md
    │   ├── shellcheck-validation.md
    │   └── proof-inspection-guide.md
    ├── complexity-report.txt
    ├── rustfmt.toml
    ├── .git/
    ├── analyze_complexity.py
    ├── cobertura.xml
    ├── analyze_modules.py
    ├── test_edge
    ├── .idea/
    │   ├── rash.iml
    │   ├── modules.xml
    │   ├── .gitignore
    │   ├── workspace.xml
    │   └── vcs.xml
    ├── Makefile
    ├── examples/
    │   ├── installer.rs
    │   ├── simple.rs
    │   ├── formal_verification.rs
    │   ├── basic.rs
    │   ├── shellcheck_validation.rs
    │   ├── debug.rs
    │   └── minimal.rs
    ├── Cargo.lock
    ├── rash/
    │   ├── proptest-regressions/
    │   │   ├── ast/
    │   │   │   └── tests.txt
    │   │   ├── formal/
    │   │   │   └── proofs.txt
    │   │   └── services/
    │   │       └── tests.txt
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── bin/
    │   │   │   ├── quality-gate.rs
    │   │   │   ├── quality-dashboard.rs
    │   │   │   ├── rash.rs
    │   │   │   └── rash-metrics.rs
    │   │   ├── validation/
    │   │   │   ├── mod.rs
    │   │   │   ├── rules.rs
    │   │   │   ├── tests.rs
    │   │   │   └── pipeline.rs
    │   │   ├── testing/
    │   │   │   ├── fuzz.rs
    │   │   │   ├── mutation_tests.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── mutation.rs
    │   │   │   ├── fuzz_tests.rs
    │   │   │   ├── cross_validation.rs
    │   │   │   ├── regression.rs
    │   │   │   ├── coverage.rs
    │   │   │   ├── stress_tests.rs
    │   │   │   ├── error_injection.rs
    │   │   │   ├── regression_tests.rs
    │   │   │   ├── boundary.rs
    │   │   │   └── stress.rs
    │   │   ├── ir/
    │   │   │   ├── mod.rs
    │   │   │   ├── effects.rs
    │   │   │   ├── tests.rs
    │   │   │   └── shell_ir.rs
    │   │   ├── formatter/
    │   │   │   ├── mod.rs
    │   │   │   ├── transforms.rs
    │   │   │   ├── contract.rs
    │   │   │   ├── engine.rs
    │   │   │   ├── logging.rs
    │   │   │   ├── types.rs
    │   │   │   ├── source_map.rs
    │   │   │   └── dialect.rs
    │   │   ├── cli/
    │   │   │   ├── mod.rs
    │   │   │   ├── args.rs
    │   │   │   ├── tests.rs
    │   │   │   ├── commands.rs
    │   │   │   └── command_tests.rs
    │   │   ├── verifier/
    │   │   │   ├── kani_harnesses.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── properties.rs
    │   │   │   └── tests.rs
    │   │   ├── ast/
    │   │   │   ├── mod.rs
    │   │   │   ├── visitor.rs
    │   │   │   ├── restricted.rs
    │   │   │   ├── tests.rs
    │   │   │   ├── restricted_test.rs
    │   │   │   └── visitor_tests.rs
    │   │   ├── models/
    │   │   │   ├── mod.rs
    │   │   │   ├── config.rs
    │   │   │   └── error.rs
    │   │   ├── emitter/
    │   │   │   ├── mod.rs
    │   │   │   ├── posix.rs
    │   │   │   ├── tests.rs
    │   │   │   └── escape.rs
    │   │   ├── formal/
    │   │   │   ├── kani_harnesses.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── semantics.rs
    │   │   │   ├── tiny_ast.rs
    │   │   │   ├── inspector.rs
    │   │   │   ├── emitter.rs
    │   │   │   ├── abstract_state.rs
    │   │   │   └── proofs.rs
    │   │   ├── lib.rs
    │   │   └── services/
    │   │       ├── mod.rs
    │   │       ├── tests.rs
    │   │       └── parser.rs
    │   ├── build.rs
    │   ├── tests/
    │   │   ├── integration_tests.rs
    │   │   └── exhaustive_tests.rs
    │   └── benches/
    │       ├── transpilation.rs
    │       ├── validation.rs
    │       └── verification.rs
    ├── final-analysis.md
    ├── scripts/
    │   ├── check-coverage.sh
    │   ├── cross-shell-validator.rs
    │   └── verify-determinism.sh
    ├── clippy.toml
    ├── test_deep.rs
    ├── rash-runtime/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── lib.sh
    │   │   └── lib.rs
    │   └── build.rs
    ├── rash-tests/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── sandbox.rs
    │       └── lib.rs
    ├── .github/
    │   └── workflows/
    │       ├── release.yml
    │       ├── quality-monitor.yml
    │       ├── main.yml
    │       ├── test-pipeline.yml
    │       └── ci.yml
    ├── paiml-analysis.json
    └── target/

📊 Total Files: 231, Total Size: 8322149 bytes
```

## Complexity Hotspots

| Function | File | Cyclomatic | Cognitive |
|----------|------|------------|-----------|

## Code Churn Analysis

**Summary:**
- Total Commits: 0
- Files Changed: 0

**Top Changed Files:**
| File | Commits | Authors |
|------|---------|---------|

## Technical Debt Analysis

**SATD Summary:**

## Dead Code Analysis

**Summary:**
- Dead Functions: 0
- Total Dead Lines: 0

## Defect Probability Analysis

**Risk Assessment:**
- Total Defects Predicted: 0
- Defect Density: 0.00 defects per 1000 lines

---
Generated by deep-context v0.21.0
