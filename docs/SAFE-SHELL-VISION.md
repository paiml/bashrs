# Safe Shell Vision: Rust-Based Bash Replacement

**Status**: 🎯 Design Phase
**Goal**: Build a memory-safe, deterministic shell that's a drop-in replacement for bash
**Target**: v8.0.0 (Safe Shell 1.0)

---

## The Problem with Bash

**Bash is fundamentally unsafe**:
1. ❌ **Memory unsafe**: C codebase, buffer overflows, use-after-free
2. ❌ **Non-deterministic**: $RANDOM, timestamps, $$, hostname baked in
3. ❌ **Non-idempotent**: Commands destructive by default (rm, mkdir)
4. ❌ **Injection-prone**: Unquoted variables, eval, command substitution
5. ❌ **Hard to test**: Side effects, global state, non-reproducible
6. ❌ **No type safety**: Everything is strings, errors at runtime

---

## The Vision: `rash` - Safe Shell

A **memory-safe, deterministic, idempotent shell** written in Rust that:

### Core Principles

1. **Memory Safe**: Written in 100% safe Rust (no unsafe blocks)
2. **Deterministic by Default**: No $RANDOM, no timestamps, reproducible
3. **Idempotent by Default**: Operations safe to re-run (mkdir -p, rm -f)
4. **Injection-Safe**: All variables quoted automatically
5. **Testable**: Pure functions, no hidden state
6. **Type-Safe**: Static analysis catches errors before execution

### Compatibility

