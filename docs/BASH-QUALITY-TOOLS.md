# Bash Quality Tools - Design Specification

**Version**: 1.0.0 (v6.9.0)
**Status**: Design Phase
**Methodology**: EXTREME TDD

---

## 🎯 Vision

Enable comprehensive quality tooling for bash scripts, providing test, lint, coverage, format, and TDG-style scoring capabilities.

**Goal**: Make bashrs the **complete quality platform** for bash script development.

---

## 📋 Feature Overview

### New CLI Commands

```bash
# Testing
bashrs test <file.sh>              # Run tests on bash script
bashrs test --watch <file.sh>      # Watch mode for continuous testing

# Coverage
bashrs coverage <file.sh>          # Generate coverage report
bashrs coverage --format html      # HTML coverage report
bashrs coverage --min 80           # Enforce minimum coverage

# Formatting
bashrs format <file.sh>            # Format bash script
bashrs format --check <file.sh>    # Check formatting without changes
bashrs format --fix <file.sh>      # Fix formatting issues

# Quality Scoring
bashrs score <file.sh>             # TDG-style quality score
bashrs score --format json         # JSON output
bashrs score --min-grade B+        # Enforce minimum grade

# Comprehensive Check
bashrs check <file.sh>             # Run all checks (test, lint, coverage, format, score)
bashrs check --strict              # Fail on any issue
```

---

## 🧪 1. Test Runner (`bashrs test`)

### Requirements

**Functional**:
- Run inline bash tests (comments with assertions)
- Support property-based testing patterns
- Integration with bats-core (if available)
- Custom test framework (built-in)
- Watch mode for continuous testing

**Quality**:
- >85% code coverage
- >90% mutation score
- All tests pass
- <100ms test discovery

### Test Format (Inline Tests)

```bash
#!/bin/bash
# test_example.sh

my_function() {
    echo "Hello, $1"
}

# TEST: my_function with name
# GIVEN: name="World"
# WHEN: my_function "World"
# THEN: output should be "Hello, World"
test_my_function_basic() {
    result=$(my_function "World")
    [[ "$result" == "Hello, World" ]] || return 1
}

# TEST: my_function with empty string
# GIVEN: name=""
# WHEN: my_function ""
# THEN: output should be "Hello, "
test_my_function_empty() {
    result=$(my_function "")
    [[ "$result" == "Hello, " ]] || return 1
}
```

### Implementation Plan

1. **Test Discovery**:
   - Parse bash file for functions starting with `test_`
   - Extract TEST comments (GIVEN/WHEN/THEN)
   - Build test registry

2. **Test Execution**:
   - Source the script in isolated environment
   - Run each test function
   - Capture output, exit codes, stderr

3. **Test Reporting**:
   - Human-readable format (default)
   - JSON format (for CI/CD)
   - JUnit XML format (for CI integration)

4. **Property-Based Testing** (Future):
   - Generate test cases from properties
   - Shrink failing cases
   - QuickCheck-style testing

### Files to Create

- `rash/src/testing/mod.rs` - Test framework module
- `rash/src/testing/runner.rs` - Test runner
- `rash/src/testing/discovery.rs` - Test discovery
- `rash/src/testing/report.rs` - Test reporting
- `rash/src/cli/commands/test.rs` - CLI handler

---

## 📊 2. Coverage Tracker (`bashrs coverage`)

### Requirements

**Functional**:
- Track line coverage for bash scripts
- Track function coverage
- Track branch coverage (if/else, case)
- Integration with kcov (if available)
- Built-in coverage tracker (fallback)

**Quality**:
- Accurate coverage metrics
- <200ms coverage analysis
- HTML report generation
- JSON output for CI/CD

### Coverage Mechanisms

**Option 1: kcov Integration** (preferred if available):
```bash
kcov --bash-dont-parse-binary-dir output_dir script.sh
```

**Option 2: Built-in Tracker** (fallback):
```bash
# Instrument script with PS4 trap
PS4='+(${BASH_SOURCE}:${LINENO}): ${FUNCNAME[0]:+${FUNCNAME[0]}(): }'
export PS4
set -x

# Run script
bash -x script.sh 2> trace.log

# Parse trace.log to extract coverage
```

### Coverage Formats

1. **Terminal** (default):
   ```
   Coverage Report: script.sh

   Lines:     45/50   (90.0%)  ✅
   Functions: 8/10    (80.0%)  ✅
   Branches:  12/15   (80.0%)  ✅

   Uncovered Lines: 23, 45, 67
   ```

