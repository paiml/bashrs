# Transpiler Bug Hunt & Probar Integration Specification

**Document ID**: SPEC-TB-2025-001
**Version**: 2.5.0
**Status**: APPROVED
**Created**: 2025-12-21
**Methodology**: Popper Falsification + Lean TDD + PMAT Quality Gates + Probar Integration
**Philosophy**: The Toyota Way (Kaizen, Genchi Genbutsu, Jidoka)

---

## Executive Summary

This unified specification documents our "Standardized Work" for ensuring the quality of the bashrs transpiler. It integrates **Kaizen** (continuous improvement) and **Jidoka** (automation with human intelligence) to eliminate **Muda** (waste) caused by transpiler bugs.

**Core Components**:
1.  **Visual Control Board**: Bug Inventory (12 identified gaps).
2.  **Poka-Yoke (Mistake Proofing)**: 130-point Popper Falsification Checklist.
3.  **Jidoka Infrastructure**: Probar integration for mutation testing, state machines, and simulation.
4.  **Standardized Work**: Dockerfile purification and verification.

**Key Insight (Genchi Genbutsu)**: Direct observation of the codebase reveals that while critical (P0) bugs are resolved, the transpiler fails to support idiomatic Rust constructs (`match`, complex expressions). This requires a rigorous, falsification-based quality assurance process.

---

## Part I: Current Condition (Bug Inventory)

### 1.1 Critical (P0) - Blocking Core Functionality

| Ticket | ID | Description | Impact |
|--------|-----|-------------|--------|
| TB-001 | TB030 | User-defined functions not transpiled | Functions disappear from output |
| TB-002 | TB031 | Function parameters not passed | Arguments lost in translation |
| TB-003 | TB033 | Multiple function definitions fail | Only main() survives |
| TB-010 | TB040 | `match` statements unsupported | Syntax error on idiomatic branching |

### 1.2 High (P1) - Major Feature Gaps

| Ticket | ID | Description | Impact |
|--------|-----|-------------|--------|
| TB-004 | TB004/TB005 | String literal validation fails | "No #[bashrs::main]" error |
| TB-005 | TB023 | Range-based for loops unsupported | `0..3` syntax rejected |
| TB-006 | TB024 | Function return values not handled | Return semantics broken |
| TB-011 | TB041 | `Option`/`Result` handling missing | Cannot use idiomatic error types |

### 1.3 Medium (P2) - Arithmetic & Utility Gaps

| Ticket | ID | Description | Impact |
|--------|-----|-------------|--------|
| TB-007 | TB012 | Multiplication not computed | `4 * 3` not evaluated |
| TB-008 | TB014 | Modulo not computed | `10 % 3` not evaluated |
| TB-009 | TB015 | Complex expressions fail | `(1 + 2) * 3` not evaluated |
| TB-012 | TB050 | Array indexing unsupported | `arr[0]` syntax rejected |

### 1.4 Root Cause Analysis (5 Whys)

**Problem**: Transpiler fails on idiomatic Rust.
1.  **Why?** The AST visitor pattern misses specific Rust nodes.
2.  **Why?** `emitter/posix.rs` implements a shallow subset of Rust syntax.
3.  **Why?** Initial focus was on "Hello World" capability, not language completeness.
4.  **Why?** Lack of a comprehensive "Falsification Checklist" during initial development.
5.  **Why?** No standard mechanism to verify idiomatic constructs against shell semantics.

**Countermeasure**: Implement the 130-point Falsification Checklist.

---

## Part II: Capability Matrix

### 2.1 bashrs Capabilities

