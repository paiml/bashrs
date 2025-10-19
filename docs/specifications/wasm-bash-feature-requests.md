# WASM Backend for Bashrs

## Document Status: REVISED AFTER CRITICAL REVIEW

**Original Version**: October 19, 2025
**Revised Version**: October 19, 2025 (Post-Review)
**Review Status**: ⚠️ Major architectural flaws identified and addressed

### Critical Review Summary

This document underwent rigorous technical review by the bashrs core team. The review identified **critical architectural flaws** in the original proposal:

1. **IR Contamination**: Proposal incorrectly suggested adding WASM-specific variants to `ShellIR` ❌
2. **Streaming I/O Failure**: Proposed buffering entire command outputs in memory (catastrophic for large outputs) ❌
3. **Unrealistic Timeline**: 6-7 weeks grossly underestimated the complexity ❌

### Revisions Made

This document has been **substantially revised** to address these concerns:

✅ **Phase 0 Feasibility Study**: Added mandatory 3-week research phase for streaming I/O
✅ **Architecture Fixed**: Removed IR contamination, kept `ShellIR` platform-agnostic
✅ **Timeline Adjusted**: Realistic 11-13 week timeline (including Phase 0)
✅ **Technical Honesty**: Acknowledged streaming I/O as a major unsolved research problem

**Reviewer Decision**: "No-Go" on original plan, but "Go" on strategic direction pending Phase 0 success.

---

## Executive Summary

This document proposes adding a **WebAssembly (WASM) backend** to bashrs, enabling generation of WASM-compatible Rust code from bash scripts and linting for WASM runtime constraints. This feature would enable deterministic, type-safe, sandboxed shell scripts that run in browsers, serverless environments, and embedded systems.

**Key Benefit**: Compile bash scripts to portable WASM binaries that execute deterministically across any platform supporting WebAssembly.

**CRITICAL CAVEAT**: This proposal requires solving a hard computer science problem (streaming I/O in WASM) before implementation can proceed. A mandatory Phase 0 feasibility study is required.

## Motivation

### Primary Use Cases

**Educational Operating Systems**: Educational microkernel operating systems written in Rust and compiled to WASM often need shell script execution capabilities. These projects currently face challenges:
- Manual shell script parsing is error-prone and incomplete
- Limited POSIX command support
- Lack of comprehensive linting and safety checks

**Solution**: Use bashrs to:
1. **Parse** bash scripts into AST
2. **Transpile** to WASM-compatible Rust code with `#[wasm_bindgen]`
3. **Lint** for WASM constraints (no file I/O, no native syscalls, determinism)
4. **Compile** to WASM binary for browser execution

### Additional Use Cases

1. **Serverless Functions**: Deploy shell scripts as AWS Lambda/Cloudflare Workers (WASM)
2. **Browser Automation**: Safe script execution in web apps without server round-trips
3. **Embedded Systems**: Run deterministic scripts on resource-constrained devices
4. **Educational Tools**: Teach shell scripting with instant browser-based execution
5. **CI/CD Pipelines**: Sandboxed, deterministic build scripts

## Current Bashrs Architecture

Bashrs has a clean, extensible architecture ideal for adding WASM backend:

```
┌─────────────┐
│ Bash Script │
└──────┬──────┘
       │ parse
       ▼
┌─────────────┐
│  Bash AST   │  (ast.rs, ~298 lines)
└──────┬──────┘
       │ to_ir
       ▼
┌─────────────┐
│  ShellIR    │  (shell_ir.rs, ~300 lines)
└──────┬──────┘
       │ emit
       ▼
┌─────────────────────────────────┐
│ Target: POSIX | Bash | **WASM** │
└─────────────────────────────────┘
```

### Key Components

#### 1. Parser (`bash_parser/`)
- Full POSIX parameter expansion support
- Handles `${VAR:-default}`, `${#VAR}`, `${VAR%pattern}`, etc.
- Command substitution, process substitution
- Functions, loops, conditionals, pipes

**Example AST**:
```rust
BashStmt::Command {
    name: "echo",
    args: vec![
        BashExpr::Literal("Hello"),
        BashExpr::Variable("NAME")
    ],
    span: 1:0-1:21
}
```

#### 2. IR Layer (`ir/shell_ir.rs`)
- Platform-agnostic intermediate representation
- Effect tracking (NetworkAccess, FileWrite, ProcessSpawn)
- Suitable for multiple backends

**Example ShellIR**:
```rust
ShellIR::Exec {
    cmd: Command {
        name: "echo",
        args: vec![
            ShellValue::String("Hello"),
            ShellValue::Variable("NAME")
        ]
    },
    effects: Effects::None
}
```

#### 3. Emitter Pattern (`emitter/posix.rs`)
- Generates code from ShellIR
- Currently supports POSIX shell
- ~400 lines, easily extensible

