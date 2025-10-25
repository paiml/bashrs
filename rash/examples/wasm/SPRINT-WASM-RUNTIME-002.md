# Sprint WASM-RUNTIME-002: Core Features

**Sprint ID**: WASM-RUNTIME-002
**Duration**: 2-3 weeks (estimated)
**Goal**: Add pipes, loops, and functions to WASM bash runtime
**Status**: 🟡 READY TO START
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)

---

## Executive Summary

Sprint WASM-RUNTIME-002 builds on the successful foundation of Sprint 001 by adding **core bash features** that enable real-world scripting patterns:

- **Pipelines**: `cmd1 | cmd2` for composing commands
- **Command substitution**: `$(cmd)` for capturing output
- **Loops**: `for` and `while` for iteration
- **Functions**: User-defined bash functions
- **Arrays**: Bash array support
- **Arithmetic**: `$((expr))` for calculations

These features transform the runtime from a **toy demo** into a **practical educational tool** for WOS and interactive.paiml.com.

---

## Prerequisites

### Sprint 001 Deliverables (COMPLETE ✅)

- ✅ Basic executor (`BashExecutor`)
- ✅ Virtual filesystem (`VirtualFilesystem`)
- ✅ Built-in commands (echo, cd, pwd)
- ✅ Variable assignment and expansion
- ✅ I/O streams (stdout/stderr capture)
- ✅ 49 tests passing (100%)
- ✅ Browser demo working
- ✅ Property-based testing infrastructure
- ✅ E2E testing with Playwright

### Infrastructure Available

- ✅ bash_parser (full bash syntax parsing)
- ✅ AST representation (all bash constructs)
- ✅ WASM build pipeline (wasm-pack)
- ✅ Testing infrastructure (unit + property + E2E)
- ✅ Documentation templates

---

## Objectives

### Primary Goals

| Objective | Description | Priority | Estimated Effort |
|-----------|-------------|----------|------------------|
| **PIPE-001** | Simple pipelines `cmd1 \| cmd2` | P0 | 3 days |
| **PIPE-002** | Multi-stage pipelines `c1 \| c2 \| c3` | P0 | 2 days |
| **SUB-001** | Command substitution `$(cmd)` | P0 | 2 days |
| **LOOP-001** | For loops `for i in ...; do ...; done` | P0 | 3 days |
| **LOOP-002** | While loops `while ...; do ...; done` | P1 | 2 days |
| **FUNC-001** | Function definition and calls | P0 | 4 days |
| **ARRAY-001** | Bash arrays `arr=(a b c)` | P1 | 2 days |
| **ARITH-001** | Arithmetic expansion `$((1 + 2))` | P1 | 2 days |

**Total**: 20 days (~3 weeks with buffer)

### Secondary Goals (Stretch)

- **REDIR-001**: Output redirection `>`, `>>`, `2>`
- **REDIR-002**: Input redirection `<`
- **COND-001**: If/else/elif conditionals
- **CASE-001**: Case statements

---

## Technical Design

### 1. Pipeline Implementation

**File**: `src/wasm/pipeline.rs`

```rust
/// Execute a pipeline of commands
pub struct PipelineExecutor {
    executor: BashExecutor,
}

impl PipelineExecutor {
    /// Execute: cmd1 | cmd2 | cmd3
    pub fn execute_pipeline(
        &mut self,
        commands: &[Command],
    ) -> Result<ExecutionResult> {
        let mut prev_stdout = Vec::new();

        for (i, cmd) in commands.iter().enumerate() {
            // Set stdin to previous stdout
            if i > 0 {
                self.executor.set_stdin(&prev_stdout);
            }

            // Execute command
            let result = self.executor.execute_command(cmd)?;

            // Capture stdout for next stage
            prev_stdout = result.stdout.as_bytes().to_vec();
        }

        Ok(ExecutionResult {
            stdout: String::from_utf8(prev_stdout)?,
            stderr: String::new(),
            exit_code: 0,
        })
    }
}
```

