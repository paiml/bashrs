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
