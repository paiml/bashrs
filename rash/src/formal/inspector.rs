//! Proof inspection and intermediate artifact generation
//!
//! This module provides tools for generating detailed proof artifacts,
//! annotated ASTs, and verification reports for inspection and debugging.

use crate::formal::semantics::{posix_semantics, rash_semantics};
use crate::formal::{AbstractState, FormalEmitter, TinyAst};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed verification report containing all intermediate proof artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    /// The original AST being verified
    pub ast: TinyAst,
    /// Generated POSIX shell code
    pub emitted_code: String,
    /// Initial state used for verification
    pub initial_state: AbstractState,
    /// Annotated AST with semantic information
    pub annotated_ast: AnnotatedAst,
    /// Step-by-step execution trace for rash semantics
    pub rash_trace: ExecutionTrace,
    /// Step-by-step execution trace for POSIX semantics
    pub posix_trace: ExecutionTrace,
    /// Final states comparison
    pub equivalence_analysis: EquivalenceAnalysis,
    /// Emitter justifications
    pub emitter_justifications: Vec<EmitterJustification>,
    /// Overall verification result
    pub verification_result: VerificationResult,
}

/// AST annotated with semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedAst {
    /// The AST node
    pub node: TinyAst,
    /// Pre-condition state (before execution)
    pub precondition: AbstractState,
    /// Post-condition state (after execution)
    pub postcondition: AbstractState,
    /// State transformation description
    pub transformation: StateTransformation,
    /// Child annotations for composite nodes
    pub children: Vec<AnnotatedAst>,
}

/// Description of how a state was transformed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransformation {
    /// Environment variable changes
    pub env_changes: HashMap<String, EnvChange>,
    /// Working directory change
    pub cwd_change: Option<CwdChange>,
    /// Filesystem changes
    pub fs_changes: Vec<FilesystemChange>,
    /// Output produced
    pub output_produced: Vec<String>,
    /// Errors produced
    pub errors_produced: Vec<String>,
    /// Exit code change
    pub exit_code_change: Option<i32>,
}

/// Environment variable change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvChange {
    Added {
        value: String,
    },
    Modified {
        old_value: String,
        new_value: String,
    },
    Removed {
        old_value: String,
    },
}

/// Working directory change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CwdChange {
    pub from: String,
    pub to: String,
}

/// Filesystem change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemChange {
    DirectoryCreated { path: String },
    FileCreated { path: String, content: String },
    ItemRemoved { path: String },
}

/// Step-by-step execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Initial state
    pub initial_state: AbstractState,
    /// Execution steps
    pub steps: Vec<ExecutionStep>,
    /// Final state
    pub final_state: AbstractState,
}

/// Single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step_number: usize,
    /// Description of the operation
    pub operation: String,
    /// State before this step
    pub state_before: AbstractState,
    /// State after this step
    pub state_after: AbstractState,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

/// Analysis of state equivalence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceAnalysis {
    /// Whether the states are equivalent
    pub are_equivalent: bool,
    /// Detailed comparison of environment variables
    pub env_comparison: EnvComparison,
    /// Working directory comparison
    pub cwd_comparison: CwdComparison,
    /// Filesystem comparison
    pub fs_comparison: FilesystemComparison,
    /// Output comparison
    pub output_comparison: OutputComparison,
    /// Exit code comparison
    pub exit_code_comparison: ExitCodeComparison,
}

/// Environment variables comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvComparison {
    pub matches: bool,
    pub rash_only: HashMap<String, String>,
    pub posix_only: HashMap<String, String>,
    pub different_values: HashMap<String, (String, String)>,
}

/// Working directory comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CwdComparison {
    pub matches: bool,
    pub rash_cwd: String,
    pub posix_cwd: String,
}

/// Filesystem comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemComparison {
    pub matches: bool,
    pub differences: Vec<String>,
}

/// Output comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputComparison {
    pub stdout_matches: bool,
    pub stderr_matches: bool,
    pub rash_stdout: Vec<String>,
    pub posix_stdout: Vec<String>,
    pub rash_stderr: Vec<String>,
    pub posix_stderr: Vec<String>,
}

/// Exit code comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeComparison {
    pub matches: bool,
    pub rash_exit_code: i32,
    pub posix_exit_code: i32,
}

/// Justification for emitter decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterJustification {
    /// The AST node being emitted
    pub ast_node: String,
    /// The generated POSIX code
    pub generated_code: String,
    /// Reasoning for the generation
    pub reasoning: String,
    /// Any special considerations
    pub considerations: Vec<String>,
}

/// Overall verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Success { confidence: f64 },
    Failure { reasons: Vec<String> },
    Partial { issues: Vec<String> },
}

/// Proof inspector for generating detailed verification artifacts
pub struct ProofInspector;

include!("inspector_incl2.rs");