2. **HTML**: Interactive coverage report
3. **JSON**: Machine-readable format
4. **LCOV**: Standard coverage format

### Files to Create

- `rash/src/coverage/mod.rs` - Coverage module
- `rash/src/coverage/tracker.rs` - Coverage tracker
- `rash/src/coverage/kcov.rs` - kcov integration
- `rash/src/coverage/builtin.rs` - Built-in tracker
- `rash/src/coverage/report.rs` - Coverage reporting
- `rash/src/cli/commands/coverage.rs` - CLI handler

---

## 🎨 3. Formatter (`bashrs format`)

### Requirements

**Functional**:
- Format bash scripts consistently
- Integration with shfmt (if available)
- Built-in formatter (fallback)
- Configurable style (Google, K&R, etc.)

**Quality**:
- Idempotent formatting (format(format(x)) == format(x))
- Preserves semantic meaning
- <100ms for typical scripts

### Formatting Rules

1. **Indentation**: 2 or 4 spaces (configurable)
2. **Line Length**: 80 or 100 chars (configurable)
3. **Function Braces**: Same line or next line
4. **Case Statements**: Consistent indentation
5. **Comments**: Preserve and align

### Example Formatting

**Before**:
```bash
function my_func(){
if [ "$1" = "test" ];then
echo "test"
else
echo "other"
fi
}
```

**After**:
```bash
function my_func() {
  if [ "$1" = "test" ]; then
    echo "test"
  else
    echo "other"
  fi
}
```

### Implementation Plan

1. **shfmt Integration** (preferred):
   ```bash
   shfmt -i 2 -ci -bn -sr script.sh
   ```

2. **Built-in Formatter** (fallback):
   - Parse to AST
   - Pretty-print AST with formatting rules
   - Preserve comments and whitespace where appropriate

### Files to Create

- `rash/src/format/mod.rs` - Format module
- `rash/src/format/shfmt.rs` - shfmt integration
- `rash/src/format/builtin.rs` - Built-in formatter
- `rash/src/format/config.rs` - Format configuration
- `rash/src/cli/commands/format.rs` - CLI handler

---

## 📈 4. Quality Scorer (`bashrs score`)

### Requirements

**Functional**:
- TDG-style quality grading (A+ to F)
- Analyze multiple quality dimensions
- Actionable improvement suggestions
- Track quality trends over time

**Quality**:
- Scoring algorithm validated by property tests
- <500ms for typical scripts
- Deterministic scoring

### Scoring Dimensions

#### 1. **Complexity** (30%)
- Cyclomatic complexity
- Cognitive complexity
- Function length
- Nesting depth

#### 2. **Safety** (25%)
- Shellcheck violations
- Quote usage
- Error handling
- Set -euo pipefail usage

#### 3. **Maintainability** (20%)
- Comment density
- Function modularity
- Variable naming
- Code duplication

#### 4. **Testing** (15%)
- Test coverage (if tests exist)
- Test quality
- Assertion density

#### 5. **Documentation** (10%)
- Function documentation
- Usage examples
- README present

### Grading Scale

| Grade | Score Range | Description |
|-------|-------------|-------------|
| **A+** | 95-100 | Exceptional quality |
| **A**  | 90-94  | Excellent quality |
| **A-** | 85-89  | Very good quality |
| **B+** | 80-84  | Good quality |
| **B**  | 75-79  | Above average |
| **B-** | 70-74  | Average |
| **C+** | 65-69  | Below average |
| **C**  | 60-64  | Poor quality |
| **C-** | 55-59  | Very poor |
| **D**  | 50-54  | Severe issues |
| **F**  | <50    | Failing |

### Score Output

```bash
$ bashrs score deploy.sh

Quality Score: B+ (82/100)

Breakdown:
  Complexity:       ██████████████░░░░░░  70/100  (B-)
  Safety:          ████████████████████  100/100 (A+)
  Maintainability: ████████████████░░░░  80/100  (B+)
  Testing:         ██████████░░░░░░░░░░  50/100  (D)
  Documentation:   ███████████████░░░░░  75/100  (B)

Top Issues:
  ⚠️  High cyclomatic complexity in deploy() (15)
  ⚠️  No test coverage found
  💡 Consider extracting helper functions
  💡 Add inline tests with TEST comments

Quality Trend:
  v6.8.0: B+  (82/100)  [current]
  v6.7.0: B   (78/100)  ⬆️ +4 points
  v6.6.0: C+  (69/100)  ⬆️ +9 points
```

