fn test_reproducible_builds_random_in_variable() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "RAND_SEED".into(),
            value: "$$RANDOM".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectRandom { .. })),
        "Should detect $$RANDOM"
    );
}

#[test]
fn test_reproducible_builds_process_id() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "TMP_FILE".into(),
            value: "/tmp/build_$$$$".into(),
            flavor: crate::make_parser::ast::VarFlavor::Recursive,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectProcessId { .. })),
        "Should detect $$$$ (process ID)"
    );
}

#[test]
fn test_reproducible_builds_hostname() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "HOST".into(),
            value: "$(shell hostname)".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect hostname"
    );
}

#[test]
fn test_reproducible_builds_git_log_timestamp() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "GIT_DATE".into(),
            value: "$(shell git log -1 --format=%cd --date=short)".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect git log timestamp"
    );
}

#[test]
fn test_reproducible_builds_mktemp_in_recipe() {
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".into(),
            prerequisites: vec![],
            recipe: vec!["mktemp -d".into(), "echo building".into()],
            phony: false,
            recipe_metadata: None,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result
            .iter()
            .any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect mktemp in recipe"
    );
}

#[test]
fn test_reproducible_builds_clean_makefile() {
    let ast = MakeAst {
        items: vec![
            MakeItem::Variable {
                name: "CC".into(),
                value: "gcc".into(),
                flavor: crate::make_parser::ast::VarFlavor::Simple,
                span: crate::make_parser::ast::Span::dummy(),
            },
            MakeItem::Target {
                name: "build".into(),
                prerequisites: vec!["main.c".into()],
                recipe: vec!["$(CC) -o build main.c".into()],
                phony: false,
                recipe_metadata: None,
                span: crate::make_parser::ast::Span::dummy(),
            },
        ],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(result.is_empty(), "Clean Makefile should have no issues");
}

#[test]
fn test_reproducible_builds_empty_ast() {
    let ast = MakeAst {
        items: vec![],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(result.is_empty());
}

// =============================================================================
// report — format_analysis_transformation coverage (via format_transformation)
// =============================================================================

#[test]
fn test_format_transformation_detect_timestamp() {
    let t = Transformation::DetectTimestamp {
        variable_name: "BUILD_TIME".into(),
        pattern: "$(shell date +%s)".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectTimestamp");
}

#[test]
fn test_format_transformation_detect_random() {
    let t = Transformation::DetectRandom {
        variable_name: "SEED".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectRandom");
}

#[test]
fn test_format_transformation_detect_process_id() {
    let t = Transformation::DetectProcessId {
        variable_name: "TMP".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectProcessId");
}

#[test]
fn test_format_transformation_suggest_source_date_epoch() {
    let t = Transformation::SuggestSourceDateEpoch {
        variable_name: "BUILD_TIME".into(),
        original_pattern: "$(shell date)".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format SuggestSourceDateEpoch");
}

#[test]
fn test_format_transformation_detect_non_deterministic_command() {
    let t = Transformation::DetectNonDeterministicCommand {
        variable_name: "HOST".into(),
        command: "hostname".into(),
        reason: "environment-dependent".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_combine_shell() {
    let t = Transformation::SuggestCombineShellInvocations {
        target_name: "build".into(),
        recipe_count: 5,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_simple_expansion() {
    let t = Transformation::SuggestSimpleExpansion {
        variable_name: "CC".into(),
        reason: "constant value".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_suffixes() {
    let t = Transformation::RecommendSuffixes {
        reason: "disable builtin rules".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_sequential_recipes() {
    let t = Transformation::DetectSequentialRecipes {
        target_name: "build".into(),
        recipe_count: 10,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_pattern_rule() {
    let t = Transformation::SuggestPatternRule {
        pattern: "%.o: %.c".into(),
        target_count: 3,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_missing_error_handling() {
    let t = Transformation::DetectMissingErrorHandling {
        target_name: "deploy".into(),
        command: "rm -rf /tmp/build".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_silent_failure() {
    let t = Transformation::DetectSilentFailure {
        target_name: "install".into(),
        command: "-cp src dest".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_delete_on_error() {
    let t = Transformation::RecommendDeleteOnError {
        reason: "prevent partial builds".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_oneshell() {
    let t = Transformation::RecommendOneshell {
        target_name: "build".into(),
        reason: "recipe uses cd".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_missing_set_e() {
    let t = Transformation::DetectMissingSetE {
        target_name: "build".into(),
        command: "gcc -o build main.c".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_bashism() {
    let t = Transformation::DetectBashism {
        target_name: "test".into(),
        construct: "[[ -f foo ]]".into(),
        posix_alternative: "[ -f foo ]".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_platform_specific() {
    let t = Transformation::DetectPlatformSpecific {
        target_name: "install".into(),
        command: "apt-get install foo".into(),
        reason: "debian/ubuntu only".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_shell_specific() {
    let t = Transformation::DetectShellSpecific {
        target_name: "run".into(),
        feature: "source".into(),
        posix_alternative: ". .env".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_non_portable_flags() {
    let t = Transformation::DetectNonPortableFlags {
        target_name: "build".into(),
        command: "cp --preserve=all src dest".into(),
        flag: "--preserve".into(),
        reason: "not available on BSD cp".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_non_portable_echo() {
    let t = Transformation::DetectNonPortableEcho {
        target_name: "info".into(),
        command: "echo -e \"\\t\"".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_loop_without_error_handling() {
    let t = Transformation::DetectLoopWithoutErrorHandling {
        target_name: "deploy".into(),
        loop_command: "for f in *.sh; do sh $$f; done".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}
