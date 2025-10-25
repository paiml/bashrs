# bashrs WASM Runtime Enhancement Roadmap

**Date**: 2025-10-24
**Status**: Phase 0 Complete - Ready for Runtime Implementation
**Goal**: Build a bash runtime in WASM for educational and interactive shell execution

---

## Vision

Enable **safe, sandboxed bash script execution** entirely in the browser using WebAssembly. This transforms bashrs from a linting/analysis tool into a **complete educational shell environment** for WOS and interactive.paiml.com.

### Use Cases

1. **WOS (Web Operating System)**
   - URL: https://interactive.paiml.com/wos/
   - Use: Interactive terminal with bash execution
   - Need: Safe, sandboxed bash runtime (no system access)

2. **interactive.paiml.com**
   - URL: https://interactive.paiml.com
   - Use: Shell scripting tutorials with live execution
   - Need: Real-time feedback, syntax highlighting, safe execution

3. **bashrs Playground**
   - Use: Test and learn bash scripts in browser
   - Need: Full bash feature support, debugger, step-through

---

## Current State Analysis

### What Exists Today ✅

#### 1. Parser Infrastructure (100% Complete)
- **bash_parser**: Full bash syntax parser
- **make_parser**: Makefile parser
- **AST**: Complete abstract syntax tree representation
- **Location**: `/home/noah/src/bashrs/rash/src/bash_parser/`

#### 2. Standard Library Mapping (30% Complete)
- **stdlib.rs**: Maps Rust stdlib to shell functions
- **Functions**: 18 stdlib functions mapped
  - String: trim, contains, len, split, replace, to_upper, to_lower
  - Filesystem: exists, read_file, write_file, copy, remove, is_file, is_dir
  - Array: len, join
  - Environment: env, env_var_or
  - Arguments: arg, args, arg_count
  - Exit: exit_code
- **Location**: `/home/noah/src/bashrs/rash/src/stdlib.rs`

#### 3. WASM API (Phase 0 Complete)
- **Config Analysis**: analyze_config() ✅
- **Purification**: purify_config() ✅
- **Version**: version() ✅
- **Location**: `/home/noah/src/bashrs/rash/src/wasm/api.rs`

#### 4. Emitter (POSIX Shell Generation)
- **posix.rs**: Generates POSIX-compliant shell
- **Determinism**: Enforces deterministic output
- **Idempotency**: Ensures safe re-run
- **Location**: `/home/noah/src/bashrs/rash/src/emitter/posix.rs`

### What's Missing ❌

#### 1. Runtime Executor (0% Complete)
- **Need**: Execute parsed bash AST in WASM
- **Blocker**: No execution engine exists
- **Estimate**: 4-6 weeks to build

#### 2. Virtual Filesystem (0% Complete)
- **Need**: In-memory filesystem for WASM
- **Blocker**: No VFS implementation
- **Estimate**: 2-3 weeks

#### 3. Process Management (0% Complete)
- **Need**: Mock process table, pipes, signals
- **Blocker**: No process abstraction
- **Estimate**: 2-3 weeks

#### 4. I/O Redirection (0% Complete)
- **Need**: stdin, stdout, stderr capture
- **Blocker**: No I/O system
- **Estimate**: 1-2 weeks

#### 5. Environment Variables (20% Complete)
- **Partial**: env() functions exist in stdlib
- **Missing**: Full environment table, export, sourcing
- **Estimate**: 1 week

---

## Architecture Design

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  JavaScript / Browser                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │  WOS Terminal / interactive.paiml.com UI          │  │
│  └───────────────────┬──────────────────────────────┘  │
│                      │                                   │
│                      ▼                                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │         bashrs WASM API (api.rs)                  │  │
│  │  - analyzeConfig()                                 │  │
│  │  - purifyConfig()                                  │  │
│  │  - executeScript() ← NEW                          │  │
│  │  - createShell() ← NEW                            │  │
│  └───────────────────┬──────────────────────────────┘  │
└────────────────────┬─┴────────────────────────────────┘
                     │
           ┌─────────▼──────────┐
           │  WASM Runtime       │
           │  (Rust compiled)    │
           └─────────┬───────────┘
                     │
      ┌──────────────┼──────────────┐
      │              │               │
      ▼              ▼               ▼
┌──────────┐  ┌──────────┐   ┌──────────┐
│  Parser   │  │ Executor │   │   VFS    │
│  (exists) │  │  (NEW)   │   │  (NEW)   │
└──────────┘  └────┬─────┘   └──────────┘
                   │
            ┌──────┴──────┐
            │             │
            ▼             ▼
      ┌──────────┐  ┌──────────┐
      │ Builtins │  │ Stdlib   │
      │  (NEW)   │  │ (partial)│
      └──────────┘  └──────────┘
