# bashrs Tracing Architectural Refinements

**Document Version**: 2.0 (Architectural Review Feedback)
**Date**: 2025-10-29
**Status**: REFINEMENTS TO BE INTEGRATED INTO MAIN SPEC
**Reviewer Feedback**: Toyota Way (*Jidoka*, *Genchi Genbutsu*) Applied

---

## Overview

This document contains critical architectural refinements based on peer review feedback. These improvements ensure the tracing system is not only powerful but also **practical, usable, and robust**.

**Key Themes**:
1. **Jidoka** (Automation with Human Touch): Guide developer attention to critical information
2. **Genchi Genbutsu** (Go and See): Design for novice, intermediate, and expert users
3. **Performance First**: Two-tiered buffering, diff-based storage, zero-copy design
4. **Determinism Guarantees**: Transformation purity validation
5. **Quality Assurance**: Golden trace testing (Phase 9)
6. **Risk Mitigation**: Early performance validation, user-centric rollout

---

## Refinement 1: Trace Significance Metric (Jidoka)

### Problem
The tracing system can produce thousands of events for complex scripts. Without ranking significance, developers are overwhelmed with noise, obscuring critical transformations.

### Solution: Trace Significance Scoring

**Event Significance Levels**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TraceSignificance {
    Critical   = 4,  // Transformation conflicts, security violations
    High       = 3,  // Transformations applied, parse errors
    Medium     = 2,  // Transformations skipped, rule evaluations
    Low        = 1,  // Parse nodes, generation steps
    Trace      = 0,  // Internal engine events
}

impl TraceEvent {
    /// Calculate significance of this trace event
    /// Higher significance events bubble to the top in UI
    fn significance(&self) -> TraceSignificance {
        match self {
            // CRITICAL: Conflicts and security violations
            TraceEvent::TransformationConflict { .. } => TraceSignificance::Critical,
            TraceEvent::RuleEvaluated {
                violation: Some(v), ..
            } if v.category == Category::Security => TraceSignificance::Critical,

            // HIGH: Transformations applied, parse/generation errors
            TraceEvent::TransformationApplied { .. } => TraceSignificance::High,
            TraceEvent::ParseError { .. } => TraceSignificance::High,

            // MEDIUM: Transformations skipped, rule evaluations
            TraceEvent::TransformationSkipped { .. } => TraceSignificance::Medium,
            TraceEvent::RuleEvaluated { .. } => TraceSignificance::Medium,

            // LOW: Parse nodes, generation steps
            TraceEvent::ParseNode { .. } => TraceSignificance::Low,
            TraceEvent::GenerateCode { .. } => TraceSignificance::Low,

            // TRACE: Start/complete events (structural only)
            TraceEvent::ParseStart { .. } => TraceSignificance::Trace,
            TraceEvent::PurifyStart { .. } => TraceSignificance::Trace,
            _ => TraceSignificance::Trace,
        }
    }
}
```

**UI Integration**:

```bash
# Default: Show HIGH + CRITICAL only
$ bashrs purify deploy.sh --trace
[HIGH]     Line 42: IDEM001 applied → mkdir -p
[HIGH]     Line 45: IDEM002 applied → rm -f
[CRITICAL] Line 50: CONFLICT: IDEM003 vs SEC001
[HIGH]     Line 55: DET001 applied → removed $RANDOM

# Verbose: Show MEDIUM + HIGH + CRITICAL
$ bashrs purify deploy.sh --trace --verbose
[MEDIUM]   Line 30: IDEM001 skipped (already has -p)
[HIGH]     Line 42: IDEM001 applied → mkdir -p
[MEDIUM]   Line 43: SEC002 evaluated (passed)
[HIGH]     Line 45: IDEM002 applied → rm -f

# Full: Show everything including TRACE/LOW
$ bashrs purify deploy.sh --trace --verbose --all
[TRACE]    Parsing started
[LOW]      Line 1: ParseNode (Shebang)
[LOW]      Line 2: ParseNode (Comment)
[MEDIUM]   Line 30: IDEM001 skipped
[HIGH]     Line 42: IDEM001 applied
```

**REPL Filter Commands**:

```bash
bashrs> :trace filter critical    # Show CRITICAL only
bashrs> :trace filter high+        # Show HIGH + CRITICAL (default)
bashrs> :trace filter medium+      # Show MEDIUM + HIGH + CRITICAL
bashrs> :trace filter all          # Show everything
```

**Property Test**:
```rust
proptest! {
    #[test]
    fn prop_critical_events_never_filtered(event in any::<TraceEvent>()) {
        let sig = event.significance();
        if sig == TraceSignificance::Critical {
            // Property: Critical events ALWAYS visible in default mode
            prop_assert!(sig >= TraceSignificance::High);
        }
    }
}
```

---

## Refinement 2: User Personas and Modes (Genchi Genbutsu)

### Problem
The full tracing system (slicing, dependency graphs, WhyLine queries) has a steep learning curve. Novice users need **simplified, guided** output. Experts need **full power**. One-size-fits-all doesn't work.

### Solution: Three User Personas with Adaptive Modes

**Persona 1: Learner Mode** 🎓

**Target User**: Students, bash beginners, those learning shell script purification

**Features**:
- **Default output**: Natural language explanations only
- **No complex commands**: Hide `:slice`, `:forward`, `:dependency-graph`
- **Interactive guidance**: Suggest next questions to ask
- **Educational focus**: Teach bash concepts, not tracing internals

**Example**:
```bash
$ bashrs purify deploy.sh --mode learner

