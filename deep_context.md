# Deep Context Analysis

## Executive Summary

Generated: 2025-06-04 01:16:36.429829855 UTC
Version: 0.21.0
Analysis Time: 0.03s
Cache Hit Rate: 0.0%

## Quality Scorecard

- **Overall Health**: ✅ (88.5/100)
- **Maintainability Index**: 95.1
- **Technical Debt**: 1.0 hours estimated

## Project Structure

```
└── /
    ├── README.md
    ├── LICENSE
    ├── Cargo.toml
    ├── .gitignore
    ├── test_output.sh
    ├── TESTING_REPORT.md
    ├── dependency-graph.mmd
    ├── rash-analysis.json
    ├── test-project/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── main.rs
    │   └── rash.toml
    ├── docs/
    │   └── rash-spec.md
    ├── rustfmt.toml
    ├── .git/
    ├── examples/
    │   ├── installer.rs
    │   ├── simple.rs
    │   ├── basic.rs
    │   ├── debug.rs
    │   └── minimal.rs
    ├── Cargo.lock
    ├── rash/
    │   ├── proptest-regressions/
    │   │   └── services/
    │   │       └── tests.txt
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── bin/
    │   │   │   └── rash.rs
    │   │   ├── ir/
    │   │   │   ├── mod.rs
    │   │   │   ├── effects.rs
    │   │   │   ├── tests.rs
    │   │   │   └── shell_ir.rs
    │   │   ├── cli/
    │   │   │   ├── mod.rs
    │   │   │   ├── args.rs
    │   │   │   └── commands.rs
    │   │   ├── verifier/
    │   │   │   ├── mod.rs
    │   │   │   └── properties.rs
    │   │   ├── ast/
    │   │   │   ├── mod.rs
    │   │   │   ├── visitor.rs
    │   │   │   ├── restricted.rs
    │   │   │   ├── tests.rs
    │   │   │   └── restricted_test.rs
    │   │   ├── models/
    │   │   │   ├── mod.rs
    │   │   │   ├── config.rs
    │   │   │   └── error.rs
    │   │   ├── emitter/
    │   │   │   ├── mod.rs
    │   │   │   ├── posix.rs
    │   │   │   ├── tests.rs
    │   │   │   └── escape.rs
    │   │   ├── lib.rs
    │   │   └── services/
    │   │       ├── mod.rs
    │   │       ├── tests.rs
    │   │       └── parser.rs
    │   ├── tests/
    │   │   └── integration_tests.rs
    │   └── benches/
    │       ├── transpilation.rs
    │       └── verification.rs
    ├── final-analysis.md
    ├── clippy.toml
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
    │       └── ci.yml
    └── target/

📊 Total Files: 60, Total Size: 322425 bytes
```

## Enhanced AST Analysis

### ./test-project/src/main.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `install` (private) at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./examples/installer.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `main` (private) at line 1
  - `echo` (private) at line 1
  - `mkdir` (private) at line 1
  - `touch` (private) at line 1
  - `concat` (private) at line 1

**Defect Probability:** 0.0%

### ./examples/simple.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `install` (private) at line 1
  - `echo` (private) at line 1
  - `mkdir` (private) at line 1

**Defect Probability:** 0.0%

### ./examples/basic.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `install` (private) at line 1
  - `echo` (private) at line 1

**Defect Probability:** 0.0%

### ./examples/debug.rs

**Language:** rust
**Total Symbols:** 1
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `main` (private) at line 1

**Defect Probability:** 0.0%

### ./examples/minimal.rs

**Language:** rust
**Total Symbols:** 1
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `install` (private) at line 1

**Defect Probability:** 0.0%

### ./rash/src/bin/rash.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `main` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ir/mod.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 5 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `from_ast` (public) at line 1
  - `optimize` (public) at line 1
  - `constant_fold` (private) at line 1
  - `eliminate_dead_code` (private) at line 1
  - `transform_ir` (private) at line 1

**Structs:**
  - `IrConverter` (private) with 0 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ir/effects.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 4 | **Structs:** 1 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `analyze_command_effects` (public) at line 1
  - `test_pure_effect_set` (private) at line 1
  - `test_effect_set_union` (private) at line 1
  - `test_command_effect_analysis` (private) at line 1

