# bashrs WASM Testing Specification
## SQLite-Inspired Elite Testing for Shell Script Analysis in WebAssembly

**Version**: 1.0
**Date**: 2025-10-22
**Methodology**: Adapted from SQLite Testing + WOS Canary Tests + interactive.paiml.com WASM validation
**Target**: NASA-level quality for production WASM deployment

---

## Executive Summary

### Mission-Critical Quality Standards

bashrs WASM provides shell script analysis in browsers for:
- **WOS (Web Operating System)**: Shell script execution and linting
- **interactive.paiml.com**: Educational bash/shell tutorials
- **Production websites**: Real-time config file validation

**Zero tolerance for failure** - These are mission-critical tools that users depend on.

### Testing Philosophy: SQLite + WOS + interactive.paiml.com

| Standard | bashrs WASM Application | Implementation |
|----------|------------------------|----------------|
| **SQLite TCL Tests** | Browser Canary Tests | Playwright E2E validating real user workflows |
| **WOS Canary Tests** | Config Analysis Workflows | Test critical bashrs config analysis paths |
| **interactive.paiml.com** | WASM Execution Tests | Actual bashrs execution in Pyodide-like environment |
| **Anomaly Testing** | Error Injection | localStorage failures, OOM, network errors |
| **Performance** | <100ms analysis | Real-time feedback for user actions |
| **Cross-Browser** | Chromium/Firefox/WebKit | Full browser compatibility |

---

## 1. Four-Harness Testing Framework

### Harness 1: Browser Canary Tests (BCT)

**Purpose**: Validate critical bashrs workflows in production browsers
**Inspiration**: WOS Canary Tests (60 tests, 8-second runtime)
**Coverage Target**: 100% of critical config analysis workflows

#### Test Categories

**B01-B10: Config Analysis Workflows** (10 tests)
- B01: Load WASM module successfully
- B02: Analyze .bashrc with CONFIG-001 (PATH deduplication)
- B03: Analyze .zshrc with CONFIG-002 (Quote variable expansions)
- B04: Analyze config with CONFIG-003 (Duplicate aliases)
- B05: Analyze config with CONFIG-004 (Non-deterministic constructs)
- B06: Display issues in UI with correct line numbers
- B07: Purify config and show fixed output
- B08: Handle large config files (>10KB)
- B09: Handle malformed config files gracefully
- B10: Performance: Analyze 1KB config in <100ms

**B11-B20: Streaming I/O Performance** (10 tests)
- B11: Stream 1MB output via JavaScript callbacks
- B12: Stream 10MB output with >10 MB/s throughput
- B13: Callback latency <1ms average
- B14: No memory leaks after 1000 callbacks
- B15: Handle callback errors gracefully
- B16: Backpressure handling (slow consumer)
- B17: Chunk size optimization (test 1KB, 10KB, 100KB)
- B18: Binary data streaming (WASM binary output)
- B19: Multiple concurrent streams
- B20: Performance: Stream 10MB in <1 second

**B21-B30: Error Handling & Anomalies** (10 tests)
- B21: Handle localStorage quota exceeded
- B22: Handle OOM during analysis
- B23: Handle network failure during WASM load
- B24: Handle corrupted WASM binary
- B25: Handle invalid JavaScript callbacks
- B26: Handle tab suspension/resume
- B27: Handle page reload during analysis
- B28: Handle browser back/forward navigation
- B29: Handle concurrent analysis requests
- B30: Recover from WASM panic

**B31-B40: Cross-Browser Compatibility** (10 tests)
- B31: Chromium: Full functionality
- B32: Firefox: Full functionality
- B33: WebKit/Safari: Full functionality
- B34: Mobile Chrome: Touch interactions
- B35: Mobile Safari: iOS-specific behavior
- B36: Edge: Chromium-based compatibility
- B37: Older browsers: Graceful degradation
- B38: WebAssembly feature detection
- B39: Fallback for non-WASM browsers
- B40: Performance across all browsers

#### Implementation

