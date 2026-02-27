// Test expression and arithmetic purification for Bash scripts
//
// Handles: TestExpr (comparisons, file tests, logical ops) and ArithExpr

use super::{PurificationResult, Purifier};
use crate::bash_parser::ast::*;

impl Purifier {
    pub(super) fn purify_test_expr(&mut self, test: &TestExpr) -> PurificationResult<TestExpr> {
        match test {
            TestExpr::StringEq(a, b)
            | TestExpr::StringNe(a, b)
            | TestExpr::IntEq(a, b)
            | TestExpr::IntNe(a, b)
            | TestExpr::IntLt(a, b)
            | TestExpr::IntLe(a, b)
            | TestExpr::IntGt(a, b)
            | TestExpr::IntGe(a, b) => {
                let purified_a = self.purify_expression(a)?;
                let purified_b = self.purify_expression(b)?;

                Ok(match test {
                    TestExpr::StringEq(_, _) => TestExpr::StringEq(purified_a, purified_b),
                    TestExpr::StringNe(_, _) => TestExpr::StringNe(purified_a, purified_b),
                    TestExpr::IntEq(_, _) => TestExpr::IntEq(purified_a, purified_b),
                    TestExpr::IntNe(_, _) => TestExpr::IntNe(purified_a, purified_b),
                    TestExpr::IntLt(_, _) => TestExpr::IntLt(purified_a, purified_b),
                    TestExpr::IntLe(_, _) => TestExpr::IntLe(purified_a, purified_b),
                    TestExpr::IntGt(_, _) => TestExpr::IntGt(purified_a, purified_b),
                    TestExpr::IntGe(_, _) => TestExpr::IntGe(purified_a, purified_b),
                    _ => unreachable!(),
                })
            }

            TestExpr::FileExists(p)
            | TestExpr::FileReadable(p)
            | TestExpr::FileWritable(p)
            | TestExpr::FileExecutable(p)
            | TestExpr::FileDirectory(p) => {
                let purified_p = self.purify_expression(p)?;

                Ok(match test {
                    TestExpr::FileExists(_) => TestExpr::FileExists(purified_p),
                    TestExpr::FileReadable(_) => TestExpr::FileReadable(purified_p),
                    TestExpr::FileWritable(_) => TestExpr::FileWritable(purified_p),
                    TestExpr::FileExecutable(_) => TestExpr::FileExecutable(purified_p),
                    TestExpr::FileDirectory(_) => TestExpr::FileDirectory(purified_p),
                    _ => unreachable!(),
                })
            }

            TestExpr::StringEmpty(s) | TestExpr::StringNonEmpty(s) => {
                let purified_s = self.purify_expression(s)?;

                Ok(match test {
                    TestExpr::StringEmpty(_) => TestExpr::StringEmpty(purified_s),
                    TestExpr::StringNonEmpty(_) => TestExpr::StringNonEmpty(purified_s),
                    _ => unreachable!(),
                })
            }

            TestExpr::And(a, b) | TestExpr::Or(a, b) => {
                let purified_a = self.purify_test_expr(a)?;
                let purified_b = self.purify_test_expr(b)?;

                Ok(match test {
                    TestExpr::And(_, _) => {
                        TestExpr::And(Box::new(purified_a), Box::new(purified_b))
                    }
                    TestExpr::Or(_, _) => TestExpr::Or(Box::new(purified_a), Box::new(purified_b)),
                    _ => unreachable!(),
                })
            }

            TestExpr::Not(t) => {
                let purified_t = self.purify_test_expr(t)?;
                Ok(TestExpr::Not(Box::new(purified_t)))
            }
        }
    }

    pub(super) fn purify_arithmetic(&mut self, arith: &ArithExpr) -> PurificationResult<ArithExpr> {
        match arith {
            ArithExpr::Variable(name) => {
                if self.non_deterministic_vars.contains(name)
                    && self.options.remove_non_deterministic
                {
                    self.report.determinism_fixes.push(format!(
                        "Removed non-deterministic variable in arithmetic: {}",
                        name
                    ));
                    return Ok(ArithExpr::Number(0));
                }
                Ok(arith.clone())
            }

            ArithExpr::Add(a, b)
            | ArithExpr::Sub(a, b)
            | ArithExpr::Mul(a, b)
            | ArithExpr::Div(a, b)
            | ArithExpr::Mod(a, b) => {
                let purified_a = self.purify_arithmetic(a)?;
                let purified_b = self.purify_arithmetic(b)?;

                Ok(match arith {
                    ArithExpr::Add(_, _) => {
                        ArithExpr::Add(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Sub(_, _) => {
                        ArithExpr::Sub(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Mul(_, _) => {
                        ArithExpr::Mul(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Div(_, _) => {
                        ArithExpr::Div(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Mod(_, _) => {
                        ArithExpr::Mod(Box::new(purified_a), Box::new(purified_b))
                    }
                    _ => unreachable!(),
                })
            }

            ArithExpr::Number(_) => Ok(arith.clone()),
        }
    }
}
