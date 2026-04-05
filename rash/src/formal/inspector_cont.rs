#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_inspection() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "TEST_VAR".to_string(),
                    value: "test_value".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["Hello".to_string()],
                },
            ],
        };

        let initial_state = AbstractState::new();
        let report = ProofInspector::inspect(&ast, initial_state);

        // Verify we have a report
        assert!(!report.emitted_code.is_empty());
        assert!(matches!(
            report.verification_result,
            VerificationResult::Success { .. }
        ));
        assert!(!report.emitter_justifications.is_empty());

        // Verify annotated AST has correct structure
        assert_eq!(report.annotated_ast.children.len(), 2);

        // Generate human-readable report
        let readable_report = ProofInspector::generate_report(&report);
        assert!(readable_report.contains("Formal Verification Report"));
        assert!(readable_report.contains("SUCCESS"));
    }

    #[test]
    fn test_transformation_analysis() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "NEW_VAR".to_string(),
            value: "new_value".to_string(),
        };

        let initial_state = AbstractState::new();
        let report = ProofInspector::inspect(&ast, initial_state);

        // Check that transformation detected the environment change
        assert!(!report.annotated_ast.transformation.env_changes.is_empty());
        assert!(report
            .annotated_ast
            .transformation
            .env_changes
            .contains_key("NEW_VAR"));
    }
}