**Test Coverage**:
- Simple 2-stage pipeline: `echo "hello" | wc -c`
- Multi-stage: `echo "a b c" | tr ' ' '\n' | wc -l`
- Error propagation: Failed command in pipeline
- Property test: Pipeline is associative

### 2. Command Substitution

**File**: `src/wasm/substitution.rs`

```rust
/// Expand command substitutions
pub fn expand_command_substitution(
    script: &str,
    executor: &mut BashExecutor,
) -> Result<String> {
    // Find all $(...)
    let re = Regex::new(r"\$\(([^)]+)\)").unwrap();

    let mut result = script.to_string();
    for cap in re.captures_iter(script) {
        let cmd = &cap[1];

        // Execute command
        let output = executor.execute(cmd)?;

        // Replace $(cmd) with output
        result = result.replace(&cap[0], output.stdout.trim());
    }

    Ok(result)
}
```

**Test Coverage**:
- Basic: `echo "Today is $(date)"`
- Nested: `echo "Count: $(echo $(expr 1 + 1))"`
- In assignment: `count=$(wc -l < file.txt)`
- Property test: Deterministic expansion

### 3. For Loops

**File**: `src/wasm/loops.rs`

```rust
/// Execute a for loop
pub fn execute_for_loop(
    &mut self,
    var: &str,
    items: &[String],
    body: &[Statement],
) -> Result<()> {
    for item in items {
        // Set loop variable
        self.env.insert(var.to_string(), item.to_string());

        // Execute loop body
        for stmt in body {
            self.execute_statement(stmt)?;
        }
    }
    Ok(())
}
```

**Test Coverage**:
- Simple: `for i in 1 2 3; do echo $i; done`
- Glob expansion: `for f in *.txt; do cat $f; done` (VFS)
- Nested loops
- Break/continue support
- Property test: Loop executes N times for N items

### 4. Functions

**File**: `src/wasm/functions.rs`

```rust
/// User-defined bash function
pub struct BashFunction {
    name: String,
    params: Vec<String>,
    body: Vec<Statement>,
}

/// Function table
pub struct FunctionTable {
    functions: HashMap<String, BashFunction>,
}

impl FunctionTable {
    /// Define a function
    pub fn define(&mut self, func: BashFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Call a function
    pub fn call(
        &self,
        name: &str,
        args: &[String],
        ctx: &mut ExecutionContext,
    ) -> Result<i32> {
        let func = self.functions.get(name)
            .ok_or(format!("Function not found: {}", name))?;

        // Set positional parameters $1, $2, ...
        for (i, arg) in args.iter().enumerate() {
            ctx.env.insert((i + 1).to_string(), arg.to_string());
        }

        // Execute function body
        for stmt in &func.body {
            ctx.execute_statement(stmt)?;
        }

        Ok(ctx.exit_code)
    }
}
```

**Test Coverage**:
- Simple function: `greet() { echo "Hello, $1"; }; greet World`
- Return values: `add() { echo $(($ 1 + $2)); }; result=$(add 3 5)`
- Recursion: Fibonacci function
- Local variables
- Property test: Functions are deterministic

### 5. Arrays

**File**: `src/wasm/arrays.rs`

```rust
/// Bash array support
pub struct BashArray {
    elements: Vec<String>,
}

impl BashArray {
    /// Create array: arr=(a b c)
    pub fn new(elements: Vec<String>) -> Self {
        Self { elements }
    }

    /// Access element: ${arr[0]}
    pub fn get(&self, index: usize) -> Option<&String> {
        self.elements.get(index)
    }

    /// Set element: arr[0]="value"
    pub fn set(&mut self, index: usize, value: String) {
        if index >= self.elements.len() {
            self.elements.resize(index + 1, String::new());
        }
        self.elements[index] = value;
    }

    /// Get all elements: ${arr[@]}
    pub fn all(&self) -> &[String] {
        &self.elements
    }

    /// Length: ${#arr[@]}
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}
```

