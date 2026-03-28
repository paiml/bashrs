//! Formal verification specifications for the bashrs workspace.
//!
//! This crate provides design-by-contract specifications using Verus-style
//! pre/postconditions and Kani-style bounded model checking harnesses.

// Contract assertions from YAML (pv codegen)
#[macro_use]
#[allow(unused_macros)]
mod generated_contracts;
pub mod verification_specs;
