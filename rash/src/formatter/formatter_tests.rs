#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for formatter/contract.rs and formatter/dialect.rs
//! Targets uncovered branches in ContractSystem, TypeInferenceEngine,
//! unification, DialectScorer, ShellDialect inference, and compatibility.

use super::*;

// ═══════════════════════════════════════════════════════════════════════════════
// contract.rs coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_COV_FMT_001_contract_system_builtins() {
    let system = ContractSystem::default();
    assert!(system.get_variable_type("x").is_none());

    let mut sys = ContractSystem::new();

    // echo param -> Array(String)
    let _t = sys.infer_variable_type(
        "echo_arg",
        &TypeContext::FunctionCall {
            function: "echo".to_string(),
            param_index: 0,
        },
    );
    sys.solve_constraints().unwrap();
    assert!(matches!(
        sys.get_variable_type("echo_arg").unwrap(),
        ShellType::Array(_)
    ));

    // test param -> String
    let mut sys2 = ContractSystem::new();
    let _t = sys2.infer_variable_type(
        "test_expr",
        &TypeContext::FunctionCall {
            function: "test".to_string(),
            param_index: 0,
        },
    );
    sys2.solve_constraints().unwrap();
    assert_eq!(
        *sys2.get_variable_type("test_expr").unwrap(),
        ShellType::String
    );

    // read param -> Array(String)
    let mut sys3 = ContractSystem::new();
    let _t = sys3.infer_variable_type(
        "read_var",
        &TypeContext::FunctionCall {
            function: "read".to_string(),
            param_index: 0,
        },
    );
    sys3.solve_constraints().unwrap();
    assert!(matches!(
        sys3.get_variable_type("read_var").unwrap(),
        ShellType::Array(_)
    ));
}

#[test]
fn test_COV_FMT_002_infer_existing_and_edge_cases() {
    let mut system = ContractSystem::new();

    // Infer + solve -> then re-infer returns cached
    let _t = system.infer_variable_type(
        "counter",
        &TypeContext::Assignment {
            value_type: ShellType::Integer,
        },
    );
    system.solve_constraints().unwrap();
    let t2 = system.infer_variable_type("counter", &TypeContext::Arithmetic);
    assert_eq!(t2, ShellType::Integer);

    // Nonexistent function -> type var, no constraint
    let mut sys2 = ContractSystem::new();
    let t = sys2.infer_variable_type(
        "arg",
        &TypeContext::FunctionCall {
            function: "no_such".to_string(),
            param_index: 0,
        },
    );
    assert!(matches!(t, ShellType::TypeVar(_)));
    sys2.solve_constraints().unwrap();

    // Out-of-bounds param index
    let mut sys3 = ContractSystem::new();
    let t = sys3.infer_variable_type(
        "extra",
        &TypeContext::FunctionCall {
            function: "echo".to_string(),
            param_index: 5,
        },
    );
    assert!(matches!(t, ShellType::TypeVar(_)));
    sys3.solve_constraints().unwrap();
}

#[test]
fn test_COV_FMT_003_unification_variants() {
    // Array type
    let mut sys = ContractSystem::new();
    let _t = sys.infer_variable_type(
        "list",
        &TypeContext::Assignment {
            value_type: ShellType::Array(Box::new(ShellType::String)),
        },
    );
    sys.solve_constraints().unwrap();
    assert_eq!(
        *sys.get_variable_type("list").unwrap(),
        ShellType::Array(Box::new(ShellType::String))
    );

    // AssocArray type
    let mut sys2 = ContractSystem::new();
    let assoc = ShellType::AssocArray {
        key: Box::new(ShellType::String),
        value: Box::new(ShellType::Integer),
    };
    let _t = sys2.infer_variable_type(
        "map",
        &TypeContext::Assignment {
            value_type: assoc.clone(),
        },
    );
    sys2.solve_constraints().unwrap();
    assert_eq!(*sys2.get_variable_type("map").unwrap(), assoc);

    // Union type
    let mut sys3 = ContractSystem::new();
    let union = ShellType::Union(vec![ShellType::String, ShellType::Integer]);
    let _t = sys3.infer_variable_type(
        "mixed",
        &TypeContext::Assignment {
            value_type: union.clone(),
        },
    );
    sys3.solve_constraints().unwrap();
    assert_eq!(*sys3.get_variable_type("mixed").unwrap(), union);
}

