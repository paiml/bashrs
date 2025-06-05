//! Contract system for shell-specific type inference and validation

use crate::formatter::types::*;
use std::collections::HashMap;

/// Contract-based type system for shell scripts
#[derive(Debug, Clone)]
pub struct ContractSystem {
    /// Type environment for variables
    type_env: HashMap<String, ShellType>,

    /// Function signatures
    function_sigs: HashMap<String, FunctionSignature>,

    /// Active contracts in current scope
    active_contracts: Vec<Contract>,

    /// Type inference engine
    inference_engine: TypeInferenceEngine,
}

/// Function signature with pre/post conditions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: ShellType,
    pub preconditions: Vec<Contract>,
    pub postconditions: Vec<Contract>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: ShellType,
    pub is_optional: bool,
}

/// Contract specification
#[derive(Debug, Clone)]
pub struct Contract {
    pub kind: ContractKind,
    pub condition: ContractCondition,
    pub description: String,
    pub location: Span,
}

/// Contract condition expressed as logical formula
#[derive(Debug, Clone)]
pub enum ContractCondition {
    /// Type constraint: variable has specific type
    TypeConstraint {
        var: String,
        expected_type: ShellType,
    },

    /// Range constraint: numeric variable in range
    RangeConstraint {
        var: String,
        min: Option<i64>,
        max: Option<i64>,
    },

    /// Non-null constraint: variable is defined
    NonNull {
        var: String,
    },

    /// File system constraint: path exists, is readable, etc.
    FileSystemConstraint {
        path: String,
        constraint: FsConstraint,
    },

    /// Custom predicate with shell expression
    CustomPredicate {
        expression: String,
    },