**Structs:**
  - `EffectSet` (public) with 1 field (derives: derive) at line 1

**Enums:**
  - `Effect` (public) with 7 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ir/tests.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 16 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_simple_ast_to_ir_conversion` (private) at line 1
  - `test_function_call_to_command` (private) at line 1
  - `test_shell_value_constant_detection` (private) at line 1
  - `test_shell_value_constant_string_extraction` (private) at line 1
  - `test_command_builder` (private) at line 1
  - `test_shell_ir_effects_calculation` (private) at line 1
  - `test_optimization_constant_folding` (private) at line 1
  - `test_optimization_disabled` (private) at line 1
  - `test_if_statement_conversion` (private) at line 1
  - `test_return_statement_conversion` (private) at line 1
  - ... and 6 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ir/shell_ir.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `Command` (public) with 2 fields (derives: derive) at line 1

**Enums:**
  - `ShellIR` (public) with 6 variants at line 1
  - `ShellValue` (public) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/cli/mod.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/cli/args.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Structs:**
  - `Cli` (public) with 4 fields (derives: derive) at line 1

**Enums:**
  - `Commands` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/cli/commands.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 7 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `execute_command` (public) at line 1
  - `build_command` (private) at line 1
  - `check_command` (private) at line 1
  - `init_command` (private) at line 1
  - `verify_command` (private) at line 1
  - `generate_proof` (private) at line 1
  - `normalize_shell_script` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/verifier/mod.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `verify` (public) at line 1
  - `verify_basic` (private) at line 1
  - `verify_strict` (private) at line 1
  - `verify_paranoid` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/verifier/properties.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 18 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `verify_no_command_injection` (public) at line 1
  - `verify_deterministic` (public) at line 1
  - `verify_idempotency` (public) at line 1
  - `verify_resource_safety` (public) at line 1
  - `walk_ir` (private) at line 1
  - `check_command_safety` (private) at line 1
  - `check_value_safety` (private) at line 1
  - `contains_shell_metacharacters` (private) at line 1
  - `is_dangerous_command` (private) at line 1
  - `is_nondeterministic_command` (private) at line 1
  - ... and 8 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ast/mod.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `validate` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ast/visitor.rs

**Language:** rust
**Total Symbols:** 7
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `walk_ast` (public) at line 1
  - `transform_exprs` (public) at line 1
  - `transform_stmt_exprs` (private) at line 1
  - `transform_expr` (private) at line 1

**Traits:**
  - `Visitor` (public) at line 1
  - `VisitorMut` (public) at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ast/restricted.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 0 | **Structs:** 3 | **Enums:** 6 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `RestrictedAst` (public) with 2 fields (derives: derive) at line 1
  - `Function` (public) with 4 fields (derives: derive) at line 1
  - `Parameter` (public) with 2 fields (derives: derive) at line 1

**Enums:**
  - `Type` (public) with 5 variants at line 1
  - `Stmt` (public) with 4 variants at line 1
  - `Expr` (public) with 6 variants at line 1
  - `Literal` (public) with 3 variants at line 1
  - `BinaryOp` (public) with 12 variants at line 1
  - ... and 1 more enums

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ast/tests.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 12 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_restricted_ast_validation` (private) at line 1
  - `test_missing_entry_point` (private) at line 1
  - `test_function_validation` (private) at line 1
  - `test_recursion_detection` (private) at line 1
  - `test_indirect_recursion_detection` (private) at line 1
  - `test_allowed_types` (private) at line 1
  - `test_complex_types_allowed` (private) at line 1
  - `test_expression_validation` (private) at line 1
  - `test_statement_validation` (private) at line 1
  - `test_function_call_collection` (private) at line 1
  - ... and 2 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/ast/restricted_test.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `test_restricted_ast_validation` (private) at line 1
  - `test_missing_entry_point` (private) at line 1
  - `test_function_validation` (private) at line 1
  - `test_recursion_detection` (private) at line 1
  - `test_indirect_recursion_detection` (private) at line 1
  - `test_type_validation` (private) at line 1
  - `test_expression_validation` (private) at line 1
  - `test_statement_validation` (private) at line 1
  - `test_function_call_collection` (private) at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/models/mod.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/models/config.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 1 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Structs:**
  - `Config` (public) with 4 fields (derives: derive) at line 1

