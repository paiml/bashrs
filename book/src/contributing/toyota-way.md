# Toyota Way Principles

Rash (bashrs) follows the **Toyota Way** manufacturing philosophy, adapted for software development. These principles ensure **NASA-level quality** through rigorous quality gates, zero-defect policies, and continuous improvement.

## Philosophy Overview

The Toyota Way emphasizes building quality into the development process, not testing it in afterward. This translates to:

- **Zero tolerance for defects** - All tests must pass before committing
- **Stop the line** - Immediately halt work when bugs are discovered
- **Continuous improvement** - Every iteration should improve quality metrics
- **Direct observation** - Validate against real-world usage, not just theory

These principles are embedded in **EXTREME TDD methodology** and enforced through automated quality gates.

## Core Principles

### 🚨 Jidoka (自働化) - Build Quality In

**Japanese**: 自働化 (Jidoka)
**English**: "Automation with a human touch" or "Build quality in"

**Definition**: Build quality into the development process from the start. Don't rely on testing to find defects - prevent them through design.

#### How Rash Applies Jidoka

1. **Automated Quality Gates**
   ```bash
   # Pre-commit hooks enforce quality automatically
   git commit
   # → Runs tests (6321+ tests)
   # → Runs clippy (zero warnings required)
   # → Checks formatting
   # → Verifies complexity <10
   # → REJECTS commit if any check fails
   ```

2. **Bash Purification Validation**
   ```bash
   # Every purified script MUST pass shellcheck
   bashrs purify script.sh --output purified.sh
   shellcheck -s sh purified.sh  # Automatic POSIX validation
   ```

3. **Test Coverage Requirements**
   - **Target**: >85% coverage on all modules
   - **Current**: 6321+ tests passing (100% pass rate)
   - **Enforcement**: CI/CD fails if coverage drops below threshold

4. **Never Ship Incomplete Code**
   - All purifier outputs must be fully safe
   - All generated shell must pass quality gates
   - All linter rules must have >80% mutation kill rate

#### Real Example: SEC001 Mutation Testing

**Jidoka Applied**:
```bash
# Before committing SEC001 rule, verify quality
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib

# Result: 100% mutation kill rate (16/16 mutants caught)
# Quality built in - not tested in afterward
```

**If mutation testing had failed** (<90% kill rate):
```text
🚨 STOP THE LINE - Quality Gate Failed 🚨

Mutation kill rate: 75% (below 90% threshold)
Action: Add targeted tests to catch missed mutants
Status: COMMIT REJECTED until quality gate passes
```

This is Jidoka - **build quality in from the start**.

### 🎯 Genchi Genbutsu (現地現物) - Go and See

**Japanese**: 現地現物 (Genchi Genbutsu)
**English**: "Go and see for yourself" - Direct observation at the source

**Definition**: Understand problems and validate solutions through direct observation of real-world usage, not assumptions or theory.

#### How Rash Applies Genchi Genbutsu

1. **Test Against Real Shells**
   ```bash
   # Don't assume - test on actual target shells
   for shell in sh dash ash bash busybox; do
       echo "Testing with: $shell"
       $shell purified_script.sh
   done
   ```

2. **Profile Actual Scenarios**
   ```bash
   # Test real-world use cases in production-like environments
   docker run -it alpine:latest sh
   # Install bashrs and test bootstrap installers
   wget https://example.com/install.sh
   bashrs purify install.sh --output safe_install.sh
   sh safe_install.sh  # Verify it works in minimal environment
   ```

3. **Verify Purification Preserves Behavior**
   ```bash
   # Original bash script
   bash original.sh > original_output.txt

   # Purified POSIX sh
   sh purified.sh > purified_output.txt

   # VERIFY: Outputs must be identical
   diff original_output.txt purified_output.txt
   # Expected: No differences (behavioral equivalence)
   ```

