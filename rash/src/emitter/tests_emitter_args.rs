/// RED TEST: args() should emit "$@" syntax
/// Tests that ShellValue::Arg { position: None } generates "$@" in shell
#[test]
fn test_args_emits_all_args_syntax() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "all".to_string(),
        value: ShellValue::Arg { position: None },
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement args() emission
    assert!(
        output.contains("\"$@\""),
        "args() should emit $@ with quotes, got: {}",
        output
    );
    assert!(
        output.contains("all=\"$@\""),
        "Should assign quoted all args to variable, got: {}",
        output
    );
}

/// RED TEST: arg_count() should emit "$#" syntax
/// Tests that ShellValue::ArgCount generates "$#" in shell
#[test]
fn test_arg_count_emits_count_syntax() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "count".to_string(),
        value: ShellValue::ArgCount,
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement ArgCount emission
    assert!(
        output.contains("\"$#\""),
        "arg_count() should emit $# with quotes, got: {}",
        output
    );
    assert!(
        output.contains("count=\"$#\""),
        "Should assign quoted arg count to variable, got: {}",
        output
    );
}

/// RED TEST: Arguments must be quoted for safety
/// Tests that all argument accesses include proper quoting
#[test]
fn test_args_quoted_for_safety() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Sequence(vec![
        crate::ir::shell_ir::ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::Arg { position: Some(1) },
            effects: crate::ir::effects::EffectSet::pure(),
        },
        crate::ir::shell_ir::ShellIR::Let {
            name: "y".to_string(),
            value: ShellValue::Arg { position: None },
            effects: crate::ir::effects::EffectSet::pure(),
        },
    ]);

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: Must have quotes around $1 and $@ for safety
    // Should NOT have unquoted versions like =$ 1 or =$1 (without quotes)
    // The proper form is ="$1" and ="$@"

    // Must have quoted versions
    assert!(
        output.contains("\"$1\""),
        "Should have quoted $1: {}",
        output
    );
    assert!(
        output.contains("\"$@\""),
        "Should have quoted $@: {}",
        output
    );
}

/// RED TEST: Multiple arg() calls in sequence
/// Tests that multiple positional arguments can be accessed together
#[test]
fn test_multiple_args_in_sequence() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Sequence(vec![
        crate::ir::shell_ir::ShellIR::Let {
            name: "first".to_string(),
            value: ShellValue::Arg { position: Some(1) },
            effects: crate::ir::effects::EffectSet::pure(),
        },
        crate::ir::shell_ir::ShellIR::Let {
            name: "second".to_string(),
            value: ShellValue::Arg { position: Some(2) },
            effects: crate::ir::effects::EffectSet::pure(),
        },
        crate::ir::shell_ir::ShellIR::Let {
            name: "count".to_string(),
            value: ShellValue::ArgCount,
            effects: crate::ir::effects::EffectSet::pure(),
        },
    ]);

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: All three should be emitted correctly
    assert!(
        output.contains("first=\"$1\""),
        "Should contain first=$1, got: {}",
        output
    );
    assert!(
        output.contains("second=\"$2\""),
        "Should contain second=$2, got: {}",
        output
    );
    assert!(
        output.contains("count=\"$#\""),
        "Should contain count=$#, got: {}",
        output
    );
}

// ============= Sprint 27c: Exit Code Handling - RED PHASE =============

/// RED TEST: exit_code() should emit "$?" syntax
/// Tests that ShellValue::ExitCode generates "$?" in shell
#[test]
fn test_exit_code_emits_question_mark_syntax() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "status".to_string(),
        value: ShellValue::ExitCode,
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement ExitCode emission
    assert!(
        output.contains("\"$?\""),
        "exit_code() should emit $? with quotes, got: {}",
        output
    );
    assert!(
        output.contains("status=\"$?\""),
        "Should assign quoted exit code to variable, got: {}",
        output
    );
}

/// RED TEST: exit_code() in comparison context
/// Tests that exit_code() works in if condition comparisons
#[test]
fn test_exit_code_in_comparison() {
    use crate::ir::shell_ir::ComparisonOp;
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::StrEq,
            left: Box::new(ShellValue::ExitCode),
            right: Box::new(ShellValue::String("0".to_string())),
        },
        then_branch: Box::new(crate::ir::shell_ir::ShellIR::Echo {
            value: ShellValue::String("success".to_string()),
        }),
        else_branch: None,
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until ExitCode is implemented in comparison
    assert!(
        output.contains("\"$?\""),
        "Should contain exit code in comparison, got: {}",
        output
    );
    assert!(
        output.contains("[ \"$?\" = "),
        "Should emit exit code comparison, got: {}",
        output
    );
}

