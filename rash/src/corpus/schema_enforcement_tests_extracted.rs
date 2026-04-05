#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat, output: &str) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: String::new(),
            expected_output: output.to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
    }

    #[test]
    fn test_grammar_category_code() {
        assert_eq!(GrammarCategory::MissingQuoting.code(), "GRAM-001");
        assert_eq!(GrammarCategory::Bashism.code(), "GRAM-002");
        assert_eq!(GrammarCategory::TabSpaceConfusion.code(), "GRAM-003");
        assert_eq!(GrammarCategory::ShellFormCmd.code(), "GRAM-004");
        assert_eq!(GrammarCategory::UndefinedVariable.code(), "GRAM-005");
        assert_eq!(GrammarCategory::InvalidArithmetic.code(), "GRAM-006");
        assert_eq!(GrammarCategory::MissingFrom.code(), "GRAM-007");
        assert_eq!(GrammarCategory::CircularDependency.code(), "GRAM-008");
    }

    #[test]
    fn test_grammar_category_display() {
        assert_eq!(format!("{}", GrammarCategory::Bashism), "GRAM-002");
    }

    #[test]
    fn test_grammar_category_all() {
        assert_eq!(GrammarCategory::all().len(), 8);
    }

    #[test]
    fn test_grammar_category_applicable_format() {
        assert_eq!(
            GrammarCategory::MissingQuoting.applicable_format(),
            CorpusFormat::Bash
        );
        assert_eq!(
            GrammarCategory::TabSpaceConfusion.applicable_format(),
            CorpusFormat::Makefile
        );
        assert_eq!(
            GrammarCategory::MissingFrom.applicable_format(),
            CorpusFormat::Dockerfile
        );
    }

    #[test]
    fn test_grammar_category_description() {
        assert!(!GrammarCategory::MissingQuoting.description().is_empty());
        assert!(!GrammarCategory::Bashism.fix_pattern().is_empty());
    }

    #[test]
    fn test_validation_layer_display() {
        assert_eq!(format!("{}", ValidationLayer::Lexical), "L1:Lexical");
        assert_eq!(format!("{}", ValidationLayer::Syntactic), "L2:Syntactic");
        assert_eq!(format!("{}", ValidationLayer::Semantic), "L3:Semantic");
        assert_eq!(format!("{}", ValidationLayer::Behavioral), "L4:Behavioral");
    }

    #[test]
    fn test_validate_bash_clean() {
        let entry = make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nset -eu\necho \"hello\"\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
        assert!(result.violations.is_empty());
        assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    }

    #[test]
    fn test_validate_bash_bashism() {
        let entry = make_entry(
            "B-002",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo ok; fi\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].category, GrammarCategory::Bashism);
    }

    #[test]
    fn test_validate_bash_unquoted_expansion() {
        let entry = make_entry("B-003", CorpusFormat::Bash, "#!/bin/sh\necho $HOME\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::MissingQuoting
        );
    }

    #[test]
    fn test_validate_bash_quoted_expansion_ok() {
        let entry = make_entry("B-004", CorpusFormat::Bash, "#!/bin/sh\necho \"$HOME\"\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_assignment_not_flagged() {
        let entry = make_entry("B-005", CorpusFormat::Bash, "#!/bin/sh\nFOO=$HOME\n");
        let result = validate_entry(&entry);
        // Assignments are not flagged for unquoted expansions
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_invalid_arithmetic() {
        let entry = make_entry("B-006", CorpusFormat::Bash, "#!/bin/sh\n(( x = x + 1 ))\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::InvalidArithmetic
        );
    }

    #[test]
    fn test_validate_bash_posix_arithmetic_ok() {
        let entry = make_entry("B-007", CorpusFormat::Bash, "#!/bin/sh\nx=$((x + 1))\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_clean() {
        let entry = make_entry(
            "M-001",
            CorpusFormat::Makefile,
            "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_space_recipe() {
        let entry = make_entry("M-002", CorpusFormat::Makefile, "all:\n    echo hello\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::TabSpaceConfusion
        );
    }

    #[test]
    fn test_validate_dockerfile_clean() {
        let entry = make_entry(
            "D-001",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nRUN apk add --no-cache curl\nCMD [\"curl\", \"https://example.com\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_dockerfile_missing_from() {
        let entry = make_entry("D-002", CorpusFormat::Dockerfile, "RUN apt-get update\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::MissingFrom);
    }

    #[test]
    fn test_validate_dockerfile_shell_form_cmd() {
        let entry = make_entry(
            "D-003",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD echo hello\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::ShellFormCmd);
    }

    #[test]
    fn test_validate_dockerfile_exec_form_ok() {
        let entry = make_entry(
            "D-004",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD [\"echo\", \"hello\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

include!("schema_enforcement_tests_extracted_validate.rs");
