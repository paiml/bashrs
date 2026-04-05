
#[test]
fn test_COV_POSIX_055_test_logical_and_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_056_test_logical_or_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(ShellValue::Bool(true)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_057_test_logical_and_runtime() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::Variable("b".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("&&"));
}

// ---------------------------------------------------------------------------
// Selective runtime emission
// ---------------------------------------------------------------------------

fn make_runtime_call_ir(func: &str) -> ShellIR {
    ShellIR::Exec {
        cmd: Command {
            program: func.to_string(),
            args: vec![ShellValue::Variable("x".to_string())],
        },
        effects: EffectSet::pure(),
    }
}

#[test]
fn test_COV_POSIX_058_runtime_rash_print() {
    let ir = make_runtime_call_ir("rash_print");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_print()"));
}

#[test]
fn test_COV_POSIX_059_runtime_rash_eprintln() {
    let ir = make_runtime_call_ir("rash_eprintln");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_eprintln()"));
}

#[test]
fn test_COV_POSIX_060_runtime_rash_require() {
    let ir = make_runtime_call_ir("rash_require");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_require()"));
}

#[test]
fn test_COV_POSIX_061_runtime_rash_download_verified() {
    let ir = make_runtime_call_ir("rash_download_verified");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_download_verified()"));
}

#[test]
fn test_COV_POSIX_062_runtime_rash_string_trim() {
    let ir = make_runtime_call_ir("rash_string_trim");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_trim()"));
}

#[test]
fn test_COV_POSIX_063_runtime_rash_string_contains() {
    let ir = make_runtime_call_ir("rash_string_contains");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_contains()"));
}

#[test]
fn test_COV_POSIX_064_runtime_rash_string_len() {
    let ir = make_runtime_call_ir("rash_string_len");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_len()"));
}

#[test]
fn test_COV_POSIX_065_runtime_rash_string_replace() {
    let ir = make_runtime_call_ir("rash_string_replace");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_replace()"));
}

#[test]
fn test_COV_POSIX_066_runtime_rash_string_to_upper() {
    let ir = make_runtime_call_ir("rash_string_to_upper");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_to_upper()"));
}

#[test]
fn test_COV_POSIX_067_runtime_rash_string_to_lower() {
    let ir = make_runtime_call_ir("rash_string_to_lower");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_to_lower()"));
}

#[test]
fn test_COV_POSIX_068_runtime_rash_fs_exists() {
    let ir = make_runtime_call_ir("rash_fs_exists");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_exists()"));
}

#[test]
fn test_COV_POSIX_069_runtime_rash_fs_read_file() {
    let ir = make_runtime_call_ir("rash_fs_read_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_read_file()"));
}

#[test]
fn test_COV_POSIX_070_runtime_rash_fs_write_file() {
    let ir = make_runtime_call_ir("rash_fs_write_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_write_file()"));
}

#[test]
fn test_COV_POSIX_071_runtime_rash_fs_copy() {
    let ir = make_runtime_call_ir("rash_fs_copy");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_copy()"));
}

#[test]
fn test_COV_POSIX_072_runtime_rash_fs_remove() {
    let ir = make_runtime_call_ir("rash_fs_remove");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_remove()"));
}

#[test]
fn test_COV_POSIX_073_runtime_rash_fs_is_file() {
    let ir = make_runtime_call_ir("rash_fs_is_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_is_file()"));
}

#[test]
fn test_COV_POSIX_074_runtime_rash_fs_is_dir() {
    let ir = make_runtime_call_ir("rash_fs_is_dir");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_is_dir()"));
}

#[test]
fn test_COV_POSIX_075_runtime_rash_string_split() {
    let ir = make_runtime_call_ir("rash_string_split");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_split()"));
}

#[test]
fn test_COV_POSIX_076_runtime_rash_array_len() {
    let ir = make_runtime_call_ir("rash_array_len");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_array_len()"));
}

#[test]
fn test_COV_POSIX_077_runtime_rash_array_join() {
    let ir = make_runtime_call_ir("rash_array_join");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_array_join()"));
}

// ---------------------------------------------------------------------------
// separate_functions with non-Sequence IR
// ---------------------------------------------------------------------------

include!("posix_coverage_tests_COV_COV_COV_COV_COV.rs");
