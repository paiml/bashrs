//! Formal verification module for the rash emitter
//!
//! This module implements formal verification of the emitter's correctness
//! for a tiny subset of the rash AST, proving semantic equivalence between
//! rash AST nodes and their emitted POSIX shell commands.

pub mod abstract_state;
pub mod emitter;
pub mod inspector;
pub mod semantics;
pub mod tiny_ast;

#[cfg(test)]
pub mod proofs;

#[cfg(kani)]
pub mod kani_harnesses;

pub use abstract_state::*;
pub use emitter::*;
pub use inspector::*;
pub use semantics::*;
pub use tiny_ast::*;
