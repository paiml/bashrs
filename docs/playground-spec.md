# Playground-Style REPL Specification for rash

## 1. System Architecture

### 1.1 Core Components

The playground REPL implements a reactive architecture with three primary subsystems:

```rust
pub struct PlaygroundSystem {
    // Document management with CRDT-like properties
    document_store: DocumentStore,
    
    // Lock-free incremental computation graph
    computation_dag: Arc<ComputationGraph>,
    
    // Zero-copy rendering pipeline
    render_pipeline: RenderPipeline,
}

struct DocumentStore {
    rope: ropey::Rope,                    // B-tree rope, O(log n) edits
    syntax_tree: tree_sitter::Tree,       // Incremental parse tree
    version_vector: VersionVector,        // Lamport timestamps for edits
    checkpoint_trie: CheckpointTrie,      // Persistent data structure
}

struct ComputationGraph {
    nodes: dashmap::DashMap<NodeId, ComputeNode>,
    edges: petgraph::Graph<NodeId, Dependency>,
    dirty_set: crossbeam::queue::SegQueue<NodeId>,
}
```

### 1.2 Data Flow Architecture

The system implements a push-based dataflow with backpressure:

```
┌─────────────┐  InputEdit   ┌──────────────┐  AST Delta   ┌─────────────┐
│   Terminal  │─────────────▶│   Document   │────────────▶│  Transpiler │
│   (60 FPS)  │              │     Store    │              │  (Parallel) │
└─────────────┘              └──────────────┘              └─────────────┘
       ▲                            │                             │
       │                            │ Version Vector              │ Shell IR
       │                            ▼                             ▼
┌─────────────┐  Draw Commands ┌──────────────┐  Diagnostics ┌─────────────┐
│   Viewport  │◀───────────────│    Render    │◀─────────────│  Validator  │
│   Manager   │                │   Pipeline   │              │   (SIMD)    │
└─────────────┘                └──────────────┘              └─────────────┘
```

### 1.3 Memory Layout

Optimized for cache efficiency and NUMA awareness:

```rust
#[repr(C, align(64))]  // Cache line aligned
struct EditOperation {
    timestamp: u64,                    // 8 bytes
    start_byte: u32,                   // 4 bytes
    old_end_byte: u32,                 // 4 bytes
    new_end_byte: u32,                 // 4 bytes
    content_hash: u64,                 // 8 bytes
    _padding: [u8; 36],                // Pad to 64 bytes
}

// Lock-free ring buffer for edits
struct EditRingBuffer {
    buffer: Box<[EditOperation; 4096]>,  // 256KB, fits in L2
    head: AtomicU64,
    tail: AtomicU64,
}
```

## 2. Incremental Computation Model

### 2.1 Dependency Graph

The computation graph tracks fine-grained dependencies between AST nodes and transpilation outputs:

```rust
enum ComputeNode {
    Parse { range: ByteRange, version: u64 },
    Validate { ast_node: AstNodeId, rules: BitSet },
    Transpile { ir_node: IrNodeId, dialect: ShellDialect },
    Highlight { line_range: LineRange, theme: ThemeId },
}

impl ComputationGraph {
    fn mark_dirty(&self, edit: &EditOperation) {
        // Find affected parse nodes via interval tree
        let affected = self.interval_tree.query(edit.byte_range());
        
        // Propagate dirtiness through dependency edges
        let mut wavefront = VecDeque::from(affected);
        let mut visited = BitVec::new(self.nodes.len());
        
        while let Some(node) = wavefront.pop_front() {
            if visited.set(node.0, true) {
                continue;
            }
            
            // Mark node dirty
            self.dirty_set.push(node);
            
            // Add dependents to wavefront
            for edge in self.edges.edges_directed(node, Direction::Outgoing) {
                wavefront.push_back(edge.target());
            }
        }
    }
    
    fn recompute(&self) -> Result<ComputationResult> {
        // Parallel execution with work-stealing
        rayon::scope(|scope| {
            while let Some(node_id) = self.dirty_set.pop() {
                let node = self.nodes.get(&node_id)?;
                
                scope.spawn(move |_| {
                    match node.value() {
                        ComputeNode::Parse { range, .. } => {
                            self.reparse_range(range)
                        }
                        ComputeNode::Transpile { ir_node, dialect } => {
                            self.retranspile_node(ir_node, dialect)
                        }
                        // ...
                    }
                });
            }
        })
    }
}
```