#[test]
fn test_COV_FMT_004_validate_contracts() {
    // Type contract passes
    let mut sys = ContractSystem::new();
    let _t = sys.infer_variable_type(
        "x",
        &TypeContext::Assignment {
            value_type: ShellType::String,
        },
    );
    sys.solve_constraints().unwrap();
    sys.add_contract(Contract {
        kind: ContractKind::TypeAnnotation,
        condition: ContractCondition::TypeConstraint {
            var: "x".to_string(),
            expected_type: ShellType::String,
        },
        description: "x is string".to_string(),
        location: Span::new(BytePos(0), BytePos(5)),
    });
    assert!(sys.validate_contracts().is_empty());

    // NonNull passes when variable defined
    let mut sys2 = ContractSystem::new();
    let _t = sys2.infer_variable_type(
        "dv",
        &TypeContext::Assignment {
            value_type: ShellType::Boolean,
        },
    );
    sys2.solve_constraints().unwrap();
    sys2.add_contract(Contract {
        kind: ContractKind::Precondition,
        condition: ContractCondition::NonNull {
            var: "dv".to_string(),
        },
        description: "must exist".to_string(),
        location: Span::new(BytePos(0), BytePos(10)),
    });
    assert!(sys2.validate_contracts().is_empty());
}

#[test]
fn test_COV_FMT_005_validate_passthrough_conditions() {
    let mut sys = ContractSystem::new();

    // Range, FS, Custom, Or, Not all pass through
    for cond in [
        ContractCondition::RangeConstraint {
            var: "c".to_string(),
            min: Some(0),
            max: Some(100),
        },
        ContractCondition::FileSystemConstraint {
            path: "/etc/hosts".to_string(),
            constraint: FsConstraint::Exists,
        },
        ContractCondition::CustomPredicate {
            expression: "test -d /tmp".to_string(),
        },
        ContractCondition::Or(
            Box::new(ContractCondition::NonNull {
                var: "a".to_string(),
            }),
            Box::new(ContractCondition::NonNull {
                var: "b".to_string(),
            }),
        ),
        ContractCondition::Not(Box::new(ContractCondition::CustomPredicate {
            expression: "false".to_string(),
        })),
    ] {
        sys.add_contract(Contract {
            kind: ContractKind::Invariant,
            condition: cond,
            description: "pass".to_string(),
            location: Span::new(BytePos(0), BytePos(10)),
        });
    }
    assert!(sys.validate_contracts().is_empty());
}

#[test]
fn test_COV_FMT_006_register_function_and_infer() {
    let mut sys = ContractSystem::new();
    sys.register_function(FunctionSignature {
        name: "my_func".to_string(),
        parameters: vec![
            Parameter {
                name: "input".to_string(),
                param_type: ShellType::String,
                is_optional: false,
            },
            Parameter {
                name: "count".to_string(),
                param_type: ShellType::Integer,
                is_optional: true,
            },
        ],
        return_type: ShellType::ExitCode,
        preconditions: vec![],
        postconditions: vec![],
    });
    let _t = sys.infer_variable_type(
        "c",
        &TypeContext::FunctionCall {
            function: "my_func".to_string(),
            param_index: 1,
        },
    );
    sys.solve_constraints().unwrap();
    assert_eq!(*sys.get_variable_type("c").unwrap(), ShellType::Integer);
}

#[test]
fn test_COV_FMT_007_type_context_and_fs_constraint() {
    let loc = TypeContext::Arithmetic.location();
    assert!(loc.is_empty());
    let loc2 = TypeContext::Assignment {
        value_type: ShellType::String,
    }
    .location();
    assert_eq!(loc2.start, BytePos(0));

    assert_eq!(FsConstraint::Exists, FsConstraint::Exists);
    assert_ne!(FsConstraint::IsReadable, FsConstraint::IsWritable);
    assert_ne!(FsConstraint::IsDirectory, FsConstraint::IsRegularFile);
    assert_ne!(FsConstraint::IsExecutable, FsConstraint::Exists);
}