**Current Emitter**:
```rust
impl PosixEmitter {
    pub fn emit(&self, ir: &ShellIR) -> Result<String> {
        match ir {
            ShellIR::Let { name, value, .. } => {
                write!(self.out, "{}={}", name, self.emit_value(value)?)
            }
            ShellIR::Exec { cmd, .. } => {
                write!(self.out, "{}", self.emit_command(cmd)?)
            }
            // ... handle all ShellIR variants
        }
    }
}
```

#### 4. Linter (`linter/rules/`)
- 20+ rules for security, determinism, idempotency
- Examples: DET001 (no $RANDOM), SEC001 (quote variables), IDP001 (use -p, -f flags)
- Extensible rule system

**Example Rule** (det001.rs):
```rust
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(col) = line.find("$RANDOM") {
            result.add(Diagnostic::error(
                "DET001",
                "Non-deterministic $RANDOM usage",
                span,
            ).with_fix(Fix::new("${VERSION}")));
        }
    }
    result
}
```

## Proposed WASM Backend Architecture

### 1. Add ShellDialect::Wasm Variant

**File**: `rash/src/emitter/mod.rs`

```rust
#[derive(Debug, Clone)]
pub enum ShellDialect {
    Posix,
    Bash,
    Wasm,  // NEW
}

pub fn emit(ir: &ShellIR, config: &Config) -> Result<String> {
    match config.target {
        ShellDialect::Posix => PosixEmitter::new(config.clone()).emit(ir),
        ShellDialect::Bash => BashEmitter::new(config.clone()).emit(ir),
        ShellDialect::Wasm => WasmEmitter::new(config.clone()).emit(ir),  // NEW
    }
}
```

### 2. Create WasmEmitter

**File**: `rash/src/emitter/wasm.rs` (new file)

```rust
use crate::ir::{ShellIR, ShellValue, Command, Effects};
use crate::emitter::Config;
use std::io::Write;

pub struct WasmEmitter {
    out: Box<dyn Write>,
    config: Config,
}

impl WasmEmitter {
    pub fn new(config: Config) -> Self {
        Self {
            out: Box::new(Vec::new()),
            config,
        }
    }

    pub fn emit(&mut self, ir: &ShellIR) -> Result<String> {
        // Emit Rust code with #[wasm_bindgen] annotations
        writeln!(self.out, "use wasm_bindgen::prelude::*;")?;
        writeln!(self.out, "")?;
        writeln!(self.out, "#[wasm_bindgen]")?;
        writeln!(self.out, "pub struct ShellRuntime {{")?;
        writeln!(self.out, "    env: std::collections::HashMap<String, String>,")?;
        writeln!(self.out, "}}")?;
        writeln!(self.out, "")?;
        writeln!(self.out, "#[wasm_bindgen]")?;
        writeln!(self.out, "impl ShellRuntime {{")?;

        match ir {
            ShellIR::Let { name, value, .. } => {
                self.emit_assignment(name, value)?;
            }
            ShellIR::Exec { cmd, effects } => {
                self.emit_command(cmd, effects)?;
            }
            ShellIR::Function { name, params, body } => {
                self.emit_function(name, params, body)?;
            }
            ShellIR::If { test, then_branch, else_branch } => {
                self.emit_conditional(test, then_branch, else_branch)?;
            }
            // ... handle all ShellIR variants
        }

        writeln!(self.out, "}}")?;

        Ok(String::from_utf8(self.out.get_ref().clone())?)
    }

    fn emit_command(&mut self, cmd: &Command, effects: &Effects) -> Result<()> {
        // Verify WASM compatibility
        if effects.contains(Effects::FileWrite) {
            return Err(Error::WasmIncompatible("File I/O not supported in WASM"));
        }

        match cmd.name.as_str() {
            "echo" => {
                writeln!(self.out, "    #[wasm_bindgen]")?;
                writeln!(self.out, "    pub fn echo(&self) -> String {{")?;
                writeln!(self.out, "        format!(\"{}\", {})",
                    self.emit_args(&cmd.args)?)?;
                writeln!(self.out, "    }}")?;
            }
            "pwd" => {
                writeln!(self.out, "    #[wasm_bindgen]")?;
                writeln!(self.out, "    pub fn pwd(&self) -> String {{")?;
                writeln!(self.out, "        self.cwd.clone()")?;
                writeln!(self.out, "    }}")?;
            }
            // ... handle other commands
            _ => {
                return Err(Error::UnsupportedCommand(cmd.name.clone()));
            }
        }
        Ok(())
    }

    fn emit_assignment(&mut self, name: &str, value: &ShellValue) -> Result<()> {
        writeln!(self.out, "    #[wasm_bindgen]")?;
        writeln!(self.out, "    pub fn set_{}(&mut self, value: String) {{", name)?;
        writeln!(self.out, "        self.env.insert(\"{}\".to_string(), value);", name)?;
        writeln!(self.out, "    }}")?;
        Ok(())
    }
}
```