```

### Components to Build

#### 1. Runtime Executor
**File**: `src/wasm/executor.rs`

```rust
/// Execute a bash script in WASM runtime
pub struct BashExecutor {
    /// Virtual filesystem
    vfs: VirtualFilesystem,

    /// Environment variables
    env: HashMap<String, String>,

    /// Process table (for pipelines)
    processes: Vec<Process>,

    /// I/O streams
    io: IoStreams,

    /// Exit code
    exit_code: i32,
}

impl BashExecutor {
    /// Execute a bash script
    pub fn execute(&mut self, source: &str) -> ExecutionResult {
        // 1. Parse to AST
        let ast = parse_bash(source)?;

        // 2. Walk AST and execute
        self.execute_ast(&ast)
    }

    /// Execute AST node
    fn execute_ast(&mut self, node: &AstNode) -> Result<Value> {
        match node {
            AstNode::Command(cmd) => self.execute_command(cmd),
            AstNode::Pipeline(cmds) => self.execute_pipeline(cmds),
            AstNode::If(cond, then_block, else_block) => {
                self.execute_if(cond, then_block, else_block)
            }
            // ... other node types
        }
    }
}
```

#### 2. Virtual Filesystem
**File**: `src/wasm/vfs.rs`

```rust
/// In-memory virtual filesystem for WASM
pub struct VirtualFilesystem {
    /// File tree
    root: VfsNode,

    /// Current working directory
    cwd: PathBuf,

    /// Open file handles
    handles: HashMap<u32, FileHandle>,
}

pub enum VfsNode {
    File { content: Vec<u8>, perms: u32 },
    Directory { children: HashMap<String, VfsNode> },
    Symlink { target: PathBuf },
}

impl VirtualFilesystem {
    /// Read file
    pub fn read(&self, path: &Path) -> Result<Vec<u8>> { }

    /// Write file
    pub fn write(&mut self, path: &Path, content: &[u8]) -> Result<()> { }

    /// List directory
    pub fn readdir(&self, path: &Path) -> Result<Vec<String>> { }

    /// Create directory
    pub fn mkdir(&mut self, path: &Path) -> Result<()> { }
}
```

#### 3. Built-in Commands
**File**: `src/wasm/builtins.rs`

```rust
/// Bash built-in commands
pub struct Builtins;

impl Builtins {
    /// Execute built-in command
    pub fn execute(
        &self,
        name: &str,
        args: &[String],
        ctx: &mut ExecutionContext,
    ) -> Result<i32> {
        match name {
            "echo" => self.echo(args, ctx),
            "cd" => self.cd(args, ctx),
            "pwd" => self.pwd(ctx),
            "export" => self.export(args, ctx),
            "source" => self.source(args, ctx),
            "test" | "[" => self.test(args, ctx),
            _ => Err(format!("Unknown builtin: {}", name)),
        }
    }

    fn echo(&self, args: &[String], ctx: &mut ExecutionContext) -> Result<i32> {
        let output = args.join(" ");
        ctx.io.stdout.write_all(output.as_bytes())?;
        ctx.io.stdout.write_all(b"\n")?;
        Ok(0)
    }

    fn cd(&self, args: &[String], ctx: &mut ExecutionContext) -> Result<i32> {
        let path = args.get(0).map(|s| s.as_str()).unwrap_or("~");
        ctx.vfs.chdir(path)?;
        Ok(0)
    }

    // ... other builtins
}
```

#### 4. I/O Streams
**File**: `src/wasm/io.rs`

```rust
/// I/O streams for bash execution
pub struct IoStreams {
    pub stdin: Box<dyn Read>,
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
}

