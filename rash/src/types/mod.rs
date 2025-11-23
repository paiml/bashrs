//! Type system for bashrs
//!
//! This module provides type checking and taint tracking for injection safety.

pub mod taint;

pub use taint::{Taint, Type, TypeChecker};