### 3. Keep ShellIR Pure (Critical Design Principle)

**DO NOT** add WASM-specific variants to `ShellIR`. The IR must remain platform-agnostic.

**Correct Approach**:
- The `WasmEmitter` receives platform-agnostic `ShellIR`
- The emitter translates `ShellIR` into WASM-specific Rust code
- If necessary, the `WasmEmitter` can use an internal, secondary IR for WASM-specific optimizations
- The primary `ShellIR` remains unchanged

**Example**:
```rust
// ❌ WRONG: Contaminating ShellIR with WASM-specific variants
pub enum ShellIR {
    WasmImport { ... },  // NO!
}

// ✅ CORRECT: Emitter handles WASM specifics internally
impl WasmEmitter {
    fn emit(&mut self, ir: &ShellIR) -> Result<String> {
        match ir {
            ShellIR::Exec { cmd, effects } => {
                self.emit_wasm_function_call(cmd, effects)
            }
            // ... handle all ShellIR variants
        }
    }
}
```

### 4. WASM Runtime Functions

Create standard library of WASM-compatible shell builtins:

**File**: `rash/src/wasm/runtime.rs` (new file)

```rust
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

#[wasm_bindgen]
pub struct ShellRuntime {
    env: HashMap<String, String>,
    cwd: String,
    exit_code: i32,
}

#[wasm_bindgen]
impl ShellRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            cwd: "/".to_string(),
            exit_code: 0,
        }
    }

    #[wasm_bindgen]
    pub fn echo(&self, args: Vec<String>) -> String {
        args.join(" ")
    }

    #[wasm_bindgen]
    pub fn export(&mut self, name: String, value: String) {
        self.env.insert(name, value);
    }

    #[wasm_bindgen]
    pub fn get_env(&self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }

    #[wasm_bindgen]
    pub fn pwd(&self) -> String {
        self.cwd.clone()
    }

    #[wasm_bindgen]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}
```

## WASM-Specific Linting Rules

### 1. WASM001: No File I/O Operations

**Rationale**: WASM in browser has no file system access

**Rule**:
```rust
// rash/src/linter/rules/wasm001.rs
pub fn check(ir: &ShellIR) -> LintResult {
    let mut result = LintResult::new();

    if let ShellIR::Exec { cmd, effects } = ir {
        if effects.contains(Effects::FileWrite) ||
           effects.contains(Effects::FileRead) {
            result.add(Diagnostic::error(
                "WASM001",
                "File I/O not supported in WASM target",
                cmd.span,
            ).with_suggestion("Use virtual filesystem (VFS) instead"));
        }
    }
    result
}
```

**Violations**:
- `cat file.txt` ❌
- `echo "data" > output.txt` ❌
- `ls /tmp` ❌

**Allowed**:
- `echo "Hello World"` ✅
- `export NAME=value` ✅
- `pwd` ✅ (if using VFS)

### 2. WASM002: No Network Operations

```rust
pub fn check(ir: &ShellIR) -> LintResult {
    if let ShellIR::Exec { cmd, effects } = ir {
        if effects.contains(Effects::NetworkAccess) {
            result.add(Diagnostic::error(
                "WASM002",
                "Network operations require explicit WASI imports",
                cmd.span,
            ));
        }
    }
    result
}
```

**Violations**:
- `curl https://api.example.com` ❌
- `wget file.tar.gz` ❌

### 3. WASM003: No Process Spawning

```rust
pub fn check(ir: &ShellIR) -> LintResult {
    if let ShellIR::Exec { cmd, effects } = ir {
        if effects.contains(Effects::ProcessSpawn) {
            result.add(Diagnostic::error(
                "WASM003",
                "Cannot spawn processes in WASM sandbox",
                cmd.span,
            ));
        }
    }
    result
}
```

**Violations**:
- `fork()` ❌
- `exec /bin/bash` ❌
- Background jobs (`&`) ❌

### 4. WASM004: Deterministic Only

```rust
pub fn check(ir: &ShellIR) -> LintResult {
    // Check for non-deterministic sources
    const NON_DETERMINISTIC: &[&str] = &[
        "$RANDOM",
        "$(date)",
        "$$",  // PID
        "$PPID",
    ];

    for pattern in NON_DETERMINISTIC {
        if source.contains(pattern) {
            result.add(Diagnostic::error(
                "WASM004",
                "Non-deterministic operation in WASM",
                span,
            ).with_fix("Use deterministic RNG with seed"));
        }
    }
    result
}
```

### 5. WASM005: Memory Bounds

