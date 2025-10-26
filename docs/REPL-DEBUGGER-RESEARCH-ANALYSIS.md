# Shell REPL and Debugger Implementation Research Analysis

**Document Status**: Comprehensive Research Report for bashrs REPL/Debugger Specification
**Date**: 2025-10-26
**Version**: 1.0
**Research Scope**: 2020-2025 Academic Sources, Production Implementations, Design Patterns

---

## Executive Summary

This research analysis examines shell REPL and debugger implementations to inform the design of a comprehensive REPL and debugging system for bashrs (the Rash shell safety tool). The analysis covers:

1. **Ruchy Project Architectures** - Production-ready REPL and debugging implementations
2. **Academic Research** (2020-2025) - Shell script verification, deterministic execution, program analysis
3. **Production Tools** - bashdb, zsh debugging, PowerShell debugger analysis
4. **Recommended Architecture** - Integrated design for bashrs

**Key Findings**:
- Modern debuggers should integrate REPL functionality (debugger-as-REPL paradigm)
- Shell script debugging requires special consideration for determinism and idempotency
- NASA-level quality frameworks provide proven patterns for safety-critical debugging tools
- Real-world validation through dogfooding accelerates quality and adoption

---

## 1. Ruchy Project Analysis

### 1.1 REPL Architecture (Primary Source: /home/noah/src/ruchy)

**File**: `/home/noah/src/ruchy/src/runtime/repl/mod.rs`

**Core Architecture**:

```rust
pub struct Repl {
    commands: CommandRegistry,      // Command handling (:help, :quit, etc.)
    state: ReplState,               // Session state management
    evaluator: Evaluator,           // Expression evaluation engine
    completion: CompletionEngine,   // Tab completion
    work_dir: PathBuf,              // Working directory context
}
```

**Key Design Patterns Identified**:

1. **Modular Architecture** (Lines 11-16):
   - Separate modules: commands, completion, evaluation, formatting, state
   - Clean separation of concerns enables independent testing
   - Each module <10 cyclomatic complexity (extreme TDD discipline)

2. **Configuration-Driven Design** (Lines 36-58):
   ```rust
   pub struct ReplConfig {
       max_memory: usize,           // Memory limits for safety
       timeout: Duration,           // Execution timeout
       maxdepth: usize,             // Recursion depth protection
       debug: bool,                 // Debug mode toggle
   }
   ```
   - Safety-first design with resource limits
   - Sandboxed mode for untrusted input (lines 106-115)
   - Configurable constraints prevent runaway execution

3. **Mode-Based Interaction** (Lines 185-190):
   - Normal mode: Standard evaluation
   - Debug mode: Enhanced output with tracing
   - AST mode: Show parsed abstract syntax tree
   - Transpile mode: Show generated code
   - **Lesson**: Multi-modal UX provides power without complexity

4. **History and State Management** (Lines 127-155):
   - Persistent command history
   - State preservation across sessions
   - Graceful error handling (Ctrl-C, EOF, errors)

**Strengths**:
- ✅ Extreme quality focus (all functions <10 complexity)
- ✅ Comprehensive testing (90%+ coverage)
- ✅ Resource safety (memory, time, recursion limits)
- ✅ Clean modular design

**Applicability to bashrs**:
- ✅ Configuration pattern: Directly applicable for bash safety constraints
- ✅ Sandboxed mode: Critical for testing purified bash safely
- ✅ Multi-mode design: bashrs could support: Normal, Purify, Lint, Explain modes
- ✅ Resource limits: Essential for preventing $RANDOM, timestamp, infinite loops

---

### 1.2 Debugger Architecture (Primary Source: /home/noah/src/ruchy)

**File**: `/home/noah/src/ruchy/src/debugger/mod.rs`

**Core Architecture**:

```rust
pub struct Debugger {
    breakpoints: Vec<Breakpoint>,
    is_running: bool,
    is_paused: bool,
    current_line: usize,
    current_function: String,
    call_stack: Vec<StackFrame>,
    watches: Vec<Watch>,
    events: Vec<DebugEvent>,
    local_variables: HashMap<String, String>,
    output: String,
    watch_notifications_enabled: bool,
    watch_changes: HashMap<usize, Vec<WatchChange>>,
}
```

**Key Design Patterns Identified**:

1. **Breakpoint System** (Lines 26-33):
   ```rust
   pub struct Breakpoint {
       file: String,
       line: usize,
       condition: Option<String>,       // Conditional breakpoints
       hit_count_target: Option<usize>, // Hit count breakpoints
       current_hit_count: usize,
   }
   ```
   - **Advanced**: Conditional and hit-count breakpoints
   - **Lesson**: Rich breakpoint types enable sophisticated debugging

2. **Call Stack Tracking** (Lines 35-40):
   ```rust
   pub struct StackFrame {
       function_name: String,
       line: usize,
       file: String,
   }
   ```
   - Full stack traces for context
   - Essential for understanding bash function calls

3. **Watch Expressions** (Lines 51-60):
   ```rust
   struct Watch {
       expression: String,
       value: Option<String>,
   }

   pub struct WatchChange {
       old_value: String,
       new_value: String,
   }
   ```
   - Track variable changes across execution
   - Change notification system for reactivity

4. **Debug Events** (Lines 42-48):
   ```rust
   pub enum DebugEvent {
       BreakpointHit(usize),
       StepComplete,
       ProgramTerminated,
       ExceptionThrown(String),
   }
   ```
   - Event-driven architecture
   - Decouples debugger from execution engine

5. **Source Context** (Lines 360-367):
   - `get_source_context()` shows surrounding lines
   - Improves developer orientation
   - Critical for shell scripts (context is everything)

6. **Line/Offset Conversion** (Lines 327-358):
   - Bidirectional mapping between line numbers and byte offsets
   - Essential for accurate breakpoint placement
   - **Lesson**: Shell scripts need precise source mapping

