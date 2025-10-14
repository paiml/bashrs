# Sprint 27a: Environment Variables Support - EXTREME TDD

```yaml
sprint:
  name: "Sprint 27a - Environment Variables Only"
  status: "in_progress"
  duration: "2-3 hours"
  priority: "P1_HIGH"
  philosophy: "自働化 (Jidoka) - Build quality in through EXTREME TDD"
  parent_sprint: "Sprint 27 - Core Shell Features Enhancement"

scope:
  focus: "Environment variable access only"
  deferred:
    - "Command-line arguments ($1, $2, $@) - Sprint 27b"
    - "Exit code handling ($?) - Sprint 27c"
    - "Subshell support - Sprint 27d"
    - "Pipe operator support - Sprint 27e"

objectives:
  primary:
    - "Implement env(var_name) stdlib function"
    - "Implement env_var_or(var_name, default) stdlib function"
    - "Generate safe ${VAR} syntax in shell output"
    - "Maintain 100% test pass rate"
    - "Apply EXTREME TDD (RED-GREEN-REFACTOR)"

  quality_targets:
    - "10-15 new tests (unit + property)"
    - "100% mutation kill rate on new functions"
    - "Zero regressions in existing 813 tests"
    - "All generated shell passes shellcheck"

current_state:
  environment_variables:
    examples: "env() calls exist in examples/environment-setup.rs"
    stdlib: "NOT IMPLEMENTED - functions are stubs only"
    ast: "Expr::Variable exists but unused for env vars"
    ir: "No ShellValue variant for env var access"
    emitter: "No ${VAR} generation logic"
    tests: "No env var tests"

  existing_infrastructure:
    - "stdlib.rs module exists with 6 functions"
    - "IR converter handles function calls"
    - "Emitter generates shell function calls"
    - "Property test framework in place"

technical_design:
  stdlib_functions:
    env:
      signature: "env(var_name: &str) -> String"
      behavior: "Returns value of environment variable, empty string if unset"
      shell_output: "${VAR_NAME}"
      safety: "Proper quoting with double quotes"
      example:
        rust: 'let home = env("HOME");'
        shell: 'home="${HOME}"'

    env_var_or:
      signature: "env_var_or(var_name: &str, default: &str) -> String"
      behavior: "Returns env var value, or default if unset/empty"
      shell_output: "${VAR_NAME:-default_value}"
      safety: "Proper quoting with double quotes"
      example:
        rust: 'let prefix = env_var_or("PREFIX", "/usr/local");'
        shell: 'prefix="${PREFIX:-/usr/local}"'

  implementation_layers:
    layer_1_stdlib:
      file: "rash/src/stdlib.rs"
      changes:
        - "Add env to is_stdlib_function() match"
        - "Add env_var_or to is_stdlib_function() match"
        - "Add StdlibFunction entries for both"

    layer_2_ir:
      file: "rash/src/ir/shell_ir.rs"
      changes:
        - "Add EnvVar variant to ShellValue enum"
        - "Store var_name and optional default value"

    layer_3_converter:
      file: "rash/src/ir/mod.rs"
      changes:
        - "Detect env() calls in convert_expr()"
        - "Convert to ShellValue::EnvVar"
        - "Handle both env() and env_var_or()"

    layer_4_emitter:
      file: "rash/src/emitter/mod.rs"
      changes:
        - "Generate ${VAR} for env()"
        - "Generate ${VAR:-default} for env_var_or()"
        - "Proper quoting for safety"

  test_strategy:
    red_phase:
      - "test_stdlib_env_function_recognized"
      - "test_stdlib_env_var_or_function_recognized"
      - "test_env_call_converts_to_ir"
      - "test_env_var_or_call_converts_to_ir"
      - "test_env_emits_dollar_brace_syntax"
      - "test_env_var_or_emits_with_default"
      - "test_env_var_quoted_for_safety"
      - "test_env_integration_end_to_end"

    green_phase:
      - "Implement stdlib additions"
      - "Implement IR variant"
      - "Implement converter logic"
      - "Implement emitter logic"
      - "All tests pass"

    refactor_phase:
      - "Extract helper functions if needed"
      - "Improve error messages"
      - "Add inline documentation"
      - "Run clippy and fix warnings"

  property_tests:
    - "prop_env_calls_are_safe (no injection)"
    - "prop_env_vars_always_quoted"
    - "prop_env_var_or_preserves_default"

  examples_to_update:
    - "examples/environment-setup.rs - Already uses env()"
    - "examples/node-installer.rs - Could use env_var_or()"
    - "examples/rust-installer.rs - Could use env_var_or()"

success_criteria:
  all_must_pass:
    - "env() function callable from Rust code"
    - "env_var_or() function callable from Rust code"
    - "Generated shell uses ${VAR} syntax"
    - "Generated shell uses ${VAR:-default} syntax"
    - "All 813 existing tests still pass"
    - "10-15 new tests added and passing"
    - "Property tests validate safety"
    - "examples/environment-setup.rs transpiles successfully"
    - "Shellcheck passes on generated output"
    - "No clippy warnings"

risks_and_mitigations:
  risk_1:
    description: "Variable name injection"
    example: 'env("'; rm -rf /; #")'
    mitigation: "Validate var names are alphanumeric + underscore only"
    test: "test_env_rejects_invalid_var_names"

  risk_2:
    description: "Default value injection in env_var_or"
    example: 'env_var_or("VAR", "\"; rm -rf /; echo \"")'
    mitigation: "Properly escape default values"
    test: "test_env_var_or_escapes_default"

  risk_3:
    description: "Breaking existing tests"
    mitigation: "Run full test suite after each change"
    test: "make test (all 813 tests)"

toyota_way_principles:
  jidoka:
    application: "EXTREME TDD - Write failing test first, then implement"
    practice: "RED-GREEN-REFACTOR cycle for every feature"

  hansei:
    application: "Review each test failure to understand root cause"
    practice: "If test fails unexpectedly, analyze why before fixing"

  kaizen:
    application: "Start with simplest implementation, improve iteratively"
    practice: "env() first, then env_var_or(), then safety enhancements"

  genchi_genbutsu:
    application: "Test against real shells (dash, bash, ash)"
    practice: "Verify generated scripts work in actual environments"

timeline:
  phase_1_red:
    duration: "30 minutes"
    tasks:
      - "Write 8 failing unit tests"
      - "Write 2-3 failing property tests"
      - "Verify all tests fail for right reasons"

  phase_2_green:
    duration: "60-90 minutes"
    tasks:
      - "Add env/env_var_or to stdlib.rs"
      - "Add ShellValue::EnvVar to IR"
      - "Implement converter logic"
      - "Implement emitter logic"
      - "Fix security issues (var name validation, escaping)"
      - "All tests pass"

  phase_3_refactor:
    duration: "30 minutes"
    tasks:
      - "Extract helper functions"
      - "Add documentation"
      - "Run clippy, fix warnings"
      - "Verify examples work"

  phase_4_documentation:
    duration: "30 minutes"
    tasks:
      - "Update ROADMAP.md"
      - "Update CHANGELOG.md"
      - "Create Sprint 27a completion report"
      - "Commit and push changes"

deliverables:
  code:
    - "rash/src/stdlib.rs - env() and env_var_or() added"
    - "rash/src/ir/shell_ir.rs - EnvVar variant added"
    - "rash/src/ir/mod.rs - Converter logic added"
    - "rash/src/emitter/mod.rs - Shell generation added"

  tests:
    - "rash/src/stdlib/tests.rs - 2-3 unit tests"
    - "rash/src/ir/tests.rs - 3-4 unit tests"
    - "rash/src/emitter/tests.rs - 3-4 unit tests"
    - "rash/src/property_tests.rs - 2-3 property tests"

  documentation:
    - "docs/specifications/SPRINT_27A.md - This spec"
    - ".quality/sprint27a-complete.md - Completion report"
    - "ROADMAP.md - Updated with Sprint 27a"
    - "CHANGELOG.md - v1.4.0 prep entry"

next_steps:
  sprint_27b:
    name: "Command-Line Arguments ($1, $2, $@)"
    duration: "2-3 hours"
    depends_on: "Sprint 27a complete"

  sprint_27c:
    name: "Exit Code Handling ($?)"
    duration: "1-2 hours"
    depends_on: "Sprint 27b complete"

  sprint_27d:
    name: "Subshell Support"
    duration: "2-3 hours"
    depends_on: "Sprint 27c complete"

  sprint_27e:
    name: "Pipe Operator Support"
    duration: "2-3 hours"
    depends_on: "Sprint 27d complete"

references:
  posix_spec: "https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_02"
  shellcheck: "https://www.shellcheck.net/"
  bash_manual: "https://www.gnu.org/software/bash/manual/bash.html#Shell-Parameter-Expansion"
```