Found issue with your script! Let me explain:

Line 42: mkdir /tmp/releases
Problem: This fails if /tmp/releases already exists (not idempotent)
Solution: Use `mkdir -p` to create directory only if missing
Fixed:   mkdir -p /tmp/releases

Want to learn more? Try these questions:
  :why mkdir -p        - Why was -p added?
  :next                - Show next issue
  :help idempotency    - Learn about idempotency
```

**Persona 2: Developer Mode** 💻

**Target User**: Professional developers, DevOps engineers, experienced scripters

**Features**:
- **Full command set**: All tracing features enabled
- **Concise output**: Show transformation diffs, rule IDs
- **Advanced queries**: Program slicing, causality graphs
- **Performance metrics**: Show overhead, transformation counts

**Example**:
```bash
$ bashrs purify deploy.sh --mode developer

Applied 5 transformations (12ms, <1% overhead):
  [IDEM001] Line 42: mkdir /tmp/releases → mkdir -p /tmp/releases
  [IDEM002] Line 45: rm /app/current → rm -f /app/current
  [DET001]  Line 50: $RANDOM removed → deterministic ID
  [IDEM003] Line 55: ln -s → ln -sf
  [SEC002]  Line 60: quoted variable expansion

Commands: :trace, :why, :slice, :forward, :dependency-graph, :replay
```

**Persona 3: Rule Author Mode** 🔧

**Target User**: bashrs contributors, rule authors, debugging tracing engine itself

**Features**:
- **Deep internals**: AST node IDs, rule evaluation metrics
- **Performance profiling**: Per-rule overhead, bottlenecks
- **Fault localization**: SBFL rankings for transformation bugs
- **Debug output**: Internal engine state, buffer contents

**Example**:
```bash
$ bashrs purify deploy.sh --mode rule-author --debug

=== Purification Performance Profile ===
Total: 12.3ms (11.8ms engine, 0.5ms tracing overhead: 4.0%)

Rule Evaluations:
  IDEM001 (mkdir idempotency):
    - Evaluated: 3 nodes
    - Applied: 2 nodes (66.7% match rate)
    - Avg time: 0.15ms/node
    - AST pattern: Command { name: "mkdir", args: [...] }

  DET001 (remove $RANDOM):
    - Evaluated: 10 nodes
    - Applied: 1 node (10% match rate)
    - Avg time: 0.05ms/node
    - AST pattern: ParameterExpansion { name: "RANDOM" }

Trace Buffer: 347/1024 events (33.9% full)
Fault Localization: No suspicious transformations detected

Commands: :profile, :sbfl, :ast-dump, :buffer-stats, :rule-metrics
```

**Mode Detection and Switching**:

```bash
# Auto-detect based on usage patterns
$ bashrs purify deploy.sh  # Defaults to Learner mode (first-time user)
$ bashrs purify deploy.sh  # Upgrades to Developer mode (after 10 sessions)

# Explicit mode selection
$ bashrs purify deploy.sh --mode learner
$ bashrs purify deploy.sh --mode developer
$ bashrs purify deploy.sh --mode rule-author

# REPL mode switching
bashrs> :mode learner       # Switch to Learner mode
bashrs> :mode developer     # Switch to Developer mode
bashrs> :mode rule-author   # Switch to Rule Author mode
bashrs> :mode               # Show current mode
```

**Configuration File** (~/.bashr/config.toml):
```toml
[tracing]
default_mode = "developer"
auto_upgrade = true  # Upgrade Learner → Developer after 10 sessions

[modes.learner]
show_rule_ids = false
show_ast_nodes = false
max_explanation_length = 200
suggest_next_questions = true

[modes.developer]
show_rule_ids = true
show_performance = true
default_significance = "high+"

[modes.rule_author]
show_internals = true
profile_rules = true
sbfl_enabled = true
```

---

## Refinement 3: Two-Tiered Trace Buffer (Performance + Omniscience)

### Problem
The current single circular buffer (1024 events) balances memory and coverage. However:
- **Complex scripts**: May require >10,000 events for full causality chains
- **Interactive REPL**: Needs fast, low-memory tracing
- **Post-hoc analysis**: Requires complete event history

One buffer size can't satisfy all use cases.

### Solution: Two-Tiered Buffer System

**Tier 1: In-Memory Circular Buffer** (Real-Time, Interactive)

```rust
/// Fast, fixed-size circular buffer for live REPL use
/// Oldest events evicted when full
struct CircularTraceBuffer {
    events: VecDeque<TraceEvent>,
    capacity: usize,  // Default: 1024
    total_events: u64,
    evicted_count: u64,
}