**Strengths**:
- ✅ Rich feature set (conditional breakpoints, watches, stack traces)
- ✅ Non-invasive design (doesn't alter program semantics)
- ✅ Event-driven architecture (loosely coupled)
- ✅ Comprehensive state tracking

**Applicability to bashrs**:
- ✅ Breakpoint patterns: Directly applicable to bash scripts
- ✅ Watch system: Track bash variables across purification
- ✅ Stack traces: Essential for debugging bash function calls
- ✅ Source context: Critical for understanding bash scripts

**Gaps** (opportunities for bashrs):
- ⚠️ No time-travel debugging (backward stepping)
- ⚠️ No program slicing (causality analysis)
- ⚠️ No integration with purification engine
- ⚠️ No determinism/idempotency tracking

---

### 1.3 RuchyRuchy Debugging Tools Specification

**File**: `/home/noah/src/ruchyruchy/docs/specifications/ruchyruchy-debugging-tools-spec.md`

**World-Class Design Principles**:

This 2169-line specification represents **NASA-level engineering standards** applied to debugging tools. Key insights:

1. **Symbiotic Compiler-Debugger Architecture** (Lines 285-323):
   ```
   ┌─────────────────────────────────────────────────────┐
   │                  RuchyDbg Debugger                  │
   │  ┌───────────────────────────────────────────────┐ │
   │  │         Embedded Ruchy Compiler               │ │
   │  │  ┌─────────┬──────────┬──────────┬─────────┐ │ │
   │  │  │ Stage 0 │ Stage 1  │ Stage 2  │ Stage 3 │ │ │
   │  │  │ Lexer   │ Parser   │ TypeChk  │ CodeGen │ │ │
   │  └───────────────────────────────────────────────┘ │
   │         ↓                                            │
   │  ┌───────────────────────────────────────────────┐ │
   │  │        Debugging Intelligence Layer           │ │
   │  │  • Time-Travel Engine                         │ │
   │  │  • Program Slicer                             │ │
   │  │  • Causality Analyzer                         │ │
   │  └───────────────────────────────────────────────┘ │
   └─────────────────────────────────────────────────────┘
   ```

   **Key Insight**: Embed the ENTIRE compiler into the debugger for maximum semantic awareness.

   **Applicability to bashrs**:
   - ✅ Embed bash parser and purifier into debugger
   - ✅ Debugger has full AST access
   - ✅ Can show purified vs original bash side-by-side
   - ✅ Can trace non-deterministic patterns ($RANDOM, $$, timestamps)

2. **Time-Travel Debugging** (Lines 505-556):
   - **Record-Replay Architecture**: Log all state-mutating operations
   - **Backward Stepping**: Replay from checkpoints
   - **Formal Verification**: Mathematical correctness proofs

   **Relevance to bashrs**:
   - Track bash variable mutations
   - Show history: "x was 5, then became 10 at line 42"
   - Replay purified bash to verify idempotency
   - **Critical**: Prove purified bash produces same result on replay

3. **Program Slicing** (Lines 560-590):
   - **Backward Slice**: What code influenced this variable?
   - **Forward Slice**: What does this variable influence?
   - **Dependency Analysis**: Data-flow and control-flow tracking

   **Relevance to bashrs**:
   - "Why did this mkdir fail?" → Show preceding commands
   - "What files will this rm affect?" → Show forward dependencies
   - Isolate non-idempotent operations

4. **WhyLine Queries** (Lines 593-622):
   - Natural language queries: "Why is x > 10?"
   - Causality explanations: Show statement that caused condition

   **Relevance to bashrs**:
   - "Why did this script fail?" → Show exact problematic line
   - "Why is this non-deterministic?" → Show $RANDOM usage
   - "Why is this non-idempotent?" → Show mkdir without -p

5. **Tiered Quality Gates** (Lines 223-280):
   - **Tier 1 (Pre-Commit)**: <1 second, catches trivial errors
   - **Tier 2 (CI Pipeline)**: 5-10 minutes, integration tests
   - **Tier 3 (Nightly)**: Hours, exhaustive verification

   **Lesson**: Balance speed vs thoroughness, prevent developer overburden

   **Applicability to bashrs**:
   ```
   Tier 1: bashrs lint --fast (syntax, obvious errors)
   Tier 2: bashrs lint + bashrs test (purification correctness)
   Tier 3: bashrs prove (property tests, mutation tests, formal verification)
   ```

6. **Anti-Fraud Validation** (Lines 696-926):
   - **Systematic Validation**: Every tool must pass smoke, error, integration tests
   - **Differential Testing**: Compare against GDB, Chrome DevTools
   - **Consensus Validation**: Multiple tools must agree on same bug

   **Critical Insight**: A broken debugging tool that gives wrong information is WORSE than no tool at all.

   **Applicability to bashrs**:
   ```bash
   # Tier 2 Integration Test (MANDATORY)
   test_all_bashrs_tools_on_single_script() {
       script="deploy.sh"

       # Step 1: Parse script
       ast=$(bashrs parse $script)

       # Step 2: Lint script
       lint_results=$(bashrs lint $script)

       # Step 3: Purify script
       purified=$(bashrs purify $script)

       # Step 4: Debug purified script
       debug_results=$(bashrs debug $purified)

       # CRITICAL: All tools must agree on line numbers!
       # If lint says error at line 10, debug must stop at line 10
       # If purify changes line 10, debug must map correctly
       verify_tool_consensus($ast, $lint_results, $purified, $debug_results)
   }
   ```

7. **DevEx Validation** (Lines 1127-1266):
   - **Cognitive Walkthroughs**: Can user complete task without documentation?
   - **Usability Testing**: 5 developers, realistic tasks, measure time/errors
   - **Comparative Studies**: Benchmark against existing tools

   **Example Metrics**:
   ```
   Task: Find bug in 200-line bash script
   GDB (Manual):        18.3 min, 60% success
   RuchyDbg (Slicing):   4.1 min, 100% success
   → 4.5x improvement!
   ```

   **Applicability to bashrs**:
   - Test bashrs debugger against manual debugging
   - Measure: Time to find non-deterministic code
   - Measure: Time to identify non-idempotent operations
   - Target: 3-5x improvement over manual inspection

8. **Fast-Feedback Ruchy Integration** (Lines 1610-1858):
   - **Dogfooding**: Test debugger on Ruchy compiler itself
   - **Pre-Commit Hooks**: <6 second validation on every commit
   - **Real-World Validation**: 50K+ LOC, 390K+ tests

   **Milestone Example** (Week 4):
   ```bash
   $ ruchy debug source-map src/compiler/parser.ruchy
   ✅ Generated source map: 847 lines, 847 mappings
   ✅ Breakpoint accuracy: 100% (±1 line)
   ```

   **Applicability to bashrs**:
   ```bash
   # Dogfood on bashrs itself!
   $ bashrs debug rash/src/parser.rs
   ✅ Parsed: 1247 lines
   ✅ Purified: 0 non-deterministic patterns
   ✅ Verified: Idempotent operations

   # Pre-commit hook
   $ git commit -m "Fix parser bug"
   🔍 Validating bashrs purifier...
      ✅ All test scripts purify correctly
      ✅ Shellcheck passes on purified output
   ✅ Commit allowed
   ```

---

### 1.4 Key Lessons from Ruchy Projects

**Architecture Patterns**:
1. ✅ **Modular Design**: Separate concerns (REPL, debugger, evaluator, parser)
2. ✅ **Configuration-Driven**: Resource limits, safety constraints, modes
3. ✅ **Event-Driven**: Decouple debugging from execution
4. ✅ **Symbiotic Integration**: Embed compiler/parser in debugger

**Quality Patterns**:
1. ✅ **Extreme TDD**: All functions <10 complexity
2. ✅ **Tiered Gates**: Fast pre-commit, thorough nightly
3. ✅ **Anti-Fraud Validation**: Tools must agree, differential testing
4. ✅ **DevEx Focus**: Cognitive walkthroughs, usability testing

**Safety Patterns**:
1. ✅ **Resource Limits**: Memory, time, recursion depth
2. ✅ **Sandboxed Execution**: Untrusted code isolation
3. ✅ **Formal Verification**: Mathematical correctness proofs
4. ✅ **NASA-Level Standards**: Zero defects, redundancy, fault tolerance

**Implementation Patterns**:
1. ✅ **Dogfooding**: Test on project itself (Ruchy debugs Ruchy)
2. ✅ **Fast Feedback**: <6 second pre-commit validation
3. ✅ **Real-World Validation**: 50K+ LOC, 390K+ tests
4. ✅ **Vertical Slices**: Deliver thin end-to-end features early

---

## 2. Academic Research (2020-2025)

### 2.1 Shell Script Verification and Safety

**Paper 1**: "Practically Correct, Just-in-Time Shell Script Parallelization"
**Authors**: Kallas et al.
**Venue**: USENIX OSDI 2022
**URL**: https://www.usenix.org/system/files/osdi22-kallas.pdf

**Key Contributions**:
1. **PASH-JIT**: Just-in-time parallelization of POSIX shell scripts
2. **Safety Guarantees**: Preserves script semantics while parallelizing
3. **Testing**: 1007 assertions, 494 test cases, 29K LOC of shell scripts

**Relevance to bashrs**:
- Proves safety of shell script transformations (like purification)
- Shows importance of comprehensive test suites for shell tools
- Demonstrates POSIX compliance verification techniques

**Citation**: Kallas, K., Mustafa, O., Benetopoulos, A., & Greenberg, M. (2022). Practically Correct, Just-in-Time Shell Script Parallelization. In *16th USENIX Symposium on Operating Systems Design and Implementation (OSDI 22)* (pp. 1-18).

---

**Paper 2**: "From Ahead-of-Time to Just-in-Time and Back Again: Static Analysis for Unix Shell Programs"
**Authors**: Lazarek et al.
**Venue**: ACM HOTOS 2025
**URL**: https://regmedia.co.uk/2025/04/29/sash-hotos25-paper.pdf

**Key Contributions**:
1. **Static Analysis for Shell**: Techniques for analyzing shell scripts without execution
2. **ShellCheck Integration**: Leverages existing static analysis tools
3. **Timing**: Ahead-of-time vs just-in-time analysis trade-offs

**Relevance to bashrs**:
- Static analysis can detect non-determinism without execution
- Integration with ShellCheck validates purified output
- Shows importance of multi-phase analysis (parse → analyze → verify)

**Citation**: Lazarek, J., Chong, S., & Greenberg, M. (2025). From Ahead-of-Time to Just-in-Time and Back Again: Static Analysis for Unix Shell Programs. In *Proceedings of the Workshop on Hot Topics in Operating Systems (HOTOS 25)* (pp. 1-6).

---

### 2.2 Debugger-REPL Integration

**Source**: "A Debugger is a REPL is a Debugger"
**Author**: Alexey Kladov (matklad)
**Date**: March 25, 2025
**URL**: https://matklad.github.io/2025/03/25/debugger-is-repl-is-debugger.html

**Key Insights**:
1. **Unified Interface**: Debuggers and REPLs should be the same tool
2. **Bidirectional Flow**:
   - REPL needs breakpoints (to access program context)
   - Debugger needs REPL (to evaluate expressions at breakpoints)
3. **Modern UX**: 2D program text editing, not 1D command line
4. **Run to Cursor**: Place yourself in middle of execution

**Relevance to bashrs**:
- bashrs REPL should have debugging capabilities built-in
- Set breakpoints in bash scripts, evaluate purified bash at breakpoints
- Interactive exploration of non-deterministic code

**Quote**: "The medium has progressed from 1D to 2D: from command line to program text. This is a vi interface, not ed interface."

---

### 2.3 Program Comprehension and Debugging

**Paper 3**: "Debugging by starting a REPL at a breakpoint is fun"
**Author**: Julia Evans (jvns)
**Date**: September 16, 2021
**URL**: https://jvns.ca/blog/2021/09/16/debugging-in-a-repl-is-fun/

**Key Insights**:
1. **Context Initialization**: REPL at breakpoint has full program context
2. **Interactive Experimentation**: Try solutions without recompiling
3. **Python ipdb**: Example of REPL-based debugging in practice

**Relevance to bashrs**:
- bashrs debugger should drop into REPL at purification points
- Allow interactive testing: "What if I make this idempotent?"
- Example workflow:
  ```bash
  # Script paused at: mkdir /tmp/mydir
  bashrs> test "mkdir -p /tmp/mydir"  # Try purified version
  bashrs> diff original purified      # See differences
  bashrs> continue                    # Resume with purified version
  ```

---

### 2.4 Deterministic Execution Research

**Search Query Results**: "deterministic shell execution idempotency IEEE 2021-2025"

**Finding**: Limited direct research on shell idempotency, but extensive work on:
- Deterministic concurrency control (databases)
- Deterministic execution in distributed systems
- Application-level determinism

**Key IEEE Papers**:

1. **Discrete-Event-Based Deterministic Execution** (IEEE 2017):
   - Proposes deterministic execution environment for industrial systems
   - Timestamp-based mechanism for reproducibility

   **Relevance to bashrs**: Timestamps are enemy of determinism in bash scripts

2. **Application-Level Determinism in Distributed Systems** (IEEE 2016):
   - Deterministic execution eases development and debugging
   - Performance costs of enforcing determinism

   **Relevance to bashrs**: Purification trades performance for determinism (acceptable trade-off)

**Gap in Literature**: No specific research on shell script idempotency guarantees in recent IEEE publications (opportunity for bashrs contribution!)

---

## 3. Production Tool Analysis

### 3.1 bashdb (Bash Debugger)

**Version**: 5.2-1.2.0 (2024)
**URL**: https://bashdb.sourceforge.net/
**Architecture**: GDB-like debugger for bash

**Implementation Details**:

1. **Integration**: Runs inside bash shell itself
   ```bash
   bash --debugger script.sh
   # OR
   bashdb script.sh
   ```

2. **Performance Optimization**:
   - **Zero Overhead Until Invoked**: No performance penalty until `_Dbg_debugger` called
   - **Advantage**: Large config scripts have zero overhead
   - **Lesson for bashrs**: Don't instrument unless debugging requested

3. **Entry Points**:
   ```bash
   # Method 1: Command-line
   bash --debugger script.sh

   # Method 2: Source and call
   source bashdb/dbg-trace.sh
   _Dbg_debugger
   ```

4. **Configuration**: `.bashdbrc` files
   - Per-user: `~/.bashdbrc`
   - Per-project: `./.bashdbrc`
   - Supports: Breakpoint setup, watch expressions, aliases

**Strengths**:
- ✅ GDB-compatible commands (familiar to developers)
- ✅ Zero overhead when not debugging
- ✅ Configurable per-user/per-project

**Weaknesses**:
- ❌ No time-travel (backward stepping)
- ❌ No integration with purification/linting
- ❌ No determinism tracking
- ❌ No idempotency analysis

**Applicability to bashrs**:
- ✅ GDB-style commands: Familiar interface
- ✅ Configuration system: Good UX pattern
- ⚠️ Zero overhead design: May conflict with purification (need instrumentation)
- ✅ Source-based integration: bashrs could inject debug hooks during purification

---

### 3.2 Shell Debugging Techniques (POSIX)

**Sources**: Stack Overflow, Linux.com, Shell-Tips

**Common Techniques**:

1. **Execution Tracing** (`set -x`):
   ```bash
   set -x    # Turn on tracing
   # ... commands ...
   set +x    # Turn off tracing
   ```
   - Pros: Simple, built-in, no external tools
   - Cons: Verbose, no control flow analysis

2. **Extended Debug Mode** (`set -o extdebug`):
   ```bash
   shopt -s extdebug
   ```
   - Enables: Interactive debugger, errtrace, functrace
   - Pros: More powerful than basic tracing
   - Cons: Bash-specific, not POSIX

3. **Manual Breakpoints** (spawning subshell):
   ```bash
   # At desired breakpoint:
   bash --rcfile <(echo "PS1='DEBUG> '")
   # Inspect variables, test commands
   # Exit to continue script
   ```
   - Pros: REPL-like experience, full context
   - Cons: Manual, no automation

**Relevance to bashrs**:
- bashrs purifier could inject `set -x` blocks automatically
- bashrs debugger could use `extdebug` for advanced features
- bashrs REPL could improve upon manual breakpoint technique

---

### 3.3 PowerShell Debugger (Comparison Point)

**Not directly researched but worth noting**:
- PowerShell has integrated debugging (Set-PSBreakpoint)
- Supports: Conditional breakpoints, watch variables, call stack
- Lesson: Modern shells should have debugging built-in, not bolted-on

---

## 4. Recommended Architecture for bashrs

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│              bashrs REPL + Debugger                     │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │     Embedded Bash Parser + Purifier               │ │
│  │  ┌──────────┬────────────┬──────────────────┐    │ │
│  │  │  Parser  │  Analyzer  │  Purifier/Emit   │    │ │
│  │  │  (AST)   │  (Lint)    │  (POSIX sh)      │    │ │
│  │  └──────────┴────────────┴──────────────────┘    │ │
│  │         ↓                                          │ │
│  │    AST, Lint Results, Purified Output             │ │
│  └───────────────────────────────────────────────────┘ │
│                        ↓                                │
│  ┌───────────────────────────────────────────────────┐ │
│  │      Debugging Intelligence Layer                 │ │
│  │  • Execution Tracer (set -x integration)          │ │
│  │  • Determinism Checker ($RANDOM, $$, timestamps)  │ │
│  │  • Idempotency Analyzer (mkdir→mkdir -p)          │ │
│  │  • Purification Explainer (why changes made)      │ │
│  └───────────────────────────────────────────────────┘ │
│                        ↓                                │
│  ┌───────────────────────────────────────────────────┐ │
│  │           REPL + Interactive Debugger             │ │
│  │  • Evaluate bash expressions                      │ │
│  │  • Set breakpoints in bash scripts                │ │
│  │  • Step through execution                         │ │
│  │  • Compare original vs purified                   │ │
│  │  • Explain purification decisions                 │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Core Components

#### Component 1: REPL Interface

**Inspired by**: Ruchy REPL (mod.rs), matklad's debugger-REPL unification

**Features**:
```rust
pub struct BashrsRepl {
    config: ReplConfig,
    state: ReplState,
    parser: BashParser,
    purifier: BashPurifier,
    debugger: BashDebugger,
    history: CommandHistory,
}
```

**Modes**:
1. **Normal**: Evaluate bash commands (in sandboxed sh)
2. **Purify**: Show purified version of input
3. **Lint**: Show linting results
4. **Debug**: Step through bash scripts
5. **Explain**: Explain why purification made changes

**Example Session**:
```bash
$ bashrs repl
bashrs> mkdir /tmp/test
Executed: mkdir /tmp/test

bashrs> :mode purify
Mode: Purify

bashrs> mkdir /tmp/test
Original:  mkdir /tmp/test
Purified:  mkdir -p /tmp/test
Reason:    Added -p flag for idempotency

bashrs> :lint deploy.sh
deploy.sh:10: SEC001: Unquoted variable $file
deploy.sh:15: DET001: Non-deterministic: $RANDOM
deploy.sh:20: IDEM001: Non-idempotent: mkdir (should be mkdir -p)

bashrs> :debug deploy.sh
Breakpoint 1 at deploy.sh:15
(bashrs-dbg) continue
Hit breakpoint at line 15: SESSION_ID=$RANDOM
(bashrs-dbg) print $RANDOM
5847
(bashrs-dbg) explain
Non-deterministic: $RANDOM produces different value each run
Purification: Replace with input parameter or fixed seed
```

---

#### Component 2: Debugger Engine

**Inspired by**: Ruchy debugger (mod.rs), bashdb, RuchyRuchy spec

**Features**:
1. **Breakpoints**: Line-based, conditional, watch-based
2. **Stepping**: Step, next, continue, finish
3. **Inspection**: Variables, call stack, environment
4. **Comparison**: Original vs purified bash side-by-side
5. **Tracing**: Record execution for replay

**Implementation Strategy**:
```rust
pub struct BashDebugger {
    breakpoints: Vec<Breakpoint>,
    execution_state: ExecutionState,
    trace_log: Vec<TraceEvent>,
    purified_comparison: Option<PurifiedScript>,
}

impl BashDebugger {
    /// Set breakpoint at line (supports conditional)
    pub fn set_breakpoint(&mut self, line: usize, condition: Option<String>);

    /// Step to next line
    pub fn step(&mut self) -> Result<ExecutionState>;

    /// Continue until breakpoint or completion
    pub fn continue_execution(&mut self) -> Result<ExecutionState>;

    /// Show variables at current line
    pub fn inspect_variables(&self) -> HashMap<String, String>;

    /// Compare original vs purified at current line
    pub fn show_purified_diff(&self) -> Diff;

    /// Explain why purification made changes at this line
    pub fn explain_purification(&self) -> String;
}
```

**Integration with Purification**:
- When debugging, show purified version side-by-side
- Highlight differences (original vs purified)
- Explain each transformation:
  ```
  Original:  mkdir /app/logs
  Purified:  mkdir -p /app/logs
  Reason:    Added -p flag because mkdir fails if /app exists
             This makes operation idempotent (safe to re-run)
  ```

---

#### Component 3: Determinism Tracker

**Inspired by**: RuchyRuchy time-travel debugging, IEEE deterministic execution research

**Features**:
1. **Pattern Detection**: Identify $RANDOM, $$, timestamps
2. **Replay Verification**: Execute twice, compare outputs
3. **Diff Explanation**: Show what changed between runs

**Implementation**:
```rust
pub struct DeterminismChecker {
    patterns: Vec<NonDeterministicPattern>,
    execution_log: Vec<ExecutionTrace>,
}

impl DeterminismChecker {
    /// Scan bash script for non-deterministic patterns
    pub fn scan(&self, script: &str) -> Vec<DeterminismViolation>;

    /// Execute script twice and compare outputs
    pub fn verify_determinism(&self, script: &str) -> DeterminismReport;

    /// Show what changed between executions
    pub fn explain_differences(&self, run1: &Trace, run2: &Trace) -> Diff;
}

pub struct DeterminismViolation {
    line: usize,
    pattern: NonDeterministicPattern,
    suggestion: String,
}

pub enum NonDeterministicPattern {
    Random,           // $RANDOM
    ProcessId,        // $$
    Timestamp,        // date +%s, $(date), etc.
    Hostname,         // $(hostname), $HOSTNAME (if varies)
    RandomFile,       // mktemp without fixed seed
}
```

**Example Output**:
```bash
$ bashrs check-determinism deploy.sh

Non-Deterministic Patterns Found:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Line 15: SESSION_ID=$RANDOM
  Pattern: $RANDOM (generates random number)
  Issue:   Different value each run → non-deterministic
  Fix:     Pass SESSION_ID as parameter or use fixed seed

Line 20: TIMESTAMP=$(date +%s)
  Pattern: $(date +%s) (Unix timestamp)
  Issue:   Different value each run → non-deterministic
  Fix:     Pass TIMESTAMP as parameter or use fixed timestamp

Line 25: TMPFILE=$(mktemp)
  Pattern: mktemp (random temp filename)
  Issue:   Different filename each run → non-deterministic
  Fix:     Use mktemp -p /tmp --suffix=.deploy for predictable names

Verification Test (2 runs):
  Run 1: SESSION_ID=5847, TIMESTAMP=1730000000, TMPFILE=/tmp/tmp.Xy7pQ3
  Run 2: SESSION_ID=2491, TIMESTAMP=1730000005, TMPFILE=/tmp/tmp.A1bC2d
  Result: ❌ FAILED - Outputs differ

Purified Script Verification (2 runs):
  Run 1: SESSION_ID=input1, TIMESTAMP=input1, TMPFILE=/tmp/deploy.tmp
  Run 2: SESSION_ID=input1, TIMESTAMP=input1, TMPFILE=/tmp/deploy.tmp
  Result: ✅ PASSED - Outputs identical
```

---

#### Component 4: Idempotency Analyzer

**Inspired by**: bashrs core mission, infrastructure-as-code tools (Ansible, Terraform)

**Features**:
1. **Operation Analysis**: Identify non-idempotent operations
2. **Suggested Fixes**: Show how to make operations idempotent
3. **Verification**: Execute twice, verify same result

**Implementation**:
```rust
pub struct IdempotencyAnalyzer {
    patterns: Vec<NonIdempotentPattern>,
}

impl IdempotencyAnalyzer {
    /// Scan for non-idempotent operations
    pub fn analyze(&self, script: &str) -> Vec<IdempotencyViolation>;

    /// Verify script is idempotent by running twice
    pub fn verify(&self, script: &str) -> IdempotencyReport;
}

pub struct IdempotencyViolation {
    line: usize,
    operation: String,
    issue: String,
    fix: String,
}

pub enum NonIdempotentPattern {
    MkdirWithoutP,     // mkdir dir (fails if exists)
    RmWithoutF,        // rm file (fails if not exists)
    LnWithoutF,        // ln -s (fails if exists)
    MvWithoutF,        // mv (fails if target exists)
    TeeAppend,         // >> (accumulates on re-run)
}
```

**Example Output**:
```bash
$ bashrs check-idempotency deploy.sh

Non-Idempotent Operations Found:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Line 10: mkdir /app/logs
  Issue:   Fails if /app/logs already exists (2nd run fails)
  Fix:     mkdir -p /app/logs
  Impact:  Script cannot be re-run safely

Line 15: ln -s /app/current /app/latest
  Issue:   Fails if /app/latest already exists
  Fix:     ln -sf /app/current /app/latest (force overwrite)
  Impact:  Deployment fails on re-run

Line 20: echo "deployed" >> /var/log/deploy.log
  Issue:   Appends on each run (log grows indefinitely)
  Fix:     Use > instead of >>, or log with timestamp and rotate
  Impact:  Log file grows without bounds

Idempotency Test (2 runs):
  Run 1: ✅ SUCCESS (created /app/logs, created symlink, wrote log)
  Run 2: ❌ FAILED (mkdir: cannot create directory '/app/logs': File exists)
  Result: ❌ NON-IDEMPOTENT

Purified Script Idempotency Test (3 runs):
  Run 1: ✅ SUCCESS
  Run 2: ✅ SUCCESS (all operations succeed, same result)
  Run 3: ✅ SUCCESS (verified: truly idempotent)
  Result: ✅ IDEMPOTENT
```

---

#### Component 5: Purification Explainer

**Inspired by**: RuchyRuchy WhyLine queries, Ko & Myers (2004) "Why did..." debugging

**Features**:
1. **Transformation Explanation**: Why each change was made
2. **Safety Rationale**: How change improves safety
3. **Alternative Suggestions**: Other ways to achieve safety

**Implementation**:
```rust
pub struct PurificationExplainer {
    transformations: Vec<Transformation>,
}

pub struct Transformation {
    line: usize,
    original: String,
    purified: String,
    reason: String,
    safety_benefit: String,
    alternatives: Vec<String>,
}

impl PurificationExplainer {
    /// Explain all transformations in purified script
    pub fn explain_all(&self) -> Vec<Transformation>;

    /// Explain specific transformation at line
    pub fn explain_line(&self, line: usize) -> Option<Transformation>;

    /// Why was this pattern changed?
    pub fn why_changed(&self, pattern: &str) -> String;
}
```

**Example Output**:
```bash
$ bashrs explain deploy.sh

Purification Changes Explained:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Line 10: mkdir /app/logs → mkdir -p /app/logs

  Why Changed?
    Original command fails if /app/logs already exists.
    This makes script non-idempotent (cannot re-run safely).

  Safety Benefit:
    With -p flag, mkdir succeeds even if directory exists.
    Script can now be re-run without errors.
    Matches infrastructure-as-code best practices.

  Alternative Fixes:
    1. Add check: [ -d /app/logs ] || mkdir /app/logs
    2. Use install: install -d /app/logs
    3. Keep mkdir, handle error: mkdir /app/logs 2>/dev/null || true

  Recommended:
    mkdir -p is idiomatic, concise, and POSIX-compliant.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Line 15: SESSION_ID=$RANDOM → SESSION_ID="${1:-default}"

  Why Changed?
    $RANDOM produces different value each run.
    This makes script non-deterministic (unpredictable behavior).

  Safety Benefit:
    Using parameter ${1:-default} makes SESSION_ID controllable.
    Same input always produces same output.
    Enables testing and reproducible deployments.

  Alternative Fixes:
    1. Use fixed seed: SESSION_ID=12345
    2. Use date-based: SESSION_ID=$(date +%Y%m%d)
    3. Environment var: SESSION_ID="${SESSION_ID:-default}"

  Recommended:
    Parameter passing gives caller control.
    Can be overridden per-deployment without editing script.
```

---

### 4.3 Integration with bashrs Workflow

**End-to-End Experience**:

```bash
# Step 1: Lint existing script
$ bashrs lint deploy.sh
deploy.sh:10: IDEM001: mkdir without -p (non-idempotent)
deploy.sh:15: DET001: $RANDOM (non-deterministic)
deploy.sh:20: SEC001: Unquoted variable $file

# Step 2: Purify script
$ bashrs purify deploy.sh -o deploy-purified.sh
Purified: deploy.sh → deploy-purified.sh
Changes: 3 transformations applied
  - Line 10: Added -p to mkdir
  - Line 15: Replaced $RANDOM with parameter
  - Line 20: Quoted variable $file

# Step 3: Explain changes
$ bashrs explain deploy-purified.sh
[Shows detailed explanations like above]

# Step 4: Verify with debugger
$ bashrs debug deploy-purified.sh
(bashrs-dbg) break 10
Breakpoint 1 at line 10: mkdir -p /app/logs
(bashrs-dbg) run
Hit breakpoint at line 10
(bashrs-dbg) compare-original
Original:  mkdir /app/logs
Purified:  mkdir -p /app/logs
(bashrs-dbg) why-changed
Added -p flag for idempotency (see 'bashrs explain' for details)
(bashrs-dbg) continue
Script completed successfully

# Step 5: Verify idempotency
$ bashrs verify-idempotent deploy-purified.sh --runs=3
Run 1: ✅ SUCCESS
Run 2: ✅ SUCCESS (same result)
Run 3: ✅ SUCCESS (verified idempotent)
Result: ✅ IDEMPOTENT (3/3 runs identical)

# Step 6: Interactive REPL for testing
$ bashrs repl --script deploy-purified.sh
bashrs> :load deploy-purified.sh
Loaded: deploy-purified.sh (45 lines)
bashrs> :step 10
Stopped at line 10: mkdir -p /app/logs
bashrs> test-idempotency
Testing: mkdir -p /app/logs
  Run 1: Created /app/logs
  Run 2: /app/logs exists, no error (✅ idempotent)
bashrs> :continue
Script completed
bashrs> :quit
```

---

### 4.4 Safety Integration Points

**bashrs Unique Safety Features**:

1. **Purification-Aware Debugging**:
   - Show original vs purified side-by-side
   - Explain why each transformation happened
   - Verify purified version is actually safer

2. **Determinism Verification**:
   - Run script twice, compare outputs
   - Highlight any differences
   - Suggest fixes for non-determinism

3. **Idempotency Testing**:
   - Run script N times, verify same result
   - Check filesystem state after each run
   - Report any cumulative effects (appending logs, etc.)

4. **Sandbox Execution**:
   - REPL runs in isolated namespace (à la Ruchy sandboxed mode)
   - Filesystem changes in temporary directory
   - Network calls blocked or logged

5. **ShellCheck Integration**:
   - Automatically run shellcheck on purified output
   - Report any warnings/errors
   - Ensure POSIX compliance

---

### 4.5 Performance Considerations

**Inspired by**: bashdb zero-overhead design, Ruchy tiered gates

**Strategy**:

1. **Zero Overhead by Default**:
   - Purification doesn't add runtime overhead (it's a transformation)
   - Debugging is opt-in (--debug flag)
   - Linting is pre-execution (no runtime cost)

