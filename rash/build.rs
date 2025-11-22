//! Build script for bashrs crate.
//!
//! This build script configures cargo to recognize the `kani` cfg for verification.

fn main() {
    // Allow kani cfg for verification
    println!("cargo::rustc-check-cfg=cfg(kani)");
}
