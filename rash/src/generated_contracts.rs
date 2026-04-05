// Auto-generated contract assertions — bashrs-only subset (9 macros).

/// Preconditions for equation `parse_playbook`.
/// Domain-specific. Call: `contract_pre_parse_playbook!(slice_expr)`
macro_rules! contract_pre_parse_playbook {
    () => {{}};
    ($input:expr) => {{
        let input = &*(&$input);
        debug_assert!(
            input.len() > 0,
            "Contract parse_playbook: precondition violated — input.len() > 0"
        );
    }};
}

/// Preconditions for equation `configuration`.
/// Domain-specific. Call: `contract_pre_configuration!(slice_expr)`
macro_rules! contract_pre_configuration {
    () => {{}};
    ($input:expr) => {{
        // ShellIR/config types are structs (non-ZST by construction); binding proves liveness.
        let _contract_val = &$input;
    }};
}

/// Preconditions for equation `roundtrip`.
/// Call at function entry: `contract_pre_roundtrip!(input_expr)`
macro_rules! contract_pre_roundtrip {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
        debug_assert!(
            !_contract_input.is_empty(),
            "Contract roundtrip: precondition violated — !input.is_empty()"
        );
    }};
}

/// Preconditions for equation `roundtrip`.
/// Domain-specific. Call: `contract_pre_roundtrip!(slice_expr)`
macro_rules! contract_pre_roundtrip {
    () => {{}};
    ($input:expr) => {{
        let input = &*(&$input);
        debug_assert!(
            input.len() > 0,
            "Contract roundtrip: precondition violated — input.len() > 0"
        );
    }};
}

/// Preconditions for equation `roundtrip`.
/// Domain-specific. Call: `contract_pre_roundtrip!(slice_expr)`
macro_rules! contract_pre_roundtrip {
    () => {{}};
    ($input:expr) => {{
        let input = &*(&$input);
        debug_assert!(
            input.len() > 0,
            "Contract roundtrip: precondition violated — input.len() > 0"
        );
    }};
}

/// Preconditions for equation `lex`.
/// Domain-specific. Call: `contract_pre_lex!(slice_expr)`
macro_rules! contract_pre_lex {
    () => {{}};
    ($input:expr) => {{
        let input = &*(&$input);
        debug_assert!(
            input.len() > 0,
            "Contract lex: precondition violated — input.len() > 0"
        );
    }};
}

/// Preconditions for equation `parse`.
/// Domain-specific. Call: `contract_pre_parse!(slice_expr)`
macro_rules! contract_pre_parse {
    () => {{}};
    ($input:expr) => {{
        let input = &*(&$input);
        debug_assert!(
            input.len() > 0,
            "Contract parse: precondition violated — input.len() > 0"
        );
    }};
}

/// Preconditions for equation `semantic_analyze`.
/// Domain-specific. Call: `contract_pre_semantic_analyze!(slice_expr)`
macro_rules! contract_pre_semantic_analyze {
    () => {{}};
    ($input:expr) => {{
        // BashAst is a struct (non-ZST by construction); binding proves liveness.
        let _contract_val = &$input;
    }};
}

/// Preconditions for equation `classify_filesystem`.
/// Domain-specific. Call: `contract_pre_classify_filesystem!(slice_expr)`
macro_rules! contract_pre_classify_filesystem {
    () => {{}};
    ($input:expr) => {{
        let source = &$input;
        debug_assert!(
            !source.is_empty(),
            "Contract classify_filesystem: precondition violated — !source.is_empty()"
        );
        debug_assert!(
            source.len() <= 1_000_000,
            "Contract classify_filesystem: precondition violated — source.len() <= 1_000_000"
        );
    }};
}

/// Preconditions for equation `classify_injection`.
/// Domain-specific. Call: `contract_pre_classify_injection!(slice_expr)`
macro_rules! contract_pre_classify_injection {
    () => {{}};
    ($input:expr) => {{
        let source = &$input;
        debug_assert!(
            !source.is_empty(),
            "Contract classify_injection: precondition violated — !source.is_empty()"
        );
        debug_assert!(
            source.len() <= 1_000_000,
            "Contract classify_injection: precondition violated — source.len() <= 1_000_000"
        );
    }};
}

/// Preconditions for equation `classify_secrets`.
/// Domain-specific. Call: `contract_pre_classify_secrets!(slice_expr)`
macro_rules! contract_pre_classify_secrets {
    () => {{}};
    ($input:expr) => {{
        let source = &$input;
        debug_assert!(
            !source.is_empty(),
            "Contract classify_secrets: precondition violated — !source.is_empty()"
        );
        debug_assert!(
            source.len() <= 1_000_000,
            "Contract classify_secrets: precondition violated — source.len() <= 1_000_000"
        );
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