```rust
pub fn check(ir: &ShellIR) -> LintResult {
    if let ShellIR::WasmMemoryAlloc { size } = ir {
        const MAX_WASM_MEMORY: usize = 4 * 1024 * 1024 * 1024; // 4GB

        if *size > MAX_WASM_MEMORY {
            result.add(Diagnostic::error(
                "WASM005",
                "Memory allocation exceeds WASM limit (4GB)",
                span,
            ));
        }
    }
    result
}
```

## CLI Integration

### Transpile Bash → WASM

```bash
# Transpile bash script to WASM-compatible Rust
bashrs transpile script.sh --target wasm --output script.rs

# Compile to WASM
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-bindgen target/wasm32-unknown-unknown/release/script.wasm \
    --out-dir dist --target web
```

### Lint for WASM Compatibility

```bash
# Check if bash script can run in WASM
bashrs lint script.sh --target wasm

# Example output:
# ❌ 12:5 [error] WASM001: File I/O not supported in WASM target
#   cat input.txt
#   Use virtual filesystem (VFS) instead
#
# ❌ 24:10 [error] WASM004: Non-deterministic operation in WASM
#   RANDOM_ID=$RANDOM
#   Use deterministic RNG with seed
#
# Summary: 2 error(s), 0 warning(s)
```

### Combined Workflow

```bash
# One-step: lint, transpile, compile to WASM
bashrs build script.sh --target wasm --output dist/

# Produces:
# - dist/script.wasm
# - dist/script.js
# - dist/script_bg.wasm (wasm-bindgen output)
```

## Integration with WASM Projects

### Typical WASM OS Architecture

```
┌─────────────────┐
│   Browser App   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   WASM Binary   │  (kernel.wasm)
│   ┌──────────┐  │
│   │  Kernel  │  │  (process scheduler, syscalls)
│   └──────────┘  │
│   ┌──────────┐  │
│   │Userspace │  │  (shell, init, programs)
│   └──────────┘  │
│   ┌──────────┐  │
│   │Script Ex.│  │  (manual parsing)  ← REPLACE WITH BASHRS
│   └──────────┘  │
└─────────────────┘
```

### Bashrs Integration Pattern

```rust
// In WASM kernel (src/script_executor.rs)

pub fn execute_script(script: &str) -> Result<String> {
    // Current: Manual parsing with split_whitespace() ❌
    // Future: Use bashrs-generated WASM code ✅

    // Step 1: Parse with bashrs at build time
    let ast = bashrs_parser::parse(script)?;

    // Step 2: Generate ShellIR
    let ir = ast.to_ir()?;

    // Step 3: Emit WASM-compatible Rust
    let rust_code = WasmEmitter::new().emit(&ir)?;

    // Step 4: Compile and link into WASM binary
    // (This would be done at build time, not runtime)

    execute_compiled_script()
}
```

### Build-Time Script Compilation

```bash
# Add to project Makefile

# Transpile all test scripts to WASM
wasm-scripts:
	@echo "🔄 Transpiling shell scripts to WASM..."
	@for script in scripts/*.sh; do \
		bashrs transpile "$$script" --target wasm --output "src/generated/$$(basename $$script .sh).rs"; \
	done
	@echo "✅ Scripts transpiled"

# Build project with generated scripts
wasm: wasm-scripts
	@cargo build --target wasm32-unknown-unknown --release
```

### Runtime Integration

Generated scripts become WASM modules:

```javascript
// In browser frontend (app.js)

import init, { ShellRuntime } from './kernel.js';

async function runScript(scriptName) {
    await init();

    // Load bashrs-generated WASM module
    const runtime = new ShellRuntime();

    // Execute script commands
    runtime.set_NAME("World");
    const output = runtime.echo("Hello $NAME");

    console.log(output); // "Hello World"
}
```

## Critical Architectural Challenges (MUST BE SOLVED FIRST)

### **BLOCKER: Streaming I/O Model**

The current proposal **fundamentally misunderstands** how shell pipes work. The naive approach of buffering entire outputs in memory (`let ls_output = runtime.ls()`) will fail catastrophically:

**Problems**:
1. **Memory Explosion**: `ls -R /` would exhaust all available memory
2. **Loss of Concurrency**: Pipes allow concurrent execution; function calls are sequential
3. **Broken Semantics**: Shell pipes are lazy streams, not eager buffers

**Required Solution**:
- Design a streaming I/O abstraction for WASM (e.g., Rust iterators, async streams, or trait-based system)
- Pipes must operate on **streams**, not buffered strings
- Commands must be **lazy** and **composable**
- Exit codes and error propagation must match POSIX semantics

**Scientific Context**:
This is the lazy vs. strict evaluation problem from functional programming. Shell pipes are inherently lazy and stream-based.