```typescript
// e2e/bashrs-wasm-canary.spec.ts
import { test, expect, Page } from '@playwright/test';

// Helper: Load bashrs WASM
async function loadBashrsWasm(page: Page): Promise<void> {
  await page.goto('/examples/wasm/index.html');
  await page.waitForSelector('#status:has-text("✅ WASM module loaded successfully")', {
    timeout: 30000
  });
}

// Helper: Analyze config
async function analyzeConfig(page: Page, config: string): Promise<void> {
  await page.locator('#config-input').fill(config);
  await page.locator('#analyze-btn').click();
  await page.waitForSelector('#results', { state: 'visible' });
}

// B01: Load WASM module successfully
test('B01: WASM module loads successfully', async ({ page }) => {
  const startTime = Date.now();

  await page.goto('/examples/wasm/index.html');

  // Wait for WASM to load
  await page.waitForSelector('#status:has-text("✅ WASM module loaded successfully")', {
    timeout: 30000
  });

  // Verify version displayed
  const version = await page.locator('#version').textContent();
  expect(version).toMatch(/\d+\.\d+\.\d+/);

  // Performance check
  const duration = Date.now() - startTime;
  expect(duration).toBeLessThan(5000); // Load in <5s
  console.log(`WASM load: ${duration}ms (target: <5000ms)`);
});

// B02: Analyze .bashrc with CONFIG-001 (PATH deduplication)
test('B02: CONFIG-001 - PATH deduplication', async ({ page }) => {
  await loadBashrsWasm(page);

  const bashrc = `
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # CONFIG-001: Duplicate!
`;

  const startTime = Date.now();
  await analyzeConfig(page, bashrc);
  const duration = Date.now() - startTime;

  // Verify CONFIG-001 detected
  const issuesContainer = page.locator('#issues-container');
  const issueText = await issuesContainer.textContent();
  expect(issueText).toContain('CONFIG-001');
  expect(issueText).toContain('PATH');

  // Verify line number correct
  expect(issueText).toContain('Line 3');

  // Performance check
  expect(duration).toBeLessThan(100); // Analyze in <100ms
  console.log(`Analysis: ${duration}ms (target: <100ms)`);
});

// B11: Stream 1MB output via JavaScript callbacks
test('B11: Stream 1MB via callbacks', async ({ page }) => {
  await loadBashrsWasm(page);

  // Test streaming API
  const result = await page.evaluate(async () => {
    // @ts-ignore: bashrs WASM API
    const { streamOutput } = await import('./pkg/bashrs.js');

    let bytesReceived = 0;
    const chunks: string[] = [];

    const stats = await streamOutput(
      'x'.repeat(1024 * 1024), // 1MB data
      10240, // 10KB chunks
      (chunk: string) => {
        chunks.push(chunk);
        bytesReceived += chunk.length;
      }
    );

    return {
      bytesReceived,
      chunkCount: chunks.length,
      throughputMbps: stats.throughput_mbps,
    };
  });

  // Verify all data received
  expect(result.bytesReceived).toBe(1024 * 1024);

  // Verify throughput
  expect(result.throughputMbps).toBeGreaterThan(10); // >10 MB/s
  console.log(`Streaming throughput: ${result.throughputMbps} MB/s (target: >10 MB/s)`);
});

// B21: Handle localStorage quota exceeded
test('B21: localStorage quota exceeded', async ({ page }) => {
  await loadBashrsWasm(page);

  // Fill localStorage to quota
  await page.evaluate(() => {
    try {
      const large = 'x'.repeat(1024 * 1024); // 1MB chunks
      for (let i = 0; i < 100; i++) {
        localStorage.setItem(`test${i}`, large);
      }
    } catch (e) {
      // Quota exceeded - expected
    }
  });

  // Try to analyze config (should handle gracefully)
  const bashrc = 'export PATH="/usr/local/bin:$PATH"';
  await analyzeConfig(page, bashrc);

  // Verify no crash
  const status = await page.locator('#status').textContent();
  expect(status).not.toContain('Error');

  // Verify analysis still works
  const results = await page.locator('#results').textContent();
  expect(results).toBeTruthy();
});
```

### Harness 2: Unit Tests (Rust)

**Purpose**: Test bashrs core logic before WASM compilation
**Coverage**: 100% of config analysis rules