/// Capture stdout/stderr
pub struct CaptureWriter {
    buffer: Vec<u8>,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
```

---

## Implementation Phases

### Phase 1: Minimal Viable Runtime (3 weeks)

**Goal**: Execute simple bash commands in WASM

**Deliverables**:
1. ✅ Execute echo, pwd, cd
2. ✅ Variable assignment and expansion
3. ✅ Simple if/else conditionals
4. ✅ Virtual filesystem (basic)
5. ✅ Capture stdout/stderr

**API**:
```javascript
import { createShell } from './pkg/bashrs.js';

const shell = await createShell();
const result = await shell.execute(`
  echo "Hello from WASM bash!"
  name="Claude"
  echo "Hello, $name"
`);

console.log(result.stdout); // "Hello from WASM bash!\nHello, Claude\n"
console.log(result.exit_code); // 0
```

**Test Coverage**: 85%+
**Mutation Score**: 90%+

### Phase 2: Core Features (3 weeks)

**Goal**: Support common bash patterns

**Deliverables**:
1. ✅ Pipelines (cmd1 | cmd2)
2. ✅ Command substitution $(cmd)
3. ✅ For/while loops
4. ✅ Functions
5. ✅ Arrays
6. ✅ Arithmetic expansion $((expr))

**API**:
```javascript
const result = await shell.execute(`
  for i in 1 2 3; do
    echo "Number: $i"
  done

  count=$(echo "1 + 2 + 3" | bc)
  echo "Total: $count"
`);
```

**Test Coverage**: 85%+
**Mutation Score**: 90%+

### Phase 3: Advanced Features (4 weeks)

**Goal**: Full bash compatibility

**Deliverables**:
1. ✅ Process substitution <(cmd)
2. ✅ Here documents <<EOF
3. ✅ Case statements
4. ✅ Traps and signals
5. ✅ Subshells and background jobs
6. ✅ Pattern matching (globs)

**API**:
```javascript
const result = await shell.execute(`
  case "$OS" in
    Linux) echo "Linux system" ;;
    Darwin) echo "macOS system" ;;
    *) echo "Unknown system" ;;
  esac

  # Trap cleanup on exit
  trap 'echo "Cleaning up..."' EXIT
`);
```

**Test Coverage**: 85%+
**Mutation Score**: 90%+

### Phase 4: Interactive Shell (2 weeks)

**Goal**: REPL for WOS terminal

**Deliverables**:
1. ✅ Line editing (history, cursor movement)
2. ✅ Tab completion
3. ✅ Job control
4. ✅ PS1 prompt customization
5. ✅ .bashrc sourcing

**API**:
```javascript
const shell = await createInteractiveShell({
  prompt: "bash$ ",
  history: true,
  completion: true
});

shell.onOutput((text) => terminal.write(text));
shell.onError((error) => terminal.writeError(error));

// Send user input
shell.sendInput("ls -la\n");
```

**Test Coverage**: 85%+
**E2E Tests**: Playwright

---

## Integration Points

### 1. WOS Terminal Integration

**File**: Create in WOS repo: `/home/noah/src/wos/src/terminal/bash_runtime.rs`

```rust
use bashrs_wasm::{BashExecutor, IoStreams};

pub struct WosTerminal {
    executor: BashExecutor,
}

impl WosTerminal {
    pub fn new() -> Self {
        Self {
            executor: BashExecutor::new(),
        }
    }