### Files to Create

- `rash/src/score/mod.rs` - Scoring module
- `rash/src/score/dimensions.rs` - Scoring dimensions
- `rash/src/score/grading.rs` - Grading algorithm
- `rash/src/score/report.rs` - Score reporting
- `rash/src/score/trends.rs` - Quality trend tracking
- `rash/src/cli/commands/score.rs` - CLI handler

---

## 🔍 5. Comprehensive Check (`bashrs check`)

### Requirements

**Functional**:
- Run all checks in sequence
- Fast-fail or collect all errors
- Parallel execution where possible
- CI/CD friendly output

**Quality**:
- <2 seconds for typical script
- Clear failure reasons
- Exit codes for CI integration

### Check Workflow

```bash
$ bashrs check --strict deploy.sh

Running checks on deploy.sh...

✅ Lint:     0 errors, 2 warnings
✅ Format:   Already formatted
✅ Test:     8/8 tests passed
⚠️  Coverage: 78% (target: 85%)
✅ Score:    B+ (82/100)

Overall: PASS (with warnings)

To fix warnings:
  bashrs lint --fix deploy.sh
  bashrs test --coverage deploy.sh
```

### Files to Create

- `rash/src/cli/commands/check.rs` - Comprehensive check handler

---

## 🏗️ Implementation Phases

### Phase 1: Foundation (Sprint 1)
- [ ] Create module structure
- [ ] Design CLI commands
- [ ] Write failing tests (RED)
- [ ] Basic test runner implementation
- [ ] Basic linter integration

### Phase 2: Core Features (Sprint 2)
- [ ] Coverage tracker (built-in)
- [ ] Formatter (shfmt integration)
- [ ] Quality scorer (basic dimensions)
- [ ] Comprehensive check command

### Phase 3: Advanced Features (Sprint 3)
- [ ] kcov integration
- [ ] HTML coverage reports
- [ ] Quality trend tracking
- [ ] Property-based testing support

### Phase 4: Polish (Sprint 4)
- [ ] Documentation (README, book)
- [ ] Examples and tutorials
- [ ] CI/CD integration guide
- [ ] Performance optimization

---

## 📊 Quality Gates

All new code must meet:

- ✅ >85% code coverage
- ✅ >90% mutation score
- ✅ Complexity <10
- ✅ 100% test pass rate
- ✅ Zero clippy warnings
- ✅ All property tests pass

---

## 🧪 Testing Strategy

### Unit Tests
- Test each module in isolation
- Mock external dependencies (kcov, shfmt)
- Property-based tests for scoring algorithm

### Integration Tests
- End-to-end CLI tests with assert_cmd
- Test with real bash scripts
- Test with and without external tools

### Performance Tests
- Benchmark test discovery (<100ms)
- Benchmark coverage analysis (<200ms)
- Benchmark formatting (<100ms)
- Benchmark scoring (<500ms)

---

## 📚 Documentation Requirements

### README.md Updates
- New "Bash Quality Tools" section
- Command examples
- Quick start guide

### Book Updates
- New chapter: "Quality Tools"
- Subsections: Test, Coverage, Format, Score, Check
- CI/CD integration examples
- Best practices

### API Documentation
- Rustdoc comments on all public APIs
- Examples in doc comments
- Link to book chapters

---

## 🔮 Future Enhancements (v7.0+)

1. **IDE Integration**: LSP server for bash
2. **Watch Mode**: Continuous testing and coverage
3. **Git Integration**: Quality trends over commits
4. **Team Dashboard**: Web UI for quality metrics
5. **AI Suggestions**: LLM-powered code improvements

---

## 🎯 Success Criteria

This feature is complete when:

1. ✅ All 5 commands implemented and tested
2. ✅ >85% code coverage on new modules
3. ✅ >90% mutation score
4. ✅ Documentation complete (README, book)
5. ✅ CI/CD examples provided
6. ✅ Performance targets met
7. ✅ Zero regressions in existing features
8. ✅ User feedback positive

---

**Status**: Design Complete, Ready for Implementation
**Next Step**: Phase 1 - Foundation (Sprint 1)
**Estimated Timeline**: 4 sprints (2-3 weeks)

---

**Generated**: 2025-10-28
**Design Owner**: Claude Code
**Methodology**: EXTREME TDD + Toyota Way Quality Standards
