use super::effects::EffectSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseArm {
    pub pattern: CasePattern,
    pub guard: Option<ShellValue>,
    pub body: Box<ShellIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CasePattern {
    Literal(String), // Literal value to match
    Wildcard,        // * pattern
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellIR {
    /// Variable assignment: readonly NAME=VALUE
    Let {
        name: String,
        value: ShellValue,
        effects: EffectSet,
    },

    /// Command execution
    Exec { cmd: Command, effects: EffectSet },

    /// Conditional execution
    If {
        test: ShellValue,
        then_branch: Box<ShellIR>,
        else_branch: Option<Box<ShellIR>>,
    },

    /// Exit with code
    Exit { code: u8, message: Option<String> },

    /// Sequence of operations
    Sequence(Vec<ShellIR>),

    /// No-op
    Noop,

    /// Function definition
    Function {
        name: String,
        params: Vec<String>,
        body: Box<ShellIR>,
    },

    /// Echo a value (for function returns)
    Echo { value: ShellValue },

    /// For loop with range
    For {
        var: String,
        start: ShellValue,
        end: ShellValue,
        body: Box<ShellIR>,
    },

    /// Case statement (for match expressions)
    Case {
        scrutinee: ShellValue,
        arms: Vec<CaseArm>,
    },

    /// While loop
    While {
        condition: ShellValue,
        body: Box<ShellIR>,
    },

    /// Break statement
    Break,

    /// Continue statement
    Continue,
}

impl ShellIR {
    /// Get all effects from this IR node and its children
    pub fn effects(&self) -> EffectSet {
        match self {
            ShellIR::Let { effects, .. } | ShellIR::Exec { effects, .. } => effects.clone(),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                let mut combined = then_branch.effects();
                if let Some(else_ir) = else_branch {
                    combined = combined.union(&else_ir.effects());
                }
                combined
            }
            ShellIR::Sequence(items) => items
                .iter()
                .fold(EffectSet::pure(), |acc, item| acc.union(&item.effects())),
            ShellIR::Exit { .. } | ShellIR::Noop | ShellIR::Echo { .. } => EffectSet::pure(),
            ShellIR::Function { body, .. } => body.effects(),
            ShellIR::For { body, .. } => body.effects(),
            ShellIR::While { body, .. } => body.effects(),
            ShellIR::Case { arms, .. } => arms
                .iter()
                .fold(EffectSet::pure(), |acc, arm| acc.union(&arm.body.effects())),
            ShellIR::Break | ShellIR::Continue => EffectSet::pure(),
        }
    }

    /// Check if this IR node is pure (has no side effects)
    pub fn is_pure(&self) -> bool {
        self.effects().is_pure()
    }

    /// Collect all function names referenced by Exec nodes and CommandSubst values.
    /// Used by the emitter to determine which runtime functions to include.
    pub fn collect_used_functions(&self) -> std::collections::HashSet<String> {
        let mut used = std::collections::HashSet::new();
        self.collect_functions_recursive(&mut used);
        used
    }

    fn collect_functions_recursive(&self, used: &mut std::collections::HashSet<String>) {
        match self {
            ShellIR::Exec { cmd, .. } => {
                used.insert(cmd.program.clone());
                for arg in &cmd.args {
                    arg.collect_functions(used);
                }
            }
            ShellIR::Let { value, .. } => {
                value.collect_functions(used);
            }
            ShellIR::Echo { value } => {
                value.collect_functions(used);
            }
            ShellIR::If {
                test,
                then_branch,
                else_branch,
            } => {
                test.collect_functions(used);
                then_branch.collect_functions_recursive(used);
                if let Some(eb) = else_branch {
                    eb.collect_functions_recursive(used);
                }
            }
            ShellIR::Sequence(items) => {
                for item in items {
                    item.collect_functions_recursive(used);
                }
            }
            ShellIR::Function { body, .. } => {
                body.collect_functions_recursive(used);
            }
            ShellIR::For {
                start, end, body, ..
            } => {
                start.collect_functions(used);
                end.collect_functions(used);
                body.collect_functions_recursive(used);
            }
            ShellIR::While { condition, body } => {
                condition.collect_functions(used);
                body.collect_functions_recursive(used);
            }
            ShellIR::Case { scrutinee, arms } => {
                scrutinee.collect_functions(used);
                for arm in arms {
                    arm.body.collect_functions_recursive(used);
                    if let Some(guard) = &arm.guard {
                        guard.collect_functions(used);
                    }
                }
            }
            ShellIR::Exit { .. } | ShellIR::Noop | ShellIR::Break | ShellIR::Continue => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub program: String,
    pub args: Vec<ShellValue>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: ShellValue) -> Self {
        self.args.push(arg);
        self
    }

    pub fn args(mut self, args: Vec<ShellValue>) -> Self {
        self.args.extend(args);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellValue {
    /// String literal
    String(String),

    /// Boolean value (converted to "true"/"false")
    Bool(bool),

    /// Variable reference
    Variable(String),

    /// Concatenated values
    Concat(Vec<ShellValue>),

    /// Command substitution
    CommandSubst(Command),

    /// Comparison operation (for test conditions)
    Comparison {
        op: ComparisonOp,
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Arithmetic operation (for $((expr)))
    Arithmetic {
        op: ArithmeticOp,
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical AND (&&) operation
    LogicalAnd {
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical OR (||) operation
    LogicalOr {
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical NOT (!) operation
    LogicalNot { operand: Box<ShellValue> },

    /// Environment variable expansion: ${VAR} or ${VAR:-default}
    /// Sprint 27a: Environment Variables Support
    EnvVar {
        name: String,
        default: Option<String>,
    },

    /// Command-line argument access: $1, $2, $@, etc.
    /// Sprint 27b: Command-Line Arguments Support
    Arg {
        position: Option<usize>, // None = all args ($@)
    },

    /// Command-line argument with default value: ${1:-default}
    /// P0-POSITIONAL-PARAMETERS: Support for args.get(N).unwrap_or(default)
    ArgWithDefault { position: usize, default: String },

    /// Argument count: $#
    /// Sprint 27b: Command-Line Arguments Support
    ArgCount,

    /// Exit code of last command: $?
    /// Sprint 27c: Exit Code Handling
    ExitCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// -eq: numeric equality
    NumEq,
    /// -ne: numeric inequality
    NumNe,
    /// -gt: numeric greater than
    Gt,
    /// -ge: numeric greater than or equal
    Ge,
    /// -lt: numeric less than
    Lt,
    /// -le: numeric less than or equal
    Le,
    /// =: string equality
    StrEq,
    /// !=: string inequality
    StrNe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticOp {
    /// + : addition
    Add,
    /// - : subtraction
    Sub,
    /// * : multiplication
    Mul,
    /// / : division
    Div,
    /// % : modulo
    Mod,
}

impl ShellValue {
    /// Check if this value is a constant (doesn't depend on variables or commands)
    pub fn is_constant(&self) -> bool {
        match self {
            ShellValue::String(_) | ShellValue::Bool(_) => true,
            ShellValue::Variable(_)
            | ShellValue::CommandSubst(_)
            | ShellValue::EnvVar { .. }
            | ShellValue::Arg { .. }
            | ShellValue::ArgWithDefault { .. }
            | ShellValue::ArgCount
            | ShellValue::ExitCode => false,
            ShellValue::Concat(parts) => parts.iter().all(|p| p.is_constant()),
            ShellValue::Comparison { left, right, .. }
            | ShellValue::Arithmetic { left, right, .. }
            | ShellValue::LogicalAnd { left, right }
            | ShellValue::LogicalOr { left, right } => left.is_constant() && right.is_constant(),
            ShellValue::LogicalNot { operand } => operand.is_constant(),
        }
    }

    /// Collect function names referenced by command substitutions in this value.
    pub fn collect_functions(&self, used: &mut std::collections::HashSet<String>) {
        match self {
            ShellValue::CommandSubst(cmd) => {
                used.insert(cmd.program.clone());
                for arg in &cmd.args {
                    arg.collect_functions(used);
                }
            }
            ShellValue::Concat(parts) => {
                for part in parts {
                    part.collect_functions(used);
                }
            }
            ShellValue::Comparison { left, right, .. }
            | ShellValue::Arithmetic { left, right, .. }
            | ShellValue::LogicalAnd { left, right }
            | ShellValue::LogicalOr { left, right } => {
                left.collect_functions(used);
                right.collect_functions(used);
            }
            ShellValue::LogicalNot { operand } => {
                operand.collect_functions(used);
            }
            ShellValue::String(_)
            | ShellValue::Bool(_)
            | ShellValue::Variable(_)
            | ShellValue::EnvVar { .. }
            | ShellValue::Arg { .. }
            | ShellValue::ArgWithDefault { .. }
            | ShellValue::ArgCount
            | ShellValue::ExitCode => {}
        }
    }

    /// Get the string representation for constant values
    pub fn as_constant_string(&self) -> Option<String> {
        match self {
            ShellValue::String(s) => Some(s.clone()),
            ShellValue::Bool(b) => Some(if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            ShellValue::Concat(parts) => {
                if parts.iter().all(|p| p.is_constant()) {
                    let mut result = String::new();
                    for part in parts {
                        if let Some(s) = part.as_constant_string() {
                            result.push_str(&s);
                        } else {
                            return None;
                        }
                    }
                    Some(result)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellExpression {
    String(String),
    Variable(String, bool), // (name, is_quoted)
    Command(String),
    Arithmetic(String),
}

impl ShellExpression {
    pub fn is_quoted(&self) -> bool {
        match self {
            ShellExpression::String(s) => s.starts_with('"') && s.ends_with('"'),
            ShellExpression::Variable(_, quoted) => *quoted,
            ShellExpression::Command(_) => false,
            ShellExpression::Arithmetic(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ShellIR tests =====

    #[test]
    fn test_shell_ir_noop_is_pure() {
        assert!(ShellIR::Noop.is_pure());
    }

    #[test]
    fn test_shell_ir_exit_effects() {
        let exit = ShellIR::Exit {
            code: 0,
            message: None,
        };
        assert!(exit.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_echo_effects() {
        let echo = ShellIR::Echo {
            value: ShellValue::String("hello".to_string()),
        };
        assert!(echo.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_break_continue_effects() {
        assert!(ShellIR::Break.effects().is_pure());
        assert!(ShellIR::Continue.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_sequence_effects() {
        let seq = ShellIR::Sequence(vec![ShellIR::Noop, ShellIR::Noop]);
        assert!(seq.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_if_effects() {
        let if_ir = ShellIR::If {
            test: ShellValue::Bool(true),
            then_branch: Box::new(ShellIR::Noop),
            else_branch: Some(Box::new(ShellIR::Noop)),
        };
        assert!(if_ir.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_if_no_else() {
        let if_ir = ShellIR::If {
            test: ShellValue::Bool(true),
            then_branch: Box::new(ShellIR::Noop),
            else_branch: None,
        };
        assert!(if_ir.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_function_effects() {
        let func = ShellIR::Function {
            name: "test".to_string(),
            params: vec!["arg".to_string()],
            body: Box::new(ShellIR::Noop),
        };
        assert!(func.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_for_effects() {
        let for_ir = ShellIR::For {
            var: "i".to_string(),
            start: ShellValue::String("1".to_string()),
            end: ShellValue::String("10".to_string()),
            body: Box::new(ShellIR::Noop),
        };
        assert!(for_ir.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_while_effects() {
        let while_ir = ShellIR::While {
            condition: ShellValue::Bool(true),
            body: Box::new(ShellIR::Noop),
        };
        assert!(while_ir.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_case_effects() {
        let case_ir = ShellIR::Case {
            scrutinee: ShellValue::String("x".to_string()),
            arms: vec![CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(ShellIR::Noop),
            }],
        };
        assert!(case_ir.effects().is_pure());
    }

    #[test]
    fn test_shell_ir_let_with_pure_effects() {
        let let_ir = ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::String("hello".to_string()),
            effects: EffectSet::pure(),
        };
        assert!(let_ir.is_pure());
    }

    #[test]
    fn test_shell_ir_exec_with_pure_effects() {
        let exec_ir = ShellIR::Exec {
            cmd: Command::new("echo"),
            effects: EffectSet::pure(),
        };
        assert!(exec_ir.is_pure());
    }

    // ===== Command tests =====

    #[test]
    fn test_command_new() {
        let cmd = Command::new("ls");
        assert_eq!(cmd.program, "ls");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_command_arg() {
        let cmd = Command::new("echo").arg(ShellValue::String("hello".to_string()));
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_command_args() {
        let cmd = Command::new("cp").args(vec![
            ShellValue::String("src".to_string()),
            ShellValue::String("dst".to_string()),
        ]);
        assert_eq!(cmd.args.len(), 2);
    }

    #[test]
    fn test_command_chained() {
        let cmd = Command::new("grep")
            .arg(ShellValue::String("-r".to_string()))
            .arg(ShellValue::String("pattern".to_string()));
        assert_eq!(cmd.args.len(), 2);
    }

    // ===== ShellValue tests =====

    #[test]
    fn test_shell_value_string_is_constant() {
        let val = ShellValue::String("hello".to_string());
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_bool_is_constant() {
        assert!(ShellValue::Bool(true).is_constant());
        assert!(ShellValue::Bool(false).is_constant());
    }

    #[test]
    fn test_shell_value_variable_not_constant() {
        let val = ShellValue::Variable("x".to_string());
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_command_subst_not_constant() {
        let val = ShellValue::CommandSubst(Command::new("date"));
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_env_var_not_constant() {
        let val = ShellValue::EnvVar {
            name: "HOME".to_string(),
            default: None,
        };
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_arg_not_constant() {
        let val = ShellValue::Arg { position: Some(1) };
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_arg_with_default_not_constant() {
        let val = ShellValue::ArgWithDefault {
            position: 1,
            default: "default".to_string(),
        };
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_arg_count_not_constant() {
        assert!(!ShellValue::ArgCount.is_constant());
    }

    #[test]
    fn test_shell_value_exit_code_not_constant() {
        assert!(!ShellValue::ExitCode.is_constant());
    }

    #[test]
    fn test_shell_value_concat_constant() {
        let val = ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::String(" world".to_string()),
        ]);
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_concat_not_constant() {
        let val = ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::Variable("x".to_string()),
        ]);
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_comparison_constant() {
        let val = ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::String("1".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        };
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_comparison_not_constant() {
        let val = ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        };
        assert!(!val.is_constant());
    }

    #[test]
    fn test_shell_value_arithmetic_constant() {
        let val = ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::String("1".to_string())),
            right: Box::new(ShellValue::String("2".to_string())),
        };
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_logical_and_constant() {
        let val = ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        };
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_logical_or_constant() {
        let val = ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        };
        assert!(val.is_constant());
    }

    #[test]
    fn test_shell_value_logical_not_constant() {
        let val = ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(true)),
        };
        assert!(val.is_constant());
    }

    // ===== as_constant_string tests =====

    #[test]
    fn test_as_constant_string_string() {
        let val = ShellValue::String("hello".to_string());
        assert_eq!(val.as_constant_string(), Some("hello".to_string()));
    }

    #[test]
    fn test_as_constant_string_bool_true() {
        let val = ShellValue::Bool(true);
        assert_eq!(val.as_constant_string(), Some("true".to_string()));
    }

    #[test]
    fn test_as_constant_string_bool_false() {
        let val = ShellValue::Bool(false);
        assert_eq!(val.as_constant_string(), Some("false".to_string()));
    }

    #[test]
    fn test_as_constant_string_concat() {
        let val = ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::String(" world".to_string()),
        ]);
        assert_eq!(val.as_constant_string(), Some("hello world".to_string()));
    }

    #[test]
    fn test_as_constant_string_concat_with_variable() {
        let val = ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::Variable("x".to_string()),
        ]);
        assert_eq!(val.as_constant_string(), None);
    }

    #[test]
    fn test_as_constant_string_variable() {
        let val = ShellValue::Variable("x".to_string());
        assert_eq!(val.as_constant_string(), None);
    }

    // ===== ShellExpression tests =====

    #[test]
    fn test_shell_expression_string_quoted() {
        let expr = ShellExpression::String("\"hello\"".to_string());
        assert!(expr.is_quoted());
    }

    #[test]
    fn test_shell_expression_string_not_quoted() {
        let expr = ShellExpression::String("hello".to_string());
        assert!(!expr.is_quoted());
    }

    #[test]
    fn test_shell_expression_variable_quoted() {
        let expr = ShellExpression::Variable("x".to_string(), true);
        assert!(expr.is_quoted());
    }

    #[test]
    fn test_shell_expression_variable_not_quoted() {
        let expr = ShellExpression::Variable("x".to_string(), false);
        assert!(!expr.is_quoted());
    }

    #[test]
    fn test_shell_expression_command_not_quoted() {
        let expr = ShellExpression::Command("date".to_string());
        assert!(!expr.is_quoted());
    }

    #[test]
    fn test_shell_expression_arithmetic_is_quoted() {
        let expr = ShellExpression::Arithmetic("1 + 2".to_string());
        assert!(expr.is_quoted());
    }

    // ===== ComparisonOp tests =====

    #[test]
    fn test_comparison_op_eq() {
        assert_eq!(ComparisonOp::NumEq, ComparisonOp::NumEq);
        assert_ne!(ComparisonOp::NumEq, ComparisonOp::NumNe);
    }

    #[test]
    fn test_comparison_op_clone() {
        let ops = [
            ComparisonOp::NumEq,
            ComparisonOp::NumNe,
            ComparisonOp::Gt,
            ComparisonOp::Ge,
            ComparisonOp::Lt,
            ComparisonOp::Le,
            ComparisonOp::StrEq,
            ComparisonOp::StrNe,
        ];
        for op in ops {
            let _ = op.clone();
        }
    }

    // ===== ArithmeticOp tests =====

    #[test]
    fn test_arithmetic_op_eq() {
        assert_eq!(ArithmeticOp::Add, ArithmeticOp::Add);
        assert_ne!(ArithmeticOp::Add, ArithmeticOp::Sub);
    }

    #[test]
    fn test_arithmetic_op_clone() {
        let ops = [
            ArithmeticOp::Add,
            ArithmeticOp::Sub,
            ArithmeticOp::Mul,
            ArithmeticOp::Div,
            ArithmeticOp::Mod,
        ];
        for op in ops {
            let _ = op.clone();
        }
    }

    // ===== CasePattern tests =====

    #[test]
    fn test_case_pattern_literal() {
        let pattern = CasePattern::Literal("hello".to_string());
        let cloned = pattern.clone();
        matches!(cloned, CasePattern::Literal(_));
    }

    #[test]
    fn test_case_pattern_wildcard() {
        let pattern = CasePattern::Wildcard;
        let cloned = pattern.clone();
        matches!(cloned, CasePattern::Wildcard);
    }

    // ===== CaseArm tests =====

    #[test]
    fn test_case_arm_clone() {
        let arm = CaseArm {
            pattern: CasePattern::Wildcard,
            guard: Some(ShellValue::Bool(true)),
            body: Box::new(ShellIR::Noop),
        };
        let cloned = arm.clone();
        assert!(cloned.guard.is_some());
    }
}