/// RED TEST: Exit code must be quoted for safety
/// Tests that exit code accesses include proper quoting
#[test]
fn test_exit_code_quoted_for_safety() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Sequence(vec![
        crate::ir::shell_ir::ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::ExitCode,
            effects: crate::ir::effects::EffectSet::pure(),
        },
        crate::ir::shell_ir::ShellIR::Let {
            name: "y".to_string(),
            value: ShellValue::ExitCode,
            effects: crate::ir::effects::EffectSet::pure(),
        },
    ]);

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: Must have quotes around $? for safety and consistency
    assert!(
        output.contains("\"$?\""),
        "Exit code accesses must be quoted: {}",
        output
    );

    // Should appear twice (for both variables)
    let count = output.matches("\"$?\"").count();
    assert!(
        count >= 2,
        "Should have at least 2 quoted exit code accesses, found {}: {}",
        count,
        output
    );
}

/// RED TEST: exit_code() in concatenation
/// Tests that exit_code() can be used in string concatenation
#[test]
fn test_exit_code_in_concatenation() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("Exit code: ".to_string()),
            ShellValue::ExitCode,
        ]),
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until ExitCode works in concatenation
    assert!(
        output.contains("msg=\"Exit code: $?\""),
        "Should emit concatenated exit code, got: {}",
        output
    );
}

// ============= Sprint 28: Complete Missing Stdlib Functions - RED PHASE =============

/// RED TEST: string_split() should be in runtime
/// Tests that string_split generates shell function
#[test]
fn test_string_split_in_runtime() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    // Use IR that references rash_string_split to trigger selective emission
    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_string_split")
            .arg(crate::ir::shell_ir::ShellValue::String(
                "hello world".to_string(),
            ))
            .arg(crate::ir::shell_ir::ShellValue::String(" ".to_string())),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_string_split()"),
        "Runtime should include rash_string_split function, got: {}",
        output
    );
}

/// Tests that string_split uses POSIX tools
#[test]
fn test_string_split_basic() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_string_split")
            .arg(crate::ir::shell_ir::ShellValue::String("a,b,c".to_string()))
            .arg(crate::ir::shell_ir::ShellValue::String(",".to_string())),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_string_split()"),
        "Should have string_split function"
    );

    // Should use tr or similar for POSIX split
    assert!(
        output.contains("tr") || output.contains("sed"),
        "string_split should use POSIX tools for splitting"
    );
}

/// RED TEST: array_len() should be in runtime
/// Tests that array_len generates shell function
#[test]
fn test_array_len_in_runtime() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_array_len").arg(crate::ir::shell_ir::ShellValue::String(
            "myarray".to_string(),
        )),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_array_len()"),
        "Runtime should include rash_array_len function, got: {}",
        output
    );
}

/// Tests that array_len uses POSIX counting
#[test]
fn test_array_len_basic() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_array_len").arg(crate::ir::shell_ir::ShellValue::String(
            "myarray".to_string(),
        )),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_array_len()"),
        "Should have array_len function"
    );

    // Should use wc -l for counting lines
    assert!(
        output.contains("wc -l"),
        "array_len should use wc -l for counting"
    );
}

/// RED TEST: array_join() should be in runtime
/// Tests that array_join generates shell function
#[test]
fn test_array_join_in_runtime() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_array_join")
            .arg(crate::ir::shell_ir::ShellValue::String(
                "myarray".to_string(),
            ))
            .arg(crate::ir::shell_ir::ShellValue::String(",".to_string())),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_array_join()"),
        "Runtime should include rash_array_join function, got: {}",
        output
    );
}

/// Tests that array_join uses loop for joining
#[test]
fn test_array_join_basic() {
    use crate::ir::{Command, EffectSet};
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Exec {
        cmd: Command::new("rash_array_join")
            .arg(crate::ir::shell_ir::ShellValue::String(
                "myarray".to_string(),
            ))
            .arg(crate::ir::shell_ir::ShellValue::String(",".to_string())),
        effects: EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    assert!(
        output.contains("rash_array_join()"),
        "Should have array_join function"
    );

    // Should use while loop or similar for joining
    assert!(
        output.contains("while") || output.contains("for"),
        "array_join should use while/for loop for joining"
    );
}
