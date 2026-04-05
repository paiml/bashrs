#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_system_creation() {
        let system = ContractSystem::new();
        assert!(system.function_sigs.contains_key("echo"));
        assert!(system.function_sigs.contains_key("test"));
        assert!(system.function_sigs.contains_key("read"));
    }

    #[test]
    fn test_type_inference_basic() {
        let mut system = ContractSystem::new();

        let context = TypeContext::Assignment {
            value_type: ShellType::String,
        };

        let inferred_type = system.infer_variable_type("var1", &context);
        assert!(matches!(inferred_type, ShellType::TypeVar(_)));

        // Solve constraints
        assert!(system.solve_constraints().is_ok());

        // Check that variable now has string type
        let final_type = system.get_variable_type("var1").unwrap();
        assert_eq!(*final_type, ShellType::String);
    }

    #[test]
    fn test_function_call_inference() {
        let mut system = ContractSystem::new();

        let context = TypeContext::FunctionCall {
            function: "echo".to_string(),
            param_index: 0,
        };

        let inferred_type = system.infer_variable_type("args", &context);
        assert!(matches!(inferred_type, ShellType::TypeVar(_)));

        assert!(system.solve_constraints().is_ok());

        let final_type = system.get_variable_type("args").unwrap();
        assert!(matches!(final_type, ShellType::Array(_)));
    }

    #[test]
    fn test_arithmetic_context_inference() {
        let mut system = ContractSystem::new();

        let context = TypeContext::Arithmetic;
        let _inferred_type = system.infer_variable_type("num", &context);

        assert!(system.solve_constraints().is_ok());

        let final_type = system.get_variable_type("num").unwrap();
        assert_eq!(*final_type, ShellType::Integer);
    }

    #[test]
    fn test_contract_validation() {
        let mut system = ContractSystem::new();

        // Add a variable with string type
        system
            .type_env
            .insert("var1".to_string(), ShellType::String);

        // Add a contract expecting integer type
        let contract = Contract {
            kind: ContractKind::TypeAnnotation,
            condition: ContractCondition::TypeConstraint {
                var: "var1".to_string(),
                expected_type: ShellType::Integer,
            },
            description: "var1 should be integer".to_string(),
            location: Span::new(BytePos(0), BytePos(10)),
        };

        system.add_contract(contract);

        let violations = system.validate_contracts();
        assert_eq!(violations.len(), 1);
        assert!(violations[0]
            .reason
            .contains("has type string but expected integer"));
    }

    #[test]
    fn test_non_null_contract() {
        let mut system = ContractSystem::new();

        let contract = Contract {
            kind: ContractKind::Precondition,
            condition: ContractCondition::NonNull {
                var: "undefined_var".to_string(),
            },
            description: "Variable must be defined".to_string(),
            location: Span::new(BytePos(0), BytePos(10)),
        };

        system.add_contract(contract);

        let violations = system.validate_contracts();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].reason.contains("is not defined"));
    }

    #[test]
    fn test_function_signature_registration() {
        let mut system = ContractSystem::new();

        let custom_func = FunctionSignature {
            name: "custom_func".to_string(),
            parameters: vec![Parameter {
                name: "input".to_string(),
                param_type: ShellType::String,
                is_optional: false,
            }],
            return_type: ShellType::Boolean,
            preconditions: vec![],
            postconditions: vec![],
        };

        system.register_function(custom_func);
        assert!(system.function_sigs.contains_key("custom_func"));
    }

    #[test]
    fn test_shell_type_compatibility() {
        let string_type = ShellType::String;
        let int_type = ShellType::Integer;
        let union_type = ShellType::Union(vec![ShellType::String, ShellType::Integer]);

        assert!(string_type.is_compatible(&string_type));
        assert!(!string_type.is_compatible(&int_type));
        assert!(union_type.is_compatible(&string_type));
        assert!(union_type.is_compatible(&int_type));
    }

    #[test]
    fn test_array_type_inference() {
        let mut system = ContractSystem::new();

        // Test array type unification
        let array_str = ShellType::Array(Box::new(ShellType::String));
        let type_var = system.inference_engine.fresh_type_var();

        let constraint = TypeConstraint {
            left: type_var.clone(),
            right: array_str.clone(),
            location: Span::new(BytePos(0), BytePos(10)),
            reason: ConstraintReason::Assignment,
        };

        system.inference_engine.add_constraint(constraint);
        assert!(system.solve_constraints().is_ok());
    }

    #[test]
    fn test_contract_condition_logic() {
        let type_constraint = ContractCondition::TypeConstraint {
            var: "x".to_string(),
            expected_type: ShellType::String,
        };

        let non_null_constraint = ContractCondition::NonNull {
            var: "y".to_string(),
        };

        let and_condition =
            ContractCondition::And(Box::new(type_constraint), Box::new(non_null_constraint));

        // Test that we can construct complex logical conditions
        match and_condition {
            ContractCondition::And(_, _) => {}
            _ => panic!("Should be And condition"),
        }
    }
}