**Test Coverage**:
- Declaration: `arr=(1 2 3)`
- Access: `echo ${arr[0]}`
- All elements: `echo ${arr[@]}`
- Length: `echo ${#arr[@]}`
- Iteration: `for i in ${arr[@]}; do ...; done`
- Property test: Array operations preserve data

### 6. Arithmetic Expansion

**File**: `src/wasm/arithmetic.rs`

```rust
use evalexpr::eval;

/// Evaluate arithmetic expression
pub fn eval_arithmetic(expr: &str) -> Result<i64> {
    // Replace bash variables with values
    let expanded = expand_variables(expr)?;

    // Evaluate with evalexpr crate
    let value = eval(&expanded)
        .map_err(|e| format!("Arithmetic error: {}", e))?;

    // Convert to integer
    value.as_int()
        .ok_or_else(|| "Not an integer".to_string())
}
```

**Dependencies** (add to Cargo.toml):
```toml
[dependencies]
evalexpr = "12.0"  # For arithmetic evaluation
```

**Test Coverage**:
- Basic: `echo $((1 + 2))`
- Variables: `a=5; echo $((a * 2))`
- Complex: `echo $(( (3 + 5) * 2 - 1 ))`
- Property test: Arithmetic laws (commutative, associative)

---

## Test Strategy

### Phase 1: RED (Write Failing Tests)

**Day 1**: Write all failing tests for PIPE-001, PIPE-002

```rust
#[test]
fn test_simple_pipeline() {
    let mut executor = BashExecutor::new();
    let result = executor.execute("echo 'hello world' | wc -c").unwrap();
    assert_eq!(result.stdout.trim(), "12");
}

#[test]
fn test_multi_stage_pipeline() {
    let mut executor = BashExecutor::new();
    let result = executor.execute("echo 'a b c' | tr ' ' '\\n' | wc -l").unwrap();
    assert_eq!(result.stdout.trim(), "3");
}
```

Run: `cargo test --features wasm` → **EXPECT FAILURES** ❌

### Phase 2: GREEN (Implement Features)

**Days 2-3**: Implement pipeline executor until tests pass

```bash
cargo test --features wasm test_simple_pipeline
# ✅ PASS

cargo test --features wasm test_multi_stage_pipeline
# ✅ PASS
```

### Phase 3: REFACTOR (Clean Up)

**Day 4**: Refactor pipeline code
- Extract helper functions
- Reduce complexity <10
- Add documentation
- Verify all tests still pass

### Phase 4: Property Tests

**Day 5**: Add generative tests

```rust
proptest! {
    #[test]
    fn prop_pipeline_deterministic(
        input in "[a-z ]{1,50}",
        cmd1 in "(echo|cat)",
        cmd2 in "(wc|tr)"
    ) {
        let mut executor = BashExecutor::new();
        let script = format!("{} '{}' | {}", cmd1, input, cmd2);

        let result1 = executor.execute(&script).unwrap();
        let result2 = executor.execute(&script).unwrap();

        prop_assert_eq!(result1.stdout, result2.stdout);
    }
}
```

### Phase 5: E2E Tests

**Day 6**: Add browser tests

```typescript
test('R20: Execute pipeline in browser', async ({ page }) => {
    await page.locator('#script-input').fill(`
        echo "line1" | wc -c
    `);
    await page.locator('button:has-text("Execute Script")').click();
    await expect(page.locator('#output')).toContainText('6');
});
```

---

## Iteration Plan

### Week 1: Pipelines + Command Substitution

**Days 1-3**: PIPE-001, PIPE-002
- RED: Write 20 failing tests for pipelines
- GREEN: Implement PipelineExecutor
- REFACTOR: Clean up, complexity <10
- PROPERTY: Add 5 property tests

**Days 4-5**: SUB-001
- RED: Write 15 failing tests for command substitution
- GREEN: Implement expand_command_substitution
- REFACTOR: Clean up
- PROPERTY: Add 3 property tests

**Expected**: 35 new tests, all passing ✅

