# Project Context: rust Project

Generated: 2025-06-04 01:08:51 UTC

## Summary

- Files analyzed: 40
- Functions: 177
- Structs: 10
- Enums: 13
- Traits: 2
- Implementations: 18

## Files

### ./test-project/src/main.rs

**Functions:**
- `private fn install` (line 1)

### ./examples/installer.rs

**Functions:**
- `private fn main` (line 1)
- `private fn echo` (line 1)
- `private fn mkdir` (line 1)
- `private fn touch` (line 1)
- `private fn concat` (line 1)

### ./examples/simple.rs

**Functions:**
- `private fn install` (line 1)
- `private fn echo` (line 1)
- `private fn mkdir` (line 1)

### ./examples/basic.rs

**Functions:**
- `private fn install` (line 1)
- `private fn echo` (line 1)

### ./examples/debug.rs

**Functions:**
- `private fn main` (line 1)

### ./examples/minimal.rs

**Functions:**
- `private fn install` (line 1)

### ./rash/src/bin/rash.rs

**Functions:**
- `private fn main` (line 1)

### ./rash/src/ir/mod.rs

**Modules:**
- `pub mod shell_ir` (line 1)
- `pub mod effects` (line 1)
- `private mod tests` (line 1)

**Structs:**
- `private struct IrConverter` (0 fields) (line 1)

**Functions:**
- `pub fn from_ast` (line 1)
- `pub fn optimize` (line 1)
- `private fn constant_fold` (line 1)
- `private fn eliminate_dead_code` (line 1)
- `private fn transform_ir` (line 1)

**Implementations:**
- `impl IrConverter` (line 1)

### ./rash/src/ir/effects.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct EffectSet` (1 fields) (line 1)

**Enums:**
- `pub enum Effect` (7 variants) (line 1)

**Functions:**
- `pub fn analyze_command_effects` (line 1)

**Implementations:**
- `impl EffectSet` (line 1)
- `impl Default for EffectSet` (line 1)
- `impl From for EffectSet` (line 1)
- `impl From for EffectSet` (line 1)

### ./rash/src/ir/tests.rs

**Functions:**
- `private fn test_simple_ast_to_ir_conversion` (line 1)
- `private fn test_function_call_to_command` (line 1)
- `private fn test_shell_value_constant_detection` (line 1)
- `private fn test_shell_value_constant_string_extraction` (line 1)
- `private fn test_command_builder` (line 1)
- `private fn test_shell_ir_effects_calculation` (line 1)
- `private fn test_optimization_constant_folding` (line 1)
- `private fn test_optimization_disabled` (line 1)
- `private fn test_if_statement_conversion` (line 1)
- `private fn test_return_statement_conversion` (line 1)
- `private fn test_binary_expression_conversion` (line 1)
- `private fn test_command_effect_classification` (line 1)
- `private fn test_ir_sequence_effects_aggregation` (line 1)
- `private fn test_nested_ir_effects` (line 1)
- `private fn test_error_handling_in_conversion` (line 1)
- `private fn test_complex_nested_structures` (line 1)

### ./rash/src/ir/shell_ir.rs

**Structs:**
- `pub struct Command` (2 fields) (line 1)

**Enums:**
- `pub enum ShellIR` (6 variants) (line 1)
- `pub enum ShellValue` (5 variants) (line 1)

**Implementations:**
- `impl ShellIR` (line 1)
- `impl Command` (line 1)
- `impl ShellValue` (line 1)

### ./rash/src/cli/mod.rs

**Modules:**
- `pub mod args` (line 1)
- `pub mod commands` (line 1)

### ./rash/src/cli/args.rs

**Structs:**
- `pub struct Cli` (4 fields) (line 1)

**Enums:**
- `pub enum Commands` (4 variants) (line 1)

**Implementations:**
- `impl ValueEnum for VerificationLevel` (line 1)
- `impl ValueEnum for ShellDialect` (line 1)

### ./rash/src/cli/commands.rs

**Functions:**
- `pub fn execute_command` (line 1)
- `private fn build_command` (line 1)
- `private fn check_command` (line 1)
- `private fn init_command` (line 1)
- `private fn verify_command` (line 1)
- `private fn generate_proof` (line 1)
- `private fn normalize_shell_script` (line 1)

### ./rash/src/verifier/mod.rs

**Modules:**
- `pub mod properties` (line 1)

**Functions:**
- `pub fn verify` (line 1)
- `private fn verify_basic` (line 1)
- `private fn verify_strict` (line 1)
- `private fn verify_paranoid` (line 1)

### ./rash/src/verifier/properties.rs

**Modules:**
- `private mod tests` (line 1)

**Functions:**
- `pub fn verify_no_command_injection` (line 1)
- `pub fn verify_deterministic` (line 1)
- `pub fn verify_idempotency` (line 1)
- `pub fn verify_resource_safety` (line 1)
- `private fn walk_ir` (line 1)
- `private fn check_command_safety` (line 1)
- `private fn check_value_safety` (line 1)
- `private fn contains_shell_metacharacters` (line 1)
- `private fn is_dangerous_command` (line 1)
- `private fn is_nondeterministic_command` (line 1)
- `private fn requires_idempotency_check` (line 1)
- `private fn is_network_command` (line 1)
- `private fn is_file_operation` (line 1)
- `private fn check_has_idempotency_guard` (line 1)

