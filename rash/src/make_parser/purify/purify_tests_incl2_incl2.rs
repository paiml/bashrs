fn test_purify_is_safe_detect_output_conflict() {
    let t = Transformation::DetectOutputConflict {
        target_names: vec!["a".to_string()],
        output_file: "out".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_recursive_make_handling() {
    let t = Transformation::RecommendRecursiveMakeHandling {
        target_name: "all".to_string(),
        subdirs: vec!["sub1".to_string()],
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_directory_race() {
    let t = Transformation::DetectDirectoryRace {
        target_names: vec!["a".to_string()],
        directory: "obj".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_simple_expansion() {
    let t = Transformation::SuggestSimpleExpansion {
        variable_name: "CC".to_string(),
        reason: "constant".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_suffixes() {
    let t = Transformation::RecommendSuffixes {
        reason: "disable builtin rules".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_sequential_recipes() {
    let t = Transformation::DetectSequentialRecipes {
        target_name: "install".to_string(),
        recipe_count: 5,
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_suggest_pattern_rule() {
    let t = Transformation::SuggestPatternRule {
        pattern: "%.o: %.c".to_string(),
        target_count: 3,
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_missing_error_handling() {
    let t = Transformation::DetectMissingErrorHandling {
        target_name: "build".to_string(),
        command: "gcc".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_silent_failure() {
    let t = Transformation::DetectSilentFailure {
        target_name: "test".to_string(),
        command: "@echo".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_delete_on_error() {
    let t = Transformation::RecommendDeleteOnError {
        reason: "safety".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_recommend_oneshell() {
    let t = Transformation::RecommendOneshell {
        target_name: "deploy".to_string(),
        reason: "multiline recipe".to_string(),
        safe: true,
    };
    assert!(report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_missing_set_e() {
    let t = Transformation::DetectMissingSetE {
        target_name: "test".to_string(),
        command: "bash -c".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_loop_without_error_handling() {
    let t = Transformation::DetectLoopWithoutErrorHandling {
        target_name: "deploy".to_string(),
        loop_command: "for f in *.sh; do sh $f; done".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_bashism() {
    let t = Transformation::DetectBashism {
        target_name: "test".to_string(),
        construct: "[[".to_string(),
        posix_alternative: "use [".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_platform_specific() {
    let t = Transformation::DetectPlatformSpecific {
        target_name: "check".to_string(),
        command: "uname".to_string(),
        reason: "OS-specific".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_shell_specific() {
    let t = Transformation::DetectShellSpecific {
        target_name: "run".to_string(),
        feature: "source".to_string(),
        posix_alternative: "use .".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_non_portable_flags() {
    let t = Transformation::DetectNonPortableFlags {
        target_name: "copy".to_string(),
        command: "cp --preserve=all".to_string(),
        flag: "--preserve".to_string(),
        reason: "GNU-only".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}

#[test]
fn test_purify_is_safe_detect_non_portable_echo() {
    let t = Transformation::DetectNonPortableEcho {
        target_name: "info".to_string(),
        command: "echo -e".to_string(),
        safe: false,
    };
    assert!(!report::is_safe_transformation(&t));
}
