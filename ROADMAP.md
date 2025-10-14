# Rash (bashrs) Extreme Quality Roadmap

```yaml
project:
  name: "Rash (bashrs)"
  description: "Bidirectional shell safety tool using REAL Rust (not a DSL)"
  repository: "https://github.com/paiml/bashrs"
  current_version: "1.3.0"
  status: "production_ready"
  quality_grade: "A+"

workflows:
  primary:
    name: "Rust → Safe Shell"
    status: "production_ready"
    description: "Write actual Rust code, test with standard Rust tooling, then transpile to provably safe, deterministic POSIX shell scripts"
    features:
      - "Write REAL Rust code (not a DSL!)"
      - "Use standard Rust tooling: cargo, rustc, clippy"
      - "Test with cargo test BEFORE generating shell"
      - "Get deterministic, idempotent shell output"
      - "Guaranteed safe against injection attacks"
      - "POSIX compliant (passes shellcheck)"

  secondary:
    name: "Bash → Rust → Purified Bash"
    status: "functional"
    description: "Ingest messy bash scripts, convert to Rust with tests, then transpile to purified, safe bash"
    features:
      - "Parse legacy bash (with $RANDOM, timestamps, non-idempotent code)"
      - "Convert to Rust + generate comprehensive tests"
      - "Transpile to purified bash (deterministic, idempotent, safe)"
      - "Cleans up existing bash scripts via safety pipeline"

releases:
  v1_3_0:
    version: "1.3.0"
    date: "2025-10-14"
    status: "released"
    achievement: "Mutation Testing Excellence - 100% kill rate on is_string_value"
    highlights:
      - "Sprint 26: 96.6% mutation kill rate (IR module)"
      - "Sprint 26.1: 100% kill rate on is_string_value function"
      - "Line 523 mutant caught with improved test"
      - "4 new mutation-killing tests added"
    metrics:
      tests_passing: "813/813 (100%)"
      mutation_kill_rate_is_string_value: "100% (3/3 mutants)"
      mutation_kill_rate_ir_module: "96.6% (28/29 mutants)"
      property_tests: "52 properties (~26,000+ cases)"
      code_coverage_core: "85.36%"
      code_coverage_total: "82.18%"
      transpile_time: "19.1µs"

  v1_2_1:
    version: "1.2.1"
    date: "2025-10-13"
    status: "released"
    achievement: "Audit fixes and property test improvements"
    highlights:
      - "Sprint 26.5: Fixed duplicate function names in property tests (45 min)"
      - "BUG-001: Empty functions fixed (1 hour)"
      - "BUG-002: Parse error in backup-clean.rs fixed (30 min)"
      - "809/810 tests passing"

  v1_0_0:
    version: "1.0.0"
    date: "2025-10-01"
    status: "released"
    achievement: "First stable production release"
    highlights:
      - "Test Generator Implementation Complete"
      - "Integration Testing Framework"
      - "756 tests passing (752 unit/property + 4 integration)"
      - "A+ Quality Grade"

current_metrics:
  tests:
    total: 838
    passing: 838
    pass_rate: "100%"
    ignored: 42
    property_tests: 52
    property_cases: "~26,000+"
    integration_tests: 4
    sprint_27a_tests: 10
    sprint_27b_tests: 12

  mutation_testing:
    is_string_value_kill_rate: "100% (3/3)"
    ir_module_kill_rate: "96.6% (28/29)"
    target: "≥90%"
    status: "exceeds_target"

  coverage:
    core_line: "85.36%"
    core_function: "88.65%"
    core_region: "86.88%"
    total_line: "82.18%"
    target: ">85%"
    status: "target_achieved"

  complexity:
    median_cyclomatic: 1.0
    median_cognitive: 0.0
    top_function: 15
    target: "<10"
    status: "excellent"

  performance:
    transpile_simple: "19.1µs"
    target: "<10ms"
    ratio: "523x better than target"
    status: "exceeds"

  quality_gates:
    shellcheck_pass_rate: "100%"
    shellcheck_tests: 24
    determinism_tests: 11
    edge_cases_fixed: "11/11 (100%)"

sprints:
  sprint_26_1:
    name: "Perfect Mutation Kill Rate - EXTREME TDD"
    status: "complete"
    duration: "45 minutes"
    priority: "P1_HIGH"
    philosophy: "自働化 (Jidoka) - Build quality in, never settle for good enough"
    achievement: "100% MUTATION KILL RATE ACHIEVED"

    challenge:
      description: "After Sprint 26, one mutant survived (line 523)"
      location: "rash/src/ir/mod.rs:523"
      mutant: "Replace && with || in is_string_value"
      root_cause: "Test was too indirect - checked IR conversion success, not specific behavior"

    solution:
      technique: "Use float strings to expose logic difference"
      test_input: "\"123.5\" (parses as f64 but not i64)"
      correct_logic: "true && false = false → NOT a string → uses NumEq ✅"
      mutated_logic: "true || false = true → IS a string (WRONG!) → uses StrEq ✗"

    results:
      kill_rate: "100% (3/3 mutants)"
      mutants_caught:
        - "Line 520: replace is_string_value -> bool with true"
        - "Line 520: replace is_string_value -> bool with false"
        - "Line 523: replace && with || in is_string_value"
      test_time: "3m 50s"

    files_modified:
      - path: "rash/src/ir/tests.rs"
        change: "Completely rewrote test_is_string_value_requires_both_parse_failures"
        improvement: "Now asserts IR uses NumEq (not StrEq) for float strings"

    success_criteria:
      all_achieved: true
      criteria:
        - "100% mutation kill rate on is_string_value (3/3)"
        - "Line 523 mutant confirmed caught"
        - "Test directly checks behavior affected by mutation"
        - "Completed in 45 minutes"
        - "Toyota Way principles applied"

  sprint_26:
    name: "Mutation Testing Excellence - EXTREME TDD"
    status: "complete"
    duration: "2 hours"
    priority: "P1_HIGH"
    philosophy: "自働化 (Jidoka) - Build quality in through mutation testing"
    achievement: "96.6% MUTATION KILL RATE - EXCEEDS ≥90% TARGET"

    baseline:
      kill_rate: "86.2% (25/29)"
      missed_mutants: 4

    targeted_mutants:
      - "Line 434: IrConverter::analyze_command_effects returns Default::default()"
      - "Line 437: Delete curl|wget match arm"
      - "Line 440: Delete echo|printf match arm"
      - "Line 523: Replace && with || in is_string_value"

    results:
      kill_rate: "96.6% (28/29)"
      improvement: "+10.4 percentage points"
      target_exceeded: "+6.6 percentage points"
      mutants_killed: "3/4 targeted (75%)"

    tests_added:
      - "test_ir_converter_analyze_command_effects_used"
      - "test_ir_converter_wget_command_effect"
      - "test_ir_converter_printf_command_effect"
      - "test_is_string_value_requires_both_parse_failures (improved in Sprint 26.1)"

    success_criteria:
      all_achieved: true
      criteria:
        - "Mutation kill rate ≥90% (achieved 96.6%)"
        - "813/813 tests passing"
        - "No test failures or regressions"
        - "Completed in 2 hours"

  sprint_26_5:
    name: "Property Test Fix - EXTREME TDD"
    status: "complete"
    duration: "45 minutes"
    priority: "P0_CRITICAL"
    philosophy: "自働化 (Jidoka) - Build quality in"
    achievement: "DUPLICATE FUNCTION NAMES FIXED"

    problem:
      description: "prop_valid_scripts_analyze_successfully failing"
      root_cause: "Generator created multiple functions named '_'"
      location: "rash/src/bash_parser/generators.rs:169"

    solution:
      technique: "HashSet-based deduplication in bash_script() generator"
      implementation: "Filter out duplicate function names in prop_map closure"

    results:
      tests_passing: "809/809"
      tests_ignored: 42
      mutation_testing: "unblocked"

    success_criteria:
      all_achieved: true
      criteria:
        - "Property test generates valid bash scripts (no duplicates)"
        - "809/809 tests passing (42 ignored)"
        - "Mutation testing baseline unblocked"
        - "Completed in 0.75 hours (<2 hour target)"

  sprint_25:
    name: "Test Generator & Integration Testing - EXTREME TDD"
    status: "complete"
    achievement: "AUTOMATIC TEST GENERATION IMPLEMENTED"
    features:
      - "Test Generator Module (unit, property, doctest, mutation config)"
      - "Integration Tests (4 comprehensive end-to-end tests)"
      - "Bug fixes (doctest extraction, Rust code generation)"
    results:
      tests_passing: "756/756 (100%)"
      quality_grade: "A+"
      release: "v1.0.0-rc3 → v1.0.0"

  sprint_23:
    name: "Property Test Enhancement - EXTREME TDD"
    status: "complete"
    achievement: "52 PROPERTIES - TARGET EXCEEDED"
    results:
      property_tests: 52
      property_cases: "~26,000+"
      tests_passing: "603/603 (100%)"
      release: "v0.9.2"

  sprint_24:
    name: "Mutation Testing Analysis - EXTREME TDD"
    status: "complete"
    achievement: "MUTATION TESTING BASELINE ESTABLISHED"
    results:
      mutation_tests_added: 8
      mutants_analyzed: 47
      kill_rate_baseline: "83%"
      tests_passing: "593/593 (100%)"
      release: "v0.9.1"

sprint_history:
  - sprint: 1
    focus: "Critical bug fixes"
    results: "5 bugs, 22 property tests"
  - sprint: 2
    focus: "Quality gates"
    results: "24 ShellCheck tests, determinism"
  - sprint: 3
    focus: "Security hardening"
    results: "27 adversarial tests, injection prevention"
  - sprint: 4
    focus: "Parser fixes"
    results: "100% test pass rate"
  - sprint: 5
    focus: "Coverage infrastructure"
    results: "BLOCKED → RESOLVED"
  - sprint: 7
    focus: "Complexity reduction"
    results: "96% cognitive complexity reduction"
  - sprint: 8
    focus: "Parse refactoring"
    results: "Cognitive 35→5, 86% reduction"
  - sprint: 9
    focus: "Coverage enhancement"
    results: "85.36% core coverage achieved"
  - sprint: 10
    focus: "Edge case fixes + MCP server"
    results: "5/11 fixed, MCP operational"
  - sprint: 11
    focus: "P2 edge cases"
    results: "Arithmetic + returns fixed, 7/11 total"
  - sprint: 12
    focus: "Documentation & v0.4.0 release"
    results: "CHANGELOG, README, crates.io"
  - sprint: 13-15
    focus: "Performance benchmarks"
    results: "19.1µs confirmed, docs updated"
  - sprint: 16
    focus: "For loops implementation"
    results: "TICKET-5008, 8/11 edge cases"
  - sprint: 18
    focus: "Property test expansion"
    results: "17→24 tests, +7 new"
  - sprint: 19
    focus: "Match expressions"
    results: "TICKET-5009, 9/11 edge cases"
  - sprint: 20
    focus: "11/11 edge cases + Mutation testing"
    results: "100% edge case completion"
  - sprint: 21
    focus: "While loops"
    results: "TICKET-6001, break/continue support"
  - sprint: 22
    focus: "Standard library"
    results: "6 stdlib functions, predicate support"
  - sprint: 23
    focus: "Property test enhancement"
    results: "52 properties, 26,000+ cases"
  - sprint: 24
    focus: "Mutation testing analysis"
    results: "83% kill rate baseline, 8 targeted tests"
  - sprint: 25
    focus: "Test generator & integration testing"
    results: "Automatic test generation, 756 tests"
  - sprint: "v1.0.0"
    focus: "STABLE RELEASE"
    results: "First production release, published to GitHub"
  - sprint: 26.5
    focus: "Property test fix"
    results: "Duplicate function names, P0 CRITICAL, 45 min"
  - sprint: 26
    focus: "Mutation testing excellence"
    results: "96.6% kill rate, ≥90% target exceeded"
  - sprint: 26.1
    focus: "Perfect mutation kill rate"
    results: "100% on is_string_value, line 523 caught, 45 min"
  - sprint: 27a
    focus: "Environment variables support"
    results: "env() and env_var_or() functions, 10 new tests, 824 total passing"
  - sprint: 27b
    focus: "Command-line arguments support"
    results: "arg(), args(), arg_count() functions, 12 new tests, 838 total passing"

project_goals:
  critical_invariants:
    - name: "POSIX compliance"
      requirement: "Every generated script must pass shellcheck -s sh"
      status: "enforced"
    - name: "Determinism"
      requirement: "Same Rust input must produce byte-identical shell output"
      status: "enforced"
    - name: "Safety"
      requirement: "No injection vectors in generated scripts"
      status: "enforced"
    - name: "Idempotency"
      requirement: "Operations safe to re-run"
      status: "enforced"

toyota_way_principles:
  jidoka:
    name: "自働化 - Build Quality In"
    practices:
      - "EXTREME TDD methodology (RED-GREEN-REFACTOR)"
      - "Zero defects policy (100% test pass rate)"
      - "Quality gates enforced (complexity <10)"
      - "Mutation testing for test quality"

  hansei:
    name: "反省 - Reflection & Root Cause Analysis"
    practices:
      - "Five Whys analysis on blockers"
      - "Deep analysis of surviving mutants"
      - "Root cause identification before fixes"

  kaizen:
    name: "改善 - Continuous Improvement"
    practices:
      - "96% complexity reduction achieved"
      - "Never settled for 96.6%, pursued 100%"
      - "Continuous test quality improvement"

  genchi_genbutsu:
    name: "現地現物 - Direct Observation"
    practices:
      - "Test against real shells (dash, ash, busybox sh, bash)"
      - "Profile actual scenarios"
      - "Measure with real tools (pmat, cargo-llvm-cov, criterion)"

quality_achievements:
  code_quality:
    - "Top 2 complex functions refactored (cognitive 112→4)"
    - "All functions <10 complexity"
    - "EXTREME TDD methodology proven effective"

  test_quality:
    - "813 unit tests (100% pass rate)"
    - "52 property tests (~26,000+ cases)"
    - "4 integration tests"
    - "11 idempotence tests"
    - "11 unicode tests"
    - "24 ShellCheck tests"
    - "8 mutation coverage tests"
    - "Test Generator fully operational"

  infrastructure:
    - "make coverage - HTML coverage report (just works)"
    - "make test - Runs ALL test types"
    - "make test-all - Comprehensive suite"
    - "make mutants - Mutation testing ready"
    - "CI/CD coverage job (two-phase LLVM pattern)"

current_sprint:
  sprint_27c:
    name: "Sprint 27c - Exit Code Handling (NEXT)"
    status: "pending"
    priority: "P1_HIGH"
    duration: "1-2 hours"
    description: "Implement $? support for exit code handling"

previous_sprint:
  sprint_27b:
    name: "Sprint 27b - Command-Line Arguments"
    status: "complete"
    priority: "P1_HIGH"
    duration: "2-3 hours"
    actual_duration: "~2.5 hours (RED + GREEN phases)"
    philosophy: "自働化 (Jidoka) - Build quality in through EXTREME TDD"
    parent_sprint: "Sprint 27 - Core Shell Features Enhancement"
    achievement: "COMMAND-LINE ARGUMENTS IMPLEMENTED"

    scope:
      focus: "Command-line argument access"
      completed:
        - "arg(position) stdlib function"
        - "args() stdlib function (all arguments)"
        - "arg_count() stdlib function"
        - "Safe $1, $2, $@, $# syntax generation"
        - "Position validation (must be >= 1)"
      deferred:
        - "Exit code handling ($?) - Sprint 27c"
        - "Subshell support - Sprint 27d"
        - "Pipe operator support - Sprint 27e"

    results:
      tests_passing: "838/838 (100%)"
      new_tests_added: 12
      phases_complete: "RED + GREEN"
      refactor_phase: "optional"
      files_modified: 6
      lines_changed: "+223/-17"

    implementation:
      - "IR: Added ShellValue::Arg and ShellValue::ArgCount variants"
      - "Stdlib: Registered arg/args/arg_count functions"
      - "Converter: Added AST→IR transformation with position validation"
      - "Emitter: Generates properly quoted $1, $@, $# shell syntax"

    quality:
      test_errors: 0
      clippy_warnings: 0
      security: "Position validation prevents $0 confusion (position >= 1)"
      notes: "Zero errors during implementation (improvement over Sprint 27a)"

    specification: "docs/specifications/SPRINT_27B.md"
    completion_report: ".quality/sprint27b-complete.md"

  sprint_27a:
    name: "Sprint 27a - Environment Variables"
    status: "complete"
    priority: "P1_HIGH"
    duration: "2-3 hours"
    actual_duration: "~3 hours (RED + GREEN phases)"
    philosophy: "自働化 (Jidoka) - Build quality in through EXTREME TDD"
    parent_sprint: "Sprint 27 - Core Shell Features Enhancement"
    achievement: "ENVIRONMENT VARIABLES IMPLEMENTED"

    scope:
      focus: "Environment variable access only"
      completed:
        - "env(var_name) stdlib function"
        - "env_var_or(var_name, default) stdlib function"
        - "Safe ${VAR} and ${VAR:-default} syntax generation"
        - "Security validation (variable name injection prevention)"
      deferred:
        - "Command-line arguments ($1, $2, $@) - Sprint 27b"
        - "Exit code handling ($?) - Sprint 27c"
        - "Subshell support - Sprint 27d"
        - "Pipe operator support - Sprint 27e"

    results:
      tests_passing: "824/824 (100%)"
      new_tests_added: 10
      phases_complete: "RED + GREEN"
      refactor_phase: "optional"
      files_modified: 6
      lines_changed: "+135/-56"

    implementation:
      - "IR: Added ShellValue::EnvVar variant"
      - "Stdlib: Registered env/env_var_or functions"
      - "Converter: Added AST→IR transformation with security validation"
      - "Emitter: Generates quoted ${VAR} shell syntax"

    quality:
      test_fixes: 3
      clippy_warnings: 0
      security: "Variable name validation (alphanumeric + underscore only)"

    specification: "docs/specifications/SPRINT_27A.md"
    completion_report: ".quality/sprint27a-red-complete.md"

  sprint_27b:
    name: "Sprint 27b - Command-Line Arguments"
    status: "complete"
    priority: "P1_HIGH"
    duration: "2-3 hours"
    actual_duration: "~2.5 hours (RED + GREEN phases)"
    philosophy: "自働化 (Jidoka) - Build quality in through EXTREME TDD"
    parent_sprint: "Sprint 27 - Core Shell Features Enhancement"
    achievement: "COMMAND-LINE ARGUMENTS IMPLEMENTED"

    scope:
      focus: "Command-line argument access"
      completed:
        - "arg(position) stdlib function"
        - "args() stdlib function (all arguments)"
        - "arg_count() stdlib function"
        - "Safe $1, $2, $@, $# syntax generation"
        - "Position validation (must be >= 1)"
      deferred:
        - "Exit code handling ($?) - Sprint 27c"
        - "Subshell support - Sprint 27d"
        - "Pipe operator support - Sprint 27e"

    results:
      tests_passing: "838/838 (100%)"
      new_tests_added: 12
      phases_complete: "RED + GREEN"
      refactor_phase: "optional"
      files_modified: 6
      lines_changed: "+223/-17"

    implementation:
      - "IR: Added ShellValue::Arg and ShellValue::ArgCount variants"
      - "Stdlib: Registered arg/args/arg_count functions"
      - "Converter: Added AST→IR transformation with position validation"
      - "Emitter: Generates properly quoted $1, $@, $# shell syntax"

    quality:
      test_errors: 0
      clippy_warnings: 0
      security: "Position validation prevents $0 confusion (position >= 1)"
      notes: "Zero errors during implementation (improvement over Sprint 27a)"

    specification: "docs/specifications/SPRINT_27B.md"
    completion_report: ".quality/sprint27b-complete.md"

next_priorities:
  option_1:
    name: "Sprint 27c: Exit Code Handling"
    priority: "P1_HIGH"
    duration: "1-2 hours"
    description: "Implement $? support for exit code handling"
    depends_on: "Sprint 27b complete"

  option_2:
    name: "Sprint 28: Standard Library Expansion"
    priority: "P2_MEDIUM"
    duration: "2-3 hours"
    description: "Expand stdlib beyond current 6 functions"
    tasks:
      - "String functions (split, replace, uppercase, lowercase)"
      - "Array functions (push, pop, length, join)"
      - "File functions (delete, copy, move, permissions)"
      - "Path functions (basename, dirname, extension)"
    benefits:
      - "More expressive Rust → Shell transpilation"
      - "Reduce manual shell command usage"
      - "Better abstraction layer"

  option_3:
    name: "Sprint 29: Mutation Testing - Full Coverage"
    priority: "P2_MEDIUM"
    duration: "4-6 hours"
    description: "Achieve ≥90% mutation kill rate across ALL modules"
    scope:
      - "Run full mutation testing on parser module"
      - "Run full mutation testing on emitter module"
      - "Run full mutation testing on AST module"
      - "Add targeted tests for surviving mutants"
    target: "≥90% kill rate project-wide"
    current: "100% (is_string_value), 96.6% (IR module)"

  option_4:
    name: "Sprint 30: Error Messages & Diagnostics"
    priority: "P2_MEDIUM"
    duration: "2-3 hours"
    description: "Improve error messages and add diagnostic information"
    tasks:
      - "Enhanced parse error messages with context"
      - "Better transpilation error reporting"
      - "Suggestions for common mistakes"
      - "Color-coded output for CLI"
    benefits:
      - "Better developer experience"
      - "Faster debugging"
      - "More helpful feedback"

  option_5:
    name: "Sprint 31: Bash → Rust Parser Enhancement"
    priority: "P3_LOW"
    duration: "4-6 hours"
    description: "Improve bash-to-rust conversion completeness"
    tasks:
      - "Support more bash constructs (arrays, associative arrays)"
      - "Better handling of complex quoting"
      - "Function export support"
      - "Heredoc improvements"
    benefits:
      - "More legacy scripts convertible"
      - "Better purification pipeline"
      - "Wider compatibility"

tools_and_infrastructure:
  testing:
    - "cargo test - Core test suite"
    - "cargo llvm-cov - Coverage measurement"
    - "cargo mutants - Mutation testing"
    - "cargo clippy - Linting"
    - "shellcheck - Shell script validation"
    - "pmat - Quality analysis with paiml-mcp-agent-toolkit"

  commands:
    - "make test - Core suite (unit + doc + property + examples)"
    - "make test-all - Comprehensive (adds shells + determinism)"
    - "make coverage - HTML coverage report"
    - "make mutants - Mutation testing"
    - "pmat verify - Verify transpiler correctness"
    - "pmat test - Test generated scripts"
    - "pmat analyze complexity - Complexity analysis"
    - "pmat quality-score - Overall quality score"

documentation:
  quality_reports:
    - ".quality/sprint1-complete.md"
    - ".quality/sprint2-complete.md"
    - ".quality/sprint3-complete.md"
    - ".quality/sprint4-complete.md"
    - ".quality/sprint7-ticket4001-complete.md"
    - ".quality/sprint16-18-complete.md"
    - ".quality/sprint19-complete.md"

  specifications:
    - "docs/specifications/COVERAGE.md - Two-phase LLVM coverage pattern"
    - "docs/specifications/MUTATION_TESTING.md - Sprint 26 specification"

status:
  current_version: "v1.3.0"
  release_status: "RELEASED ✅"
  production_ready: true
  next_version: "v1.4.0 (planned)"
  quality_score: "⭐⭐⭐⭐⭐ 5/5 (A+ Grade)"
```