### 2.2 Tree-sitter Integration

Incremental parsing with sub-millisecond updates:

```rust
struct IncrementalParser {
    parser: tree_sitter::Parser,
    query_cache: HashMap<&'static str, tree_sitter::Query>,
    edit_distance_threshold: usize,  // Heuristic for full reparse
}

impl IncrementalParser {
    fn apply_edit(&mut self, tree: &mut Tree, edit: &TextEdit) -> Result<ParseDelta> {
        let start = Instant::now();
        
        // Convert rope coordinates to tree-sitter format
        let ts_edit = tree_sitter::InputEdit {
            start_byte: edit.start_byte,
            old_end_byte: edit.old_end_byte,
            new_end_byte: edit.new_end_byte,
            start_position: self.byte_to_point(edit.start_byte),
            old_end_position: self.byte_to_point(edit.old_end_byte),
            new_end_position: self.byte_to_point(edit.new_end_byte),
        };
        
        tree.edit(&ts_edit);
        
        // Incremental parse with timeout
        let new_tree = self.parser.parse_with(
            &mut |offset, _| self.rope.byte_slice(offset..).as_str(),
            Some(tree),
        )?;
        
        // Compute changed nodes via tree diff
        let changes = tree.changed_ranges(&new_tree)
            .map(|range| AstRange {
                start: range.start_byte,
                end: range.end_byte,
                nodes: self.collect_nodes_in_range(&new_tree, range),
            })
            .collect();
        
        Ok(ParseDelta {
            duration: start.elapsed(),
            changed_ranges: changes,
            tree: new_tree,
        })
    }
}
```

## 3. Rendering Pipeline

### 3.1 Differential Rendering

The renderer maintains a persistent framebuffer and computes minimal diffs:

```rust
struct RenderPipeline {
    front_buffer: Buffer,
    back_buffer: Buffer,
    damage_regions: IntervalSet<Rect>,
    layout_cache: LayoutCache,
}

impl RenderPipeline {
    fn render_frame(&mut self, state: &PlaygroundState) -> Result<Vec<DrawCommand>> {
        // Clear damage regions in back buffer
        for region in &self.damage_regions {
            self.back_buffer.clear_region(region);
        }
        
        // Render only damaged regions
        let mut commands = Vec::with_capacity(self.damage_regions.len() * 10);
        
        for region in &self.damage_regions {
            match self.layout_cache.get_widget_at(region) {
                Widget::SourceEditor => {
                    self.render_source_lines(region, state, &mut commands)?;
                }
                Widget::TranspileOutput => {
                    self.render_shell_output(region, state, &mut commands)?;
                }
                Widget::DiagnosticPanel => {
                    self.render_diagnostics(region, state, &mut commands)?;
                }
            }
        }
        
        // Compute diff between buffers
        let diff_commands = self.front_buffer.diff(&self.back_buffer);
        commands.extend(diff_commands);
        
        // Swap buffers
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        self.damage_regions.clear();
        
        Ok(commands)
    }
}
```

### 3.2 Syntax Highlighting Pipeline

SIMD-accelerated token classification:

```rust
struct SyntaxHighlighter {
    theme: syntect::Theme,
    scope_stack: Vec<syntect::Scope>,
    token_cache: LruCache<LineId, Vec<StyledToken>>,
}

impl SyntaxHighlighter {
    fn highlight_line_simd(&self, line: &str, scopes: &[Scope]) -> Vec<StyledToken> {
        // Vectorized UTF-8 validation
        let valid_mask = simdutf8::basic::from_utf8(line.as_bytes())
            .map(|_| u8x32::splat(0xFF))
            .unwrap_or_else(|_| u8x32::splat(0x00));
        
        // Parallel token classification
        let tokens = line.split_whitespace()
            .par_bridge()
            .map(|token| {
                let scope = self.classify_token(token, scopes);
                let style = self.theme.style_for_scope(scope);
                
                StyledToken {
                    text: token,
                    style: Style {
                        fg: Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b),
                        bg: style.background.map(|c| Color::Rgb(c.r, c.g, c.b)),
                        modifiers: self.convert_font_style(style.font_style),
                    },
                }
            })
            .collect();
        
        tokens
    }
}
```

## 4. Performance Constraints

### 4.1 Latency Requirements

Based on empirical psychophysics research on perceived responsiveness:

