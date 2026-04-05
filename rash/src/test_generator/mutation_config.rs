//! Mutation Test Configuration Generation (Sprint 3)
//!
//! Generates .cargo-mutants.toml configuration based on code complexity

use super::core::TestGenResult;
use crate::bash_parser::ast::*;
use std::collections::HashMap;

pub struct MutationConfigGenerator {
    /// Target mutation score (0.0 - 1.0)
    target_score: f64,
    /// Base timeout in seconds
    base_timeout: u64,
}

impl Default for MutationConfigGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MutationConfigGenerator {
    pub fn new() -> Self {
        Self {
            target_score: 0.85,
            base_timeout: 60,
        }
    }

    /// Generate mutation test configuration
    pub fn generate_config(&self, ast: &BashAst) -> TestGenResult<String> {
        let complexity = self.analyze_complexity(ast);
        let config = self.build_config(ast, &complexity)?;
        Ok(config.to_toml())
    }

    /// Analyze code complexity to determine configuration
    fn analyze_complexity(&self, ast: &BashAst) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::default();

        for stmt in &ast.statements {
            self.analyze_statement(stmt, &mut metrics);
        }

        metrics
    }

    /// Analyze a single statement for complexity
    fn analyze_statement(&self, stmt: &BashStmt, metrics: &mut ComplexityMetrics) {
        match stmt {
            BashStmt::Function { name, body, .. } => {
                metrics.function_count += 1;
                metrics.total_lines += body.len();

                let func_complexity = self.calculate_cyclomatic_complexity(body);
                metrics.functions.insert(name.clone(), func_complexity);

                if func_complexity > 10 {
                    metrics.critical_functions.push(name.clone());
                }

                for stmt in body {
                    self.analyze_statement(stmt, metrics);
                }
            }
            BashStmt::If {
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                metrics.branch_count += 1;
                metrics.total_lines += then_block.len();

                for (_, block) in elif_blocks {
                    metrics.branch_count += 1;
                    metrics.total_lines += block.len();
                }

                if let Some(block) = else_block {
                    metrics.total_lines += block.len();
                }
            }
            BashStmt::While { body, .. } | BashStmt::For { body, .. } => {
                metrics.loop_count += 1;
                metrics.total_lines += body.len();

                for stmt in body {
                    self.analyze_statement(stmt, metrics);
                }
            }
            BashStmt::Assignment { value, .. } => {
                if matches!(value, BashExpr::Arithmetic(_)) {
                    metrics.arithmetic_ops += 1;
                }
            }
            _ => {}
        }
    }

    /// Calculate cyclomatic complexity for a function body
    fn calculate_cyclomatic_complexity(&self, body: &[BashStmt]) -> usize {
        let mut complexity = 1; // Base complexity

        for stmt in body {
            match stmt {
                BashStmt::If { elif_blocks, .. } => {
                    complexity += 1 + elif_blocks.len();
                }
                BashStmt::While { .. } | BashStmt::For { .. } => {
                    complexity += 1;
                }
                _ => {}
            }
        }

        complexity
    }

    /// Build configuration based on complexity metrics
    fn build_config(
        &self,
        _ast: &BashAst,
        complexity: &ComplexityMetrics,
    ) -> TestGenResult<MutationConfig> {
        // Determine timeout based on code size
        let timeout = self.calculate_timeout(complexity);

        // Determine parallel jobs based on complexity
        let parallel_jobs = self.calculate_parallel_jobs(complexity);

        // Determine which operators to enable
        let operators = self.select_operators(complexity);

        // Identify critical paths
        let critical_paths = complexity.critical_functions.clone();

        Ok(MutationConfig {
            operators,
            timeout,
            parallel_jobs,
            target_score: self.target_score,
            critical_paths,
            exclude_patterns: vec![
                "tests/*".to_string(),
                "*_test.rs".to_string(),
                "*/tests.rs".to_string(),
            ],
        })
    }

    /// Calculate appropriate timeout based on complexity
    fn calculate_timeout(&self, complexity: &ComplexityMetrics) -> u64 {
        let base = self.base_timeout;

        // Add time for each function (5 seconds per function)
        let function_overhead = complexity.function_count as u64 * 5;

        // Add time for loops (10 seconds per loop)
        let loop_overhead = complexity.loop_count as u64 * 10;

        base + function_overhead + loop_overhead
    }

    /// Calculate optimal number of parallel jobs
    fn calculate_parallel_jobs(&self, complexity: &ComplexityMetrics) -> usize {
        // Scale jobs based on complexity
        if complexity.function_count > 20 {
            8
        } else if complexity.function_count > 10 {
            4
        } else {
            2
        }
    }

    /// Select mutation operators based on code patterns
    fn select_operators(&self, complexity: &ComplexityMetrics) -> Vec<MutationOperator> {
        let mut operators = vec![MutationOperator::ReturnValue, MutationOperator::Conditional];

        if complexity.arithmetic_ops > 0 {
            operators.push(MutationOperator::ArithmeticOp);
        }

        if complexity.branch_count > 0 {
            operators.push(MutationOperator::RelationalOp);
            operators.push(MutationOperator::BooleanOp);
        }

        operators
    }
}

#[derive(Debug, Clone, Default)]
struct ComplexityMetrics {
    function_count: usize,
    branch_count: usize,
    loop_count: usize,
    arithmetic_ops: usize,
    total_lines: usize,
    functions: HashMap<String, usize>,
    critical_functions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MutationConfig {
    pub operators: Vec<MutationOperator>,
    pub timeout: u64,
    pub parallel_jobs: usize,
    pub target_score: f64,
    pub critical_paths: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl MutationConfig {
    /// Convert to TOML configuration format
    pub fn to_toml(&self) -> String {
        let mut config = String::from("# Generated mutation test configuration\n");
        config.push_str("# Auto-generated based on code complexity analysis\n\n");

        // Global settings
        config.push_str(&format!("timeout = {}\n", self.timeout));
        config.push_str(&format!("jobs = {}\n", self.parallel_jobs));
        config.push_str(&format!(
            "# Target mutation score: {:.0}%\n\n",
            self.target_score * 100.0
        ));

        // Exclude patterns
        if !self.exclude_patterns.is_empty() {
            config.push_str("exclude_globs = [\n");
            for pattern in &self.exclude_patterns {
                config.push_str(&format!("    \"{}\",\n", pattern));
            }
            config.push_str("]\n\n");
        }

        // Operators
        config.push_str("# Mutation operators to apply\n");
        for operator in &self.operators {
            match operator {
                MutationOperator::ArithmeticOp => {
                    config.push_str("# Arithmetic: +, -, *, /, %\n");
                }
                MutationOperator::RelationalOp => {
                    config.push_str("# Relational: <, <=, >, >=, ==, !=\n");
                }
                MutationOperator::BooleanOp => {
                    config.push_str("# Boolean: &&, ||, !\n");
                }
                MutationOperator::ReturnValue => {
                    config.push_str("# Return values\n");
                }
                MutationOperator::Conditional => {
                    config.push_str("# Conditionals: if/else\n");
                }
            }
        }

        // Critical paths
        if !self.critical_paths.is_empty() {
            config.push_str("\n# High-complexity functions requiring extra attention:\n");
            for func in &self.critical_paths {
                config.push_str(&format!("# - {}\n", func));
            }
        }

        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationOperator {
    ArithmeticOp,
    RelationalOp,
    BooleanOp,
    ReturnValue,
    Conditional,
}