4. **Property-Based Testing with Real Inputs**
   ```rust,ignore
   // Generate thousands of real-world test cases
   proptest! {
       #[test]
       fn prop_purification_preserves_behavior(
           bash_code in r"[a-z0-9_=\s]{1,100}"
       ) {
           let original_result = execute_bash(&bash_code);
           let purified = purify(&bash_code);
           let purified_result = execute_sh(&purified);

           // VERIFY: Same behavior on real inputs
           prop_assert_eq!(original_result, purified_result);
       }
   }
   ```

#### Real Example: v6.30.1 Parser Bug Discovery

**Genchi Genbutsu in Action**:

Property tests discovered a critical parser bug:
```bash
# Property test generated this real-world test case:
fi=1

# Parser ERROR: InvalidSyntax("Expected command name")
# This is VALID bash - keywords can be variable names!
```

**Direct Observation** revealed the problem:
```bash
# Go and see for yourself
$ bash
bash$ fi=1
bash$ echo $fi
1              # Works in real bash!

$ sh
sh$ fi=1
sh$ echo $fi
1              # Works in real POSIX sh too!
```

**Root Cause**: Parser theory was wrong - bash keywords are only special in specific syntactic positions. Direct observation with real shells revealed the specification gap.

**Fix**: Updated parser to match actual bash behavior, not assumed behavior.

This is Genchi Genbutsu - **verify against reality, not assumptions**.

### 🔍 Hansei (反省) - Reflection and Learning

**Japanese**: 反省 (Hansei)
**English**: "Reflection" - Learn from problems and fix root causes

**Definition**: Reflect on what went wrong, identify root causes, and implement systematic fixes to prevent recurrence.

#### How Rash Applies Hansei

1. **Fix Before Adding Features**
   - **Current priorities** (v6.30+ focus):
     1. Fix all SEC rules to >90% mutation kill rate (Phase 2 IN PROGRESS)
     2. Complete book documentation (3/3 critical chapters now fixed)
     3. Performance optimization (<100ms for typical scripts)
     4. THEN add new features (SEC009-SEC045 deferred to v2.x)

2. **Root Cause Analysis**
   ```markdown
   When property tests fail, don't just fix the symptom - understand WHY.

   Example: v6.30.1 Parser Bug
   - Symptom: Property test failed on "fi=1"
   - Root Cause: Parser treated keywords as special in all contexts
   - Fix: Added assignment pattern detection before keyword routing
   - Prevention: Added 14 tests for all keyword assignments
   ```

3. **Systematic Improvement**
   ```bash
   # After fixing a bug, ensure it can't happen again

   # Step 1: Add regression test
   #[test]
   fn test_issue_001_keyword_assignments() {
       // Prevent this bug from recurring
   }

   # Step 2: Document in CHANGELOG
   # "Fixed: Parser now handles keyword assignments (fi=1, for=2, etc.)"

   # Step 3: Update roadmap
   # Mark PARAM-KEYWORD-001 as completed
   ```

4. **Learn from Metrics**
   ```bash
   # SEC002 baseline: 75.0% mutation kill rate
   # Reflection: Why not 90%+?
   # Analysis: Missing tests for edge cases
   # Action: Add 8 mutation coverage tests
   # Result: Expected 87-91% after iteration
   ```

#### Real Example: SEC Batch Mutation Testing Reflection

**Hansei Applied**:

After SEC001 achieved 100% mutation kill rate, we reflected:

**Question**: Why did SEC001 succeed perfectly?
**Analysis**: Universal mutation pattern discovered (arithmetic mutations in `Span::new()`)
**Learning**: This pattern should work for ALL SEC rules
**Action**: Pre-wrote 45 tests for SEC002-SEC008 using same pattern
**Result**: 81.2% baseline average (exceeding 80% target before iteration!)

**Further Reflection**:

**Question**: Why did baseline average exceed 80% target?
**Answer**: High-quality existing tests + pattern recognition
**Learning**: Batch processing with pre-written tests saves 6-8 hours
**Action**: Apply batch approach to future rule development

This is Hansei - **reflect on success and failure, learn patterns, improve systematically**.

### 📈 Kaizen (改善) - Continuous Improvement

**Japanese**: 改善 (Kaizen)
**English**: "Continuous improvement" - Small, incremental enhancements

