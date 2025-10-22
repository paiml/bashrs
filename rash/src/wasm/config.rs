//! Config management for WASM
//!
//! Re-exports config functionality for WASM consumption

// Config modules are already implemented - just need to expose them properly
// The main work is in api.rs which calls the existing config::analyzer and config::purifier

pub use crate::config::*;