| Feature | Command | Status |
|---------|---------|--------|
| Transpile Rust → Shell | `bashrs build` | ✅ |
| Purify bash scripts | `bashrs purify` | ✅ |
| Purify Makefiles | `bashrs purify Makefile` | ✅ |
| Purify Dockerfiles | `bashrs purify Dockerfile` | ⬜ Planned |
| Lint bash/shell | `bashrs lint` | ✅ |
| Lint Dockerfiles | `bashrs lint Dockerfile` | ⬜ Planned |
| Unit test bash | `bashrs test` | ✅ |
| Unit test Dockerfiles | `bashrs test Dockerfile` | ⬜ Planned |
| Quality scoring | `bashrs score` | ✅ |
| REPL debugging | `bashrs repl` | ✅ |
| Playbook execution | `bashrs playbook` | ✅ (v6.46.0) |
| Mutation testing | `bashrs mutate` | ✅ (v6.46.0) |
| Simulation replay | `bashrs simulate` | ✅ (v6.46.0) |

### 2.2 probar Capabilities

| Feature | Module | Status |
|---------|--------|--------|
| Test harness | `harness.rs` | ✅ |
| Playbook execution | `playbook/` | ✅ |
| TUI frame testing | `tui/` | ✅ |
| Mutation testing | `playbook/mutation.rs` | ✅ |
| Simulation replay | `simulation.rs` | ✅ |
| State machines | `playbook/state_machine.rs` | ✅ |

---

## Part III: Standardized Testing Harness

### 3.1 Standard Test Context

All **[STMT]** test cases are injected into the following harness to ensure **Repatability**:

```rust
// Standard Test Harness
fn main() {
    // Preamble: Common variables to support fragments
    let x = 10;
    let y = 5;
    let z = 2;
    let s = "hello";
    let b = true;
    let arr = [1, 2, 3];
    let opt = Some(1);
    let res: Result<i32, &str> = Ok(1);

    // --- INSERT T-CODE HERE ---
    <T_CODE>
}
// Helper functions (if needed by specific tests)
fn foo(i: i32) -> i32 { i }
fn bar() {}
```

### 3.2 Integration Architecture (Value Stream Map)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     UNIFIED BASH TESTING WORKFLOW                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │  Rust Code   │───►│   bashrs     │───►│ Purified     │               │
│  │  (source)    │    │   build      │    │ Shell Script │               │
│  └──────────────┘    └──────────────┘    └──────┬───────┘               │
│                                                  │                       │
│                                                  ▼                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │  probar      │◄───│   bashrs     │◄───│ Test         │               │
│  │  playbook    │    │   test       │    │ Functions    │               │
│  └──────────────┘    └──────────────┘    └──────────────┘               │
│         │                   │                                            │
│         ▼                   ▼                                            │
│  ┌──────────────┐    ┌──────────────┐                                   │
│  │ State Machine│    │ JUnit XML    │                                   │
│  │ Verification │    │ + Coverage   │                                   │
│  └──────────────┘    └──────────────┘                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Part IV: The 130-Point Popper Falsification Checklist

**Objective**: Attempt to falsify the hypothesis that "bashrs correctly transpiles all valid Rust constructs".

### 4.1 Basic & Literals (T001-T015)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T001 | PROG | `fn main() {}` | `(empty)` | Empty main |
| T002 | STMT | `let a = 1;` | `unknown` | Integer assignment |
| T003 | STMT | `let a = -1;` | `unknown` | Negative integer |
| T004 | STMT | `let a = "hi";` | `error` | String literal |
| T005 | STMT | `let a = true;` | `unknown` | Boolean literal |
| T006 | STMT | `let a = false;` | `unknown` | Boolean false |
| T007 | STMT | `let a = 0;` | `unknown` | Zero literal |
| T008 | STMT | `let a = 999999;` | `Overflow` | Large integer |
| T009 | STMT | `let _a = 1;` | `error` | Underscore prefix |
| T10 | STMT | `let a: i32 = 1;` | `error` | Explicit type |
| T011 | STMT | `let a = 1.0;` | `(silent)` | Float rejection |
| T012 | STMT | `let a = 'a';` | `unknown` | Char literal |
| T013 | STMT | `let a = b"hi";` | `unknown` | Byte string |
| T014 | PROG | `const X: i32 = 1; fn main(){}` | Missing `readonly` | Constant |
| T015 | PROG | `static X: i32 = 1; fn main(){}` | Missing global | Static |