### Week 2: Loops + Functions

**Days 6-8**: LOOP-001, LOOP-002
- RED: Write 25 failing tests for loops
- GREEN: Implement for_loop and while_loop
- REFACTOR: Extract loop helpers
- PROPERTY: Add 6 property tests

**Days 9-12**: FUNC-001
- RED: Write 30 failing tests for functions
- GREEN: Implement FunctionTable
- REFACTOR: Clean up function handling
- PROPERTY: Add 8 property tests

**Expected**: 55 new tests, all passing ✅

### Week 3: Arrays + Arithmetic + Polish

**Days 13-14**: ARRAY-001
- RED: Write 20 failing tests for arrays
- GREEN: Implement BashArray
- REFACTOR: Clean up array operations
- PROPERTY: Add 5 property tests

**Days 15-16**: ARITH-001
- RED: Write 15 failing tests for arithmetic
- GREEN: Implement eval_arithmetic
- REFACTOR: Clean up
- PROPERTY: Add 4 property tests

**Days 17-18**: Integration + Documentation
- Add 10 E2E tests for new features
- Update RUNTIME-USAGE.md with examples
- Create new example scripts
- Update browser demo with new capabilities

**Expected**: 35 new tests, comprehensive docs ✅

---

## Quality Gates

### Before Each Commit

- [ ] ✅ All unit tests pass: `cargo test --features wasm`
- [ ] ✅ Property tests pass: 100+ cases per property
- [ ] ✅ Clippy clean: `cargo clippy --features wasm`
- [ ] ✅ Formatted: `cargo fmt -- --check`

### Before Each Feature Completion

- [ ] ✅ Feature tests pass (100%)
- [ ] ✅ Property tests added and passing
- [ ] ✅ Code coverage >85% on new code
- [ ] ✅ Complexity <10 on all functions
- [ ] ✅ Documentation updated

### Before Sprint Completion

- [ ] ✅ All 125 new tests passing (100%)
- [ ] ✅ E2E tests passing (10 new browser tests)
- [ ] ✅ Mutation score ≥90% on new modules
- [ ] ✅ Performance: <100ms for 100-line scripts
- [ ] ✅ Browser demo updated with examples
- [ ] ✅ RUNTIME-USAGE.md comprehensive
- [ ] ✅ Sprint retrospective written

---

## Success Metrics

### Quantitative

| Metric | Target | How to Measure |
|--------|--------|----------------|
| New tests | 125+ | `cargo test --features wasm \| grep "test wasm"` |
| Test pass rate | 100% | All tests must pass |
| Coverage | >85% | `cargo llvm-cov --features wasm` |
| Mutation score | ≥90% | `cargo mutants --file src/wasm/*.rs` |
| Performance | <100ms | Benchmark 100-line script |
| Browser tests | 10+ | E2E tests in Playwright |

### Qualitative

- ✅ Code is readable and well-documented
- ✅ Examples are practical and educational
- ✅ Browser demo is polished and delightful
- ✅ Documentation is comprehensive
- ✅ Zero regressions from Sprint 001

---

## Risks and Mitigation

### Risk 1: Pipeline Complexity

**Risk**: Implementing pipes may require major executor refactor
**Probability**: MEDIUM
**Impact**: HIGH (could delay sprint)

**Mitigation**:
1. Start with simplest possible implementation
2. Add complexity incrementally
3. Keep tests passing at each step
4. Budget extra time (3 days instead of 2)

### Risk 2: Parser Integration

**Risk**: bash_parser may not provide all AST nodes needed
**Probability**: LOW
**Impact**: MEDIUM (would need parser updates)

**Mitigation**:
1. Verify parser coverage on Day 1
2. Add missing AST nodes if needed
3. Coordinate with parser maintainers

### Risk 3: Performance Degradation

**Risk**: Complex features slow down execution
**Probability**: MEDIUM
**Impact**: LOW (acceptable if <200ms)