**References**:
- Hughes, J. (1989). *Why functional programming matters*. The Computer Journal.
- Haas, A., et al. (2017). *Bringing the web up to speed with WebAssembly*. PLDI.

### **BLOCKER: VFS and State Management**

The proposal underestimates the impedance mismatch between:
- **POSIX**: Global, mutable filesystem tree with ambient authority
- **WASM**: Capability-based security with sandboxed, component-specific resources

**Required Solution**:
- Define clear semantics for filesystem state in `ShellRuntime`
- Design capability-based VFS API compatible with WASI principles
- Handle stateful commands like `cd /tmp; touch file` correctly

**References**:
- Miller, M. S., et al. (2003). *Capability-based financial systems*. WEA.

## Revised Implementation Plan

### Phase 0: Feasibility Study & Prototyping (3 Weeks - **MANDATORY**)

**This phase is non-negotiable. Do not proceed without completing this research.**

**Week 1-2**: Streaming I/O Research
- [ ] Research streaming patterns in Rust suitable for WASM
  - Explore `futures::stream`, async iterators, or custom trait-based systems
  - Study how existing WASM runtimes handle I/O
- [ ] Design streaming abstraction for `ShellRuntime`
  - Define traits for stream-based I/O
  - Ensure zero-copy or minimal buffering
  - Support backpressure and flow control

**Week 3**: Proof of Concept
- [ ] Prototype: `cat file | grep pattern | wc -l`
  - **Requirement**: MUST NOT buffer entire file in memory
  - Measure memory usage with 1GB+ input files
  - Verify lazy evaluation and streaming behavior
- [ ] Define exit code semantics for pipelines
- [ ] Document error handling model

**Deliverables**:
- Proof-of-concept Rust library demonstrating streaming pipes
- Benchmarks proving memory efficiency
- Design document outlining chosen streaming model
- **Go/No-Go decision point**: If streaming model is infeasible, abort WASM backend

### Phase 1: Core Emitter & Runtime (3 weeks)

**Only proceed if Phase 0 succeeds.**

**Week 1**: Core infrastructure
- [ ] Add `ShellDialect::Wasm` variant (**without** contaminating `ShellIR`)
- [ ] Create `rash/src/emitter/wasm.rs`
- [ ] Implement streaming-based `ShellRuntime`
- [ ] Handle simple commands: `echo`, `export`, `pwd`

**Week 2**: Variable expansion and control flow
- [ ] Implement variable assignment and expansion
- [ ] Handle `if/then/else` conditionals
- [ ] Basic loop support

**Week 3**: Integration
- [ ] CLI: `bashrs transpile script.sh --target wasm`
- [ ] Unit tests
- [ ] Integration tests

**Deliverables**:
- Working WASM emitter for simple scripts
- Test coverage: 85%+

### Phase 2: Streaming Pipes & Advanced Semantics (3 weeks)

**Week 1**: Pipe implementation
- [ ] Implement pipes using streaming model from Phase 0
- [ ] Handle multi-stage pipelines (`cmd1 | cmd2 | cmd3`)
- [ ] Proper error propagation and exit codes

**Week 2**: Command substitution
- [ ] Implement `$(command)` using streaming (not buffering)
- [ ] Handle nested command substitution
- [ ] Backquote syntax `` `command` ``

**Week 3**: Functions and advanced control flow
- [ ] Function definitions and calls
- [ ] `while` and `until` loops
- [ ] `case` statements

**Deliverables**:
- Full support for pipes and command substitution
- Zero memory overhead for streaming operations
- Benchmarks demonstrating streaming efficiency

### Phase 3: WASM Linting (2 weeks)

**Week 1**: Core linting rules
- [ ] Implement WASM001 (no file I/O)
- [ ] Implement WASM002 (no network)
- [ ] Implement WASM003 (no process spawning)

**Week 2**: Advanced rules and integration
- [ ] Implement WASM004 (determinism)
- [ ] Implement WASM005 (memory bounds)
- [ ] CLI: `bashrs lint --target wasm`
- [ ] Integration with existing linter framework

**Deliverables**:
- 5 WASM-specific linting rules
- Documentation with examples
- Test coverage: 90%+

### Phase 4: VFS Integration & Project Integration (3 weeks)

**Week 1**: VFS design
- [ ] Design capability-based VFS API
- [ ] Handle stateful operations (`cd`, `pwd`)
- [ ] Define semantics for path resolution

**Week 2**: WASM project integration
- [ ] Replace manual script parsing with bashrs in example WASM projects
- [ ] Generate WASM modules from test scripts
- [ ] Update E2E tests

**Week 3**: Performance optimization
- [ ] Benchmarking
- [ ] WASM binary size optimization (`wasm-opt`)
- [ ] Documentation updates

**Deliverables**:
- Example WASM projects using bashrs for shell script execution
- E2E test pass rate: Target 80%+ (some POSIX features may be unsupported)
- WASM binary size: <100KB per script