impl CircularTraceBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            total_events: 0,
            evicted_count: 0,
        }
    }

    fn push(&mut self, event: TraceEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front(); // Evict oldest
            self.evicted_count += 1;
        }
        self.events.push_back(event);
        self.total_events += 1;
    }

    /// Efficiency metric: What % of events are retained?
    fn retention_rate(&self) -> f64 {
        if self.total_events == 0 {
            return 1.0;
        }
        (self.total_events - self.evicted_count) as f64 / self.total_events as f64
    }
}
```

**Tier 2: Disk-Based Full Trace Log** (Complete History, Post-Hoc Analysis)

```rust
/// Append-only disk log for complete trace history
/// Compressed with zstd, streamed incrementally
struct DiskTraceLog {
    path: PathBuf,
    writer: BufWriter<File>,
    compressor: zstd::Encoder<'static, BufWriter<File>>,
    event_count: u64,
}

impl DiskTraceLog {
    fn new(path: PathBuf) -> Result<Self> {
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        let compressor = zstd::Encoder::new(writer, 3)?; // Level 3: fast compression

        Ok(Self {
            path,
            writer,
            compressor,
            event_count: 0,
        })
    }

    fn append(&mut self, event: &TraceEvent) -> Result<()> {
        // Serialize to JSON (one event per line)
        let json = serde_json::to_string(event)?;
        writeln!(self.compressor, "{}", json)?;
        self.event_count += 1;
        Ok(())
    }

    fn close(mut self) -> Result<PathBuf> {
        self.compressor.finish()?;
        Ok(self.path)
    }
}
```

**Unified Trace Manager** (Coordinates Both Tiers):

```rust
enum TraceMode {
    InMemoryOnly,           // Fast, limited history (REPL default)
    InMemoryWithDisk,       // Full history to disk + circular buffer
    DiskOnly,               // Minimal memory, disk streaming (CI/CD)
}

struct TraceManager {
    mode: TraceMode,
    circular_buffer: CircularTraceBuffer,
    disk_log: Option<DiskTraceLog>,
}

impl TraceManager {
    fn new(mode: TraceMode, capacity: usize) -> Result<Self> {
        let disk_log = match mode {
            TraceMode::InMemoryWithDisk | TraceMode::DiskOnly => {
                let path = PathBuf::from("/tmp/bashrs-trace.jsonl.zst");
                Some(DiskTraceLog::new(path)?)
            },
            TraceMode::InMemoryOnly => None,
        };

        Ok(Self {
            mode,
            circular_buffer: CircularTraceBuffer::new(capacity),
            disk_log,
        })
    }

    fn record(&mut self, event: TraceEvent) -> Result<()> {
        // Always write to disk log (if enabled)
        if let Some(ref mut log) = self.disk_log {
            log.append(&event)?;
        }

        // Circular buffer (unless disk-only mode)
        if self.mode != TraceMode::DiskOnly {
            self.circular_buffer.push(event);
        }

        Ok(())
    }
}
```

**Usage Examples**:

```bash
# Interactive REPL: In-memory only (fast, low memory)
$ bashrs repl
bashrs> :trace on
bashrs> mkdir /tmp/test
[Tracing enabled, in-memory buffer: 1024 events]

# Complex script: Full disk logging
$ bashrs purify complex_deploy.sh --trace-to-disk
Trace saved to: /tmp/bashrs-trace-20251029-143022.jsonl.zst
Size: 2.3 MB (compressed), 45,231 events

# Post-hoc analysis: Replay from disk
$ bashrs trace replay /tmp/bashrs-trace-20251029-143022.jsonl.zst
Loaded 45,231 events, replaying...
[Event 1] ParseStart { source: "complex_deploy.sh", line: 1 }
[Event 2] ParseNode { type: Shebang, span: 1:1-1:11 }
...

# CI/CD: Disk-only streaming (minimal memory)
$ bashrs purify --trace-mode disk-only --trace-output ci-trace.jsonl.zst
Memory usage: 12 MB (disk streaming, no circular buffer)
```

**Configuration**:

```toml
[tracing]
# Mode selection
mode = "in-memory-with-disk"  # Options: in-memory-only, in-memory-with-disk, disk-only

# In-memory buffer size
circular_buffer_size = 1024    # 256, 512, 1024, 2048, 4096