### 4.2 Arithmetic & Bitwise (T016-T035)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T016 | STMT | `let _ = 1 + 2;` | Missing `$((` | Addition |
| T017 | STMT | `let _ = 5 - 3;` | Missing `$((` | Subtraction |
| T018 | STMT | `let _ = 4 * 3;` | Missing `$((` | Multiplication |
| T019 | STMT | `let _ = 10 / 2;` | Missing `$((` | Division |
| T020 | STMT | `let _ = 10 % 3;` | Missing `$((` | Modulo |
| T021 | STMT | `let _ = 2 + 3 * 4;` | `20` | Precedence |
| T022 | STMT | `let _ = (2 + 3) * 4;` | `14` | Grouping |
| T023 | STMT | `let _ = -5 + 3;` | `8` | Unary minus |
| T024 | STMT | `let _ = 1 << 2;` | Missing `<<` | Shift left |
| T025 | STMT | `let _ = 8 >> 2;` | Missing `>>` | Shift right |
| T026 | STMT | `let _ = 5 & 3;` | Missing `&` | Bitwise AND |
| T027 | STMT | `let _ = 5 | 3;` | Missing `|` | Bitwise OR |
| T028 | STMT | `let _ = 5 ^ 3;` | Missing `^` | Bitwise XOR |
| T029 | STMT | `let _ = !5;` | Missing `~` | Bitwise NOT |
| T030 | STMT | `let mut m = 1; m += 1;` | `m = m + 1` | Compound add |
| T031 | STMT | `let mut m = 1; m -= 1;` | Incorrect update | Compound sub |
| T032 | STMT | `let mut m = 1; m *= 2;` | Incorrect update | Compound mul |
| T033 | STMT | `let mut m = 1; m /= 2;` | Incorrect update | Compound div |
| T034 | STMT | `let mut m = 1; m %= 2;` | Incorrect update | Compound mod |
| T035 | STMT | `let _ = (1+1)==2;` | Missing `((` | Numeric comparison |

### 4.3 Control Flow & Loops (T036-T055)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T036 | STMT | `if true { }` | Missing `if/fi` | Empty if |
| T037 | STMT | `if x == 1 { }` | `[` | Numeric eq (prefer `((`) |
| T038 | STMT | `if s == "a" { }` | `((` | String eq (must be `[[`) |
| T039 | STMT | `if true { } else { }` | Missing `else` | If-else |
| T040 | STMT | `while x < 20 { break; }` | Missing `while` | Bounded loop |
| T041 | STMT | `loop { break; }` | Missing `while true` | Infinite loop |
| T042 | STMT | `for i in 0..3 { }` | Missing `{0..2}` | Range loop |
| T043 | STMT | `for i in 0..=3 { }` | Missing `{0..3}` | Inclusive range |
| T044 | STMT | `for i in (0..3).rev() { }` | Wrong order | Reverse range |
| T045 | STMT | `loop { break; }` | Outside loop | Break |
| T046 | STMT | `loop { continue; }` | Outside loop | Continue |
| T047 | STMT | `'label: loop { break 'label; }` | Missing label | Labeled break |
| T048 | STMT | `if let Some(_) = opt { }` | Syntax error | If-let |
| T049 | STMT | `while let Some(_) = opt { break; }` | Syntax error | While-let |
| T050 | STMT | `for _ in arr { }` | Missing `[@]` | Array iter |
| T051 | STMT | `if x > 1 && x < 10 { }` | `&&` inside `[` | Logical AND |
| T052 | STMT | `if x < 1 || x > 10 { }` | `||` inside `[` | Logical OR |
| T053 | STMT | `if !b { }` | Missing `!` | Logical NOT |
| T054 | STMT | `match x { 1 => {}, _ => {} }` | Missing `case` | Basic match |
| T055 | STMT | `match x { 1..=5 => {}, _ => {} }` | Missing `|` | Range match |