**Mitigation**:
1. Profile after each feature
2. Optimize hot paths
3. Add performance tests
4. Set clear thresholds

### Risk 4: Scope Creep

**Risk**: Attempting too many features
**Probability**: HIGH
**Impact**: HIGH (sprint fails)

**Mitigation**:
1. Stick to P0 objectives only
2. Defer P1 to Sprint 003 if needed
3. Mark stretch goals clearly
4. Review progress weekly

---

## Dependencies

### New Dependencies (Cargo.toml)

```toml
[dependencies]
# For arithmetic evaluation
evalexpr = "12.0"

# For regex in substitution
regex = "1.10"

# For glob patterns (arrays, loops)
globset = "0.4"
```

### Existing Dependencies

- ✅ wasm-bindgen
- ✅ js-sys
- ✅ serde / serde_json
- ✅ bash_parser (AST)

---

## Deliverables

### Code Artifacts

1. ✅ `src/wasm/pipeline.rs` - Pipeline execution
2. ✅ `src/wasm/substitution.rs` - Command substitution
3. ✅ `src/wasm/loops.rs` - For/while loops
4. ✅ `src/wasm/functions.rs` - Function table
5. ✅ `src/wasm/arrays.rs` - Bash arrays
6. ✅ `src/wasm/arithmetic.rs` - Arithmetic expansion

### Test Artifacts

1. ✅ 125+ unit tests (all passing)
2. ✅ 30+ property tests (3000+ cases)
3. ✅ 10+ E2E browser tests
4. ✅ Mutation testing reports (≥90% kill rate)

### Documentation Artifacts

1. ✅ Updated RUNTIME-USAGE.md with new features
2. ✅ 5 new example scripts showcasing features
3. ✅ API reference for new functions
4. ✅ Integration guide updates
5. ✅ SPRINT-002-RETROSPECTIVE.md

### Demo Artifacts

1. ✅ Updated runtime-demo.html with new examples
2. ✅ Example: Pipeline demo (`ls | grep | wc`)
3. ✅ Example: Loop demo (file processing)
4. ✅ Example: Function demo (helper functions)
5. ✅ Example: Array demo (data processing)

---

## Definition of Done

Sprint WASM-RUNTIME-002 is COMPLETE when:

- [x] All P0 objectives implemented and tested
- [x] 125+ tests passing (100% pass rate)
- [x] Property tests: 30+, 3000+ cases
- [x] E2E tests: 10+ browser tests passing
- [x] Mutation score ≥90% on new code
- [x] Code coverage >85%
- [x] Performance <100ms for 100-line scripts
- [x] Browser demo updated and working
- [x] Documentation comprehensive and accurate
- [x] Zero regressions from Sprint 001
- [x] Sprint retrospective written
- [x] Code reviewed and merged to main

---

## Next Steps (Sprint 003)

After Sprint 002 completion, possible directions:

**Option A**: Advanced Features (Phase 3)
- Process substitution `<(cmd)`
- Here documents `<<EOF`
- Case statements
- Traps and signals

**Option B**: Integration
- WOS terminal integration
- interactive.paiml.com integration
- Offline support (Service Worker)

**Option C**: Interactive Shell (Phase 4)
- REPL for terminal
- Line editing (history, cursor)
- Tab completion
- Job control

**Decision**: To be made in Sprint 002 retrospective based on:
- User feedback
- WOS/interactive.paiml.com priorities
- Technical learnings from Sprint 002

---

## References

- **Sprint 001 Retrospective**: `SPRINT-001-RETROSPECTIVE.md`
- **WASM Roadmap**: `WASM-RUNTIME-ROADMAP.md`
- **bash_parser**: `/home/noah/src/bashrs/rash/src/bash_parser/`
- **Existing Tests**: `src/wasm/executor.rs` (property tests)
- **E2E Tests**: `e2e/runtime-demo.spec.ts`

---

**🤖 Generated with [Claude Code](https://claude.com/claude-code)**
**Co-Authored-By: Claude <noreply@anthropic.com>**