| Operation | P50 Target | P99 Target | Hard Limit | Perceptual Threshold |
|-----------|------------|------------|------------|----------------------|
| Keystroke → Visual | 8ms | 16ms | 33ms | Instantaneous (<100ms) |
| Edit → Transpile | 20ms | 50ms | 100ms | Fluid (<100ms) |
| Transpile → Validate | 5ms | 20ms | 50ms | Seamless (<50ms) |
| Scroll → Render | 4ms | 8ms | 16ms | 60 FPS requirement |

### 4.2 Memory Constraints

```rust
const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;  // 10MB
const MAX_AST_NODES: usize = 1_000_000;
const RENDER_BUFFER_SIZE: usize = 4 * 1024 * 1024;  // 4MB (4K display)

struct MemoryBudget {
    document_store: usize,      // 15MB (rope + syntax tree)
    computation_cache: usize,   // 10MB (transpilation cache)
    render_buffers: usize,      // 8MB (double buffering)
    syntax_highlight: usize,    // 5MB (token cache)
    misc_overhead: usize,       // 2MB
    // Total: ~40MB resident set size
}
```

## 5. User Interaction Model

### 5.1 Modal Editing States

```rust
enum EditorMode {
    Normal {
        pending_operator: Option<Operator>,
        count: Option<usize>,
    },
    Insert {
        completion_ctx: CompletionContext,
        snippet_engine: SnippetEngine,
    },
    Visual {
        selection: Selection,
        mode: VisualMode,
    },
    Command {
        cmdline_buffer: String,
        history_index: usize,
    },
}

struct KeymapEngine {
    trie: KeyTrie<Action>,
    timeout_ms: u64,  // 500ms for key sequences
    
    fn process_key(&mut self, key: KeyEvent, mode: &EditorMode) -> Option<Action> {
        // Trie-based keymap matching with timeout
        match self.trie.step(key) {
            TrieResult::Match(action) => {
                self.trie.reset();
                Some(action)
            }
            TrieResult::Prefix => {
                // Start timeout for sequence completion
                self.start_timeout();
                None
            }
            TrieResult::NoMatch => {
                self.trie.reset();
                self.fallback_action(key, mode)
            }
        }
    }
}
```

### 5.2 Layout Management

Dynamic tiling with golden ratio constraints:

```rust
enum LayoutStrategy {
    Vertical { ratio: f32 },    // φ = 1.618 (golden ratio)
    Horizontal { ratio: f32 },
    Tabbed { active: TabId },
    Accordion { expanded: PanelId },
}

impl LayoutEngine {
    fn calculate_splits(&self, area: Rect, strategy: &LayoutStrategy) -> Vec<Rect> {
        match strategy {
            LayoutStrategy::Vertical { ratio } => {
                let split = (area.width as f32 * ratio) as u16;
                vec![
                    Rect { width: split, ..area },
                    Rect { x: area.x + split, width: area.width - split, ..area },
                ]
            }
            LayoutStrategy::Horizontal { ratio } => {
                // Ensure minimum 5 lines for readability
                let split = ((area.height as f32 * ratio).max(5.0)) as u16;
                vec![
                    Rect { height: split, ..area },
                    Rect { y: area.y + split, height: area.height - split, ..area },
                ]
            }
            // ...
        }
    }
}
```

## 6. Transpilation Pipeline

### 6.1 Debounced Execution

Adaptive debouncing based on edit patterns:

```rust
struct AdaptiveDebouncer {
    base_delay_ms: u64,  // 150ms
    burst_threshold: usize,  // 5 edits/second
    history: VecDeque<Instant>,
    
    fn calculate_delay(&mut self) -> Duration {
        let now = Instant::now();
        
        // Remove old entries
        while let Some(front) = self.history.front() {
            if now.duration_since(*front) > Duration::from_secs(1) {
                self.history.pop_front();
            } else {
                break;
            }
        }
        
        // Adaptive delay based on burst detection
        let delay = if self.history.len() > self.burst_threshold {
            // User is typing rapidly, increase delay
            Duration::from_millis(self.base_delay_ms * 2)
        } else {
            Duration::from_millis(self.base_delay_ms)
        };
        
        self.history.push_back(now);
        delay
    }
}
```

### 6.2 Cancellable Transpilation

Structured concurrency with graceful cancellation:

```rust
struct TranspilationTask {
    generation: AtomicU64,
    cancel_token: CancellationToken,
}

impl TranspilationController {
    async fn transpile_with_cancellation(
        &self,
        source: Arc<str>,
        generation: u64,
    ) -> Result<TranspilationOutput> {
        let token = CancellationToken::new();
        let child_token = token.child_token();
        
        // Update generation and cancel previous
        let prev_gen = self.current_generation.swap(generation, Ordering::SeqCst);
        if prev_gen < generation {
            self.cancel_token.cancel();
            self.cancel_token = token;
        }
        
        // Transpile with cancellation checks
        tokio::select! {
            result = self.transpile_internal(source, child_token) => {
                result
            }
            _ = child_token.cancelled() => {
                Err(TranspilationError::Cancelled)
            }
        }
    }
    
    async fn transpile_internal(
        &self,
        source: Arc<str>,
        cancel: CancellationToken,
    ) -> Result<TranspilationOutput> {
        // Parse phase
        cancel.check_cancelled()?;
        let ast = self.parser.parse(&source)?;
        
        // Validation phase with cancellation points
        let mut diagnostics = Vec::new();
        for node in ast.nodes() {
            cancel.check_cancelled()?;
            if let Err(e) = self.validator.validate_node(node) {
                diagnostics.push(e);
            }
        }
        
        // Transpilation phase
        cancel.check_cancelled()?;
        let shell_ir = self.transpiler.ast_to_ir(&ast)?;
        
        cancel.check_cancelled()?;
        let shell_code = self.emitter.emit(&shell_ir)?;
        
        Ok(TranspilationOutput {
            shell_code,
            diagnostics,
            metrics: self.collect_metrics(),
        })
    }
}
```

## 7. Testing Infrastructure

### 7.1 Property-Based UI Testing

```rust
#[derive(Debug, Clone, Arbitrary)]
enum UserAction {
    TypeText(String),
    MoveCursor(CursorMotion),
    DeleteRange(Selection),
    Undo,
    Redo,
    SwitchLayout(LayoutStrategy),
}

proptest! {
    #[test]
    fn ui_state_invariants(actions: Vec<UserAction>) {
        let mut playground = PlaygroundState::new();
        
        for action in actions {
            playground.apply_action(action);
            
            // Invariant 1: Cursor within document bounds
            prop_assert!(playground.cursor.offset <= playground.rope.len_bytes());
            
            // Invariant 2: Syntax tree covers entire document
            prop_assert_eq!(
                playground.syntax_tree.root_node().byte_range(),
                0..playground.rope.len_bytes()
            );
            
            // Invariant 3: No overlapping damage regions
            let regions = &playground.render_pipeline.damage_regions;
            for (i, r1) in regions.iter().enumerate() {
                for r2 in regions.iter().skip(i + 1) {
                    prop_assert!(!r1.intersects(r2));
                }
            }
            
            // Invariant 4: Transpilation matches source
            if let Some(output) = &playground.last_transpilation {
                let re_transpiled = playground.transpiler.emit(&playground.ast).unwrap();
                prop_assert_eq!(output.shell_code, re_transpiled);
            }
        }
    }
}
```

### 7.2 Differential Testing Against Reference Implementation

```rust
struct DifferentialTester {
    reference_impl: ReferenceTranspiler,  // Simple, correct implementation
    optimized_impl: PlaygroundTranspiler,  // Fast, incremental implementation
}

impl DifferentialTester {
    fn test_equivalence(&self, source: &str) -> Result<()> {
        let ref_output = self.reference_impl.transpile(source)?;
        let opt_output = self.optimized_impl.transpile(source)?;
        
        // Semantic equivalence, not textual
        let ref_ast = sh_parser::parse(&ref_output)?;
        let opt_ast = sh_parser::parse(&opt_output)?;
        
        assert_eq!(
            self.normalize_ast(ref_ast),
            self.normalize_ast(opt_ast),
            "Outputs differ semantically for input: {:?}",
            source
        );
        
        Ok(())
    }
}
```

## 8. Error Recovery

### 8.1 Graceful Degradation