## Implementation Checklist

### Phase 1: RED (Write Failing Tests)

- [ ] `test_stdlib_env_function_recognized` - Assert `is_stdlib_function("env")` returns true
- [ ] `test_stdlib_env_var_or_function_recognized` - Assert `is_stdlib_function("env_var_or")` returns true
- [ ] `test_env_call_converts_to_ir` - Parse `env("HOME")` and verify IR has EnvVar variant
- [ ] `test_env_var_or_call_converts_to_ir` - Parse `env_var_or("PREFIX", "/usr")` and verify IR
- [ ] `test_env_emits_dollar_brace_syntax` - Emit `env("HOME")` and verify output is `"${HOME}"`
- [ ] `test_env_var_or_emits_with_default` - Emit `env_var_or()` and verify `"${VAR:-default}"`
- [ ] `test_env_var_quoted_for_safety` - Verify all env var expansions are quoted
- [ ] `test_env_integration_end_to_end` - Full transpilation test from Rust to shell
- [ ] `test_env_rejects_invalid_var_names` - Security: reject invalid var names
- [ ] `test_env_var_or_escapes_default` - Security: escape default values
- [ ] `prop_env_calls_are_safe` - Property test: no injection possible
- [ ] `prop_env_vars_always_quoted` - Property test: quotes always present

