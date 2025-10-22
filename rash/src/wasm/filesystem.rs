//! Virtual Filesystem for WASM
//!
//! WASM cannot access the real filesystem. This module provides a virtual
//! filesystem backed by JavaScript.
//!
//! ## Phase 0: Deferred
//!
//! This is not needed for initial config analysis (which works on in-memory strings),
//! but will be required for full bash script execution.
//!
//! ## Future Design
//!
//! ```js
//! // JavaScript side
//! const fs = {
//!     readFile: (path) => { /* ... */ },
//!     writeFile: (path, content) => { /* ... */ },
//!     exists: (path) => { /* ... */ }
//! };
//!
//! setFilesystemProvider(fs);
//! ```

use wasm_bindgen::prelude::*;

/// Virtual filesystem interface
///
/// Placeholder for Phase 1 implementation
#[wasm_bindgen]
pub struct VirtualFilesystem {
    // Will be implemented in Phase 1
}

#[wasm_bindgen]
impl VirtualFilesystem {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        VirtualFilesystem {}
    }
}