# Disk log settings
disk_log_path = "/tmp/bashrs-trace.jsonl.zst"
compression_level = 3          # 1 (fast) to 22 (max compression)
auto_cleanup = true            # Delete trace logs older than 7 days
```

**Performance Targets**:

| Mode | Memory | Disk I/O | Overhead | Use Case |
|------|--------|----------|----------|----------|
| In-Memory Only | ~8 MB | None | <5% | Interactive REPL |
| In-Memory + Disk | ~10 MB | Streaming | <10% | Complex scripts |
| Disk Only | ~2 MB | Streaming | <15% | CI/CD pipelines |

---

## Refinement 4: Custom DAP Protocol Extensions

### Problem
DAP (Debug Adapter Protocol) is designed for **imperative, procedural debugging** (call stacks, stack frames, variables). bashrs purification is a **declarative, functional transformation pipeline**. Forcing this into standard DAP semantics is awkward:
- "What is the 'call stack' of a series of AST transformations?"
- "What is a 'variable' in a functional pipeline?"

### Solution: Extend DAP with Custom `bashrs/*` Messages

**Standard DAP Messages** (Use Where Applicable):

```typescript
// Standard DAP: Start/stop debugging
dap.initialize(...)
dap.launch({ program: "deploy.sh", trace: true })
dap.terminate()

// Standard DAP: Breakpoints
dap.setBreakpoints({ source: "deploy.sh", breakpoints: [{ line: 42 }] })

// Standard DAP: Execution control
dap.continue()
dap.pause()
```

**Custom bashrs Extensions** (bashrs-Specific Concepts):

```typescript
// Custom: Get full trace history
interface bashrsFullTraceRequest {
  command: "bashrs/fullTrace"
}
interface bashrsFullTraceResponse {
  events: TraceEvent[]          // All trace events in circular buffer
  totalEvents: number           // Total events recorded
  evictedEvents: number         // Events evicted from buffer
  retentionRate: number         // Percentage retained
}

// Custom: Query trace significance filter
interface bashrsTraceFilterRequest {
  command: "bashrs/traceFilter"
  significance: "critical" | "high+" | "medium+" | "all"
}
interface bashrsTraceFilterResponse {
  events: TraceEvent[]          // Filtered events
}

// Custom: Get causality graph (dependency between transformations)
interface bashrsCausalityGraphRequest {
  command: "bashrs/causalityGraph"
}
interface bashrsCausalityGraphResponse {
  nodes: TransformationNode[]   // Transformations as nodes
  edges: DependencyEdge[]       // Dependencies as edges
}

// Custom: WhyLine query
interface bashrsWhyLineRequest {
  command: "bashrs/whyLine"
  query: {
    type: "why" | "why-not"
    node: AstNodeId
    transformation?: RuleId
  }
}
interface bashrsWhyLineResponse {
  explanation: string           // Natural language explanation
  causalityChain: Transformation[]  // Causality chain
  relatedRules: RuleId[]
}

// Custom: Program slicing
interface bashrsSliceRequest {
  command: "bashrs/slice"
  direction: "backward" | "forward"
  criterion: {
    line: number
    transformation?: RuleId
  }
}
interface bashrsSliceResponse {
  slice: TraceEvent[]           // Relevant trace events
  dependencies: TransformationId[]
}

// Custom: Time-travel navigation
interface bashrsTimeTravelRequest {
  command: "bashrs/timeTravel"
  direction: "forward" | "backward"
  steps: number
}
interface bashrsTimeTravelResponse {
  currentEvent: TraceEvent
  currentIndex: number
  ast: BashAst                  // AST at this point in time
}

// Custom: Trace statistics
interface bashrsTraceStatsRequest {
  command: "bashrs/traceStats"
}
interface bashrsTraceStatsResponse {
  totalEvents: number
  eventsByType: Record<string, number>
  eventsBySignificance: Record<TraceSignificance, number>
  transformationsApplied: number
  transformationsSkipped: number
  conflicts: number
  violations: number
  duration_ms: number
  overheadPercent: number
}
```

**DAP Message Flow Example**:

```typescript
// Frontend (VS Code extension) requests trace
client.send({
  command: "bashrs/fullTrace"
})

// bashrs DAP adapter responds
server.respond({
  events: [
    { type: "ParseStart", source: "deploy.sh", line: 1 },
    { type: "TransformationApplied", rule: "IDEM001", ... },
    ...
  ],
  totalEvents: 347,
  evictedEvents: 0,
  retentionRate: 1.0
})

// Frontend requests causality graph
client.send({
  command: "bashrs/causalityGraph"
})

// Server responds with graph data (visualize in UI)
server.respond({
  nodes: [
    { id: "T1", rule: "IDEM001", line: 42 },
    { id: "T2", rule: "IDEM002", line: 45 },
  ],
  edges: [
    { from: "T1", to: "T2", type: "enables" }
  ]
})
```

**Why Custom Extensions Are Better**:

1. **Semantic Clarity**: `bashrs/whyLine` is self-documenting, not repurposed `evaluate()`
2. **Type Safety**: Custom TypeScript interfaces prevent misuse
3. **Future-Proof**: Add bashrs-specific features without breaking DAP compliance
4. **Debugger Compatibility**: Standard DAP features (breakpoints, continue, pause) still work

**DAP Specification Contribution**:

If bashrs tracing proves successful, we can propose these extensions to the DAP specification as a "Transformation Debugging Profile" for static analysis tools, transpilers, and linters.

---

## Refinement 5: Diff-Based TraceEvent Storage (Memory Optimization)

### Problem
The current `TransformationApplied` event stores full AST nodes:

```rust
TransformationApplied {
    rule_id: RuleId,
    node_before: AstNode,      // ❌ Full clone (expensive!)
    node_after: AstNode,       // ❌ Full clone (expensive!)
    reason: String,
    span: Span,
}
```

For large scripts, AST nodes can be >10 KB each. Cloning thousands of nodes for trace events creates **massive memory pressure** and GC overhead.

### Solution: Store Nodes by Reference, Diffs as Patches

**Optimized TraceEvent Structure**:

```rust
/// Unique identifier for AST nodes (cheap to copy)
type AstNodeId = u64;

/// Minimal diff representation (not full clone)
#[derive(Debug, Clone)]
enum AstNodePatch {
    AddedFlag { flag: String },               // e.g., "Added -p"
    RemovedFlag { flag: String },             // e.g., "Removed -f"
    ReplacedArgument { index: usize, old: String, new: String },
    ReplacedExpression { old_expr: String, new_expr: String },
    AddedQuotes { variable: String },         // e.g., $foo → "$foo"
    RemovedRandomVar,                         // $RANDOM removed
}

/// Memory-efficient transformation event
#[derive(Debug, Clone)]
struct TransformationApplied {
    rule_id: RuleId,
    node_id: AstNodeId,        // ✅ Reference to node (8 bytes)
    patch: AstNodePatch,       // ✅ Minimal diff (typically <100 bytes)
    reason: String,
    span: Span,
}
```

**AST Node Registry** (Centralized Storage):

```rust
/// Registry stores all AST nodes, events reference by ID
struct AstNodeRegistry {
    nodes: HashMap<AstNodeId, AstNode>,
    next_id: AstNodeId,
}

impl AstNodeRegistry {
    fn register(&mut self, node: AstNode) -> AstNodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    fn get(&self, id: AstNodeId) -> Option<&AstNode> {
        self.nodes.get(&id)
    }
}
```

**Replay with Patch Application**:

```rust
impl TraceReplay {
    fn apply_transformation(&self,
                           registry: &AstNodeRegistry,
                           event: &TransformationApplied) -> Result<AstNode> {
        // Get original node by reference
        let node_before = registry.get(event.node_id)
            .ok_or("Node not found")?
            .clone();

        // Apply patch to reconstruct node_after
        let node_after = match &event.patch {
            AstNodePatch::AddedFlag { flag } => {
                node_before.add_flag(flag)
            },
            AstNodePatch::RemovedFlag { flag } => {
                node_before.remove_flag(flag)
            },
            AstNodePatch::ReplacedExpression { old_expr, new_expr } => {
                node_before.replace_expression(old_expr, new_expr)
            },
            _ => node_before, // Other patches...
        };

        Ok(node_after)
    }
}
```

**Memory Comparison**:

| Storage Strategy | Memory per Event | 1000 Events | 10,000 Events |
|------------------|------------------|-------------|---------------|
| **Full Clone** (current) | ~20 KB | ~20 MB | ~200 MB |
| **Diff + Reference** (optimized) | ~150 bytes | ~150 KB | ~1.5 MB |
| **Savings** | **99.25%** | **99.25%** | **99.25%** |

**Property Test** (Verify Reconstruction):

```rust
proptest! {
    #[test]
    fn prop_patch_reconstruction_exact(
        node in arbitrary_ast_node(),
        transformation in arbitrary_transformation()
    ) {
        // Apply transformation (full)
        let node_after_full = apply_transformation_full(&node, &transformation);

        // Apply transformation (via patch)
        let patch = create_patch(&node, &node_after_full);
        let node_after_patch = apply_patch(&node, &patch);

        // Property: Reconstruction must be exact
        prop_assert_eq!(node_after_full, node_after_patch);
    }
}
```

---

## Refinement 6: Transformation Purity Validation (Determinism Guarantee)

### Problem
The time-travel replay algorithm assumes transformations are **pure functions**:
```rust
fn apply_transformation(ast: BashAst, t: Transformation) -> BashAst
```

If a transformation relies on external state (file system, environment variables, randomness), replay becomes **non-deterministic**. This breaks time-travel debugging.

### Solution: Automated Purity Validation in EXTREME TDD

**Add to Phase 2: GREEN Phase** (after implementation):

```rust
/// Validate that a transformation is a pure function
/// - No external state access (file I/O, env vars, randomness)
/// - Deterministic output (same input → same output)
/// - No side effects (no mutations outside function scope)
#[test]
fn test_IDEM001_transformation_purity() {
    // ARRANGE: Create test AST node
    let input_ast = parse_bash("mkdir /tmp/test");
    let transformation = Transformation {
        rule_id: RuleId::IDEM001,
        // ...
    };

    // ACT: Apply transformation twice in separate processes
    let output1 = apply_transformation_isolated(input_ast.clone(), transformation.clone());
    let output2 = apply_transformation_isolated(input_ast.clone(), transformation.clone());

    // ASSERT: Output must be byte-identical (deterministic)
    assert_eq!(output1, output2, "Transformation must be deterministic");

    // ASSERT: No side effects (file system unchanged)
    assert_no_file_changes("/tmp");
}

/// Apply transformation in isolated subprocess (no shared state)
fn apply_transformation_isolated(ast: BashAst, t: Transformation) -> BashAst {
    // Serialize to JSON
    let ast_json = serde_json::to_string(&ast).unwrap();
    let t_json = serde_json::to_string(&t).unwrap();

    // Run in subprocess
    let output = Command::new("cargo")
        .args(&["run", "--bin", "bashrs-transform-isolated"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Write input
    write!(output.stdin.unwrap(), "{}\n{}", ast_json, t_json).unwrap();

    // Read output
    let output_bytes = output.wait_with_output().unwrap().stdout;
    serde_json::from_slice(&output_bytes).unwrap()
}
```

**Purity Violation Detection**:

```rust
#[test]
fn test_SECURITY_transformation_purity_violation() {
    // This transformation reads environment variables (non-pure!)
    let input_ast = parse_bash("echo $USER");
    let transformation = Transformation {
        rule_id: RuleId::CUSTOM_ENV_VAR_RESOLUTION,
        // ...
    };

    // ACT: Apply transformation
    let result = apply_transformation_isolated(input_ast, transformation);

    // ASSERT: Should fail (non-deterministic)
    assert!(result.is_err(), "Non-pure transformation must be rejected");
}
```

**Integration into EXTREME TDD**:

Phase 2: GREEN Phase (Implementation)
- [ ] Write implementation
- [ ] Run unit test (verify GREEN ✅)
- [ ] **NEW**: Run purity validation test
- [ ] **STOP THE LINE** if purity test fails ❌
- [ ] Refactor to remove external state dependencies
- [ ] Re-run purity validation (verify GREEN ✅)

**Why This Matters**:
- **Time-Travel Guarantee**: Replay is deterministic
- **Regression Safety**: Refactoring can't introduce non-determinism
- **Distributed Analysis**: Transformations can run in parallel (no shared state)

---

## Refinement 7: Phase 9 - Trace Quality Validation (Golden Trace Testing)

### Problem
The current 8-phase EXTREME TDD methodology validates that the tracing engine is implemented **correctly**. It does NOT validate that the traces it produces are **useful, clear, and accurate**. A perfectly correct engine could still generate confusing causality chains or unhelpful explanations.

### Solution: Add Phase 9 - Golden Trace Testing

**Phase 9 Overview**:

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 9: Trace Quality Validation                          │
│                                                             │
│ 1. Create "Golden Trace" benchmark suite                   │
│ 2. Manually craft ideal traces for representative scripts  │
│ 3. Assert generated traces match golden traces             │
│ 4. Measure trace quality metrics (clarity, completeness)   │
└─────────────────────────────────────────────────────────────┘
```

**Golden Trace Benchmark Suite**:

```
tests/golden_traces/
├── README.md                     # Benchmark methodology
├── 01-basic-idempotency/
│   ├── input.sh                  # Input bash script
│   ├── expected_trace.jsonl      # Golden trace (manually crafted)
│   ├── expected_why_mkdir.txt    # Expected WhyLine explanation
│   └── test.rs                   # Test runner
├── 02-determinism-random/
│   ├── input.sh
│   ├── expected_trace.jsonl
│   └── test.rs
├── 03-conflict-resolution/
│   ├── input.sh
│   ├── expected_trace.jsonl
│   └── test.rs
...
├── 20-complex-multi-rule/
│   ├── input.sh
│   ├── expected_trace.jsonl
│   └── test.rs
```

**Golden Trace Creation Process**:

1. **Select Representative Scripts** (20-30 scripts covering):
   - Basic idempotency (`mkdir`, `rm`, `ln`)
   - Determinism (`$RANDOM`, `$(date +%s)`, `$$`)
   - Security violations (unquoted variables, eval, curl|sh)
   - Complex scenarios (multiple rules, conflicts, dependencies)

2. **Manually Craft Golden Traces**:
   - Run bashrs tracing on the script
   - Review generated trace, refine explanations
   - Validate causality chains are correct
   - Ensure WhyLine explanations are clear
   - Save as `expected_trace.jsonl`

3. **Commit Golden Traces to Git**:
   - Golden traces become "ground truth"
   - Changes to tracing engine must not break golden traces
   - If trace improves, update golden file (with justification)

**Example Golden Trace** (`tests/golden_traces/01-basic-idempotency/expected_trace.jsonl`):

```jsonl
{"type":"ParseStart","source":"input.sh","line":1,"col":1}
{"type":"ParseComplete","duration_ms":2}
{"type":"PurifyStart"}
{"type":"TransformationApplied","rule_id":"IDEM001","line":5,"node_type":"Command","node_before":"mkdir /tmp/releases","node_after":"mkdir -p /tmp/releases","reason":"Added `-p` flag for idempotency. Without `-p`, mkdir fails if directory exists, making the script non-idempotent (not safe to re-run).","significance":"High"}
{"type":"PurifyComplete","transformations_applied":1,"duration_ms":5}
```

**Test Runner** (`tests/golden_traces/01-basic-idempotency/test.rs`):

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_golden_trace_01_basic_idempotency() {
    let input = "tests/golden_traces/01-basic-idempotency/input.sh";
    let expected_trace = "tests/golden_traces/01-basic-idempotency/expected_trace.jsonl";

    // Run bashrs with tracing
    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("purify")
        .arg(input)
        .arg("--trace-output")
        .arg("/tmp/actual_trace.jsonl")
        .output()
        .expect("Failed to run bashrs");

    assert!(output.status.success(), "bashrs purify should succeed");

    // Load expected and actual traces
    let expected_events = fs::read_to_string(expected_trace).unwrap();
    let actual_events = fs::read_to_string("/tmp/actual_trace.jsonl").unwrap();

    // Compare traces (semantic equality, not byte-for-byte)
    let expected_trace = parse_trace(&expected_events);
    let actual_trace = parse_trace(&actual_events);

    assert_trace_equivalence(&expected_trace, &actual_trace);
}

/// Compare traces for semantic equivalence
fn assert_trace_equivalence(expected: &Trace, actual: &Trace) {
    // Same number of significant events (ignore TRACE-level)
    let expected_significant = expected.events.iter()
        .filter(|e| e.significance() >= TraceSignificance::Medium)
        .collect::<Vec<_>>();
    let actual_significant = actual.events.iter()
        .filter(|e| e.significance() >= TraceSignificance::Medium)
        .collect::<Vec<_>>();

    assert_eq!(expected_significant.len(), actual_significant.len(),
               "Trace must have same number of significant events");

    // Compare each significant event
    for (i, (exp, act)) in expected_significant.iter().zip(&actual_significant).enumerate() {
        assert_eq!(exp.event_type(), act.event_type(),
                   "Event {} type mismatch", i);
        assert_eq!(exp.rule_id(), act.rule_id(),
                   "Event {} rule_id mismatch", i);
        assert_eq!(exp.significance(), act.significance(),
                   "Event {} significance mismatch", i);

        // Allow minor variations in explanations (but must contain key phrases)
        if let (Some(exp_reason), Some(act_reason)) = (exp.reason(), act.reason()) {
            assert!(act_reason.contains_key_phrases(exp_reason),
                    "Event {} explanation must contain key phrases: expected '{}', got '{}'",
                    i, exp_reason, act_reason);
        }
    }
}
```

**Trace Quality Metrics**:

```rust
/// Measure trace quality (beyond just correctness)
struct TraceQualityMetrics {
    clarity_score: f64,        // Are explanations clear? (human-rated 1-10)
    completeness: f64,         // Are all transformations explained? (% coverage)
    causality_accuracy: f64,   // Are causality chains correct? (% correct)
    verbosity: f64,            // Is output too verbose? (events/transformation)
}

#[test]
fn test_trace_quality_metrics() {
    let golden_traces = load_all_golden_traces();

    for golden in golden_traces {
        let actual = run_bashrs_trace(&golden.input);
        let metrics = compute_quality_metrics(&golden.expected_trace, &actual);

        // Quality thresholds
        assert!(metrics.clarity_score >= 7.0, "Trace clarity must be >= 7/10");
        assert!(metrics.completeness >= 0.95, "Trace completeness must be >= 95%");
        assert!(metrics.causality_accuracy >= 0.98, "Causality accuracy must be >= 98%");
        assert!(metrics.verbosity <= 5.0, "Verbosity must be <= 5 events/transformation");
    }
}
```

**Why Phase 9 Matters**:
- **User Experience**: Validates traces are actually useful, not just correct
- **Regression Prevention**: Changes to tracing engine can't degrade quality
- **Documentation**: Golden traces serve as examples for users
- **Educational Validation**: Ensures explanations are pedagogically sound

---

## Refinement 8: Risk Mitigation Strategy (Andon Cord Moments)

### Problem
The tracing system is ambitious (21 tickets, 21 weeks). If performance is poor, UX is confusing, or causality chains are wrong, the entire project could fail after significant investment.

### Solution: Early Risk Identification and Mitigation

**Risk 1: Performance Overhead Exceeds <10% Target**

**Severity**: 🚨 STOP THE LINE (Andon Cord)

**Early Detection**:
- **Week 1**: Build simplest possible tracer (event logging only, no slicing/time-travel)
- **Week 1**: Benchmark against realistic bash scripts (100 lines, 1000 lines, 10K lines)
- **Week 1**: Measure overhead: `(time_with_tracing - time_without) / time_without * 100`

**Failure Criteria**:
- If overhead >50% in Week 1 → **STOP THE LINE** ❌
- Root cause: Likely excessive cloning, synchronous I/O, or circular buffer design

**Mitigation**:
1. **Zero-Copy First**: Implement diff-based TraceEvent (Refinement 5) before any other feature
2. **Async Disk Logging**: Disk writes must be non-blocking (use tokio::spawn)
3. **Profile Ruthlessly**: Use `cargo flamegraph` to find bottlenecks
4. **Simplify**: Remove features that don't meet overhead target

**Go/No-Go Decision (Week 1)**:
- ✅ Overhead <10%: Proceed with roadmap
- ⚠️ Overhead 10-20%: Optimize, re-benchmark, then proceed
- ❌ Overhead >20%: **ABORT** tracing system, revisit architecture

---

**Risk 2: UI/UX Complexity Becomes Unmanageable**

**Severity**: 🟨 Moderate (User Adoption Risk)

**Early Detection**:
- **Phase 1 Complete**: Immediately build simplest UI (Live REPL Tracing with WhyLine)
- **User Testing**: Get 3-5 beta testers to try REPL tracing
- **Feedback**: Ask "Is this overwhelming?" and "What would you use most?"

**Failure Criteria**:
- If >50% of testers find UI "confusing" or "too complex" → **PAUSE FEATURE ROLLOUT** ⚠️

**Mitigation**:
1. **User Personas First**: Implement Learner/Developer/RuleAuthor modes (Refinement 2) BEFORE advanced features
2. **Progressive Disclosure**: Hide advanced features (slicing, graphs) until user needs them
3. **Defaults Matter**: Default to Learner mode for first-time users
4. **Iterative Rollout**: Don't build all visualization features until REPL tracing is proven useful

**Prioritization Based on User Feedback**:
- If users love WhyLine but ignore dependency graphs → deprioritize graphs
- If users ask for timeline visualization → prioritize that feature

---

**Risk 3: Causality Chains are Ambiguous or Incorrect**

**Severity**: 🚨 CRITICAL (Breaks Trust in Tool)

**Early Detection**:
- **Phase 3 (Program Slicing)**: Create 10 test scripts with known causality chains
- **Manual Validation**: Manually trace dependencies, compare with generated chains
- **Property Test**: Generate random transformations, verify causality properties

**Failure Criteria**:
- If >5% of causality chains are incorrect → **STOP THE LINE** ❌

**Mitigation**:
1. **Ambiguous Causality Support**: Implement multi-path causality (Transformation A enables BOTH B and C)
2. **Graph Validation**: Use graph algorithms (topological sort) to validate DAG properties
3. **Golden Trace Testing**: Phase 9 golden traces include manual causality chain validation

**Example: Multi-Path Causality**:

```rust
struct CausalityChain {
    target: TransformationId,
    paths: Vec<Vec<TransformationId>>,  // Multiple paths to target
}

// Bad: Single-path (misleading)
// :why IDEM003 → Shows: DET001 → IDEM003
// Reality: DET001 → IDEM003 AND SEC002 → IDEM003

// Good: Multi-path (complete)
// :why IDEM003 → Shows:
//   Path 1: DET001 → IDEM003
//   Path 2: SEC002 → IDEM003
```

---

**Risk 4: Tracing System Becomes Technical Debt**

**Severity**: 🟨 Moderate (Maintenance Burden)

**Early Detection**:
- **Phase 8 (Documentation)**: Measure documentation coverage
- **Maintainability Metrics**: Cyclomatic complexity, module coupling

**Failure Criteria**:
- If complexity >10 per function → **REFACTOR REQUIRED** ⚠️
- If documentation <80% coverage → **DOCUMENTATION DEBT** ⚠️

**Mitigation**:
1. **EXTREME TDD Discipline**: Never skip refactoring phase
2. **Complexity Budget**: Each module has max complexity of 10
3. **Documentation First**: Write rustdoc BEFORE implementation
4. **Mutation Testing**: Maintain ≥90% kill rate (forces quality)

---

### Risk Mitigation Timeline

| Week | Checkpoint | Risk Assessed | Go/No-Go Decision |
|------|-----------|---------------|-------------------|
| 1 | Core Tracing | Performance overhead | <10%: GO, >20%: NO-GO |
| 4 | Time-Travel | Memory efficiency | <50 MB: GO |
| 8 | REPL Integration | User feedback | >70% positive: GO |
| 12 | DAP Integration | IDE compatibility | Works in VS Code: GO |
| 16 | Full Validation | All metrics | All green: RELEASE |

---

## Summary: Architectural Refinements

| Refinement | Problem Solved | Implementation Complexity | Impact |
|------------|----------------|---------------------------|--------|
| 1. Trace Significance | Information overload | Low (1 week) | High (usability) |
| 2. User Personas | Steep learning curve | Medium (2 weeks) | Critical (adoption) |
| 3. Two-Tiered Buffer | Memory vs coverage | Medium (2 weeks) | High (scalability) |
| 4. Custom DAP Extensions | DAP semantic mismatch | Low (1 week) | Medium (future-proof) |
| 5. Diff-Based Storage | Memory pressure | Medium (2 weeks) | Critical (performance) |
| 6. Purity Validation | Non-deterministic replay | Low (1 week) | Critical (correctness) |
| 7. Phase 9 (Golden Traces) | Unclear/incorrect traces | Medium (3 weeks) | Critical (quality) |
| 8. Risk Mitigation | Project failure | Ongoing | Critical (success) |

**Total Additional Work**: ~12 weeks (parallel with existing 21-week roadmap)

**Updated Timeline**: 21 weeks (main roadmap) + 4 weeks (refinement integration) = **25 weeks total**

---

## Conclusion

These refinements transform the tracing system from a **technically correct implementation** into a **practical, usable, high-quality developer tool** that embodies the Toyota Way principles:

- **🚨 Jidoka (Automation with Human Touch)**: Trace Significance guides attention to what matters
- **🔍 Genchi Genbutsu (Go and See)**: User Personas ensure design works for real developers
- **⚡ Kaizen (Continuous Improvement)**: Risk mitigation and Phase 9 validation ensure quality
- **📈 Respect for People**: Learner mode makes tracing accessible to beginners

**Next Steps**:
1. Integrate these refinements into main specification document
2. Update roadmap to include refinement tickets
3. Proceed with Phase 1 implementation (Core Tracing Infrastructure)