**Enums:**
  - `ShellDialect` (public) with 4 variants at line 1
  - `VerificationLevel` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/models/error.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Enums:**
  - `Error` (public) with 9 variants at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/emitter/mod.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `emit` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/emitter/posix.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 3 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_emit_simple_let` (private) at line 1
  - `test_emit_command` (private) at line 1
  - `test_emit_if_statement` (private) at line 1

**Structs:**
  - `PosixEmitter` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/emitter/tests.rs

**Language:** rust
**Total Symbols:** 28
**Functions:** 20 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_simple_let_emission` (private) at line 1
  - `test_command_emission` (private) at line 1
  - `test_if_statement_emission` (private) at line 1
  - `test_sequence_emission` (private) at line 1
  - `test_exit_statement_emission` (private) at line 1
  - `test_shell_value_emission` (private) at line 1
  - `test_concatenation_emission` (private) at line 1
  - `test_command_substitution_emission` (private) at line 1
  - `test_noop_emission` (private) at line 1
  - `test_header_and_footer_structure` (private) at line 1
  - ... and 10 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/emitter/escape.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 11 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `escape_shell_string` (public) at line 1
  - `escape_variable_name` (public) at line 1
  - `escape_command_name` (public) at line 1
  - `is_safe_unquoted` (private) at line 1
  - `is_valid_shell_identifier` (private) at line 1
  - `is_safe_command_name` (private) at line 1
  - `test_escape_simple_string` (private) at line 1
  - `test_escape_string_with_quotes` (private) at line 1
  - `test_variable_name_escaping` (private) at line 1
  - `test_command_name_escaping` (private) at line 1
  - ... and 1 more functions

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/lib.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `transpile` (public) at line 1
  - `check` (public) at line 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/services/mod.rs

**Language:** rust
**Total Symbols:** 1
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/services/tests.rs