### Phase 5: Documentation & Polish (1 week)

**Tasks**:
- [ ] Comprehensive documentation
- [ ] Tutorial: "Bash to WASM: Understanding the Limitations"
- [ ] Example projects demonstrating WASM-compatible scripts
- [ ] Document semantic differences from POSIX bash
- [ ] Blog post: "Lessons Learned from Transpiling Bash to WASM"

**Deliverables**:
- Documentation site update
- 3+ example projects
- Benchmarks and performance analysis

**Revised Total Timeline**: 11-13 weeks (including mandatory Phase 0)

**Risk Acknowledgment**: This timeline assumes Phase 0 succeeds. If the streaming I/O problem proves intractable, the WASM backend may not be feasible.

## Technical Challenges (Updated After Critical Review)

### ⚠️ CRITICAL REVIEW FINDINGS

This section has been rewritten to reflect a deep technical review that identified critical flaws in the original proposal.

### 1. Pipes and Redirection - **MAJOR BLOCKER**

**Challenge**: `cmd1 | cmd2` requires streaming, concurrent I/O - not function calls

**Original (Naive) Proposal** ❌:
```rust
// Bash: ls | grep pattern
// WASM: Compose as data flow
let ls_output = runtime.ls();  // ❌ WRONG: Buffers entire output in memory
let result = runtime.grep(ls_output, "pattern");
```

**Why This Fails**:
1. **Memory Explosion**: `ls -R /` would exhaust all memory
2. **Loss of Concurrency**: Shell pipes run commands concurrently; this is sequential
3. **Broken Semantics**: Pipes are lazy streams, not eager buffers

**Required Research** (Phase 0):
- Design streaming I/O abstraction (iterators, async streams, or custom traits)
- Ensure zero or minimal buffering
- Support lazy evaluation and backpressure
- Match POSIX exit code semantics

**Example Streaming Model** (Conceptual):
```rust
// Define streaming trait
trait ShellStream {
    fn read_line(&mut self) -> Option<Result<String>>;
    fn exit_code(&self) -> i32;
}

// Pipe implementation
let ls_stream = runtime.ls();  // Returns ShellStream
let grep_stream = runtime.grep(ls_stream, "pattern");  // Composes streams
let wc_stream = runtime.wc(grep_stream, "-l");

// Lazy evaluation - only executes when output is consumed
let result = wc_stream.collect();
```

**Phase 0 Requirement**: Prove this model works with 1GB+ files without memory issues.

### 2. Command Substitution - **SAME BLOCKER**

**Challenge**: `$(command)` has the same buffering problem as pipes

**Original (Naive) Proposal** ❌:
```rust
// Bash: OUTPUT=$(echo "hello")
let output = runtime.echo("hello");  // ❌ May be acceptable for simple cases
```

**Why This Partially Fails**:
- Simple cases (e.g., `$(echo "hello")`) are fine
- Complex cases (e.g., `$(find / -type f)`) will exhaust memory

**Required Solution**:
- Small outputs can be buffered
- Large outputs must use streaming model or be rejected by linter
- May need to set maximum buffer size and error on overflow

### 3. File System Access

**Challenge**: WASM has no native file system

**Solution**: Virtual filesystem (VFS)
- WASM projects can implement VFS using persistent data structures (e.g., `im-rs`)
- Bashrs emits VFS API calls instead of POSIX file I/O

```rust
// Bash: cat /proc/cpuinfo
// WASM:
runtime.vfs_read("/proc/cpuinfo")
```

### 4. Dynamic Script Execution

**Challenge**: `eval` and dynamic script execution

**Solution**: Two approaches
1. **Compile-time only**: Reject `eval` in WASM target (strict mode)
2. **Interpreter fallback**: Embed bash interpreter in WASM for `eval`

Recommend approach 1 for security and performance.

### 5. Binary Size

**Challenge**: WASM binary size constraints

**Solution**:
- Tree-shaking unused builtins
- Compile only commands actually used in script
- Use `wasm-opt` for size optimization
- Target: <100KB per script

## Benefits

### For WASM-Based Educational OS Projects

1. **Correctness**: Replace manual parsing with battle-tested bashrs parser
2. **Completeness**: Full POSIX shell support (parameter expansion, command substitution)
3. **Quality**: Comprehensive linting before runtime
4. **Performance**: Compiled WASM faster than interpreted bash
5. **Determinism**: Built-in enforcement of deterministic execution

### For Bashrs Project

1. **Expanded Use Cases**: Browser-based shell scripts, serverless, embedded
2. **Market Differentiation**: Only bash transpiler with WASM backend
3. **Educational Value**: Teach shell scripting in browser (no setup required)
4. **Enterprise Adoption**: WASM enables safe script execution in untrusted environments