```rust
// rash/src/wasm/api.rs tests
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_analyze_config_basic() {
        let config = r#"
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate
        "#;

        let result = analyze_config(config, Some(".bashrc".to_string())).unwrap();
        assert_eq!(result.issue_count, 1);
        assert!(result.issues.iter().any(|i| i.rule_id == "CONFIG-001"));
    }

    #[wasm_bindgen_test]
    fn test_purify_config_deterministic() {
        let config = "export SESSION_ID=$RANDOM";
        let purified = purify_config(config).unwrap();

        // Should remove $RANDOM
        assert!(!purified.contains("$RANDOM"));
        assert!(purified.contains("# Removed non-deterministic"));
    }
}
```

### Harness 3: Property-Based Tests

**Purpose**: Fuzz config inputs to find edge cases
**Inspiration**: SQLite dbsqlfuzz (1 billion tests/day)

```rust
// rash/tests/wasm_property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_analyze_never_panics(
        config in ".*{0,10000}"  // Any string up to 10KB
    ) {
        use bashrs::wasm::analyze_config;

        // Should never panic, regardless of input
        let result = analyze_config(&config, Some(".bashrc".to_string()));

        // Either succeeds or returns error (never panics)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn prop_purify_deterministic(
        config in "[a-zA-Z0-9=\" \n]{0,1000}"
    ) {
        use bashrs::wasm::purify_config;

        // Purify should be deterministic
        let purified1 = purify_config(&config);
        let purified2 = purify_config(&config);

        assert_eq!(purified1, purified2);
    }
}
```

### Harness 4: Mutation Testing

**Purpose**: Verify tests catch actual bugs
**Target**: >90% mutation score

```bash
# Run mutation testing on WASM modules
cargo mutants --file rash/src/wasm/api.rs
cargo mutants --file rash/src/wasm/streaming.rs
cargo mutants --file rash/src/config/analyzer.rs

# TARGET: ≥90% kill rate
```

---

## 2. Performance Baselines

All operations must meet these targets:

| Operation | Target | Test |
|-----------|--------|------|
| WASM load | <5s | B01 |
| Config analysis (1KB) | <100ms | B02-B05 |
| Config purify (1KB) | <200ms | B07 |
| Stream 10MB | <1s, >10 MB/s | B11-B12 |
| Callback latency | <1ms avg | B13 |
| Large config (10KB) | <500ms | B08 |
| Memory per analysis | <10MB | B14 |

Tests automatically **FAIL** if performance degrades.

---

## 3. Cross-Browser Matrix

| Browser | Version | Tests | Status |
|---------|---------|-------|--------|
| Chromium | Latest | All 40 | Required |
| Firefox | Latest | All 40 | Required |
| WebKit/Safari | Latest | All 40 | Required |
| Mobile Chrome | Latest | B31-B35 | Required |
| Mobile Safari | Latest | B31-B35 | Required |
| Edge | Latest | B31 | Optional |

---

## 4. Quality Gates (MANDATORY)

### Before Commit
```bash
# Run fast canary tests (<2 min)
make wasm-canary

# Verify WASM builds
make wasm-build

# Run Rust unit tests
cargo test --lib --features wasm
```

### Before Release
```bash
# Full browser matrix (~15 min)
make wasm-canary-all

# Property-based tests
cargo test --lib --features wasm --release -- --include-ignored

# Mutation testing
make wasm-mutation

# Performance benchmarks
make wasm-bench
```

### CI/CD Pipeline
```yaml
name: bashrs WASM Quality

on: [push, pull_request]

jobs:
  wasm-quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Dependencies
        run: |
          cargo install wasm-pack
          npm install -g playwright

      - name: Build WASM
        run: |
          cd rash
          wasm-pack build --target web --features wasm

      - name: Run Canary Tests
        run: make wasm-canary-all

      - name: Property Tests
        run: cargo test --lib --features wasm --release

      - name: Mutation Testing
        run: cargo mutants --file rash/src/wasm/

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: wasm-test-report
          path: e2e/playwright-report/
```

---

## 5. Deployment Targets

### Target 1: WOS (Web Operating System)

**URL**: https://wos.paiml.com
**Integration**: bashrs as system linter/analyzer
**Requirements**:
- Loads in <5s on first visit
- Works offline (Service Worker)
- <1MB WASM binary
- Zero panics/crashes

**Deployment Path**:
```bash
# Build optimized WASM
cd rash
wasm-pack build --target web --release --features wasm --out-dir ../wos-integration/pkg

# Copy to WOS
cp -r ../wos-integration/pkg /path/to/wos/dist/bashrs/

# Test in WOS
cd /path/to/wos
make wasm-canary
```

