//! Runtime library for bashrs.
//!
//! This crate provides the embedded shell runtime that contains core functions
//! and utilities included in all transpiled shell scripts.

include!(concat!(env!("OUT_DIR"), "/runtime.rs"));