**Definition**: Continuously improve processes, code quality, and efficiency through small, measurable iterations.

#### How Rash Applies Kaizen

1. **Quality Baselines**
   ```bash
   # Establish baseline, then improve incrementally

   # SEC002 Baseline: 75.0% mutation kill rate (24/32 mutants caught)
   # Iteration 1: Add 8 targeted tests
   # Expected: 87-91% kill rate (28-29/32 mutants caught)
   # Improvement: +12-16 percentage points
   ```

2. **Performance Optimization**
   ```bash
   # Continuous performance improvement

   # Baseline: 200ms transpilation time
   # Target: <100ms for typical scripts
   # Approach: Profile, optimize hot paths incrementally
   # Measure: Benchmark after each optimization
   ```

3. **Test Coverage Improvement**
   ```bash
   # Incremental coverage increases

   # v6.24.0: 6164 tests
   # v6.25.0: 6260 tests (+96 tests)
   # v6.30.0: 6321 tests (+61 tests)
   # Trend: Continuous growth, never regression
   ```

4. **Code Complexity Reduction**
   ```bash
   # v6.24.3 Complexity Reduction

   # Before refactoring:
   # - SC2178: complexity 10
   # - SEC008: complexity 12
   # - SC2168: complexity 12

   # After refactoring (v6.24.3):
   # - SC2178: complexity 9 (-1 point)
   # - SEC008: complexity 7 (-5 points, 42% reduction)
   # - SC2168: complexity 5 (-7 points, 58% reduction)

   # Total improvement: -13 points (~42% average reduction)
   ```

5. **Process Automation**
   ```bash
   # Automate repetitive quality checks

   # Manual (slow):
   cargo test --lib
   cargo clippy --all-targets
   cargo fmt

   # Automated (fast):
   git commit  # Pre-commit hook runs all checks automatically
   ```

#### Real Example: Batch Processing Efficiency (Kaizen)

**Continuous Improvement Applied**:

**Iteration 1**: Sequential mutation testing
- SEC001 baseline: 45 minutes
- Analyze results: 15 minutes
- Write tests: 30 minutes
- SEC001 iteration: 45 minutes
- **Total per rule**: ~2.25 hours

**Kaizen Improvement**: Batch processing
- Run ALL baselines in parallel
- Pre-write tests during baseline execution
- Queue iterations efficiently
- **Time saved**: 6-8 hours for 8 rules

**Measurement**:
```bash
# Old approach: ~18 hours (8 rules × 2.25h)
# New approach: ~10-12 hours (parallel execution + batch processing)
# Improvement: 33-44% time savings
```

This is Kaizen - **continuously improve efficiency through small, measurable changes**.

## Integration with EXTREME TDD

The Toyota Way principles are embedded in the **EXTREME TDD methodology**:

### EXTREME TDD Formula

**EXTREME TDD = TDD + Property Testing + Mutation Testing + Fuzz Testing + PMAT + Examples**

| Phase | Toyota Way Principle | Application |
|-------|---------------------|-------------|
| **RED** (Write failing test) | **Jidoka** | Build quality in - test written first |
| **GREEN** (Implement) | **Genchi Genbutsu** | Verify against real shells |
| **REFACTOR** (Clean up) | **Kaizen** | Continuous improvement |
| **QUALITY** (Mutation test) | **Hansei** | Reflect on test effectiveness |

### Example: SEC001 EXTREME TDD with Toyota Way

```bash
# Phase 1: RED (Jidoka - Build Quality In)
#[test]
fn test_sec001_eval_with_variable() {
    let bash_code = r#"eval "$user_input""#;
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);  # Test FAILS - good!
}

# Phase 2: GREEN (Genchi Genbutsu - Verify Reality)
# Implement SEC001 rule detection
# Test against real bash: bash -c 'eval "$user_input"' (verify it's dangerous)
# Test PASSES now

# Phase 3: REFACTOR (Kaizen - Continuous Improvement)
# Extract helper: is_dangerous_eval()
# Reduce complexity: 12 → 7 (42% reduction)
# All tests still PASS

# Phase 4: QUALITY (Hansei - Reflect on Effectiveness)
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib
# Result: 100% mutation kill rate (16/16 caught)
# Reflection: Universal pattern discovered - apply to other rules
```

