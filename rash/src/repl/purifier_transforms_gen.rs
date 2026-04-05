//! Purifier transform generators — extracted for file health.

use super::purifier::purify_bash;
use super::purifier_transforms::{Alternative, TransformationCategory, TransformationExplanation};

/// Generate alternatives for safety transformations
pub fn generate_safety_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("quot") || title.contains("variable") => vec![
            Alternative::new(
                "Use printf %q for shell-safe quoting",
                "safe=$(printf %q \"$variable\")",
                "When you need shell-escaped values",
            )
            .add_pro("Automatically escapes special characters")
            .add_pro("Safe for eval")
            .add_con("Bash-specific (not POSIX)")
            .add_con("Output may have backslashes"),
            Alternative::new(
                "Use arrays instead of strings",
                "args=(\"$var1\" \"$var2\"); command \"${args[@]}\"",
                "When handling multiple arguments",
            )
            .add_pro("Preserves word boundaries correctly")
            .add_pro("No quoting issues")
            .add_con("Bash-specific (not POSIX)")
            .add_con("More complex syntax"),
            Alternative::new(
                "Validate input before use",
                "if [[ $var =~ ^[a-zA-Z0-9_-]+$ ]]; then cmd \"$var\"; fi",
                "When you can restrict input to safe characters",
            )
            .add_pro("Explicit validation")
            .add_pro("Clear error handling")
            .add_con("May reject valid inputs")
            .add_con("Requires input constraints"),
        ],

        _ => vec![Alternative::new(
            "Use safer built-in alternatives",
            "# Use bash built-ins when possible",
            "When avoiding external commands",
        )
        .add_pro("No command injection risk")
        .add_con("Limited functionality")],
    }
}

/// Format alternatives for display
pub fn format_alternatives(alternatives: &[Alternative]) -> String {
    let mut output = String::new();

    if alternatives.is_empty() {
        return output;
    }

    output.push_str("Alternative Approaches:\n\n");

    for (i, alt) in alternatives.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, alt.approach));
        output.push_str(&format!("   Example: {}\n", alt.example));
        output.push_str(&format!("   When to use: {}\n", alt.when_to_use));

        if !alt.pros.is_empty() {
            output.push_str("   Pros:\n");
            for pro in &alt.pros {
                output.push_str(&format!("     + {}\n", pro));
            }
        }

        if !alt.cons.is_empty() {
            output.push_str("   Cons:\n");
            for con in &alt.cons {
                output.push_str(&format!("     - {}\n", con));
            }
        }

        output.push('\n');
    }

    output
}

/// Explain what changed during purification with detailed transformations
pub fn explain_purification_changes_detailed(
    original: &str,
) -> anyhow::Result<Vec<TransformationExplanation>> {
    // Purify the bash code
    let purified = purify_bash(original)?;

    // If no changes, return empty vector
    if original.trim() == purified.trim() {
        return Ok(Vec::new());
    }

    // Analyze the changes and generate detailed explanations
    let mut explanations = Vec::new();

    // Check for mkdir -p addition (Idempotency)
    if original.contains("mkdir") && !original.contains("mkdir -p") && purified.contains("mkdir -p")
    {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir → mkdir -p",
            original,
            &purified,
            "Added -p flag to mkdir command",
            "Makes directory creation safe to re-run. Won't fail if directory already exists.",
        ));
    }

    // Check for rm -f addition (Idempotency)
    if original.contains("rm ") && !original.contains("rm -f") && purified.contains("rm -f") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "rm → rm -f",
            original,
            &purified,
            "Added -f flag to rm command",
            "Makes file deletion safe to re-run. Won't fail if file doesn't exist.",
        ));
    }

    // Check for variable quoting (Safety)
    let original_has_unquoted = original.contains('$') && !original.contains("\"$");
    let purified_has_quoted = purified.contains("\"$");
    if original_has_unquoted && purified_has_quoted {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            original,
            &purified,
            "Added quotes around variables",
            "Prevents word splitting and glob expansion. Protects against injection attacks.",
        ));
    }

    // Check for ln -sf addition (Idempotency)
    if original.contains("ln -s") && !original.contains("ln -sf") && purified.contains("ln -sf") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "ln -s → ln -sf",
            original,
            &purified,
            "Added -f flag to ln -s command",
            "Makes symlink creation safe to re-run. Forces replacement if link already exists.",
        ));
    }

    // Check for $RANDOM removal (Determinism)
    if original.contains("$RANDOM") && !purified.contains("$RANDOM") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove $RANDOM",
            original,
            &purified,
            "Removed $RANDOM variable usage",
            "Non-deterministic values make scripts unpredictable and unreproducible.",
        ));
    }

    // Check for timestamp removal (Determinism)
    if (original.contains("date") || original.contains("$SECONDS"))
        && (!purified.contains("date") || !purified.contains("$SECONDS"))
    {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove timestamps",
            original,
            &purified,
            "Removed time-based values (date, $SECONDS)",
            "Time-based values make scripts non-reproducible across different runs.",
        ));
    }

    Ok(explanations)
}

/// Format a detailed transformation report from transformation explanations
pub fn format_transformation_report(transformations: &[TransformationExplanation]) -> String {
    if transformations.is_empty() {
        return "No transformations applied - code is already purified.".to_string();
    }

    let mut report = String::from("Transformation Report\n");
    report.push_str("====================\n\n");

    for (i, transformation) in transformations.iter().enumerate() {
        if i > 0 {
            report.push_str("\n\n");
        }

        // Category header
        let category_name = match transformation.category {
            TransformationCategory::Idempotency => "IDEMPOTENCY",
            TransformationCategory::Determinism => "DETERMINISM",
            TransformationCategory::Safety => "SAFETY",
        };
        report.push_str(&format!("CATEGORY: {}\n", category_name));
        report.push_str("------------------------\n");

        // Title and details
        report.push_str(&format!("Title: {}\n", transformation.title));
        report.push_str(&format!("What changed: {}\n", transformation.what_changed));
        report.push_str(&format!(
            "Why it matters: {}\n",
            transformation.why_it_matters
        ));

        // Line number if present
        if let Some(line) = transformation.line_number {
            report.push_str(&format!("Line: {}\n", line));
        }

        // Original and transformed code
        report.push_str("\nOriginal:\n");
        for line in transformation.original.lines() {
            report.push_str(&format!("  {}\n", line));
        }

        report.push_str("\nTransformed:\n");
        for line in transformation.transformed.lines() {
            report.push_str(&format!("  {}\n", line));
        }
    }

    report
}