### ./rash/src/ast/mod.rs

**Modules:**
- `pub mod restricted` (line 1)
- `pub mod visitor` (line 1)
- `private mod tests` (line 1)

**Functions:**
- `pub fn validate` (line 1)

### ./rash/src/ast/visitor.rs

**Traits:**
- `pub trait Visitor` (line 1)
- `pub trait VisitorMut` (line 1)

**Functions:**
- `pub fn walk_ast` (line 1)
- `pub fn transform_exprs` (line 1)
- `private fn transform_stmt_exprs` (line 1)
- `private fn transform_expr` (line 1)

### ./rash/src/ast/restricted.rs

**Structs:**
- `pub struct RestrictedAst` (2 fields) (line 1)
- `pub struct Function` (4 fields) (line 1)
- `pub struct Parameter` (2 fields) (line 1)

**Enums:**
- `pub enum Type` (5 variants) (line 1)
- `pub enum Stmt` (4 variants) (line 1)
- `pub enum Expr` (6 variants) (line 1)
- `pub enum Literal` (3 variants) (line 1)
- `pub enum BinaryOp` (12 variants) (line 1)
- `pub enum UnaryOp` (2 variants) (line 1)

**Implementations:**
- `impl RestrictedAst` (line 1)
- `impl Function` (line 1)
- `impl Type` (line 1)
- `impl Stmt` (line 1)
- `impl Expr` (line 1)

### ./rash/src/ast/tests.rs

**Functions:**
- `private fn test_restricted_ast_validation` (line 1)
- `private fn test_missing_entry_point` (line 1)
- `private fn test_function_validation` (line 1)
- `private fn test_recursion_detection` (line 1)
- `private fn test_indirect_recursion_detection` (line 1)
- `private fn test_allowed_types` (line 1)
- `private fn test_complex_types_allowed` (line 1)
- `private fn test_expression_validation` (line 1)
- `private fn test_statement_validation` (line 1)
- `private fn test_function_call_collection` (line 1)
- `private fn test_validate_public_api` (line 1)
- `private fn test_invalid_ast_returns_validation_error` (line 1)

### ./rash/src/ast/restricted_test.rs

**Modules:**
- `private mod tests` (line 1)

### ./rash/src/models/mod.rs

**Modules:**
- `pub mod config` (line 1)
- `pub mod error` (line 1)

### ./rash/src/models/config.rs

**Structs:**
- `pub struct Config` (4 fields) (line 1)

**Enums:**
- `pub enum ShellDialect` (4 variants) (line 1)
- `pub enum VerificationLevel` (4 variants) (line 1)

**Implementations:**
- `impl Default for Config` (line 1)

### ./rash/src/models/error.rs

**Enums:**
- `pub enum Error` (9 variants) (line 1)

### ./rash/src/emitter/mod.rs

**Modules:**
- `pub mod posix` (line 1)
- `pub mod escape` (line 1)
- `private mod tests` (line 1)

**Functions:**
- `pub fn emit` (line 1)

### ./rash/src/emitter/posix.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct PosixEmitter` (1 fields) (line 1)

**Implementations:**
- `impl PosixEmitter` (line 1)

### ./rash/src/emitter/tests.rs

**Functions:**
- `private fn test_simple_let_emission` (line 1)
- `private fn test_command_emission` (line 1)
- `private fn test_if_statement_emission` (line 1)
- `private fn test_sequence_emission` (line 1)
- `private fn test_exit_statement_emission` (line 1)
- `private fn test_shell_value_emission` (line 1)
- `private fn test_concatenation_emission` (line 1)
- `private fn test_command_substitution_emission` (line 1)
- `private fn test_noop_emission` (line 1)
- `private fn test_header_and_footer_structure` (line 1)
- `private fn test_runtime_functions_included` (line 1)
- `private fn test_test_expression_emission` (line 1)
- `private fn test_string_escaping` (line 1)
- `private fn test_variable_name_escaping` (line 1)
- `private fn test_command_name_escaping` (line 1)
- `private fn test_shell_value_emission_cases` (line 1)
- `private fn test_complex_nested_emission` (line 1)
- `private fn test_emit_public_api` (line 1)
- `private fn test_different_shell_dialects` (line 1)
- `private fn test_indentation_consistency` (line 1)

### ./rash/src/emitter/escape.rs

**Modules:**
- `private mod tests` (line 1)

**Functions:**
- `pub fn escape_shell_string` (line 1)
- `pub fn escape_variable_name` (line 1)
- `pub fn escape_command_name` (line 1)
- `private fn is_safe_unquoted` (line 1)
- `private fn is_valid_shell_identifier` (line 1)
- `private fn is_safe_command_name` (line 1)

### ./rash/src/lib.rs

