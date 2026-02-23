// Expression purification for Bash scripts
//
// Handles all BashExpr variants: Variable, CommandSubst, Array, Concat,
// Test, Arithmetic, Literal, Glob, and parameter expansion forms.

use crate::bash_parser::ast::*;
use super::{PurificationError, PurificationResult, Purifier};

impl Purifier {
    pub(super) fn purify_expression(&mut self, expr: &BashExpr) -> PurificationResult<BashExpr> {
        match expr {
            BashExpr::Variable(name) => self.purify_variable_expr(name, expr),
            BashExpr::CommandSubst(cmd) => {
                self.report
                    .warnings
                    .push("Command substitution detected - may affect determinism".to_string());
                let purified_cmd = self.purify_statement(cmd)?;
                Ok(BashExpr::CommandSubst(Box::new(purified_cmd)))
            }
            BashExpr::Array(items) => self.purify_array_expr(items),
            BashExpr::Concat(parts) => self.purify_concat_expr(parts),
            BashExpr::Test(test_expr) => {
                let purified_test = self.purify_test_expr(test_expr)?;
                Ok(BashExpr::Test(Box::new(purified_test)))
            }
            BashExpr::Arithmetic(arith) => {
                let purified_arith = self.purify_arithmetic(arith)?;
                Ok(BashExpr::Arithmetic(Box::new(purified_arith)))
            }
            BashExpr::Literal(_) | BashExpr::Glob(_) => Ok(expr.clone()),
            BashExpr::DefaultValue { variable, default } => {
                self.purify_param_expansion_with_expr(variable, default, ParamExpKind::DefaultValue)
            }
            BashExpr::AssignDefault { variable, default } => {
                self.purify_param_expansion_with_expr(variable, default, ParamExpKind::AssignDefault)
            }
            BashExpr::ErrorIfUnset { variable, message } => {
                self.purify_param_expansion_with_expr(variable, message, ParamExpKind::ErrorIfUnset)
            }
            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => self.purify_param_expansion_with_expr(
                variable,
                alternative,
                ParamExpKind::AlternativeValue,
            ),
            BashExpr::StringLength { variable } => {
                self.check_nondet_variable(variable, "String length");
                Ok(BashExpr::StringLength {
                    variable: variable.clone(),
                })
            }
            BashExpr::RemoveSuffix { variable, pattern } => {
                self.purify_pattern_removal(variable, pattern, PatternRemovalKind::Suffix)
            }
            BashExpr::RemovePrefix { variable, pattern } => {
                self.purify_pattern_removal(variable, pattern, PatternRemovalKind::Prefix)
            }
            BashExpr::RemoveLongestPrefix { variable, pattern } => {
                self.purify_pattern_removal(variable, pattern, PatternRemovalKind::LongestPrefix)
            }
            BashExpr::RemoveLongestSuffix { variable, pattern } => {
                self.purify_pattern_removal(variable, pattern, PatternRemovalKind::LongestSuffix)
            }
            BashExpr::CommandCondition(cmd) => {
                let purified_cmd = self.purify_statement(cmd)?;
                Ok(BashExpr::CommandCondition(Box::new(purified_cmd)))
            }
        }
    }

    fn purify_variable_expr(
        &mut self,
        name: &str,
        expr: &BashExpr,
    ) -> PurificationResult<BashExpr> {
        if self.non_deterministic_vars.contains(name) {
            if self.options.remove_non_deterministic {
                self.report
                    .determinism_fixes
                    .push(format!("Removed non-deterministic variable: ${}", name));
                return Ok(BashExpr::Literal("0".to_string()));
            } else if self.options.strict_idempotency {
                return Err(PurificationError::NonDeterministicConstruct(format!(
                    "Variable ${} is non-deterministic",
                    name
                )));
            }
        }
        Ok(expr.clone())
    }

    fn purify_array_expr(&mut self, items: &[BashExpr]) -> PurificationResult<BashExpr> {
        let mut purified_items = Vec::new();
        for item in items {
            purified_items.push(self.purify_expression(item)?);
        }
        Ok(BashExpr::Array(purified_items))
    }

    fn purify_concat_expr(&mut self, parts: &[BashExpr]) -> PurificationResult<BashExpr> {
        let mut purified_parts = Vec::new();
        for part in parts {
            purified_parts.push(self.purify_expression(part)?);
        }
        Ok(BashExpr::Concat(purified_parts))
    }

    fn check_nondet_variable(&mut self, variable: &str, context: &str) {
        if self.non_deterministic_vars.contains(variable) {
            self.report.determinism_fixes.push(format!(
                "{} expansion uses non-deterministic variable: ${}",
                context, variable
            ));
        }
    }

    fn purify_param_expansion_with_expr(
        &mut self,
        variable: &str,
        inner_expr: &BashExpr,
        kind: ParamExpKind,
    ) -> PurificationResult<BashExpr> {
        self.check_nondet_variable(variable, kind.label());
        let purified_inner = self.purify_expression(inner_expr)?;
        Ok(kind.build(variable.to_string(), Box::new(purified_inner)))
    }

    fn purify_pattern_removal(
        &mut self,
        variable: &str,
        pattern: &BashExpr,
        kind: PatternRemovalKind,
    ) -> PurificationResult<BashExpr> {
        self.check_nondet_variable(variable, kind.label());
        let purified_pattern = Box::new(self.purify_expression(pattern)?);
        Ok(kind.build(variable.to_string(), purified_pattern))
    }
}

/// Kind of parameter expansion with an inner expression
enum ParamExpKind {
    DefaultValue,
    AssignDefault,
    ErrorIfUnset,
    AlternativeValue,
}

impl ParamExpKind {
    fn label(&self) -> &'static str {
        match self {
            Self::DefaultValue => "Default value",
            Self::AssignDefault => "Assign default",
            Self::ErrorIfUnset => "Error-if-unset",
            Self::AlternativeValue => "Alternative value",
        }
    }

    fn build(self, variable: String, inner: Box<BashExpr>) -> BashExpr {
        match self {
            Self::DefaultValue => BashExpr::DefaultValue {
                variable,
                default: inner,
            },
            Self::AssignDefault => BashExpr::AssignDefault {
                variable,
                default: inner,
            },
            Self::ErrorIfUnset => BashExpr::ErrorIfUnset {
                variable,
                message: inner,
            },
            Self::AlternativeValue => BashExpr::AlternativeValue {
                variable,
                alternative: inner,
            },
        }
    }
}

/// Kind of pattern removal operation
enum PatternRemovalKind {
    Suffix,
    Prefix,
    LongestPrefix,
    LongestSuffix,
}

impl PatternRemovalKind {
    fn label(&self) -> &'static str {
        match self {
            Self::Suffix => "Remove suffix",
            Self::Prefix => "Remove prefix",
            Self::LongestPrefix => "Remove longest prefix",
            Self::LongestSuffix => "Remove longest suffix",
        }
    }

    fn build(self, variable: String, pattern: Box<BashExpr>) -> BashExpr {
        match self {
            Self::Suffix => BashExpr::RemoveSuffix { variable, pattern },
            Self::Prefix => BashExpr::RemovePrefix { variable, pattern },
            Self::LongestPrefix => BashExpr::RemoveLongestPrefix { variable, pattern },
            Self::LongestSuffix => BashExpr::RemoveLongestSuffix { variable, pattern },
        }
    }
}