2. **Fast Pre-Commit Checks** (<1 second):
   ```bash
   bashrs lint --fast script.sh
   # Only checks: syntax, obvious patterns
   # Skips: Full verification, property testing
   ```

3. **Thorough CI Checks** (5-10 minutes):
   ```bash
   bashrs lint script.sh                  # Full linting
   bashrs verify-determinism script.sh    # 2-run comparison
   bashrs verify-idempotent script.sh     # 3-run verification
   bashrs test script.sh                  # Unit tests
   ```

4. **Exhaustive Nightly Checks** (hours):
   ```bash
   bashrs prove script.sh                 # Property-based testing
   bashrs fuzz script.sh                  # Fuzz testing
   bashrs mutation-test script.sh         # Mutation testing
   ```

---

### 4.6 Testing Strategy

**Inspired by**: RuchyRuchy 3-layer validation, NASA quality standards

**Layer 1: Unit Tests**:
- Test each component independently
- Target: >85% code coverage
- Examples:
  - Parser: 100+ bash constructs
  - Purifier: 50+ transformation patterns
  - Debugger: 30+ debugging scenarios

**Layer 2: Integration Tests**:
- All tools on single bash script (anti-fraud measure)
- Verify tools agree on line numbers, transformations
- Example test (MANDATORY):
  ```rust
  #[test]
  fn test_all_bashrs_tools_agree_on_script() {
      let script = "deploy.sh";

      // Parse
      let ast = bashrs_parse(script);

      // Lint
      let lint = bashrs_lint(script);

      // Purify
      let purified = bashrs_purify(script);

      // Debug
      let debug = bashrs_debug(&purified);

      // CRITICAL: If lint says error at line 10,
      // purified must change line 10,
      // debug must show change at line 10
      assert_tool_consensus(ast, lint, purified, debug);
  }
  ```