### 4.4 Pattern Matching (T056-T070)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T056 | STMT | `match s { "a" => {}, _ => {} }` | Missing `case` | String match |
| T057 | STMT | `match x { 1 | 2 => {}, _ => {} }` | Missing `|` | Multiple patterns |
| T058 | STMT | `match x { _ => {} }` | Missing `*)` | Catch-all |
| T059 | STMT | `match x { y if y > 0 => {}, _ => {} }` | Missing `if` | Match guards |
| T060 | STMT | `let (a, b) = (1, 2);` | Missing assignment | Tuple destructuring |
| T061 | STMT | `struct P {x:i32} let p = P{x:1}; let P{x} = p;` | Missing assignment | Struct destructuring |
| T062 | STMT | `match opt { Some(_) => {}, None => {} }` | Missing check | Option matching |
| T063 | STMT | `match res { Ok(_) => {}, Err(_) => {} }` | Missing check | Result matching |
| T064 | STMT | `match (1, 2) { (1, 2) => {}, _ => {} }` | Missing composite | Tuple matching |
| T065 | STMT | `let [a, b, c] = arr;` | Missing index | Array destructuring |
| T066 | STMT | `if matches!(x, 1..=5) {}` | Missing check | matches! macro |
| T067 | STMT | `match x { ref y => {}, _ => {} }` | Missing ref | Ref patterns |
| T068 | STMT | `match x { mut y => {}, _ => {} }` | Missing mut | Mut patterns |
| T069 | STMT | `let _ = match x { 1 => 10, _ => 0 };` | Missing return | Match expression |
| T070 | STMT | `let a = match x { _ => 1 };` | Missing capture | Match assignment |

### 4.5 Functions & Params (T071-T090)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T071 | PROG | `fn foo() {} fn main() { foo(); }` | Missing `foo()` | Definition |
| T072 | PROG | `fn foo(x: i32) {} fn main() { foo(1); }` | Missing `$1` | Parameters |
| T073 | STMT | `foo(1);` | Missing call | Application |
| T074 | PROG | `fn foo() -> i32 { 1 } fn main() {}` | Missing `return` | Return value |
| T075 | STMT | `let _ = foo(1);` | Missing `$(foo)` | Capture return |
| T076 | PROG | `fn foo(x: &str) {} fn main() {}` | Type error | String ref param |
| T077 | PROG | `pub fn foo() {} fn main() {}` | Visibility error | Export |
| T078 | PROG | `fn foo(x:i32, y:i32){} fn main(){foo(1,2);}` | Missing `$2` | Multi-param |
| T079 | PROG | `fn foo(s:&str){} fn main(){foo("a b");}` | Word splitting | Quoted args |
| T080 | PROG | `fn f(n:i32){if n>0{f(n-1)}} fn main(){f(5)}` | Infinite loop | Recursion |
| T081 | PROG | `#[bashrs::main] fn main() {}` | Missing main | Attribute |
| T082 | PROG | `fn main() {} fn foo() {}` | Missing foo | Multiple functions |
| T083 | STMT | `/* inline hint? */` | Code duplication | Inlining |
| T084 | STMT | `let _ = |x:i32| x + 1;` | Syntax error | Closures |
| T085 | PROG | `fn foo<T>(x: T) {} fn main() {}` | Generic error | Generics |
| T086 | PROG | `fn foo() -> Result<(),()> {Ok(())} fn main(){}` | Missing exit code | Result return |
| T087 | STMT | `foo(foo(1));` | Missing nesting | Nested calls |
| T088 | STMT | `foo(1 + 2);` | Missing eval | Expr as arg |
| T089 | STMT | `println!("{}", x);` | Missing `echo` | Format macro |
| T090 | STMT | `eprintln!("{}", x);` | Missing `>&2` | Error macro |