## STOP THE LINE Protocol (Andon Cord)

The **Andon Cord** is a Toyota manufacturing concept - any worker can pull a cord to stop the production line when they discover a defect. In Rash, this translates to **STOP THE LINE when bugs are discovered**.

### When to Pull the Andon Cord

**STOP IMMEDIATELY** if you discover:

1. ❌ **Test failure** - Any test fails (RED without GREEN)
2. ❌ **Quality gate failure** - Mutation kill rate <90%, complexity >10, coverage <85%
3. ❌ **Missing implementation** - Bash construct not parsed correctly
4. ❌ **Incorrect transformation** - Purified output is wrong
5. ❌ **Non-deterministic output** - Contains $RANDOM, $$, timestamps
6. ❌ **Non-idempotent output** - Not safe to re-run
7. ❌ **POSIX violation** - Generated shell fails `shellcheck -s sh`

### STOP THE LINE Procedure

```text
🚨 STOP THE LINE - P0 BUG DETECTED 🚨

1. HALT all current work
2. Document the bug clearly
3. Create P0 ticket
4. Fix with EXTREME TDD (RED → GREEN → REFACTOR → QUALITY)
5. Verify fix with comprehensive testing
6. Update CHANGELOG and roadmap
7. ONLY THEN resume previous work
```

### Example: v6.30.1 Parser Bug (STOP THE LINE Event)

**Trigger**: Property tests failed during v6.30.0 mutation testing verification

```bash
cargo test --lib bash_transpiler::purification_property_tests

# FAILED: 5/17 tests
# - prop_no_bashisms_in_output
# - prop_purification_is_deterministic
# - prop_purification_is_idempotent
# - prop_purified_has_posix_shebang
# - prop_variable_assignments_preserved

# Minimal failing case: fi=1
# Error: InvalidSyntax("Expected command name")
```

**STOP THE LINE Decision**:
- ✅ Immediately halted v6.30.0 mutation testing work
- ✅ Created P0 ticket: "Parser rejects valid bash keyword assignments"
- ✅ Fixed with EXTREME TDD (added 14 keyword assignment tests)
- ✅ Verified all 6260 tests passing (100%)
- ✅ Updated CHANGELOG.md
- ✅ Released as v6.30.1 (patch release - critical bug fix)
- ✅ ONLY THEN resumed v6.30.0 mutation testing work

**Result**: Zero defects in production. Bug caught and fixed before release.

This is **Jidoka + Hansei** - stop the line when defects are found, fix root cause, resume only after quality is restored.

## Toyota Way in Practice

### Daily Development Workflow

1. **Before starting work** (Genchi Genbutsu):
   ```bash
   # Verify current state is good
   git pull origin main
   cargo test --lib  # All tests passing?
   git status        # Clean working directory?
   ```

2. **While developing** (Jidoka):
   ```bash
   # Build quality in from the start
   # Write test first (RED)
   # Implement feature (GREEN)
   # Run tests frequently
   cargo test --lib test_your_feature
   ```

3. **Before committing** (Kaizen):
   ```bash
   # Continuous improvement
   cargo fmt                              # Format code
   cargo clippy --all-targets -- -D warnings  # Zero warnings
   cargo test --lib                       # All tests pass
   # Pre-commit hooks enforce these automatically
   ```

4. **After commit** (Hansei):
   ```bash
   # Reflect on the change
   # - Did tests catch all edge cases?
   # - Could this be done more efficiently?
   # - What did we learn?
   # Document learnings in commit message
   ```

### Release Process (Toyota Way Applied)

Every release applies all four principles:

- **Jidoka**: All quality gates MUST pass before release
  ```bash
  cargo test --lib                    # 6321+ tests passing
  cargo clippy --all-targets -- -D warnings  # Zero warnings
  cargo fmt -- --check                 # Formatted
  ./scripts/check-book-updated.sh      # Book updated
  ```