**Layer 3: Real-World Validation**:
- Test on production bash scripts (100+ scripts from GitHub)
- Dogfood on bashrs itself (purify bashrs build scripts)
- Performance benchmarks (purification <100ms for typical scripts)

---

## 5. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-8)

**Week 1-2: REPL Core**
- Basic REPL loop (rustyline integration)
- Configuration system (ReplConfig)
- History management
- Mode switching (Normal, Purify, Lint)
- **Milestone**: `bashrs repl` command functional

**Week 3-4: Parser Integration**
- Embed bash parser into REPL
- AST display mode
- Syntax error reporting
- **Milestone**: Parse bash scripts in REPL

**Week 5-6: Purification Integration**
- Call purifier from REPL
- Show original vs purified
- Basic explanation (what changed)
- **Milestone**: Interactive purification

**Week 7-8: Linting Integration**
- Run linter from REPL
- Display lint results
- Categorize by severity
- **Milestone**: Interactive linting

**Phase 1 Deliverable**: Working REPL with parse, purify, lint modes

---

### Phase 2: Debugging Foundation (Weeks 9-16)

**Week 9-10: Breakpoint System**
- Line-based breakpoints
- Conditional breakpoints
- Hit-count breakpoints
- **Milestone**: Set breakpoints in bash scripts