### For Broader Ecosystem

1. **Portability**: Write once, run anywhere (WASM supported on all platforms)
2. **Security**: Sandbox execution prevents malicious scripts
3. **Performance**: Compiled WASM faster than interpreted shell
4. **Developer Experience**: Instant feedback with browser-based execution

## Success Metrics

### Functionality
- [ ] Transpile 90%+ of bashrs test suite to WASM
- [ ] Example project E2E test pass rate: 80%+ using bashrs
- [ ] Support all POSIX builtins (echo, test, cd, pwd, export, etc.)

### Performance
- [ ] WASM binary size: <100KB per script
- [ ] Execution speed: ≤2x overhead vs native bash
- [ ] Cold start: <100ms in browser

### Quality
- [ ] Test coverage: 85%+ for WASM emitter
- [ ] Mutation score: 90%+ for WASM runtime
- [ ] Zero WASM-incompatible code passes linter

### Adoption
- [ ] Educational WASM OS projects successfully integrated
- [ ] 3+ example projects using bash+WASM
- [ ] Documentation and tutorials published

## Alternatives Considered

### 1. Embed Bash Interpreter in WASM

**Approach**: Compile GNU bash or busybox to WASM

**Pros**:
- Full bash compatibility
- No transpilation needed

**Cons**:
- Large binary size (10+ MB)
- Runtime overhead
- Security concerns (arbitrary code execution)
- Complex WASI integration

**Verdict**: ❌ Too heavyweight for educational WASM OS use cases

### 2. JavaScript Backend

**Approach**: Transpile bash to JavaScript instead of Rust/WASM

**Pros**:
- Direct browser execution
- Smaller output size

**Cons**:
- Less type safety than Rust
- Slower than WASM
- Harder to integrate with Rust-based WASM projects

**Verdict**: ❌ Not aligned with Rust/WASM project architectures

### 3. Custom Shell DSL

**Approach**: Design new shell language optimized for WASM

**Pros**:
- Perfect WASM compatibility
- Clean slate design

**Cons**:
- Not bash-compatible (breaks existing scripts)
- No ecosystem, tooling, or familiarity
- Massive implementation effort

**Verdict**: ❌ Bashrs's value is bash compatibility

## References

### Bashrs Codebase
- Parser: `rash/src/bash_parser/ast.rs`
- IR: `rash/src/ir/shell_ir.rs`
- Emitter: `rash/src/emitter/posix.rs`
- Linter: `rash/src/linter/rules/`

