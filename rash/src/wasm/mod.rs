//! WebAssembly Support for bashrs
//!
//! Phase 0: Feasibility Study
//!
//! This module implements WASM bindings for bashrs, enabling:
//! - Browser-based shell script analysis
//! - Config linting without server round-trips
//! - Serverless function deployment
//! - Educational shell script tooling
//!
//! ## Architecture
//!
//! ```text
//! Browser/Node.js
//!     │
//!     ▼
//! JavaScript API (wasm_bindgen)
//!     │
//!     ▼
//! Rust WASM Module
//!     │
//!     ├─► Config Analyzer (CONFIG-001 to CONFIG-004)
//!     ├─► Bash Parser
//!     ├─► Linter
//!     └─► Virtual Filesystem (via JS callbacks)
//! ```
//!
//! ## Phase 0 Goals
//!
//! 1. **Streaming I/O**: Test JavaScript callback performance for streaming output
//! 2. **Config Analysis**: Port CONFIG-001 to CONFIG-004 to WASM
//! 3. **Performance**: Benchmark WASM vs native execution
//! 4. **Memory**: Validate memory usage stays <10MB
//! 5. **Decision**: Go/No-Go based on streaming I/O feasibility

#[cfg(feature = "wasm")]
pub mod api;

#[cfg(feature = "wasm")]
pub mod config;

#[cfg(feature = "wasm")]
pub mod streaming;

#[cfg(feature = "wasm")]
pub mod filesystem;

// Phase 1: Bash Runtime
#[cfg(feature = "wasm")]
pub mod io;

#[cfg(feature = "wasm")]
pub mod vfs;

#[cfg(feature = "wasm")]
pub mod builtins;

#[cfg(feature = "wasm")]
pub mod executor;

// Re-export main API when wasm feature is enabled
#[cfg(feature = "wasm")]
pub use api::*;