// ═══════════════════════════════════════════════════════════════════════════════
// dialect.rs coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_COV_FMT_050_dialect_display_names() {
    assert_eq!(ShellDialect::Posix.display_name(), "POSIX");
    assert_eq!(ShellDialect::Bash5_2.display_name(), "Bash 5.2");
    assert_eq!(ShellDialect::Ksh93uPlus.display_name(), "KornShell 93u+");
    assert_eq!(ShellDialect::Zsh5_9.display_name(), "Z shell 5.9");
    assert_eq!(ShellDialect::Dash0_5_12.display_name(), "Dash 0.5.12");

    let inferred = ShellDialect::Inferred(Box::new(InferenceConfidence {
        dialect: Box::new(ShellDialect::Bash5_2),
        confidence: 0.9,
        evidence: InferenceEvidence::Shebang("bash"),
    }));
    assert_eq!(inferred.display_name(), "Bash 5.2");
}

#[test]
fn test_COV_FMT_051_shebang_inference() {
    for (script, expected) in [
        ("#!/bin/ksh\necho hi", ShellDialect::Ksh93uPlus),
        ("#!/bin/dash\necho hi", ShellDialect::Dash0_5_12),
        ("#!/usr/bin/env bash\necho hi", ShellDialect::Bash5_2),
        ("echo hello", ShellDialect::Posix),
    ] {
        let c = ShellDialect::infer(script.as_bytes());
        assert_eq!(*c.dialect, expected, "Failed for: {}", script);
    }
}

#[test]
fn test_COV_FMT_052_feature_support() {
    // Bash supports all bash features
    for f in [
        SyntaxFeature::BashArrays,
        SyntaxFeature::BashProcessSubst,
        SyntaxFeature::BashConditionals,
        SyntaxFeature::BashArithmetic,
    ] {
        assert!(ShellDialect::Bash5_2.supports_feature(f));
    }

    // POSIX only supports PosixFunctions
    assert!(ShellDialect::Posix.supports_feature(SyntaxFeature::PosixFunctions));
    assert!(!ShellDialect::Posix.supports_feature(SyntaxFeature::BashArrays));
    assert!(!ShellDialect::Posix.supports_feature(SyntaxFeature::ZshGlobs));

    // Ksh
    assert!(ShellDialect::Ksh93uPlus.supports_feature(SyntaxFeature::KshFunctions));
    assert!(!ShellDialect::Ksh93uPlus.supports_feature(SyntaxFeature::BashArrays));

    // Dash only posix
    assert!(!ShellDialect::Dash0_5_12.supports_feature(SyntaxFeature::ZshGlobs));

    // Zsh globs
    assert!(ShellDialect::Zsh5_9.supports_feature(SyntaxFeature::ZshGlobs));
}

#[test]
fn test_COV_FMT_053_dialect_inference_scripts() {
    let zsh = "#!/bin/zsh\nzparseopts -D -- h=help\nfor f in **/*.txt; do echo $f; done";
    assert_eq!(
        *ShellDialect::infer(zsh.as_bytes()).dialect,
        ShellDialect::Zsh5_9
    );

    let ksh = "#!/bin/ksh\nfunction setup {\n  typeset -i count=0\n}";
    assert_eq!(
        *ShellDialect::infer(ksh.as_bytes()).dialect,
        ShellDialect::Ksh93uPlus
    );

    let dash = "#!/bin/dash\nlocal var=hello";
    assert_eq!(
        *ShellDialect::infer(dash.as_bytes()).dialect,
        ShellDialect::Dash0_5_12
    );

    let bash_no_shebang = "array=(a b c)\nif [[ $x == y ]]; then echo ok; fi";
    assert_eq!(
        *ShellDialect::infer(bash_no_shebang.as_bytes()).dialect,
        ShellDialect::Bash5_2
    );

    assert_eq!(*ShellDialect::infer(b"").dialect, ShellDialect::Posix);

    let proc_sub = "diff <(sort f1) <(sort f2)";
    assert_eq!(
        *ShellDialect::infer(proc_sub.as_bytes()).dialect,
        ShellDialect::Bash5_2
    );
}