### 4.6 Standard Library & OS (T091-T105)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T091 | STMT | `let _ = std::fs::read_to_string("f");` | Missing `cat` | File read |
| T092 | STMT | `std::fs::write("f", "x");` | Missing `>` | File write |
| T093 | STMT | `let _ = std::env::var("X");` | Missing `$` | Env get |
| T094 | STMT | `std::env::set_var("X", "v");` | Missing `export` | Env set |
| T095 | STMT | `std::process::exit(0);` | Missing `exit` | Exit |
| T096 | STMT | `std::fs::remove_file("f");` | Missing `rm` | Delete |
| T097 | STMT | `std::fs::create_dir("d");` | Missing `mkdir` | Mkdir |
| T098 | STMT | `std::path::Path::new("p");` | Missing string | Path wrap |
| T099 | STMT | `std::thread::sleep(std::time::Duration::from_secs(1));` | Missing `sleep` | Sleep |
| T100 | STMT | `std::process::Command::new("ls");` | Missing exec | Subprocess |
| T101 | STMT | `std::time::Instant::now();` | Missing `date +%s` | Timing |
| T102 | STMT | `std::io::stdin();` | Missing `read` | Stdin |
| T103 | STMT | `std::io::stdout();` | Missing `/dev/stdout`| Stdout |
| T104 | STMT | `std::env::args();` | Missing `$@` | CLI args |
| T105 | STMT | `std::env::current_dir();` | Missing `pwd` | CWD |

### 4.7 Advanced & Error Handling (T106-T120)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T106 | STMT | `let _ = Option::Some(1);` | Missing value | Option wrap |
| T107 | STMT | `let _ = Option::<i32>::None;` | Missing empty | Option none |
| T108 | STMT | `let _ = Result::<i32, &str>::Ok(1);` | Missing success | Result ok |
| T109 | STMT | `let _ = Result::<i32, &str>::Err("e");` | Missing failure | Result err |
| T110 | STMT | `let _ = opt.unwrap();` | Missing panic | Unwrap |
| T111 | STMT | `let _ = opt.expect("msg");` | Missing msg | Expect |
| T112 | PROG | `fn f() -> Option<i32> { Some(1)? } fn main(){}` | Missing propagation| Try operator |
| T113 | STMT | `panic!("msg");` | Missing `exit 1` | Panic |
| T114 | STMT | `assert!(x == 10);` | Missing check | Assert |
| T115 | STMT | `assert_eq!(x, 10);` | Missing check | Assert Eq |
| T116 | STMT | `let _ = vec![1, 2, 3];` | Missing array | Vec macro |
| T117 | STMT | `let mut v = vec![]; v.push(1);` | Missing append | Vec push |
| T118 | STMT | `let v = vec![1]; let _ = v.len();` | Missing `${#v[@]}` | Vec length |
| T119 | STMT | `let v = vec![1]; let _ = v[0];` | Missing `${v[0]}` | Indexing |
| T120 | STMT | `let v = vec![1]; v.contains(&1);` | Missing loop | Collection check |

### 4.8 Expansion Pack: Edge Cases (T121-T130)

| ID | Type | Code | Forbidden Pattern | Rationale |
|----|------|------|-------------------|-----------|
| T121 | STMT | `std::thread::spawn(|| {})` | Missing error | No threads |
| T122 | STMT | `print!("no newline")` | Missing `printf` | No newline echo |
| T123 | STMT | `std::env::set_var("A", "b c")` | Missing quotes | Quote export |
| T124 | STMT | `std::fs::hard_link("a", "b")` | Missing `ln` | Hard links |
| T125 | STMT | `std::fs::copy("a", "b")` | Missing `cp` | Copy file |
| T126 | STMT | `std::fs::rename("a", "b")` | Missing `mv` | Move file |
| T127 | STMT | `let s = r"a\b";` | Missing escape | Raw strings |
| T128 | STMT | `let _ = format!("x: {}", 1);` | Missing echo | String fmt |
| T129 | STMT | `vec![1, 2].iter().map(|x| x+1)` | Missing loop | Functional map |
| T130 | STMT | `vec![1].iter().filter(|x| *x>0)` | Missing if | Functional filter |

