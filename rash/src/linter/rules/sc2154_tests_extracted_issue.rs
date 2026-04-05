
    #[test]
    fn test_issue_020_sc2154_multiple_loop_variables() {
        let script = r#"
for file in *.txt; do
    echo "$file"
done

for dockerfile in docker/*/Dockerfile; do
    echo "$dockerfile"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple loop variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_loop_variable_with_command_subst() {
        let script = r#"
for dockerfile in $(find . -name "*.Dockerfile"); do
    lang="$(basename "$(dirname "$dockerfile")")"
    echo "Processing: ${lang}"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Loop and assigned variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_undefined_var_in_loop_still_flagged() {
        let script = r#"
for file in *.txt; do
    echo "$file $undefined_var"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Undefined variables in loops should still be flagged"
        );
        assert_eq!(result.diagnostics[0].code, "SC2154");
        assert!(result.diagnostics[0].message.contains("undefined_var"));
    }

    // Issue #24: Function parameter tests
    #[test]
    fn test_issue_024_sc2154_local_param_from_dollar1() {
        let script = r#"
validate_args() {
    local project_dir="$1"
    local environment="$2"

    if [[ -z "${project_dir}" ]]; then
        echo "Error: Project directory required" >&2
        exit 1
    fi
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables assigned from positional parameters should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_param_with_default() {
        let script = r#"
main() {
    local project_dir="${1:-}"
    local environment="${2:-default}"

    echo "${project_dir}"
    echo "${environment}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables with default values should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_in_function_used_later() {
        let script = r#"
validate() {
    local value="$1"
    if [[ -z "${value}" ]]; then
        return 1
    fi
    echo "Valid: ${value}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables used later in function should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_multiple_local_declarations() {
        let script = r#"
process() {
    local input="$1"
    local output="$2"
    local temp="/tmp/temp"

    echo "${input}" > "${temp}"
    cat "${temp}" > "${output}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple local variable declarations should all be recognized"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_readonly_export() {
        let script = r#"
setup() {
    local config="$1"
    readonly VERSION="1.0.0"
    export PATH="/usr/local/bin:$PATH"

    echo "${config} ${VERSION}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "local, readonly, and export declarations should all be recognized"
        );
    }

    #[test]
    fn test_issue_024_sc2154_declare_typeset() {
        let script = r#"
func() {
    declare var1="$1"
    typeset var2="$2"

    echo "${var1} ${var2}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "declare and typeset should be recognized as assignments"
        );
    }

    #[test]
    fn test_issue_024_sc2154_undefined_still_caught() {
        let script = r#"
func() {
    local defined="$1"
    echo "${defined} ${undefined}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Undefined variables should still be caught"
        );
        assert!(result.diagnostics[0].message.contains("undefined"));
    }

    #[test]
    fn test_issue_024_sc2154_declare_with_flags() {
        let script = r#"
func() {
    declare -i count="$1"
    declare -r readonly_var="$2"
    local -r local_readonly="$3"

    echo "${count} ${readonly_var} ${local_readonly}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "declare/local with flags (-i, -r) should be recognized"
        );
    }

    // Issue #95: Source command detection tests

include!("sc2154_tests_extracted_issue_issue.rs");