**Week 11-12: Execution Control**
- Step (next line)
- Next (skip over functions)
- Continue (run until breakpoint)
- Finish (exit current function)
- **Milestone**: Step through bash scripts

**Week 13-14: State Inspection**
- Variable inspection
- Environment display
- Call stack tracking
- **Milestone**: Inspect bash state at breakpoints

**Week 15-16: Integration with Purification**
- Compare original vs purified
- Highlight differences
- Explain transformations
- **Milestone**: Debug with purification awareness

**Phase 2 Deliverable**: Working debugger integrated with purification

---

### Phase 3: Advanced Features (Weeks 17-24)

**Week 17-18: Determinism Checker**
- Scan for $RANDOM, $$, timestamps
- Replay verification (2 runs)
- Diff explanation
- **Milestone**: Verify determinism

**Week 19-20: Idempotency Analyzer**
- Scan for non-idempotent operations
- Suggested fixes
- Verification (3+ runs)
- **Milestone**: Verify idempotency

**Week 21-22: Purification Explainer**
- Detailed transformation explanations
- Safety rationale
- Alternative suggestions
- **Milestone**: Explain all purification decisions

**Week 23-24: ShellCheck Integration**
- Auto-run shellcheck on purified output
- Parse shellcheck warnings
- Display in bashrs format
- **Milestone**: POSIX compliance verification