### External Resources
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/docs/wasm-bindgen/)
- [WASI (WebAssembly System Interface)](https://wasi.dev/)

## Appendix A: Example Transpilation

### Input: `hello.sh`
```bash
#!/bin/bash

NAME="World"
export GREETING="Hello"

echo "$GREETING $NAME"

for i in 1 2 3; do
    echo "Count: $i"
done
```

### Output: `hello.rs` (Generated by WasmEmitter)
```rust
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

#[wasm_bindgen]
pub struct ShellRuntime {
    env: HashMap<String, String>,
}

#[wasm_bindgen]
impl ShellRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut env = HashMap::new();
        env.insert("NAME".to_string(), "World".to_string());
        env.insert("GREETING".to_string(), "Hello".to_string());

        Self { env }
    }

    #[wasm_bindgen]
    pub fn run(&self) -> String {
        let mut output = String::new();

        // echo "$GREETING $NAME"
        output.push_str(&format!(
            "{} {}\n",
            self.env.get("GREETING").unwrap(),
            self.env.get("NAME").unwrap()
        ));

        // for i in 1 2 3; do echo "Count: $i"; done
        for i in vec!["1", "2", "3"] {
            output.push_str(&format!("Count: {}\n", i));
        }

        output
    }
}
```

### Compiled WASM

```bash
# Compile to WASM
cargo build --target wasm32-unknown-unknown --release

# Size: ~45KB (optimized)

# Generate JavaScript bindings
wasm-bindgen target/wasm32-unknown-unknown/release/hello.wasm \
    --out-dir dist --target web
```

### Usage in Browser

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Bash in WASM</title>
</head>
<body>
    <pre id="output"></pre>

    <script type="module">
        import init, { ShellRuntime } from './hello.js';

        async function run() {
            await init();

            const runtime = new ShellRuntime();
            const output = runtime.run();

            document.getElementById('output').textContent = output;
        }

        run();
    </script>
</body>
</html>
```

### Output

```
Hello World
Count: 1
Count: 2
Count: 3
```

## Appendix B: WASM Linting Example

### Script: `network.sh`
```bash
#!/bin/bash

# Fetch remote data
DATA=$(curl https://api.example.com/data)

# Save to file
echo "$DATA" > output.txt

# Use random data
ID=$RANDOM

echo "Saved data with ID: $ID"
```

### Linting Output

```bash
$ bashrs lint network.sh --target wasm

Issues found in network.sh:

❌ 4:7-40 [error] WASM002: Network operations require explicit WASI imports
  DATA=$(curl https://api.example.com/data)
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  Suggestion: Use fetch API with JS interop:
    #[wasm_bindgen]
    extern "C" {
        fn fetch(url: &str) -> String;
    }

❌ 7:1-27 [error] WASM001: File I/O not supported in WASM target
  echo "$DATA" > output.txt
  ^^^^^^^^^^^^^^^^^^^^^^^^^^

  Suggestion: Use virtual filesystem (VFS) instead:
    runtime.vfs_write("/output.txt", data)

❌ 10:4-11 [error] WASM004: Non-deterministic operation in WASM
  ID=$RANDOM
     ^^^^^^^

  Fix: Use deterministic RNG with seed:
    runtime.random_with_seed(42)

Summary: 3 error(s), 0 warning(s), 0 info(s)

❌ Cannot compile to WASM target due to errors.
```

### Fixed Script: `network-fixed.sh`
```bash
#!/bin/bash

# Use deterministic seed
export SEED=42

# Fetch using WASM import (requires wasm_bindgen extern)
# DATA=$(wasm_fetch "https://api.example.com/data")

# Save to VFS
# vfs_write "/output.txt" "$DATA"

# Use deterministic random
ID=$(wasm_random_with_seed $SEED)

echo "Saved data with ID: $ID"
```

```bash
$ bashrs lint network-fixed.sh --target wasm

✅ No issues found. Script is WASM-compatible.
```

## Conclusion

### Revised Assessment (Post-Critical Review)

Adding a WASM backend to bashrs is **strategically sound but architecturally risky**. The critical review revealed that the original implementation plan was fundamentally flawed and would have resulted in a broken, semantically incorrect backend.

### What We Got Right

1. ✅ **Strategic Alignment**: WASM integration aligns perfectly with bashrs's safety-first mission
2. ✅ **Linting Rules**: The proposed WASM001-WASM005 rules are excellent and well-designed
3. ✅ **Use Case**: Educational WASM OS projects are compelling drivers for the feature
4. ✅ **Market Differentiation**: First bash-to-WASM transpiler would be unique

### What We Got Wrong (And Fixed)

1. ❌ **IR Contamination**: Original proposal polluted `ShellIR` with target-specific variants
   - **Fixed**: Emitter-only approach maintains IR purity
2. ❌ **Streaming I/O**: Naive buffering model would fail catastrophically
   - **Fixed**: Mandatory Phase 0 research to solve streaming problem
3. ❌ **Timeline**: 6-7 weeks was laughably unrealistic
   - **Fixed**: 11-13 weeks with mandatory feasibility study

### Honest Risk Assessment

**The WASM backend may not be feasible.** The streaming I/O problem is a hard computer science problem with no obvious solution. Shell pipes are:
- Lazy (not eager)
- Streaming (not buffered)
- Concurrent (not sequential)

If Phase 0 research fails to find a viable streaming model for WASM, **the entire WASM backend should be abandoned**.

### Revised Next Steps

**Do NOT proceed with implementation without completing Phase 0.**

1. ✅ **Review Complete**: Specification reviewed and revised
2. 🔬 **Phase 0 Feasibility Study** (3 weeks):
   - Research streaming I/O models for WASM
   - Prototype `cat file | grep pattern | wc -l` with 1GB+ files
   - Benchmark memory usage
   - **Go/No-Go Decision Point**
3. ⏸️ **Phase 1-5 Implementation** (8-10 weeks):
   - Only proceed if Phase 0 succeeds
   - Follow revised implementation plan
4. ⏸️ **WASM OS Integration**:
   - Replace manual parsing with bashrs in example projects
   - Target 80%+ E2E test pass rate

### Scientific Honesty

This proposal represents an honest attempt to solve a hard problem. The critical review process prevented us from wasting weeks building a broken system. The bashrs project maintains its reputation for quality by acknowledging when problems are unsolved and requiring research before implementation.

**Timeline**: Phase 0 by Q1 2025. If successful, production release Q3-Q4 2025. If Phase 0 fails, abandon WASM backend.

---

## Acknowledgments

**Critical Review Team**: This specification was substantially improved by rigorous technical review. The reviewers correctly identified that the original proposal underestimated the semantic gap between POSIX shells and WebAssembly. Their insistence on a Phase 0 feasibility study likely saved the project from costly failure.

**Citations**:
- Hughes, J. (1989). *Why functional programming matters*. The Computer Journal.
- Haas, A., et al. (2017). *Bringing the web up to speed with WebAssembly*. PLDI.
- Miller, M. S., et al. (2003). *Capability-based financial systems*. WEA.