```rust
enum DegradationLevel {
    Full,           // All features available
    NoHighlight,    // Syntax highlighting disabled
    NoCompletion,   // Completion disabled
    BasicOnly,      // Only basic editing
    ReadOnly,       // Catastrophic failure
}

impl PlaygroundState {
    fn handle_error(&mut self, error: Error) -> Result<()> {
        match error.severity() {
            Severity::Recoverable => {
                // Log and continue
                self.error_log.push(error);
            }
            Severity::Degraded => {
                // Disable affected subsystem
                match error.subsystem() {
                    Subsystem::SyntaxHighlight => {
                        self.degradation_level = DegradationLevel::NoHighlight;
                        self.highlighter = None;
                    }
                    Subsystem::Transpiler => {
                        // Show last valid transpilation
                        self.show_stale_warning = true;
                    }
                    // ...
                }
            }
            Severity::Fatal => {
                // Attempt to save user work
                self.emergency_save()?;
                self.degradation_level = DegradationLevel::ReadOnly;
            }
        }
        Ok(())
    }
}
```

## 9. Persistence and State Management

### 9.1 Session State

```rust
#[derive(Serialize, Deserialize)]
struct SessionState {
    #[serde(with = "rope_serde")]
    document: ropey::Rope,
    cursor_position: CursorPosition,
    layout: LayoutStrategy,
    transpiler_config: TranspilerConfig,
    
    // Compressed history for undo/redo
    #[serde(with = "compressed_history")]
    history: History,
    
    // Metrics for telemetry
    session_metrics: SessionMetrics,
}

mod rope_serde {
    pub fn serialize<S>(rope: &Rope, ser: S) -> Result<S::Ok, S::Error> {
        // Serialize as chunks for efficiency
        let chunks: Vec<&str> = rope.chunks().collect();
        chunks.serialize(ser)
    }
    
    pub fn deserialize<'de, D>(de: D) -> Result<Rope, D::Error> {
        let chunks: Vec<String> = Vec::deserialize(de)?;
        Ok(Rope::from_iter(chunks.iter().map(|s| s.as_str())))
    }
}
```

### 9.2 URL State Encoding

Shareable playground URLs with compressed state:

```rust
impl SessionState {
    fn to_url(&self) -> String {
        // Extract minimal state
        let minimal = MinimalState {
            source: self.document.to_string(),
            config: self.transpiler_config.clone(),
        };
        
        // Compress with Brotli (better than gzip for small text)
        let encoded = brotli::encode(
            &serde_json::to_vec(&minimal).unwrap(),
            11,  // Maximum compression
            22,  // Window size
        );
        
        // Base64url encode
        let b64 = base64::encode_config(
            &encoded,
            base64::URL_SAFE_NO_PAD
        );
        
        format!("https://play.rash-lang.org/?c={}", b64)
    }
}
```

## 10. Production Deployment

### 10.1 Binary Size Optimization

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = "fat"           # Link-time optimization
codegen-units = 1     # Single codegen unit
strip = true          # Strip symbols
panic = "abort"       # No unwinding

[dependencies]
# Feature flags for minimal builds
ratatui = { version = "0.26", default-features = false, features = ["crossterm"] }
syntect = { version = "5.0", default-features = false, features = ["parsing", "default-themes"] }
tree-sitter = { version = "0.20", default-features = false }

# Optional features
[features]
default = ["playground"]
playground = ["ratatui", "ropey", "tree-sitter", "syntect"]
minimal = []  # Transpiler only, no REPL
```

### 10.2 Platform-Specific Optimizations

```rust
#[cfg(target_os = "linux")]
fn optimize_for_linux() {
    // Use io_uring for async I/O
    if io_uring::probe().is_ok() {
        ASYNC_BACKEND.store(AsyncBackend::IoUring, Ordering::Relaxed);
    }
    
    // Enable huge pages for large allocations
    unsafe {
        libc::madvise(
            RENDER_BUFFER.as_ptr() as *mut _,
            RENDER_BUFFER.len(),
            libc::MADV_HUGEPAGE,
        );
    }
}

#[cfg(target_os = "macos")]
fn optimize_for_macos() {
    // Use kqueue for file watching
    WATCHER_BACKEND.store(WatcherBackend::Kqueue, Ordering::Relaxed);
    
    // Optimize for M1/M2 unified memory
    if is_apple_silicon() {
        MEMORY_ALLOCATOR.store(AllocStrategy::Unified, Ordering::Relaxed);
    }
}
```

This specification provides a production-ready foundation for implementing a TypeScript Playground-style REPL for rash, with careful attention to performance, correctness, and user experience. The architecture supports sub-frame latency for all interactive operations while maintaining correctness through comprehensive testing strategies.