**Language:** rust
**Total Symbols:** 25
**Functions:** 21 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_simple_function_parsing` (private) at line 1
  - `test_multiple_functions_parsing` (private) at line 1
  - `test_literal_parsing` (private) at line 1
  - `test_function_call_parsing` (private) at line 1
  - `test_binary_expression_parsing` (private) at line 1
  - `test_method_call_parsing` (private) at line 1
  - `test_return_statement_parsing` (private) at line 1
  - `test_variable_reference_parsing` (private) at line 1
  - `test_parameter_parsing` (private) at line 1
  - `test_return_type_parsing` (private) at line 1
  - ... and 11 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/src/services/parser.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `parse` (public) at line 1
  - `convert_function` (private) at line 1
  - `convert_type` (private) at line 1
  - `convert_block` (private) at line 1
  - `convert_stmt` (private) at line 1
  - `convert_expr` (private) at line 1
  - `convert_literal` (private) at line 1
  - `convert_binary_op` (private) at line 1
  - `convert_unary_op` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/tests/integration_tests.rs

**Language:** rust
**Total Symbols:** 25
**Functions:** 19 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `test_end_to_end_simple_transpilation` (private) at line 1
  - `test_end_to_end_with_verification` (private) at line 1
  - `test_generated_script_execution` (private) at line 1
  - `test_generated_script_with_variables` (private) at line 1
  - `test_different_shell_dialects` (private) at line 1
  - `test_verification_levels` (private) at line 1
  - `test_optimization_effects` (private) at line 1
  - `test_check_function` (private) at line 1
  - `test_complex_nested_structures` (private) at line 1
  - `test_function_calls_translation` (private) at line 1
  - ... and 9 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/benches/transpilation.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `benchmark_parsing` (private) at line 1
  - `benchmark_ir_generation` (private) at line 1
  - `benchmark_optimization` (private) at line 1
  - `benchmark_emission` (private) at line 1
  - `benchmark_end_to_end` (private) at line 1
  - `benchmark_memory_usage` (private) at line 1
  - `benchmark_scalability` (private) at line 1
  - `generate_large_rust_source` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash/benches/verification.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `benchmark_verification_levels` (private) at line 1
  - `benchmark_individual_verifications` (private) at line 1
  - `benchmark_verification_scalability` (private) at line 1
  - `benchmark_verification_with_errors` (private) at line 1
  - `benchmark_effect_analysis` (private) at line 1
  - `generate_complex_rust_for_verification` (private) at line 1
  - `generate_injection_attempt` (private) at line 1
  - `generate_non_deterministic` (private) at line 1
  - `generate_resource_intensive` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash-runtime/build.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `main` (private) at line 1
  - `validate_shell_syntax` (private) at line 1
  - `minify_shell` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash-tests/src/sandbox.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Structs:**
  - `Sandbox` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Defect Probability:** 0.0%

### ./rash-tests/src/lib.rs

**Language:** rust
**Total Symbols:** 1
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Key Imports:**
  - `use statement` at line 1

**Defect Probability:** 0.0%

## Complexity Hotspots

| Function | File | Cyclomatic | Cognitive |
|----------|------|------------|-----------|
| `PosixEmitter::write_runtime` | `./rash/src/emitter/posix.rs` | 30 | 30 |
| `validate_shell_syntax` | `./rash-runtime/build.rs` | 26 | 44 |
| `PosixEmitter::emit_ir` | `./rash/src/emitter/posix.rs` | 24 | 28 |
| `convert_expr` | `./rash/src/services/parser.rs` | 22 | 26 |
| `PosixEmitter::emit_shell_value` | `./rash/src/emitter/posix.rs` | 20 | 42 |
| `convert_binary_op` | `./rash/src/services/parser.rs` | 14 | 14 |
| `IrConverter::convert_stmt` | `./rash/src/ir/mod.rs` | 12 | 13 |
| `Stmt::validate` | `./rash/src/ast/restricted.rs` | 12 | 16 |
| `Expr::validate` | `./rash/src/ast/restricted.rs` | 12 | 14 |
| `PosixEmitter::write_header` | `./rash/src/emitter/posix.rs` | 12 | 12 |

## Code Churn Analysis

**Summary:**
- Total Commits: 55
- Files Changed: 53

**Top Changed Files:**
| File | Commits | Authors |
|------|---------|---------|
| `README.md` | 2 | 1 |
| `.gitignore` | 2 | 1 |
| `docs/rash-spec.md` | 1 | 1 |
| `rash/src/services/tests.rs` | 1 | 1 |
| `rash/tests/integration_tests.rs` | 1 | 1 |
| `final-analysis.md` | 1 | 1 |
| `rash/src/emitter/tests.rs` | 1 | 1 |
| `rash/src/ir/tests.rs` | 1 | 1 |
| `rash/benches/verification.rs` | 1 | 1 |
| `rash/benches/transpilation.rs` | 1 | 1 |

## Technical Debt Analysis

**SATD Summary:**
- Low: 2

## Dead Code Analysis

**Summary:**
- Dead Functions: 0
- Total Dead Lines: 0

## Defect Probability Analysis

**Risk Assessment:**
- Total Defects Predicted: 0
- Defect Density: 0.00 defects per 1000 lines

**High-Risk Hotspots:**
| File:Line | Risk Score | Effort (hours) |
|-----------|------------|----------------|
| `./rash-runtime/build.rs:1` | 0.1 | 0.2 |
| `./rash/src/services/parser.rs:1` | 0.1 | 0.2 |
| `./rash/src/emitter/posix.rs:1` | 0.1 | 0.2 |
| `./rash/src/ast/restricted.rs:1` | 0.1 | 0.1 |
| `./rash/src/ast/visitor.rs:1` | 0.0 | 0.1 |
| `./rash/src/ir/mod.rs:1` | 0.0 | 0.1 |
| `./rash/src/emitter/mod.rs:1` | 0.0 | 0.1 |
| `./rash/src/cli/commands.rs:1` | 0.0 | 0.1 |
| `./rash/src/lib.rs:1` | 0.0 | 0.1 |
| `./rash/src/verifier/mod.rs:1` | 0.0 | 0.1 |

---
Generated by deep-context v0.21.0
