//! Streaming I/O for WASM
//!
//! ## Phase 0: Critical Feasibility Test
//!
//! This module tests the feasibility of streaming output from WASM to JavaScript.
//! Shell scripts produce streaming output (think `tail -f`, pipes, redirects), but
//! WASM has limited I/O capabilities.
//!
//! ## The Problem
//!
//! ```bash
//! # This streams output line-by-line
//! cat large_file.txt | grep pattern
//! ```
//!
//! In WASM, we can't:
//! - Block on I/O
//! - Use native file descriptors
//! - Spawn processes
//!
//! ## The Solution (Being Tested)
//!
//! Use JavaScript callbacks to stream chunks:
//!
//! ```js
//! streamOutput(bashScript, (chunk) => {
//!     console.log(chunk);  // Process each chunk as it arrives
//! });
//! ```
//!
//! ## Performance Requirements
//!
//! - Callback latency: <1ms per chunk
//! - Throughput: >10MB/s
//! - Memory: <10MB for 100MB total output
//!
//! If these aren't achievable, WASM implementation is not feasible.

use js_sys::Function;
use wasm_bindgen::prelude::*;

/// Callback type for streaming output
///
/// JavaScript function signature: `(chunk: string) => void`
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(chunk: string) => void")]
    pub type OutputCallback;
}

/// Stream output to JavaScript callback
///
/// This function tests the feasibility of streaming by:
/// 1. Breaking input into chunks
/// 2. Calling JavaScript callback for each chunk
/// 3. Measuring performance
///
/// # Example (JavaScript)
///
/// ```js
/// import { streamOutput } from 'bashrs.wasm';
///
/// let received = "";
/// const startTime = performance.now();
///
/// await streamOutput(
///     "Large text content...",
///     1024,  // 1KB chunks
///     (chunk) => {
///         received += chunk;
///         console.log(`Got chunk: ${chunk.length} bytes`);
///     }
/// );
///
/// const endTime = performance.now();
/// console.log(`Streamed in ${endTime - startTime}ms`);
/// ```
#[wasm_bindgen]
pub fn stream_output(
    content: &str,
    chunk_size: usize,
    callback: &Function,
) -> Result<StreamStats, JsValue> {
    let start = js_sys::Date::now();
    let mut chunks_sent = 0;
    let mut bytes_sent = 0;

    // Split content into chunks and stream via callback
    for chunk in content.as_bytes().chunks(chunk_size) {
        let chunk_str = String::from_utf8_lossy(chunk);
        let js_chunk = JsValue::from_str(&chunk_str);

        // Call JavaScript callback with chunk
        callback.call1(&JsValue::NULL, &js_chunk)?;

        chunks_sent += 1;
        bytes_sent += chunk.len();
    }

    let end = js_sys::Date::now();
    let duration_ms = end - start;

    Ok(StreamStats {
        chunks_sent,
        bytes_sent,
        duration_ms,
        throughput_mbps: (bytes_sent as f64 / 1_000_000.0) / (duration_ms / 1000.0),
    })
}

/// Statistics from streaming operation
#[wasm_bindgen]
pub struct StreamStats {
    chunks_sent: usize,
    bytes_sent: usize,
    duration_ms: f64,
    throughput_mbps: f64,
}

#[wasm_bindgen]
impl StreamStats {
    #[wasm_bindgen(getter)]
    pub fn chunks_sent(&self) -> usize {
        self.chunks_sent
    }

    #[wasm_bindgen(getter)]
    pub fn bytes_sent(&self) -> usize {
        self.bytes_sent
    }

    #[wasm_bindgen(getter)]
    pub fn duration_ms(&self) -> f64 {
        self.duration_ms
    }

    #[wasm_bindgen(getter)]
    pub fn throughput_mbps(&self) -> f64 {
        self.throughput_mbps
    }

    /// Check if performance meets requirements
    ///
    /// Requirements:
    /// - Throughput: >10 MB/s
    /// - Latency: <1ms per chunk (implied by >10MB/s)
    #[wasm_bindgen(getter)]
    pub fn meets_requirements(&self) -> bool {
        self.throughput_mbps >= 10.0
    }
}

/// Benchmark streaming performance
///
/// Tests different chunk sizes to find optimal configuration.
///
/// # Example (JavaScript)
///
/// ```js
/// const results = await benchmarkStreaming(
///     10 * 1024 * 1024,  // 10MB test
///     (chunk) => { } // process each chunk
/// );
///
/// console.log(`Best chunk size: ${results.optimal_chunk_size}`);
/// console.log(`Max throughput: ${results.max_throughput_mbps} MB/s`);
/// ```
#[wasm_bindgen]
pub fn benchmark_streaming(
    test_size_bytes: usize,
    callback: &Function,
) -> Result<BenchmarkResult, JsValue> {
    // Generate test data
    let test_data = "x".repeat(test_size_bytes);

    // Test different chunk sizes
    let chunk_sizes = vec![256, 512, 1024, 2048, 4096, 8192, 16384];
    let mut results = Vec::new();

    for chunk_size in chunk_sizes {
        let stats = stream_output(&test_data, chunk_size, callback)?;
        results.push((chunk_size, stats.throughput_mbps));
    }

    // Find optimal chunk size
    let (optimal_chunk_size, max_throughput_mbps) = results
        .iter()
        .max_by(|(_, a), (_, b)| {
            a.partial_cmp(b)
                .expect("throughput comparison failed (NaN detected)")
        })
        .map(|(size, throughput)| (*size, *throughput))
        .unwrap_or((1024, 0.0));

    Ok(BenchmarkResult {
        test_size_bytes,
        optimal_chunk_size,
        max_throughput_mbps,
        meets_requirements: max_throughput_mbps >= 10.0,
    })
}

/// Benchmark result
#[wasm_bindgen]
pub struct BenchmarkResult {
    test_size_bytes: usize,
    optimal_chunk_size: usize,
    max_throughput_mbps: f64,
    meets_requirements: bool,
}

#[wasm_bindgen]
impl BenchmarkResult {
    #[wasm_bindgen(getter)]
    pub fn test_size_bytes(&self) -> usize {
        self.test_size_bytes
    }

    #[wasm_bindgen(getter)]
    pub fn optimal_chunk_size(&self) -> usize {
        self.optimal_chunk_size
    }

    #[wasm_bindgen(getter)]
    pub fn max_throughput_mbps(&self) -> f64 {
        self.max_throughput_mbps
    }

    #[wasm_bindgen(getter)]
    pub fn meets_requirements(&self) -> bool {
        self.meets_requirements
    }
}
