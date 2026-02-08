//! Decision tracing for Tarantula fault localization (§11.10.1).
//!
//! Records emitter decisions during transpilation so they can be fed into
//! the SBFL module to rank which decisions correlate with corpus failures.

use serde::{Deserialize, Serialize};

/// A single decision made by the emitter during transpilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspilerDecision {
    /// Category of decision (e.g., "ir_dispatch", "assignment_value")
    pub decision_type: String,
    /// What was chosen (e.g., "Let", "single_quote", "seq_range")
    pub choice: String,
    /// IR node context (e.g., "Let", "If", "For")
    pub ir_node: String,
}

/// Ordered trace of decisions from a single transpilation run.
pub type DecisionTrace = Vec<TranspilerDecision>;