- **Genchi Genbutsu**: Verify release works for real users
  ```bash
  cargo publish --dry-run              # Test the package
  cargo install bashrs --version X.Y.Z # Test installation
  bashrs --version                     # Verify version
  bashrs lint examples/security/sec001_eval.sh  # Test real usage
  ```

- **Kaizen**: Continuously improve release automation
  ```bash
  # v1.0: Manual release checklist
  # v2.0: Automated quality gates
  # v3.0: One-command release script (future)
  ```

- **Hansei**: Reflect on release process
  ```markdown
  After each release:
  - What went well?
  - What could be improved?
  - How can we automate more?
  - Document improvements in CHANGELOG
  ```

## Quality Metrics (Toyota Way Evidence)

The Toyota Way principles produce measurable quality improvements:

### Test Quality (Jidoka + Kaizen)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Test count | Growing | 6321+ | ✅ Continuous growth |
| Pass rate | 100% | 100% | ✅ Zero defects |
| Coverage | >85% | 87.3% | ✅ Exceeds target |
| Mutation kill rate | >90% | 81.2% baseline → 87-91% expected | 🔄 Improving |

### Code Quality (Kaizen + Hansei)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Complexity | <10 | <10 (all functions) | ✅ Maintained |
| Clippy warnings | 0 | 0 | ✅ Zero tolerance |
| POSIX compliance | 100% | 100% | ✅ All purified scripts pass shellcheck |

### Process Quality (Genchi Genbutsu + Jidoka)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Pre-commit hooks | 100% enforcement | 100% | ✅ Automated |
| Shellcheck validation | All purified scripts | All purified scripts | ✅ Automatic |
| Real shell testing | dash, ash, bash, busybox | dash, ash, bash, busybox | ✅ Multi-shell validation |

### Efficiency Gains (Kaizen)

| Improvement | Before | After | Gain |
|-------------|--------|-------|------|
| Batch mutation testing | 18h (sequential) | 10-12h (parallel) | 33-44% faster |
| Complexity reduction | 12 avg (3 rules) | 7 avg (3 rules) | 42% reduction |
| Test count growth | 6164 (v6.24) | 6321 (v6.30) | +157 tests |

## Common Patterns

### Pattern 1: Fix-First Philosophy (Hansei)

**Don't add features when bugs exist**:
```bash
# ❌ WRONG: Add SEC009 while SEC002 is at 75% mutation kill rate
# ✅ RIGHT: Fix SEC002 to 90%+ THEN add SEC009
```

### Pattern 2: Zero-Defect Policy (Jidoka)

**All tests must pass before committing**:
```bash
# ❌ WRONG: git commit --no-verify (skip pre-commit hooks)
# ✅ RIGHT: Fix issues, then commit normally
cargo test --lib  # Fix failures first
cargo fmt         # Format code
git commit        # Hooks pass automatically
```

### Pattern 3: Incremental Improvement (Kaizen)

**Small, measurable improvements**:
```bash
# ❌ WRONG: "Rewrite entire linter to be 100% perfect"
# ✅ RIGHT: "Improve SEC002 from 75% to 87% mutation kill rate"
```

### Pattern 4: Empirical Validation (Genchi Genbutsu)

**Test on real shells, not assumptions**:
```bash
# ❌ WRONG: "This should work in POSIX sh" (assumption)
# ✅ RIGHT: sh purified.sh (empirical validation)
```

## Further Reading

- [Toyota Way (Wikipedia)](https://en.wikipedia.org/wiki/The_Toyota_Way)
- [Jidoka and Andon](https://en.wikipedia.org/wiki/Jidoka)
- [Kaizen](https://en.wikipedia.org/wiki/Kaizen)
- [Genchi Genbutsu](https://en.wikipedia.org/wiki/Genchi_Genbutsu)
- [EXTREME TDD Chapter](./extreme-tdd.md)
- [Release Process Chapter](./release.md)

---

**Quality Guarantee**: Rash follows Toyota Way principles to ensure NASA-level quality. Every commit, every release, and every feature is built with zero-defect philosophy and continuous improvement mindset.