    /// Logical operators
    And(Box<ContractCondition>, Box<ContractCondition>),
    Or(Box<ContractCondition>, Box<ContractCondition>),
    Not(Box<ContractCondition>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsConstraint {
    Exists,
    IsReadable,
    IsWritable,
    IsExecutable,
    IsDirectory,
    IsRegularFile,
}

/// Type inference engine with constraint solving
#[derive(Debug, Clone, Default)]
pub struct TypeInferenceEngine {
    /// Type constraints to solve
    constraints: Vec<TypeConstraint>,

    /// Fresh type variable counter
    next_type_var: u32,
}

#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub left: ShellType,
    pub right: ShellType,
    pub location: Span,
    pub reason: ConstraintReason,
}

#[derive(Debug, Clone)]
pub enum ConstraintReason {
    Assignment,
    FunctionCall,
    Arithmetic,
    Comparison,
    ArrayAccess,
}

impl ContractSystem {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            function_sigs: Self::builtin_functions(),
            active_contracts: Vec::new(),
            inference_engine: TypeInferenceEngine::default(),
        }
    }

    /// Initialize with built-in shell functions
    fn builtin_functions() -> HashMap<String, FunctionSignature> {
        let mut functions = HashMap::new();

        // echo: echo [string...] -> exit_code
        functions.insert(
            "echo".to_string(),
            FunctionSignature {
                name: "echo".to_string(),
                parameters: vec![Parameter {
                    name: "args".to_string(),
                    param_type: ShellType::Array(Box::new(ShellType::String)),
                    is_optional: true,
                }],
                return_type: ShellType::ExitCode,
                preconditions: vec![],
                postconditions: vec![],
            },
        );

        // test: test expression -> boolean
        functions.insert(
            "test".to_string(),
            FunctionSignature {
                name: "test".to_string(),
                parameters: vec![Parameter {
                    name: "expression".to_string(),
                    param_type: ShellType::String,
                    is_optional: false,
                }],
                return_type: ShellType::Boolean,
                preconditions: vec![],
                postconditions: vec![],
            },
        );

        // read: read [var...] -> exit_code
        functions.insert(
            "read".to_string(),
            FunctionSignature {
                name: "read".to_string(),
                parameters: vec![Parameter {
                    name: "variables".to_string(),
                    param_type: ShellType::Array(Box::new(ShellType::String)),
                    is_optional: true,
                }],
                return_type: ShellType::ExitCode,
                preconditions: vec![],
                postconditions: vec![Contract {
                    kind: ContractKind::Postcondition,
                    condition: ContractCondition::CustomPredicate {
                        expression: "read variables are defined".to_string(),
                    },
                    description: "Variables are defined after successful read".to_string(),
                    location: Span::new(BytePos(0), BytePos(0)),
                }],
            },
        );

        functions
    }

    /// Add a contract to the current scope
    pub fn add_contract(&mut self, contract: Contract) {
        self.active_contracts.push(contract);
    }

    /// Infer type for a variable
    pub fn infer_variable_type(&mut self, var_name: &str, context: &TypeContext) -> ShellType {
        // Check existing type environment
        if let Some(existing_type) = self.type_env.get(var_name) {
            return existing_type.clone();
        }

        // Create fresh type variable
        let type_var = self.inference_engine.fresh_type_var();

        // Add constraints based on context
        match context {
            TypeContext::Assignment { value_type } => {
                self.inference_engine.add_constraint(TypeConstraint {
                    left: type_var.clone(),
                    right: value_type.clone(),
                    location: context.location(),
                    reason: ConstraintReason::Assignment,
                });
            }
            TypeContext::FunctionCall {
                function,
                param_index,
            } => {
                if let Some(sig) = self.function_sigs.get(function) {
                    if let Some(param) = sig.parameters.get(*param_index) {
                        self.inference_engine.add_constraint(TypeConstraint {
                            left: type_var.clone(),
                            right: param.param_type.clone(),
                            location: context.location(),
                            reason: ConstraintReason::FunctionCall,
                        });
                    }
                }
            }
            TypeContext::Arithmetic => {
                self.inference_engine.add_constraint(TypeConstraint {
                    left: type_var.clone(),
                    right: ShellType::Integer,
                    location: context.location(),
                    reason: ConstraintReason::Arithmetic,
                });
            }
        }

        // Store and return the type variable
        self.type_env.insert(var_name.to_string(), type_var.clone());
        type_var
    }

    /// Solve all type constraints
    pub fn solve_constraints(&mut self) -> Result<(), TypeError> {
        let mut substitution = HashMap::new();

        for constraint in &self.inference_engine.constraints {
            match self.unify(&constraint.left, &constraint.right, &mut substitution) {
                Ok(_) => {}
                Err(e) => {
                    return Err(TypeError {
                        kind: e,
                        location: constraint.location,
                        constraint_reason: constraint.reason.clone(),
                    })
                }
            }
        }

        // Apply substitution to type environment
        let mut updated_env = HashMap::new();
        for (var, var_type) in &self.type_env {
            let new_type = self.apply_substitution(var_type, &substitution);
            updated_env.insert(var.clone(), new_type);
        }
        self.type_env = updated_env;

        Ok(())
    }

    /// Unification algorithm for type inference
    #[allow(clippy::only_used_in_recursion)]
    fn unify(
        &self,
        t1: &ShellType,
        t2: &ShellType,
        substitution: &mut HashMap<u32, ShellType>,
    ) -> Result<(), TypeErrorKind> {
        match (t1, t2) {
            // Identical types
            (a, b) if a == b => Ok(()),

            // Type variables
            (ShellType::TypeVar(id), other) | (other, ShellType::TypeVar(id)) => {
                if let Some(existing) = substitution.get(id).cloned() {
                    self.unify(&existing, other, substitution)
                } else {
                    substitution.insert(*id, other.clone());
                    Ok(())
                }
            }

            // Array types
            (ShellType::Array(inner1), ShellType::Array(inner2)) => {
                self.unify(inner1, inner2, substitution)
            }

            // Associative array types
            (
                ShellType::AssocArray { key: k1, value: v1 },
                ShellType::AssocArray { key: k2, value: v2 },
            ) => {
                self.unify(k1, k2, substitution)?;
                self.unify(v1, v2, substitution)
            }

            // Union types (simplified - would need more sophisticated handling)
            (ShellType::Union(types), other) | (other, ShellType::Union(types)) => {
                if types.iter().any(|t| t.is_compatible(other)) {
                    Ok(())
                } else {
                    Err(TypeErrorKind::IncompatibleTypes)
                }
            }

            // Incompatible types
            _ => Err(TypeErrorKind::IncompatibleTypes),
        }
    }

    /// Apply type substitution
    #[allow(clippy::only_used_in_recursion)]
    fn apply_substitution(
        &self,
        shell_type: &ShellType,
        substitution: &HashMap<u32, ShellType>,
    ) -> ShellType {
        match shell_type {
            ShellType::TypeVar(id) => substitution
                .get(id)
                .cloned()
                .unwrap_or_else(|| shell_type.clone()),
            ShellType::Array(inner) => {
                ShellType::Array(Box::new(self.apply_substitution(inner, substitution)))
            }
            ShellType::AssocArray { key, value } => ShellType::AssocArray {
                key: Box::new(self.apply_substitution(key, substitution)),
                value: Box::new(self.apply_substitution(value, substitution)),
            },
            ShellType::Union(types) => ShellType::Union(
                types
                    .iter()
                    .map(|t| self.apply_substitution(t, substitution))
                    .collect(),
            ),
            _ => shell_type.clone(),
        }
    }

    /// Validate contracts in current scope
    pub fn validate_contracts(&self) -> Vec<ContractViolation> {
        let mut violations = Vec::new();

        for contract in &self.active_contracts {
            if let Some(violation) = self.check_contract(contract) {
                violations.push(violation);
            }
        }

        violations
    }

    /// Check a specific contract
    fn check_contract(&self, contract: &Contract) -> Option<ContractViolation> {
        match &contract.condition {
            ContractCondition::TypeConstraint { var, expected_type } => {
                if let Some(actual_type) = self.type_env.get(var) {
                    if !actual_type.is_compatible(expected_type) {
                        return Some(ContractViolation {
                            contract: contract.clone(),
                            reason: format!(
                                "Variable '{}' has type {} but expected {}",
                                var,
                                actual_type.display(),
                                expected_type.display()
                            ),
                        });
                    }
                }
            }
            ContractCondition::NonNull { var } => {
                if !self.type_env.contains_key(var) {
                    return Some(ContractViolation {
                        contract: contract.clone(),
                        reason: format!("Variable '{var}' is not defined"),
                    });
                }
            }
            // Other constraint types would be implemented here
            _ => {
                // For now, assume other constraints pass
            }
        }

        None
    }

    /// Get the current type of a variable
    pub fn get_variable_type(&self, var_name: &str) -> Option<&ShellType> {
        self.type_env.get(var_name)
    }

    /// Register a function signature
    pub fn register_function(&mut self, signature: FunctionSignature) {
        self.function_sigs.insert(signature.name.clone(), signature);
    }
}

/// Context for type inference
#[derive(Debug, Clone)]
pub enum TypeContext {
    Assignment {
        value_type: ShellType,
    },
    FunctionCall {
        function: String,
        param_index: usize,
    },
    Arithmetic,
}

impl TypeContext {
    pub fn location(&self) -> Span {
        // Simplified - would contain actual location information
        Span::new(BytePos(0), BytePos(0))
    }
}

impl TypeInferenceEngine {
    pub fn fresh_type_var(&mut self) -> ShellType {
        let id = self.next_type_var;
        self.next_type_var += 1;
        ShellType::TypeVar(id)
    }

    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
}

/// Type error information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub location: Span,
    pub constraint_reason: ConstraintReason,
}

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    IncompatibleTypes,
    UndefinedVariable(String),
    UndefinedFunction(String),
    ArityMismatch { expected: usize, actual: usize },
}

/// Contract violation
#[derive(Debug, Clone)]
pub struct ContractViolation {
    pub contract: Contract,
    pub reason: String,
}

impl Default for ContractSystem {
    fn default() -> Self {
        Self::new()
    }
}

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
