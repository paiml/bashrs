# bashrs WASM Phase 0: Feasibility Study

**Status**: 🔬 Research Phase
**Goal**: Validate that streaming I/O is feasible in WASM before full implementation

## What We're Testing

This Phase 0 implementation tests whether WASM can handle shell script streaming I/O requirements:

1. **Streaming Performance**: Can we stream output via JavaScript callbacks at >10 MB/s?
2. **Config Analysis**: Can CONFIG-001 to CONFIG-004 work in browser?
3. **Memory Usage**: Can we stay under 10MB for typical config files?
4. **Latency**: Is callback overhead <1ms per chunk?

## Building WASM Module

```bash
# Install wasm-pack (if not already installed)
cargo install wasm-pack

# Build the WASM module
cd rash
wasm-pack build --target web --features wasm --out-dir ../examples/wasm/pkg

# This generates:
# - pkg/bashrs.js (JavaScript glue code)
# - pkg/bashrs_bg.wasm (compiled WebAssembly)
# - pkg/bashrs.d.ts (TypeScript definitions)
```

## Running Examples

### Using Ruchy (RECOMMENDED - Optimized for WASM)

```bash
# Ruchy serve is optimized for WASM with proper MIME types
cd examples/wasm
ruchy serve --port 8000 --watch-wasm

# Open browser to:
# http://localhost:8000/index.html
```

**Why Ruchy?**
- ✅ Correct MIME types for `.wasm` files (`application/wasm`)
- ✅ CORS headers for local development
- ✅ Watch mode for auto-rebuild
- ✅ Verified by bashrs (not Python!)
- ✅ Zero configuration needed

### Alternative: Using Basic HTTP Server

```bash
# If ruchy not available, bash one-liner works too
cd examples/wasm
bash -c 'while true; do printf "HTTP/1.1 200 OK\nContent-Type: text/html\n\n$(cat index.html)" | nc -l 8000; done'
```

## Example: Config Analysis

```javascript
import init, { analyzeConfig } from './pkg/bashrs.js';

await init();

const bashrc = `
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # CONFIG-001: Duplicate!
export SESSION_ID=$RANDOM          # CONFIG-004: Non-deterministic!
`;

const result = analyzeConfig(bashrc, ".bashrc");
console.log(`Found ${result.issue_count} issues`);
const issues = JSON.parse(result.issues_json);
issues.forEach(issue => {
    console.log(`[${issue.rule_id}] Line ${issue.line}: ${issue.message}`);
});
```

## Example: Streaming Benchmark

```javascript
import init, { benchmarkStreaming } from './pkg/bashrs.js';

await init();

const testSize = 10 * 1024 * 1024;  // 10MB
const result = await benchmarkStreaming(testSize, (chunk) => {
    // Process each chunk
    // (In real usage, this would update UI, write to file, etc.)
});

console.log(`Optimal chunk size: ${result.optimal_chunk_size} bytes`);
console.log(`Max throughput: ${result.max_throughput_mbps} MB/s`);
console.log(`Meets requirements (>10 MB/s): ${result.meets_requirements}`);
```

## Go/No-Go Decision Criteria

After Phase 0, we'll make a Go/No-Go decision based on:

### Go Criteria (All must be true)
- ✅ Streaming throughput: **>10 MB/s**
- ✅ Callback latency: **<1ms average**
- ✅ Memory usage: **<10MB for typical files**
- ✅ Config analysis: **100% feature parity with native**

### No-Go Criteria (Any one true)
- ❌ Streaming throughput: **<5 MB/s**
- ❌ Callback latency: **>5ms average**
- ❌ Memory usage: **>50MB for typical files**
- ❌ Config analysis: **Missing critical features**

## Phase 0 Deliverables

1. ✅ **Basic WASM infrastructure** (this directory)
2. 🔄 **Streaming benchmark results** (pending browser testing)
3. 🔄 **Config analysis demo** (pending browser testing)
4. 🔄 **Performance report** (pending benchmarks)
5. 🔄 **Go/No-Go decision document** (pending results)

## Next Steps (If Go)

If Phase 0 succeeds:
- **Phase 1**: Full WASM emitter (3 weeks)
- **Phase 2**: Command emulation (2 weeks)
- **Phase 3**: Config management in WASM (2 weeks)
- **Phase 4**: Integration & examples (1-2 weeks)

If Phase 0 fails:
- Document findings
- Archive WASM approach
- Return to Option 2 (cleanup) or Option 3 (100% ShellCheck)

## Files

- `README.md` - This file
- `index.html` - Browser-based demo
- `benchmark.html` - Streaming performance tests
- `pkg/` - Compiled WASM module (generated)