### Phase 2: GREEN (Make Tests Pass)

- [ ] Add `"env"` and `"env_var_or"` to `is_stdlib_function()` in `stdlib.rs`
- [ ] Add `StdlibFunction` entries for both functions in `STDLIB_FUNCTIONS`
- [ ] Add `EnvVar { name: String, default: Option<String> }` to `ShellValue` enum
- [ ] Implement converter logic in `convert_function_call()` to detect env calls
- [ ] Implement emitter logic in `emit_value()` to generate `${VAR}` syntax
- [ ] Add var name validation (alphanumeric + underscore only)
- [ ] Add default value escaping for safety
- [ ] Run `cargo test` - verify all 813 + new tests pass

### Phase 3: REFACTOR (Clean Up)

- [ ] Extract `validate_var_name()` helper function
- [ ] Extract `escape_default_value()` helper function
- [ ] Add inline documentation for new code
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt` to format code
- [ ] Verify `examples/environment-setup.rs` transpiles successfully
- [ ] Run shellcheck on generated output

### Phase 4: DOCUMENTATION

- [ ] Update `ROADMAP.md` with Sprint 27a completion
- [ ] Update `CHANGELOG.md` with v1.4.0 prep entry
- [ ] Create `.quality/sprint27a-complete.md` report
- [ ] Update `README.md` with env() example if needed
- [ ] Commit changes with detailed commit message
- [ ] Push to GitHub

---

**Status**: 🔴 **READY TO START** - Specification complete, awaiting RED phase
**Estimated Completion**: 2-3 hours from start
**Quality Target**: ⭐⭐⭐⭐⭐ A+ grade (zero defects, 100% coverage, EXTREME TDD)