#[test]
fn test_COV_FMT_054_dialect_scorer_all_evidence_types() {
    for (evidence, weight, expected) in [
        (InferenceEvidence::Shebang("sh"), 0.7, ShellDialect::Posix),
        (
            InferenceEvidence::Shebang("ksh"),
            0.7,
            ShellDialect::Ksh93uPlus,
        ),
        (
            InferenceEvidence::Shebang("dash"),
            0.7,
            ShellDialect::Dash0_5_12,
        ),
        (InferenceEvidence::Shebang("zsh"), 0.7, ShellDialect::Zsh5_9),
        (
            InferenceEvidence::Shebang("bash"),
            0.7,
            ShellDialect::Bash5_2,
        ),
        (
            InferenceEvidence::Syntax(SyntaxFeature::ZshGlobs),
            0.5,
            ShellDialect::Zsh5_9,
        ),
        (
            InferenceEvidence::Builtins(BuiltinProfile::DashLocal),
            0.5,
            ShellDialect::Dash0_5_12,
        ),
        (
            InferenceEvidence::Builtins(BuiltinProfile::KshTypeset),
            0.5,
            ShellDialect::Ksh93uPlus,
        ),
        (
            InferenceEvidence::Builtins(BuiltinProfile::ZshZparseopts),
            0.5,
            ShellDialect::Zsh5_9,
        ),
    ] {
        let mut scorer = DialectScorer::new();
        scorer.add_evidence(evidence, weight);
        assert_eq!(*scorer.compute_confidence().dialect, expected);
    }

    // Default scorer -> POSIX
    let ds = DialectScorer::default();
    let c = ds.compute_confidence();
    assert_eq!(*c.dialect, ShellDialect::Posix);
    assert!((c.confidence - 0.1).abs() < f32::EPSILON);

    // Multiple evidence
    let mut ms = DialectScorer::new();
    ms.add_evidence(InferenceEvidence::Shebang("bash"), 0.7);
    ms.add_evidence(InferenceEvidence::Syntax(SyntaxFeature::BashArrays), 0.2);
    ms.add_evidence(
        InferenceEvidence::Builtins(BuiltinProfile::BashReadarray),
        0.1,
    );
    let mc = ms.compute_confidence();
    assert_eq!(*mc.dialect, ShellDialect::Bash5_2);
    assert!(mc.confidence > 0.8);
}

#[test]
fn test_COV_FMT_055_core_dialect_conversion() {
    assert_eq!(CoreDialect::Posix.to_shell_dialect(), ShellDialect::Posix);
    assert_eq!(
        CoreDialect::Bash5_2.to_shell_dialect(),
        ShellDialect::Bash5_2
    );
    assert_eq!(
        CoreDialect::Ksh93uPlus.to_shell_dialect(),
        ShellDialect::Ksh93uPlus
    );
    assert_eq!(CoreDialect::Zsh5_9.to_shell_dialect(), ShellDialect::Zsh5_9);
    assert_eq!(
        CoreDialect::Dash0_5_12.to_shell_dialect(),
        ShellDialect::Dash0_5_12
    );
}

#[test]
fn test_COV_FMT_056_compatibility() {
    // PosixFunctions compatible everywhere
    assert_eq!(
        check_compatibility(
            ShellDialect::Bash5_2,
            ShellDialect::Posix,
            SyntaxFeature::PosixFunctions
        ),
        Compatibility::Direct
    );
    assert_eq!(
        check_compatibility(
            ShellDialect::Bash5_2,
            ShellDialect::Dash0_5_12,
            SyntaxFeature::PosixFunctions
        ),
        Compatibility::Direct
    );

    // ZshGlobs incompatible with non-zsh
    assert_eq!(
        check_compatibility(
            ShellDialect::Zsh5_9,
            ShellDialect::Bash5_2,
            SyntaxFeature::ZshGlobs
        ),
        Compatibility::Incompatible
    );

    // Ksh to Ksh
    assert_eq!(
        check_compatibility(
            ShellDialect::Ksh93uPlus,
            ShellDialect::Ksh93uPlus,
            SyntaxFeature::KshFunctions
        ),
        Compatibility::Direct
    );
}