### Target 2: interactive.paiml.com

**URL**: https://interactive.paiml.com
**Integration**: Real-time shell script tutorials
**Requirements**:
- Instant feedback (<100ms)
- Educational error messages
- Syntax highlighting integration
- Mobile-friendly

**Deployment Path**:
```bash
# Build with educational features
cd rash
wasm-pack build --target web --release --features wasm,pretty-errors

# Copy to interactive.paiml.com
cp -r pkg /path/to/interactive.paiml.com/public/bashrs/

# Test educational workflows
cd /path/to/interactive.paiml.com
deno test tests/wasm/test-bashrs-integration.ts
```

---

## 6. Anomaly Testing (Critical)

### Memory Anomalies

```typescript
test('OOM during analysis', async ({ page }) => {
  // Fill memory
  await page.evaluate(() => {
    const arrays = [];
    try {
      while (true) {
        arrays.push(new Uint8Array(1024 * 1024));
      }
    } catch (e) {
      // OOM
    }
  });

  // Try to analyze (should handle gracefully)
  await analyzeConfig(page, 'export PATH="/usr/local/bin"');

  // Should not crash
  const status = await page.locator('#status').textContent();
  expect(status).not.toContain('crashed');
});
```

### Storage Anomalies

```typescript
test('localStorage corruption', async ({ page }) => {
  // Corrupt localStorage
  await page.evaluate(() => {
    localStorage.setItem('bashrs_state', 'CORRUPTED!!!');
  });

  // Load WASM (should handle gracefully)
  await loadBashrsWasm(page);

  // Should still work
  const status = await page.locator('#status').textContent();
  expect(status).toContain('✅');
});
```

### Network Anomalies

```typescript
test('WASM load failure', async ({ page }) => {
  // Block .wasm file
  await page.route('**/*.wasm', route => route.abort());

  await page.goto('/examples/wasm/index.html');

  // Should show error message
  const status = await page.locator('#status').textContent();
  expect(status).toContain('Failed to load WASM');

  // Should not leave page broken
  expect(await page.locator('body').isVisible()).toBe(true);
});
```

---

## 7. Test Execution

### Development Workflow

```bash
# Fast feedback loop (<2 min)
make wasm-canary

# Watch mode (rebuild on changes)
cd examples/wasm
ruchy serve --port 8000 --watch-wasm
```

### Pre-Release Checklist

- [ ] All 40 canary tests pass (5 browsers)
- [ ] Property tests pass (1000+ cases)
- [ ] Mutation score >90%
- [ ] Performance baselines met
- [ ] Zero panics/crashes
- [ ] Works offline
- [ ] Mobile-friendly
- [ ] Accessibility (WCAG 2.1 AA)
- [ ] Cross-browser compatible
- [ ] Documentation updated
- [ ] WOS integration tested
- [ ] interactive.paiml.com integration tested

---

## 8. Documentation Requirements

Every WASM feature MUST have:

1. **API Documentation** (rustdoc)
2. **Browser Demo** (examples/wasm/*)
3. **E2E Tests** (40+ canary tests)
4. **Performance Benchmarks**
5. **Integration Guide** (WOS + interactive.paiml.com)
6. **Troubleshooting Guide**

---

## 9. Success Metrics

### Phase 0 (Feasibility)
- [x] WASM builds successfully
- [ ] Loads in browser (<5s)
- [ ] Config analysis works
- [ ] Streaming I/O >10 MB/s
- [ ] No panics

### Phase 1 (Production Ready)
- [ ] All 40 canary tests pass
- [ ] Cross-browser compatible
- [ ] Performance baselines met
- [ ] Integrated with WOS
- [ ] Integrated with interactive.paiml.com
- [ ] Zero defects

---

## 10. Contact & Resources

**Questions?**
- WOS Testing: `/home/noahgift/src/wos/e2e/tests/canary/README.md`
- interactive.paiml.com: `/home/noahgift/src/interactive.paiml.com/tests/wasm/`
- SQLite Testing: https://sqlite.org/testing.html

**Issues?**
- Check `PHASE0-RESULTS.md` for current status
- Run `make wasm-canary-debug` for detailed output
- Review browser console for WASM errors

---

*NASA-level quality for mission-critical shell script analysis* 🚀