**Phase 3 Deliverable**: Full safety verification suite

---

### Phase 4: Polish & Production (Weeks 25-32)

**Week 25-26: DevEx Improvements**
- Tab completion for commands
- Syntax highlighting in REPL
- Better error messages
- Help system
- **Milestone**: Professional UX

**Week 27-28: Performance Optimization**
- Fast linting (<1 second)
- Incremental parsing
- Caching
- **Milestone**: Pre-commit speed

**Week 29-30: Documentation**
- User guide
- Tutorial: "Your first purification"
- API documentation
- Examples repository
- **Milestone**: Complete docs

**Week 31-32: Testing & Validation**
- 3-layer validation suite
- Real-world script testing
- Dogfooding on bashrs itself
- Performance benchmarks
- **Milestone**: Production ready

**Phase 4 Deliverable**: Production-ready bashrs REPL + Debugger

---

## 6. Key Design Decisions

### Decision 1: Unified REPL + Debugger

**Rationale**: Following matklad's insight, debuggers and REPLs should be unified.

**Implementation**: Single `bashrs repl` command with debugging capabilities built-in.

**Alternatives Considered**:
- ❌ Separate `bashrs repl` and `bashrs debug` commands (fragmented UX)
- ❌ REPL without debugging (limited utility)
- ✅ Unified tool with modes (optimal UX)

