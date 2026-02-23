// Error handling analysis for Makefiles (Sprint 83 - Day 6)
//
// Detects missing error handling patterns: unchecked commands, silent failures,
// missing .ONESHELL, missing set -e, loops without error handling.

use super::{MakeAst, MakeItem, Transformation};

/// Check if Makefile has .DELETE_ON_ERROR directive
fn has_delete_on_error_directive(ast: &MakeAst) -> bool {
    ast.items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".DELETE_ON_ERROR"))
}

/// Collect all targets for analysis (both regular targets and pattern rules)
fn collect_targets_and_patterns(ast: &MakeAst) -> Vec<(&String, &Vec<String>)> {
    let mut targets = Vec::new();
    for item in &ast.items {
        match item {
            MakeItem::Target { name, recipe, .. } => {
                targets.push((name, recipe));
            }
            MakeItem::PatternRule {
                target_pattern,
                recipe,
                ..
            } => {
                targets.push((target_pattern, recipe));
            }
            _ => {}
        }
    }
    targets
}

/// Detect commands without error handling
fn detect_missing_error_handling_in_commands(
    targets: &[(&String, &Vec<String>)],
) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    let critical_commands = ["mkdir", "gcc", "cp", "mv"];

    for (target_name, recipes) in targets {
        for recipe in *recipes {
            let trimmed = recipe.trim();
            for cmd in &critical_commands {
                if trimmed.starts_with(cmd)
                    && !trimmed.starts_with(&format!("{} -", cmd))
                    && !recipe.contains("||")
                    && !recipe.contains("&&")
                {
                    transformations.push(Transformation::DetectMissingErrorHandling {
                        target_name: (*target_name).clone(),
                        command: trimmed.to_string(),
                        safe: false,
                    });
                }
            }
        }
    }
    transformations
}

/// Detect @ prefix that may hide errors
fn detect_silent_failures(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.trim().starts_with('@') {
                let without_at = recipe.trim().trim_start_matches('@').trim();
                if !without_at.starts_with("echo") {
                    transformations.push(Transformation::DetectSilentFailure {
                        target_name: (*target_name).clone(),
                        command: recipe.trim().to_string(),
                        safe: false,
                    });
                }
            }
        }
    }
    transformations
}

/// Detect multiline recipes without .ONESHELL
fn detect_missing_oneshell(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        if recipes.len() >= 2 {
            let has_cd = recipes.iter().any(|r| r.trim().starts_with("cd "));
            let has_command_separator = recipes.iter().any(|r| r.contains("&&") || r.contains(";"));

            if has_cd && !has_command_separator {
                transformations.push(Transformation::RecommendOneshell {
                    target_name: (*target_name).clone(),
                    reason:
                        "Use .ONESHELL or combine commands with && to ensure cd works across lines"
                            .to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect bash -c without set -e
fn detect_missing_set_e(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.contains("bash -c") && !recipe.contains("set -e") {
                transformations.push(Transformation::DetectMissingSetE {
                    target_name: (*target_name).clone(),
                    command: recipe.trim().to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect for loops without error handling
fn detect_loop_without_error_handling(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.contains("for ")
                && recipe.contains("do ")
                && !recipe.contains("|| exit")
                && !recipe.contains("|| return")
            {
                transformations.push(Transformation::DetectLoopWithoutErrorHandling {
                    target_name: (*target_name).clone(),
                    loop_command: recipe.trim().to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Check if .DELETE_ON_ERROR should be recommended
fn should_recommend_delete_on_error_directive(
    has_delete_on_error: bool,
    transformations: &[Transformation],
    targets: &[(&String, &Vec<String>)],
) -> bool {
    !has_delete_on_error && !transformations.is_empty() && !targets.is_empty()
}

pub(super) fn analyze_error_handling(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    let has_delete_on_error = has_delete_on_error_directive(ast);
    let targets = collect_targets_and_patterns(ast);

    // Detect commands without error handling
    transformations.extend(detect_missing_error_handling_in_commands(&targets));

    // Detect silent failures
    transformations.extend(detect_silent_failures(&targets));

    // Detect missing .ONESHELL
    transformations.extend(detect_missing_oneshell(&targets));

    // Detect missing set -e
    transformations.extend(detect_missing_set_e(&targets));

    // Detect loops without error handling
    transformations.extend(detect_loop_without_error_handling(&targets));

    // Recommend .DELETE_ON_ERROR if issues detected
    if should_recommend_delete_on_error_directive(has_delete_on_error, &transformations, &targets) {
        transformations.push(Transformation::RecommendDeleteOnError {
            reason: "Add .DELETE_ON_ERROR to automatically remove targets if recipe fails"
                .to_string(),
            safe: false,
        });
    }

    transformations
}
