// Reproducible builds analysis for Makefiles (Sprint 83 - Day 4)
//
// Detects non-deterministic patterns that compromise build reproducibility:
// timestamps, $RANDOM, process IDs, hostname, git timestamps, mktemp.

use super::{MakeAst, MakeItem, Transformation};

/// Analyze Makefile for reproducible builds issues (Sprint 83 - Day 4)
pub(super) fn analyze_reproducible_builds(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    // Collect all variables for analysis
    for item in &ast.items {
        if let MakeItem::Variable { name, value, .. } = item {
            // Analysis 1: Detect $(shell date) patterns
            if value.contains("date") && (value.contains("$(shell") || value.contains("${shell")) {
                transformations.push(Transformation::DetectTimestamp {
                    variable_name: name.clone(),
                    pattern: value.clone(),
                    safe: false,
                });

                // Also suggest SOURCE_DATE_EPOCH
                transformations.push(Transformation::SuggestSourceDateEpoch {
                    variable_name: name.clone(),
                    original_pattern: value.clone(),
                    safe: false,
                });
            }

            // Analysis 2: Detect $RANDOM usage (Makefile syntax: $$RANDOM)
            if value.contains("$$RANDOM") || value.contains("$RANDOM") {
                transformations.push(Transformation::DetectRandom {
                    variable_name: name.clone(),
                    safe: false,
                });
            }

            // Analysis 3: Detect process ID $$ usage (Makefile syntax: $$$$)
            if value.contains("$$$$") {
                transformations.push(Transformation::DetectProcessId {
                    variable_name: name.clone(),
                    safe: false,
                });
            }

            // Analysis 4: Detect hostname command
            if value.contains("hostname")
                && (value.contains("$(shell") || value.contains("${shell"))
            {
                transformations.push(Transformation::DetectNonDeterministicCommand {
                    variable_name: name.clone(),
                    command: "hostname".to_string(),
                    reason: "hostname is environment-dependent and makes builds non-reproducible"
                        .to_string(),
                    safe: false,
                });
            }

            // Analysis 5: Detect git timestamp commands
            if value.contains("git")
                && value.contains("log")
                && (value.contains("%cd") || value.contains("--date"))
            {
                transformations.push(Transformation::DetectNonDeterministicCommand {
                    variable_name: name.clone(),
                    command: "git log timestamp".to_string(),
                    reason: "git commit timestamps are non-deterministic".to_string(),
                    safe: false,
                });
            }
        }

        // Check recipes for mktemp usage
        if let MakeItem::Target { name, recipe, .. } = item {
            for recipe_line in recipe {
                // Analysis 6: Detect mktemp usage
                if recipe_line.contains("mktemp") {
                    transformations.push(Transformation::DetectNonDeterministicCommand {
                        variable_name: name.clone(),
                        command: "mktemp".to_string(),
                        reason: "mktemp creates random temporary file names".to_string(),
                        safe: false,
                    });
                }
            }
        }
    }

    transformations
}