---

## Part V: Property-Based Testing (Poka-Yoke)

To prevent defects from passing downstream, we enforce the following properties:

### 5.1 Symmetry Property
For any Rust expression `E`, if `transpile(E) -> S`, then the exit code and stdout of `E` (compiled with `rustc`) MUST match `S` (executed with `sh`).

### 5.2 Idempotency Property
`transpile(transpile(E))` should be stable (where applicable).

### 5.3 Quoting Safety Property (Security Poka-Yoke)
Any string literal `L` containing shell metacharacters (`$`, `` ` ``, `\`, `"`) MUST be emitted in a way that prevents shell expansion.

---

## Part VI: Probar Integration (Jidoka)

We use **Jidoka** (automation) to detect abnormalities.

### 6.1 Feature: Playbook-Driven Bash Testing

**Problem**: Manual testing of complex state is slow and error-prone.
**Solution**: State machine automation.

```yaml
# bash_install.playbook.yaml
version: "1.0"
machine:
  id: "install_script"
  initial: "uninstalled"
  # ... (State definitions)
```

**CLI Integration**:
```bash
bashrs playbook install.playbook.yaml --run
```

### 6.2 Feature: Mutation Testing (Falsification)

**Problem**: Tests may pass without actually verifying logic.
**Solution**: Mutation testing injects defects to verify test quality.

```bash
bashrs mutate install.sh --config mutation.config.yaml
# Goal: Kill rate > 90%
```

### 6.3 Feature: Deterministic Simulation

**Problem**: Non-deterministic scripts (Time, Random, PIDs) are hard to debug.
**Solution**: Simulation replay.

```bash
bashrs simulate purified.sh --seed 42 --verify
```

---

## Part VII: Dockerfile Support (Standardized Work)

### 7.1 Dockerfile Purification
Eliminate variation in build environments.

```bash
bashrs purify Dockerfile -o Dockerfile.pure
```

### 7.2 Dockerfile Linting (Andon)
Signal abnormalities immediately.

```bash
bashrs lint Dockerfile
```

### 7.3 Dockerfile Unit Testing
Verify each instruction operates as expected.

```bash
bashrs test Dockerfile
```

### 7.4 Dockerfile T-Codes (D001-D030)
(See previous version for full table, D001-D015 preserved)

---

## Part VIII: CLI Commands

```bash
# Core
bashrs build     # Rust → Shell
bashrs purify    # Determinism
bashrs lint      # Safety (Andon)
bashrs test      # Verification
bashrs score     # Measurement

# Jidoka (Advanced)
bashrs playbook  # State Machine
bashrs mutate    # Falsification
bashrs simulate  # Replay
```

---

## Part IX: Tooling Constraints (Zero Python Policy)

**CRITICAL**: All tooling MUST be implemented in Rust or POSIX shell. Python is prohibited.

### 9.1 Approved Languages
| Language | Usage | Rationale |
|----------|-------|-----------|
| Rust | Primary implementation, tests | Type safety, performance |
| POSIX sh | Generated output, scripts | Target platform |
| Bash | Development scripts only | Extended shell features |

### 9.2 Prohibited Languages
| Language | Status | Reason |
|----------|--------|--------|
| Python | ❌ BANNED | External dependency, not verified by bashrs |
| Node.js | ❌ BANNED | External dependency |
| Ruby | ❌ BANNED | External dependency |

### 9.3 QA Automation Requirements
QA falsification tests MUST use:
- `cargo test` with Rust test harnesses
- `bashrs test` for shell script verification
- POSIX shell scripts validated by `bashrs lint`

**Anti-pattern**: `scripts/*.py` - DELETE any Python scripts found.

---

## Part X: Acceptance Criteria

### 10.1 Transpiler T-Codes
- [x] All 130 T-codes implemented as Rust tests (142 tests in `transpiler_tcode_tests.rs`)
- [x] Quality gates pass (PMAT 133/134, Grade A+)

### 10.2 Advanced Testing
- [x] Playbook execution verified via `bashrs playbook` (v6.46.0)
- [x] Mutation testing via `bashrs mutate` (v6.46.0)
- [x] Simulation replay via `bashrs simulate` (v6.46.0)
- [x] Dockerfile linting active via `bashrs lint` (31 D-code tests in `dockerfile_dcode_tests.rs`)

### 10.3 Tooling Compliance
- [x] Zero Python scripts in repository (verified: `find . -name "*.py" | wc -l` = 0)
- [x] All QA via Rust tests or shell scripts

### 10.4 Examples & Documentation
- [x] All cargo examples pass (`cargo run --example <name>`):
  - [x] `linting_demo` - Linting workflow demonstration
  - [x] `quality_tools_demo` - Quality tools integration
  - [x] `optimizer_benchmark` - Constant folding optimization
  - [x] `makefile_purify_with_tests` - Makefile purification
  - [x] `xtask_custom_build` - Custom build workflows
- [x] Book examples pass (`mdbook test book`)

---

## Part XI: Lean TDD Process

1.  **Stop the Line**: If a T-code fails.
2.  **5 Whys**: Analyze the root cause.
3.  **Implement**: Minimal fix.
4.  **Verify**: Green test.
5.  **Standardize**: Add to regression suite.

---

## Part XII: Scientific Foundation & Citations

This specification is grounded in rigorous software engineering and lean manufacturing principles.

### 12.1 Falsification & Scientific Method
The "Popper Falsification Checklist" derives from Karl Popper's criterion of falsifiability, asserting that software correctness can only be provisionally accepted by failing to find bugs through rigorous testing.

*   **Popper, K.** (1959). *The Logic of Scientific Discovery*. Hutchinson.
*   **Dijkstra, E. W.** (1970). *Notes on Structured Programming*. (Famous quote: "Program testing can be used to show the presence of bugs, but never to show their absence!")

### 12.2 Mutation Testing
We employ mutation testing to evaluate the quality of our test suite, ensuring it can distinguish correct from incorrect behavior.

*   **Jia, Y., & Harman, M.** (2011). *An analysis and survey of the development of mutation testing*. IEEE Transactions on Software Engineering, 37(5), 649-678.
*   **DeMillo, R. A., Lipton, R. J., & Sayward, F. G.** (1978). *Hints on test data selection: Help for the practicing programmer*. Computer, 11(4), 34-41.

### 12.3 Lean & Toyota Way
The methodology adopts Lean principles to minimize waste and ensure quality at the source.

*   **Liker, J. K.** (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill.
*   **Womack, J. P., Jones, D. T., & Roos, D.** (1990). *The Machine That Changed the World*. Rawson Associates.
*   **Ohno, T.** (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press.

### 12.4 Formal Methods in Shell Scripting
References for challenges in shell script verification.

*   **Mazurak, K., & Zdancewic, S.** (2007). *ABASH: Finding Bugs in Bash Scripts*. In *Proceedings of the LISP User Group Meeting*.
*   **Jeon, S., Cha, S. K., & Ryu, S.** (2020). *BashReducer: Reducing Bash Scripts with Structure Preservation*. In *Proceedings of the 28th ACM Joint Meeting on European Software Engineering Conference and Symposium on the Foundations of Software Engineering*.

---

**Status**: APPROVED v2.5.0 - "Full Probar Integration + Examples" Edition