---

### Decision 2: Embedded Parser + Purifier

**Rationale**: RuchyRuchy symbiotic compiler-debugger architecture shows power of embedding entire compiler.

**Implementation**: REPL contains full bash parser and purifier, enabling:
- Real-time AST display
- Interactive purification
- Transformation explanation
- Safety verification

**Alternatives Considered**:
- ❌ Call bashrs CLI as subprocess (slow, fragmented)
- ❌ Parse-only REPL (can't show purified output)
- ✅ Embedded approach (fast, integrated)

---

### Decision 3: Tiered Quality Gates

**Rationale**: RuchyRuchy tiered gates prevent developer overburden while maintaining quality.

**Implementation**:
- Tier 1 (Pre-Commit): <1 second, basic checks
- Tier 2 (CI): 5-10 minutes, integration tests
- Tier 3 (Nightly): Hours, exhaustive verification

**Alternatives Considered**:
- ❌ All checks on every commit (too slow, blocks developers)
- ❌ No pre-commit checks (catch bugs too late)
- ✅ Tiered approach (balance speed vs thoroughness)

---

### Decision 4: Purification-Aware Debugging

**Rationale**: bashrs unique value is bash purification, debugger should leverage this.

**Implementation**: Debugger shows original vs purified side-by-side, explains transformations.

**Alternatives Considered**:
- ❌ Generic bash debugger (no integration with purification)
- ❌ Purification-only tool (no debugging)
- ✅ Integrated approach (unique value proposition)

---

### Decision 5: Determinism & Idempotency Verification

**Rationale**: bashrs core mission is deterministic, idempotent bash. Debugger should verify this.

**Implementation**: Built-in verification commands:
- `verify-determinism`: Run twice, compare outputs
- `verify-idempotent`: Run N times, check same result

**Alternatives Considered**:
- ❌ Manual verification (error-prone)
- ❌ External tools (fragmented)
- ✅ Built-in verification (seamless UX)

---

## 7. Success Metrics

### Metric 1: Adoption

**Target**: 100+ GitHub repositories using bashrs within 6 months of v1.0 release

**Measurement**:
- GitHub search: `filename:.bashrs.toml` (config file)
- NPM/Cargo downloads
- Stars/forks on GitHub

---

### Metric 2: Defect Density

**Target**: <1 defect per 1000 lines of purified bash (observed in production)

**Measurement**:
- Track bug reports from users
- Analyze purified scripts that failed
- Calculate: (bugs found) / (total LOC purified)

---

### Metric 3: Purification Accuracy

**Target**: 95%+ of purifications improve safety without changing semantics

**Measurement**:
- Test suite: 1000+ bash scripts
- Compare: original behavior vs purified behavior
- Success: Behavior identical, safety improved

---

### Metric 4: Developer Productivity

**Target**: 3-5x faster to find non-deterministic code with bashrs debugger vs manual inspection

**Measurement**:
- User study: 20 developers, realistic debugging tasks
- Compare: bashrs debugger vs manual grep/analysis
- Calculate: Time savings, task success rate

---

### Metric 5: Test Coverage

**Target**: >85% code coverage, 100% mutation score

**Measurement**:
- `cargo llvm-cov` for line coverage
- `cargo mutants` for mutation score
- Enforce in CI

---

## 8. Risks and Mitigations

### Risk 1: REPL Security

**Risk**: Executing untrusted bash in REPL could compromise system

**Mitigation**:
- Sandboxed execution mode (à la Ruchy)
- Filesystem isolation (temp directories)
- Network blocking
- Resource limits (CPU, memory, time)

**Validation**:
- Fuzz testing: 10K+ malicious bash scripts
- Security audit before v1.0
- Bug bounty program post-release

---

### Risk 2: Purification Correctness

**Risk**: Purified bash behaves differently than original

**Mitigation**:
- Comprehensive test suite (1000+ scripts)
- Property-based testing (semantic equivalence)
- Real-world validation (GitHub scripts)
- Anti-fraud integration tests

**Validation**:
- Run original and purified side-by-side
- Compare outputs (must be identical)
- ShellCheck verification (both must pass)

---

### Risk 3: Performance

**Risk**: Debugging/purification too slow for interactive use

**Mitigation**:
- Incremental parsing
- Caching (AST, lint results)
- Fast path for common operations
- Tiered gates (fast pre-commit)

**Validation**:
- Performance benchmarks (target: <100ms for 1000-line script)
- Profiling (identify bottlenecks)
- Optimization iterations

---

### Risk 4: Bash Compatibility

**Risk**: bashrs breaks on non-standard bash extensions

**Mitigation**:
- Target POSIX sh (not bash-specific)
- ShellCheck integration (POSIX compliance)
- Comprehensive parsing (handle bash extensions gracefully)
- Clear error messages for unsupported features

**Validation**:
- Test against 1000+ GitHub bash scripts
- POSIX compliance test suite
- bash 4.x, 5.x, dash, ash testing

---

## 9. References

### Academic Papers (2020-2025)

1. Kallas, K., Mustafa, O., Benetopoulos, A., & Greenberg, M. (2022). "Practically Correct, Just-in-Time Shell Script Parallelization." In *16th USENIX Symposium on Operating Systems Design and Implementation (OSDI 22)*, pp. 1-18. https://www.usenix.org/system/files/osdi22-kallas.pdf

2. Lazarek, J., Chong, S., & Greenberg, M. (2025). "From Ahead-of-Time to Just-in-Time and Back Again: Static Analysis for Unix Shell Programs." In *Proceedings of the Workshop on Hot Topics in Operating Systems (HOTOS 25)*, pp. 1-6. https://regmedia.co.uk/2025/04/29/sash-hotos25-paper.pdf

3. Ko, A. J., & Myers, B. A. (2004). "Designing the whyline: a debugging interface for asking questions about program behavior." In *Proceedings of the SIGCHI conference on Human factors in computing systems*, pp. 151-158.

4. Ko, A. J., & Myers, B. A. (2008). "Debugging reinvented: asking and answering why and why not questions about program behavior." In *Proceedings of the 30th international conference on Software engineering*, pp. 301-310.

5. Pothier, G., & Tanter, É. (2009). "Back-in-time debugging for object-oriented languages." In *Proceedings of the 23rd European conference on Object-oriented programming*, pp. 242-266.

### Industry Sources

6. Kladov, A. (2025). "A Debugger is a REPL is a Debugger." https://matklad.github.io/2025/03/25/debugger-is-repl-is-debugger.html

7. Evans, J. (2021). "Debugging by starting a REPL at a breakpoint is fun." https://jvns.ca/blog/2021/09/16/debugging-in-a-repl-is-fun/

8. bashdb Project. (2024). "BASH with Debugger and Improved Debug Support and Error Handling." https://bashdb.sourceforge.net/

### Ruchy Project Sources

9. Ruchy REPL Implementation. `/home/noah/src/ruchy/src/runtime/repl/mod.rs` (Lines 1-200+)

10. Ruchy Debugger Implementation. `/home/noah/src/ruchy/src/debugger/mod.rs` (Lines 1-521)

11. RuchyRuchy Debugging Tools Specification. `/home/noah/src/ruchyruchy/docs/specifications/ruchyruchy-debugging-tools-spec.md` (Lines 1-2169)

---

## 10. Appendices

### Appendix A: Bash vs Ruchy Comparison

| Feature | Ruchy | bashrs | Notes |
|---------|-------|--------|-------|
| **Language Type** | General-purpose programming | Shell scripting | Different domains |
| **REPL Purpose** | Interactive development | Interactive purification | bashrs adds safety focus |
| **Debugger Focus** | Time-travel, type-aware | Determinism, idempotency | bashrs safety-centric |
| **Safety Model** | Type safety, ownership | Determinism, idempotency | Complementary approaches |
| **Target Output** | Rust code | POSIX sh scripts | Different compilation targets |

**Key Insight**: bashrs can learn from Ruchy's architecture while focusing on shell-specific safety concerns.

---

### Appendix B: bashrs Commands Reference

**REPL Commands**:
```
bashrs repl                    Start interactive REPL
  :help                        Show help
  :quit                        Exit REPL
  :mode <normal|purify|lint>   Switch mode
  :load <script>               Load bash script
  :debug <script>              Start debugging script
  :explain <line>              Explain purification at line
```

**Debugging Commands** (within REPL debug mode):
```
break <line>                   Set breakpoint
break <line> if <condition>    Conditional breakpoint
continue                       Continue execution
step                           Step to next line
next                           Step over functions
finish                         Run until function returns
print <var>                    Print variable value
watch <var>                    Watch variable for changes
compare-original               Show original vs purified
why-changed                    Explain purification
backtrace                      Show call stack
```

**Standalone Commands**:
```
bashrs lint <script>           Lint bash script
bashrs purify <script>         Purify bash script
bashrs explain <script>        Explain purifications
bashrs verify-determinism <script>   Verify determinism (2 runs)
bashrs verify-idempotent <script>    Verify idempotency (3+ runs)
bashrs debug <script>          Debug bash script
```

---

### Appendix C: Example REPL Session (Extended)

```bash
$ bashrs repl
Welcome to bashrs v1.0.0 - Bash Safety REPL
Type ':help' for commands, ':quit' to exit

bashrs> :load deploy.sh
Loaded: deploy.sh (45 lines, 3 functions)

bashrs> :mode lint
Mode: Lint

bashrs> deploy.sh
Linting deploy.sh...
  Line 10: IDEM001: mkdir /app/logs (should be mkdir -p)
  Line 15: DET001: SESSION_ID=$RANDOM (non-deterministic)
  Line 20: SEC001: Unquoted variable $file
  Line 25: IDEM002: rm /tmp/old (should be rm -f)
Result: 4 issues found

bashrs> :mode purify
Mode: Purify

bashrs> deploy.sh
Purifying deploy.sh...
  ✅ Line 10: mkdir → mkdir -p
  ✅ Line 15: $RANDOM → parameter ${1:-default}
  ✅ Line 20: $file → "$file"
  ✅ Line 25: rm → rm -f
Result: 4 transformations applied

bashrs> :explain 15
Transformation at line 15:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Original:  SESSION_ID=$RANDOM
Purified:  SESSION_ID="${1:-default}"

Why Changed?
  $RANDOM produces different value each run (non-deterministic).
  This makes deployments unpredictable and testing difficult.

Safety Benefit:
  Using parameter ${1:-default} makes SESSION_ID controllable.
  Same input always produces same output (deterministic).
  Enables reproducible deployments and automated testing.

Alternative Fixes:
  1. Fixed value: SESSION_ID=12345
  2. Environment: SESSION_ID="${SESSION_ID:-default}"
  3. Date-based: SESSION_ID=$(date +%Y%m%d)

Recommended:
  Parameter passing gives caller control.
  Can override per-deployment: ./deploy.sh my-session-id

bashrs> :mode debug
Mode: Debug

bashrs> :debug deploy.sh
Starting debugger for deploy.sh...

(bashrs-dbg) break 15
Breakpoint 1 at line 15: SESSION_ID="${1:-default}"

(bashrs-dbg) run default-session
Starting execution with args: [default-session]
Hit breakpoint at line 15

(bashrs-dbg) print SESSION_ID
SESSION_ID = "default-session"

(bashrs-dbg) compare-original
Original:  SESSION_ID=$RANDOM
Purified:  SESSION_ID="${1:-default}"
Actual:    SESSION_ID="default-session"

(bashrs-dbg) why-changed
Non-deterministic pattern detected: $RANDOM
Purification replaced with parameter for determinism.
See ':explain 15' for full details.

(bashrs-dbg) verify-determinism
Running 2 times to verify determinism...
  Run 1: SESSION_ID=default-session
  Run 2: SESSION_ID=default-session
  Result: ✅ DETERMINISTIC (outputs identical)

(bashrs-dbg) continue
Breakpoint 2 at line 20: rm -f /tmp/old

(bashrs-dbg) verify-idempotency
Running 3 times to verify idempotency...
  Run 1: Removed /tmp/old
  Run 2: /tmp/old not found, no error (✅ idempotent)
  Run 3: /tmp/old not found, no error (✅ idempotent)
  Result: ✅ IDEMPOTENT (3/3 runs successful)

(bashrs-dbg) continue
Script completed successfully

(bashrs-dbg) quit
Exiting debugger

bashrs> :quit
Goodbye!
```

---

## 11. Conclusion

This research analysis has examined production REPL and debugger implementations (Ruchy, RuchyRuchy), academic research on shell verification and debugging (USENIX OSDI 2022, ACM HOTOS 2025), and production tools (bashdb, POSIX debugging techniques).

**Key Recommendations for bashrs**:

1. **Unified REPL + Debugger**: Follow matklad's insight - debuggers and REPLs should be one tool
2. **Embedded Architecture**: Embed bash parser and purifier into debugger (à la RuchyRuchy symbiotic design)
3. **Purification-Aware**: Show original vs purified side-by-side, explain transformations
4. **Safety Verification**: Built-in determinism and idempotency verification
5. **Tiered Quality Gates**: Balance speed (pre-commit) vs thoroughness (nightly)
6. **Anti-Fraud Testing**: Integration tests ensure all tools agree
7. **DevEx Focus**: Cognitive walkthroughs, usability testing, comparative studies

**Implementation Priority**:
- Phase 1 (Weeks 1-8): REPL foundation with parse, purify, lint modes
- Phase 2 (Weeks 9-16): Debugger with breakpoints, stepping, state inspection
- Phase 3 (Weeks 17-24): Advanced features (determinism, idempotency, explainer)
- Phase 4 (Weeks 25-32): Polish, performance, production readiness

**Expected Outcomes**:
- 3-5x faster debugging of bash scripts (vs manual inspection)
- 95%+ purification accuracy (semantic equivalence)
- <1 second pre-commit linting (developer-friendly)
- >85% test coverage, 100% mutation score (NASA-level quality)

This comprehensive architecture positions bashrs as a world-class tool for safe, deterministic, idempotent bash scripting with an integrated REPL and debugging experience.

---

**END OF RESEARCH ANALYSIS**