    pub fn execute_command(&mut self, cmd: &str) -> String {
        match self.executor.execute(cmd) {
            Ok(result) => result.stdout,
            Err(e) => format!("Error: {}", e),
        }
    }
}
```

### 2. interactive.paiml.com Integration

**File**: Create in interactive.paiml.com repo

```javascript
// In tutorial page
import { createShell } from '@bashrs/wasm';

const shell = await createShell();

// Student writes code
const userScript = editor.getValue();

// Execute and show output
const result = await shell.execute(userScript);
outputPanel.setValue(result.stdout);

if (result.exit_code !== 0) {
  errorPanel.setValue(result.stderr);
}
```

---

## Testing Strategy

### 1. Unit Tests (EXTREME TDD)

```rust
#[test]
fn test_execute_echo() {
    let mut executor = BashExecutor::new();
    let result = executor.execute("echo 'hello world'").unwrap();
    assert_eq!(result.stdout, "hello world\n");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_variable_expansion() {
    let mut executor = BashExecutor::new();
    let result = executor.execute(r#"
        name="Claude"
        echo "Hello, $name"
    "#).unwrap();
    assert_eq!(result.stdout, "Hello, Claude\n");
}
```

### 2. Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_echo_never_panics(s in ".*") {
        let mut executor = BashExecutor::new();
        let cmd = format!("echo '{}'", s.replace("'", "'\\''"));
        let _ = executor.execute(&cmd);
        // Should never panic
    }

    #[test]
    fn prop_variable_assignment_deterministic(
        name in "[a-zA-Z_][a-zA-Z0-9_]*",
        value in ".*"
    ) {
        let mut executor = BashExecutor::new();
        let cmd = format!("{}='{}'; echo \"${}\"", name, value, name);
        let result1 = executor.execute(&cmd).unwrap();
        let result2 = executor.execute(&cmd).unwrap();
        assert_eq!(result1.stdout, result2.stdout);
    }
}
```

### 3. E2E Tests (Playwright)

```typescript
test('WOS terminal executes bash commands', async ({ page }) => {
  await page.goto('http://localhost:8000');

  // Type command in terminal
  await page.locator('#terminal-input').fill('echo "Hello, WOS!"\n');

  // Verify output
  const output = await page.locator('#terminal-output').textContent();
  expect(output).toContain('Hello, WOS!');
});
```

### 4. Mutation Testing

```bash
cargo mutants --file src/wasm/executor.rs --test-package bashrs

# Target: ≥90% kill rate
```

---

## Performance Targets

| Operation | Target | Justification |
|-----------|--------|---------------|
| WASM load | <5s | User patience threshold |
| Execute echo | <10ms | Interactive feel |
| Execute script (100 lines) | <100ms | Acceptable delay |
| Virtual FS operation | <1ms | Avoid lag |
| Memory usage | <50MB | Browser constraint |

---

## Security Considerations

### Sandboxing Requirements

1. **No System Access**: WASM runtime cannot access host filesystem
2. **No Network**: Cannot make HTTP requests (unless explicitly allowed)
3. **No Process Spawn**: Cannot spawn real processes
4. **Memory Limits**: Enforce max heap size
5. **CPU Limits**: Prevent infinite loops (timeout after 30s)

### Implementation

```rust
pub struct ExecutionLimits {
    pub max_memory: usize,      // 50MB default
    pub max_cpu_time: Duration, // 30s default
    pub max_stdout: usize,      // 1MB default
}

impl BashExecutor {
    pub fn execute_with_limits(
        &mut self,
        source: &str,
        limits: ExecutionLimits,
    ) -> Result<ExecutionResult> {
        // Start timeout timer
        let start = Instant::now();

        // Execute with checks
        loop {
            if start.elapsed() > limits.max_cpu_time {
                return Err("Timeout: Script exceeded 30s CPU limit".into());
            }

            if self.memory_usage() > limits.max_memory {
                return Err("Out of memory: Exceeded 50MB limit".into());
            }

            // ... execute next instruction
        }
    }
}
```

---

## Roadmap Timeline

| Phase | Duration | Deliverables | Status |
|-------|----------|--------------|--------|
| Phase 0 | ✅ 2 days | WASM build, linting, tests | COMPLETE |
| Phase 1 | 3 weeks | Minimal runtime (echo, cd, vars) | READY TO START |
| Phase 2 | 3 weeks | Core features (pipes, loops, functions) | PENDING |
| Phase 3 | 4 weeks | Advanced features (subshells, traps) | PENDING |
| Phase 4 | 2 weeks | Interactive shell (REPL) | PENDING |
| **Total** | **12 weeks** | Full bash runtime in WASM | - |

---

## Dependencies

### New Crates Needed

```toml
[dependencies]
# For readline-like functionality
rustyline = { version = "14.0", optional = true, features = ["wasm"] }

# For glob pattern matching
globset = "0.4"

# For regex in pattern matching
regex = "1.10"

# For arithmetic evaluation
evalexpr = "12.0"
```

### Existing Crates (Already Available)

- `wasm-bindgen` ✅
- `js-sys` ✅
- `web-sys` ✅
- `serde` / `serde_json` ✅

---

## Success Metrics

### Technical Metrics
- ✅ 85%+ test coverage
- ✅ 90%+ mutation score
- ✅ <5s WASM load time
- ✅ <100ms script execution (100 lines)
- ✅ Zero security vulnerabilities

### User Metrics
- ✅ 95%+ bash compatibility (common patterns)
- ✅ <10ms perceived latency (interactive)
- ✅ Works in Chromium, Firefox, Safari
- ✅ Mobile-friendly (touch input)

---

## Next Steps

**Immediate (Week 1)**:
1. Create `src/wasm/executor.rs` skeleton
2. Create `src/wasm/vfs.rs` basic implementation
3. Create `src/wasm/builtins.rs` with echo, cd, pwd
4. Write RED tests for Phase 1 features
5. Begin GREEN implementation

**Commands**:
```bash
# Create new files
touch src/wasm/executor.rs
touch src/wasm/vfs.rs
touch src/wasm/builtins.rs
touch src/wasm/io.rs

# Update lib.rs
echo "pub mod executor;" >> src/wasm/mod.rs
echo "pub mod vfs;" >> src/wasm/mod.rs
echo "pub mod builtins;" >> src/wasm/mod.rs
echo "pub mod io;" >> src/wasm/mod.rs
```

---

## Conclusion

Building a bash runtime in WASM transforms bashrs from a **static analysis tool** into a **complete educational shell environment**. This enables:

1. **WOS Terminal**: Real bash execution in browser OS
2. **interactive.paiml.com**: Live shell tutorials with instant feedback
3. **bashrs Playground**: Test and learn bash safely

**Estimated Effort**: 12 weeks (3 months)
**Risk**: MEDIUM (complex but well-understood domain)
**Value**: **VERY HIGH** (enables entirely new use cases)

**Recommendation**: ✅ **PROCEED** after Phase 0 completion

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
Co-Authored-By: Claude <noreply@anthropic.com>