**Modules:**
- `pub mod ast` (line 1)
- `pub mod cli` (line 1)
- `pub mod emitter` (line 1)
- `pub mod ir` (line 1)
- `pub mod models` (line 1)
- `pub mod services` (line 1)
- `pub mod verifier` (line 1)

**Functions:**
- `pub fn transpile` (line 1)
- `pub fn check` (line 1)

### ./rash/src/services/mod.rs

**Modules:**
- `pub mod parser` (line 1)
- `private mod tests` (line 1)

### ./rash/src/services/tests.rs

**Functions:**
- `private fn test_simple_function_parsing` (line 1)
- `private fn test_multiple_functions_parsing` (line 1)
- `private fn test_literal_parsing` (line 1)
- `private fn test_function_call_parsing` (line 1)
- `private fn test_binary_expression_parsing` (line 1)
- `private fn test_method_call_parsing` (line 1)
- `private fn test_return_statement_parsing` (line 1)
- `private fn test_variable_reference_parsing` (line 1)
- `private fn test_parameter_parsing` (line 1)
- `private fn test_return_type_parsing` (line 1)
- `private fn test_error_on_no_main_function` (line 1)
- `private fn test_error_on_multiple_main_functions` (line 1)
- `private fn test_error_on_non_function_items` (line 1)
- `private fn test_complex_expression_parsing` (line 1)
- `private fn test_unary_expression_parsing` (line 1)
- `private fn test_type_conversion_edge_cases` (line 1)
- `private fn test_literal_parsing_cases` (line 1)
- `private fn test_error_handling_invalid_syntax` (line 1)
- `private fn test_nested_expression_parsing` (line 1)
- `private fn test_empty_function_body_handling` (line 1)
- `private fn test_parser_maintains_source_information` (line 1)

### ./rash/src/services/parser.rs

**Functions:**
- `pub fn parse` (line 1)
- `private fn convert_function` (line 1)
- `private fn convert_type` (line 1)
- `private fn convert_block` (line 1)
- `private fn convert_stmt` (line 1)
- `private fn convert_expr` (line 1)
- `private fn convert_literal` (line 1)
- `private fn convert_binary_op` (line 1)
- `private fn convert_unary_op` (line 1)

### ./rash/tests/integration_tests.rs

**Functions:**
- `private fn test_end_to_end_simple_transpilation` (line 1)
- `private fn test_end_to_end_with_verification` (line 1)
- `private fn test_generated_script_execution` (line 1)
- `private fn test_generated_script_with_variables` (line 1)
- `private fn test_different_shell_dialects` (line 1)
- `private fn test_verification_levels` (line 1)
- `private fn test_optimization_effects` (line 1)
- `private fn test_check_function` (line 1)
- `private fn test_complex_nested_structures` (line 1)
- `private fn test_function_calls_translation` (line 1)
- `private fn test_error_handling_invalid_source` (line 1)
- `private fn test_shell_escaping_safety` (line 1)
- `private fn test_runtime_functions_included` (line 1)
- `private fn test_script_header_and_footer` (line 1)
- `private fn test_deterministic_output` (line 1)
- `private fn test_large_input_handling` (line 1)
- `private fn test_proof_generation` (line 1)
- `private fn test_concurrent_transpilation` (line 1)
- `private fn test_memory_safety` (line 1)

### ./rash/benches/transpilation.rs

**Functions:**
- `private fn benchmark_parsing` (line 1)
- `private fn benchmark_ir_generation` (line 1)
- `private fn benchmark_optimization` (line 1)
- `private fn benchmark_emission` (line 1)
- `private fn benchmark_end_to_end` (line 1)
- `private fn benchmark_memory_usage` (line 1)
- `private fn benchmark_scalability` (line 1)
- `private fn generate_large_rust_source` (line 1)

### ./rash/benches/verification.rs

**Functions:**
- `private fn benchmark_verification_levels` (line 1)
- `private fn benchmark_individual_verifications` (line 1)
- `private fn benchmark_verification_scalability` (line 1)
- `private fn benchmark_verification_with_errors` (line 1)
- `private fn benchmark_effect_analysis` (line 1)
- `private fn generate_complex_rust_for_verification` (line 1)
- `private fn generate_injection_attempt` (line 1)
- `private fn generate_non_deterministic` (line 1)
- `private fn generate_resource_intensive` (line 1)

### ./rash-runtime/src/lib.rs

### ./rash-runtime/build.rs

**Functions:**
- `private fn main` (line 1)
- `private fn validate_shell_syntax` (line 1)
- `private fn minify_shell` (line 1)

### ./rash-tests/src/sandbox.rs

**Structs:**
- `pub struct Sandbox` (1 fields) (line 1)

**Implementations:**
- `impl Sandbox` (line 1)

### ./rash-tests/src/lib.rs

**Modules:**
- `pub mod sandbox` (line 1)

### ./target/debug/build/rash-runtime-ebb082129b44155d/out/runtime.rs

**Functions:**
- `pub fn get_runtime` (line 1)

---
Generated by paiml-mcp-agent-toolkit
