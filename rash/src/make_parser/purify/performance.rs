// Performance optimization analysis for Makefiles (Sprint 83 - Day 5)
//
// Detects opportunities to improve Makefile build performance:
// recursive variable expansion, sequential recipes, pattern rule consolidation.

use super::{MakeAst, MakeItem, Transformation};
use crate::make_parser::ast::VarFlavor;

/// Detect variables using = with $(shell) that should use :=
fn detect_recursive_var_expansion(
    variables: &[(&String, &String, &VarFlavor)],
) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for (var_name, value, flavor) in variables {
        if matches!(flavor, VarFlavor::Recursive) {
            // Check if value contains $(shell) - should use :=
            if value.contains("$(shell") || value.contains("${shell") {
                transformations.push(Transformation::SuggestSimpleExpansion {
                    variable_name: (*var_name).clone(),
                    reason: "Use := instead of = to avoid re-expanding $(shell) multiple times"
                        .to_string(),
                    safe: false,
                });
            }
            // Check if value is simple (no variable references) - could use :=
            else if !value.contains("$(") && !value.contains("${") {
                transformations.push(Transformation::SuggestSimpleExpansion {
                    variable_name: (*var_name).clone(),
                    reason:
                        "Use := instead of = for simple variables to avoid unnecessary re-expansion"
                            .to_string(),
                    safe: false,
                });
            }
        }
    }

    transformations
}

/// Detect targets with multiple sequential recipe lines that could be combined
fn detect_sequential_recipes(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for (target_name, recipes) in targets {
        if recipes.len() >= 3 {
            // Check for sequential commands (not using && or ;)
            let has_command_separator = recipes.iter().any(|r| r.contains("&&") || r.contains(';'));
            if !has_command_separator {
                transformations.push(Transformation::DetectSequentialRecipes {
                    target_name: (*target_name).clone(),
                    recipe_count: recipes.len(),
                    safe: false,
                });

                transformations.push(Transformation::SuggestCombineShellInvocations {
                    target_name: (*target_name).clone(),
                    recipe_count: recipes.len(),
                    safe: false,
                });
            }
        }

        // Detect multiple rm commands that could be combined
        let rm_count = recipes
            .iter()
            .filter(|r| r.trim().starts_with("rm "))
            .count();
        if rm_count >= 2 {
            transformations.push(Transformation::DetectSequentialRecipes {
                target_name: (*target_name).clone(),
                recipe_count: rm_count,
                safe: false,
            });
        }
    }

    transformations
}

/// Detect repeated explicit rules that could be pattern rules
fn detect_pattern_rule_opportunities(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    // Group targets by their recipe pattern
    let mut rule_patterns: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (target_name, recipes) in targets {
        // Look for .o: .c pattern
        if target_name.ends_with(".o") && recipes.iter().any(|r| r.contains("-c")) {
            let pattern = "%.o: %.c compilation".to_string();
            rule_patterns
                .entry(pattern.clone())
                .or_default()
                .push((*target_name).clone());
        }
    }

    // Report if we found 3+ similar rules
    for (_pattern, target_names) in rule_patterns {
        if target_names.len() >= 3 {
            transformations.push(Transformation::SuggestPatternRule {
                pattern: "%.o: %.c".to_string(),
                target_count: target_names.len(),
                safe: false,
            });
        }
    }

    transformations
}

/// Analyze Makefile for performance optimization opportunities (Sprint 83 - Day 5)
pub(super) fn analyze_performance_optimization(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    // Check if Makefile has .SUFFIXES directive
    let has_suffixes = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".SUFFIXES"));

    // Collect all targets and variables for analysis
    let mut targets: Vec<(&String, &Vec<String>)> = Vec::new();
    let mut variables: Vec<(&String, &String, &VarFlavor)> = Vec::new();

    for item in &ast.items {
        if let MakeItem::Target { name, recipe, .. } = item {
            targets.push((name, recipe));
        } else if let MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } = item
        {
            variables.push((name, value, flavor));
        }
    }

    // Analysis 1: Detect variables using = with $(shell) that should use :=
    transformations.extend(detect_recursive_var_expansion(&variables));

    // Analysis 3+4: Detect sequential recipes and combinable commands
    transformations.extend(detect_sequential_recipes(&targets));

    // Analysis 5: Detect repeated explicit rules that could be pattern rules
    transformations.extend(detect_pattern_rule_opportunities(&targets));

    // Final Analysis: Recommend .SUFFIXES: only if we found other performance issues
    // This ensures idempotency - we don't recommend .SUFFIXES: on already-clean Makefiles
    if !has_suffixes && !targets.is_empty() && !transformations.is_empty() {
        transformations.push(Transformation::RecommendSuffixes {
            reason: "Add .SUFFIXES: to disable builtin rules for better performance".to_string(),
            safe: false,
        });
    }

    transformations
}
