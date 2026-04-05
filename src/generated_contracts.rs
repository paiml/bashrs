// Auto-generated contract assertions from YAML — DO NOT EDIT.
// Zero cost in release builds (debug_assert!).
// Regenerate: pv codegen contracts/ -o src/generated_contracts.rs
// Include:   #[macro_use] #[allow(unused_macros)] mod generated_contracts;

// Auto-generated from contracts/cli-lint-v1.yaml — DO NOT EDIT
// Contract: cli-lint-v1

/// Preconditions for equation `exit_code_dispatch`.
/// Domain-specific. Call: `contract_pre_exit_code_dispatch!(slice_expr)`
macro_rules! contract_pre_exit_code_dispatch {
    () => {{}};
    ($input:expr) => {{
        let _pv_args = &$input;
        debug_assert!(_pv_args.len() >= 2,
            "Contract exit_code_dispatch: precondition violated — args.len() >= 2");
        debug_assert!(_pv_args[0] == "lint",
            "Contract exit_code_dispatch: precondition violated — args[0] == \"lint\"");
    }};
}

/// Preconditions for equation `finding_determinism`.
/// Call at function entry: `contract_pre_finding_determinism!(input_expr)`
macro_rules! contract_pre_finding_determinism {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
    }};
}

/// Preconditions for equation `output_format_validity`.
/// Call at function entry: `contract_pre_output_format_validity!(input_expr)`
macro_rules! contract_pre_output_format_validity {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
    }};
}

/// Preconditions for equation `severity_ordering`.
/// Domain-specific. Call: `contract_pre_severity_ordering!(slice_expr)`
macro_rules! contract_pre_severity_ordering {
    () => {{}};
    ($input:expr) => {{
        let _pv_diagnostics = &$input;
        debug_assert!(_pv_diagnostics.len() >= 0,
            "Contract severity_ordering: precondition violated — diagnostics.len() >= 0");
    }};
}

// Auto-generated from contracts/encoder-roundtrip-v1.yaml — DO NOT EDIT
// Contract: encoder-roundtrip-v1

/// Preconditions for equation `emit_posix`.
/// Call at function entry: `contract_pre_emit_posix!(input_expr)`
macro_rules! contract_pre_emit_posix {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
        debug_assert!(!_contract_input.is_empty(),
            "Contract emit_posix: precondition violated — !input.is_empty()");
    }};
}

/// Preconditions for equation `emit_purified`.
/// Call at function entry: `contract_pre_emit_purified!(input_expr)`
macro_rules! contract_pre_emit_purified {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
        debug_assert!(!_contract_input.is_empty(),
            "Contract emit_purified: precondition violated — !input.is_empty()");
    }};
}

/// Preconditions for equation `roundtrip`.
/// Call at function entry: `contract_pre_roundtrip!(input_expr)`
macro_rules! contract_pre_roundtrip {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
        debug_assert!(!_contract_input.is_empty(),
            "Contract roundtrip: precondition violated — !input.is_empty()");
    }};
}

// Auto-generated from contracts/parser-soundness-v1.yaml — DO NOT EDIT
// Contract: parser-soundness-v1

/// Preconditions for equation `lex`.
/// Domain-specific. Call: `contract_pre_lex!(slice_expr)`
macro_rules! contract_pre_lex {
    () => {{}};
    ($input:expr) => {{
        let _pv_input = &$input;
        debug_assert!(_pv_input.len() > 0,
            "Contract lex: precondition violated — input.len() > 0");
    }};
}

/// Preconditions for equation `parse`.
/// Domain-specific. Call: `contract_pre_parse!(slice_expr)`
macro_rules! contract_pre_parse {
    () => {{}};
    ($input:expr) => {{
        let _pv_input = &$input;
        debug_assert!(_pv_input.len() > 0,
            "Contract parse: precondition violated — input.len() > 0");
    }};
}

/// Preconditions for equation `semantic_analyze`.
/// Domain-specific. Call: `contract_pre_semantic_analyze!(slice_expr)`
macro_rules! contract_pre_semantic_analyze {
    () => {{}};
    ($input:expr) => {{
        let _pv_input = &$input;
        debug_assert!(_pv_input.len() > 0,
            "Contract semantic_analyze: precondition violated — input.len() > 0");
    }};
}

// Auto-generated from contracts/safety-classifier-v1.yaml — DO NOT EDIT
// Contract: safety-classifier-v1

/// Preconditions for equation `classify_filesystem`.
/// Domain-specific. Call: `contract_pre_classify_filesystem!(slice_expr)`
macro_rules! contract_pre_classify_filesystem {
    () => {{}};
    ($input:expr) => {{
        let _pv_source = &$input;
        debug_assert!(!_pv_source.is_empty(),
            "Contract classify_filesystem: precondition violated — !source.is_empty()");
        debug_assert!(_pv_source.len() <= 1_000_000,
            "Contract classify_filesystem: precondition violated — source.len() <= 1_000_000");
    }};
}

/// Preconditions for equation `classify_injection`.
/// Domain-specific. Call: `contract_pre_classify_injection!(slice_expr)`
macro_rules! contract_pre_classify_injection {
    () => {{}};
    ($input:expr) => {{
        let _pv_source = &$input;
        debug_assert!(!_pv_source.is_empty(),
            "Contract classify_injection: precondition violated — !source.is_empty()");
        debug_assert!(_pv_source.len() <= 1_000_000,
            "Contract classify_injection: precondition violated — source.len() <= 1_000_000");
    }};
}

/// Preconditions for equation `classify_secrets`.
/// Domain-specific. Call: `contract_pre_classify_secrets!(slice_expr)`
macro_rules! contract_pre_classify_secrets {
    () => {{}};
    ($input:expr) => {{
        let _pv_source = &$input;
        debug_assert!(!_pv_source.is_empty(),
            "Contract classify_secrets: precondition violated — !source.is_empty()");
        debug_assert!(_pv_source.len() <= 1_000_000,
            "Contract classify_secrets: precondition violated — source.len() <= 1_000_000");
    }};
}

/// Preconditions for equation `lint_shell`.
/// Call at function entry: `contract_pre_lint_shell!(input_expr)`
macro_rules! contract_pre_lint_shell {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
    }};
}

// Total: 15 preconditions, 0 postconditions from 4 contracts

// --- Manual stubs for verification_specs.rs (not yet in YAML) ---

/// Preconditions for `configuration` validation.
/// Stub — no domain assertions yet; add to YAML when invariants are specified.
macro_rules! contract_pre_configuration {
    () => {{}};
    ($input:expr) => {{ let _ = &$input; }};
}

/// Postconditions for `configuration` validation.
/// Stub — no domain assertions yet.
macro_rules! contract_post_configuration {
    () => {{}};
    ($output:expr) => {{ let _ = &$output; }};
}

/// Preconditions for `serialize_roundtrip`.
/// Stub — delegates to byte-slice liveness check.
macro_rules! contract_pre_serialize_roundtrip {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
        debug_assert!(!_contract_input.is_empty(),
            "Contract serialize_roundtrip: precondition violated — !input.is_empty()");
    }};
}
