# Provable Contracts

Popperian falsification contracts for bashrs. Each YAML defines claims
that are tested by the corresponding `*_contract_tests.rs` file.

## Contract → Test Mapping

| Contract | Tests | Domain |
|----------|-------|--------|
| transpiler-core-v1.yaml | transpiler_core_contract_tests.rs | Determinism, POSIX, safety |
| encoder-roundtrip-v1.yaml | encoder_roundtrip_contract_tests.rs | Escape/injection prevention |
| parser-soundness-v1.yaml | parser_soundness_contract_tests.rs | Lex/parse/AST correctness |
| linter-security-rules-v1.yaml | linter_security_contract_tests.rs | SEC001-008 |
| linter-det-idem-v1.yaml | linter_det_idem_contract_tests.rs | DET001-004, IDEM001-003 |
| linter-docker-make-v1.yaml | linter_docker_make_contract_tests.rs | DOCKER/MAKE rules |
| purification-pipeline-v1.yaml | purification_contract_tests.rs | Purify invariants |
| property-invariants-v1.yaml | property_falsification_tests.rs | Universal properties (proptest) |
| transpiler-stdlib-v1.yaml | transpile_stdlib_tests.rs | Stdlib function emission |
| linter-coverage-v1.yaml | linter_coverage_contract_tests.rs | Rule coverage per format (GAP-4) |
| linter-lexer-context-v1.yaml | rash/src/linter/lexer_context_tests.rs | Lexer-context false positives: $(( )), nested quoting, heredoc bodies, escaped backticks, Makefile comments, printf formats (PMAT-248, GH-235/237/241/242/252/255/258/261) |
| corpus-registry-v1.yaml | corpus_registry_contract_tests.rs | Corpus is present, whole, and not filler (PMAT-245, #284) |
| training-config-v1.yaml | `corpus::training_config` lib tests (F-TC-001..008) | Training config derives from counts; tests no longer walk the corpus (PMAT-245, CI run 34368312280) |
| corpus-derived-generators-v1.yaml | `corpus::{model_card,ssc_report,contract_validation,baselines}` lib tests (F-CDG-001..007) | Corpus-derived generators take an injected registry; unit tests use tier-1, never the full corpus (PMAT-247) |
| dogfood-selflint-v1.yaml | rash/tests/dogfood_selflint_gate.rs (F-DOG-001..005) | Gate S: bashrs on its own tracked scripts with a per-file error ratchet; unmeasured is a failure (PMAT-263, quorum D4) |

## Running

```sh
make test-contracts       # Run all 120 falsification tests
make validate-contracts   # Tests + inventory dashboard
```

## Adding Contracts

1. Add YAML to `provable-contracts/contracts/`
2. Symlink: `ln -s ../provable-contracts/contracts/NAME.yaml contracts/`
3. Write `rash/tests/NAME_contract_tests.rs`
4. Add to `make test-contracts` target in Makefile