- ✅ **POSIX-compatible syntax** (mostly bash-compatible)
- ✅ **Runs existing bash scripts** (with safety warnings)
- ✅ **Drop-in replacement**: `chsh -s /usr/bin/rash`
- ✅ **Interactive mode**: REPL with syntax highlighting

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                     Rash Safe Shell                          │
├─────────────────────────────────────────────────────────────┤
│  Interactive REPL  │  Script Executor  │  Linter/Analyzer   │
├─────────────────────────────────────────────────────────────┤
│                    Parser (bash AST)                         │
├─────────────────────────────────────────────────────────────┤
│              Safety Transformations Layer                     │
│  • Determinism   • Idempotency   • Injection Prevention      │
├─────────────────────────────────────────────────────────────┤
│                    IR (Intermediate Representation)          │
├─────────────────────────────────────────────────────────────┤
│                    Interpreter / JIT                         │
├─────────────────────────────────────────────────────────────┤
│              Safe Runtime (Rust std lib)                     │
└─────────────────────────────────────────────────────────────┘
```

### Components

#### 1. Parser (EXISTING - bash_parser)
- ✅ Parse bash syntax to AST
- ✅ Handle POSIX sh, bash extensions
- 🔨 **TODO**: Full bash compatibility (arrays, [[ ]], etc.)

#### 2. Safety Transformer (EXISTING - transformers)
- ✅ Remove $RANDOM → version-based IDs
- ✅ Add -p, -f flags for idempotency
- ✅ Quote all variables
- 🔨 **TODO**: Sandboxing, capability-based security

#### 3. Interpreter (NEW - PRIORITY)
- ❌ Execute bash AST directly in Rust
- ❌ No shell → string → shell round-trip
- ❌ Memory-safe execution
- ❌ Builtin commands in Rust

#### 4. REPL (NEW - PRIORITY)
- ❌ Interactive shell
- ❌ Syntax highlighting (rustyline)
- ❌ Command history
- ❌ Tab completion

#### 5. WASM Runtime (PARTIAL)
- ✅ Config analyzer works
- 🔨 **TODO**: Full shell execution in browser
- 🔨 **TODO**: Streaming I/O
- 🔨 **TODO**: Virtual filesystem

---

## Implementation Phases

### Phase 1: WASM Browser Runtime (v7.0) - **PRIORITY**

**Goal**: Run bashrs linter in browser for WOS and interactive.paiml.com

**Tasks**:
- [ ] Complete WASM canary tests (40 tests, all browsers)
- [ ] Implement streaming I/O for large scripts
- [ ] Virtual filesystem for browser
- [ ] Performance optimization (<100ms for typical scripts)
- [ ] Integration with WOS
- [ ] Integration with interactive.paiml.com

**Deliverable**: Fully functional bash linter in browser

**Timeline**: 2-3 weeks
**Status**: Phase 0 complete (feasibility proven)

---

### Phase 2: Safe Shell Interpreter (v8.0) - **NEXT**

**Goal**: Execute bash scripts directly in Rust without spawning /bin/sh

**Tasks**:
- [ ] Design interpreter architecture
- [ ] Implement builtin commands (cd, echo, pwd, etc.)
- [ ] Variable scope management
- [ ] Function call stack
- [ ] Pipeline execution
- [ ] Redirection handling
- [ ] Job control (fg, bg, jobs)
- [ ] Signal handling

**Deliverable**: Execute simple bash scripts in safe Rust interpreter

**Timeline**: 4-6 weeks
**Example**:
```rust
// Input: "echo hello && cd /tmp && pwd"
let ast = parse("echo hello && cd /tmp && pwd");
let result = interpret(ast); // Executes in Rust, no subprocess
assert_eq!(result.stdout, "hello\n/tmp\n");
```

---

### Phase 3: Interactive REPL (v8.5)

**Goal**: Interactive shell you can use daily

**Tasks**:
- [ ] rustyline integration
- [ ] Syntax highlighting
- [ ] Tab completion
- [ ] Command history (.rash_history)
- [ ] Prompt customization
- [ ] Vi/Emacs keybindings
- [ ] Multi-line editing

**Deliverable**: `rash` as your default shell

**Timeline**: 3-4 weeks
**Example**:
```bash
$ rash
rash> echo "Hello from safe shell!"
Hello from safe shell!
rash> mkdir -p /tmp/test  # Idempotent by default
rash> cd /tmp/test
rash> pwd
/tmp/test
```

---

### Phase 4: Full Bash Compatibility (v9.0)

**Goal**: Run 95% of existing bash scripts

**Tasks**:
- [ ] Bash arrays: `arr=(a b c)`
- [ ] Bash test operators: `[[ ]]`
- [ ] Process substitution: `<(cmd)`
- [ ] Here-documents: `<< EOF`
- [ ] Case statements
- [ ] For/while loops
- [ ] Functions with local variables
- [ ] Exported functions
- [ ] Bash arithmetic: `$((1+1))`

**Deliverable**: Drop-in bash replacement

**Timeline**: 8-12 weeks

---

### Phase 5: Advanced Features (v10.0+)

- [ ] JIT compilation for hot paths
- [ ] Static analysis (type checking)
- [ ] Capability-based security (landlock, seccomp)
- [ ] Container integration (rootless, cgroups)
- [ ] Distributed execution (SSH, Kubernetes)
- [ ] Time-travel debugging
- [ ] AI-powered auto-completion

---

## Immediate Next Steps (This Week)

### Option A: WASM Browser Testing (RECOMMENDED)
**Why**: Phase 0 done, ready for browser validation
**Tasks**:
1. Set up Playwright browser tests
2. Implement 40 canary tests (B01-B40)
3. Test in Chromium, Firefox, WebKit
4. Measure performance baselines
5. Deploy to WOS staging

**Value**: Immediate impact for WOS and interactive.paiml.com users

### Option B: Safe Shell Interpreter
**Why**: Core vision, high risk
**Tasks**:
1. Design interpreter API
2. Implement 10 builtin commands
3. Execute simple scripts (no pipes)
4. Benchmark vs /bin/sh
5. Write integration tests

**Value**: Foundation for safe shell, longer timeline

---

## Success Metrics

### WASM (v7.0)
- ✅ Loads in <5s
- ✅ Analyzes 1KB config in <100ms
- ✅ Streams 10MB in <1s
- ✅ Works in Chrome, Firefox, Safari
- ✅ Zero crashes (memory safe)

### Safe Shell Interpreter (v8.0)
- ✅ Executes 1000 commands/sec
- ✅ 100% memory safe (no unsafe)
- ✅ Passes 90% of bash test suite
- ✅ Zero command injection vulnerabilities
- ✅ Deterministic (same input → same output)

### Interactive REPL (v8.5)
- ✅ Startup in <100ms
- ✅ Tab completion <10ms
- ✅ Syntax highlighting real-time
- ✅ Compatible with bash_completion
- ✅ Works as login shell

---

## Technical Decisions

### Why Not Fork Bash?
- Bash is 120K lines of C (memory unsafe)
- Architecture designed for 1980s
- Technical debt insurmountable
- Better to start fresh in Rust

### Why Not Use Existing Rust Shells?
- **nushell**: Different paradigm (structured data, not POSIX)
- **ion**: RedoxOS-specific, not bash-compatible
- **dash**: C, not memory-safe
- **oil**: Python-based, not compiled

**bashrs/rash**: POSIX/bash-compatible + memory-safe + deterministic

### Why Interpreter, Not Transpiler?
**Current (v6.2.0)**: Rust → Shell transpiler
- ❌ Still executes in /bin/sh (memory unsafe)
- ❌ Can't prevent injection at runtime
- ❌ No sandboxing

**Future (v8.0+)**: Native interpreter
- ✅ Execute in Rust (memory safe)
- ✅ Runtime safety checks
- ✅ Sandboxing possible (seccomp, landlock)
- ✅ Controllable side effects

---

## Call to Action

**Recommendation**: Start with **WASM browser testing** (Phase 1)

**Rationale**:
1. Infrastructure exists (Phase 0 complete)
2. Clear deliverable (WOS + interactive.paiml.com)
3. Lower risk than interpreter
4. Immediate user value
5. Foundation for future work

**Next Command**:
```bash
cd rash/examples/wasm
npm install playwright
npx playwright test  # Run 40 canary tests
```

---

**Status**: 🎯 Ready to implement
**Decision needed**: WASM (Phase 1) or Interpreter (Phase 2)?
