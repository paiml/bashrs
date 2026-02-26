// Report generation for Makefile purification transformations
//
// Generates human-readable reports from transformation results
// and classifies transformations by safety level.

use super::Transformation;

/// Check if transformation can be applied safely
pub(super) fn is_safe_transformation(transformation: &Transformation) -> bool {
    match transformation {
        Transformation::WrapWithSort { safe, .. } => *safe,
        Transformation::AddComment { safe, .. } => *safe,
        // Sprint 83 - Parallel Safety
        Transformation::RecommendNotParallel { safe, .. } => *safe,
        Transformation::DetectRaceCondition { safe, .. } => *safe,
        Transformation::RecommendOrderOnlyPrereq { safe, .. } => *safe,
        Transformation::DetectMissingDependency { safe, .. } => *safe,
        Transformation::DetectOutputConflict { safe, .. } => *safe,
        Transformation::RecommendRecursiveMakeHandling { safe, .. } => *safe,
        Transformation::DetectDirectoryRace { safe, .. } => *safe,
        // Sprint 83 - Reproducible Builds (Day 4)
        Transformation::DetectTimestamp { safe, .. } => *safe,
        Transformation::DetectRandom { safe, .. } => *safe,
        Transformation::DetectProcessId { safe, .. } => *safe,
        Transformation::SuggestSourceDateEpoch { safe, .. } => *safe,
        Transformation::DetectNonDeterministicCommand { safe, .. } => *safe,
        // Sprint 83 - Performance Optimization (Day 5)
        Transformation::SuggestCombineShellInvocations { safe, .. } => *safe,
        Transformation::SuggestSimpleExpansion { safe, .. } => *safe,
        Transformation::RecommendSuffixes { safe, .. } => *safe,
        Transformation::DetectSequentialRecipes { safe, .. } => *safe,
        Transformation::SuggestPatternRule { safe, .. } => *safe,
        // Sprint 83 - Error Handling (Day 6)
        Transformation::DetectMissingErrorHandling { safe, .. } => *safe,
        Transformation::DetectSilentFailure { safe, .. } => *safe,
        Transformation::RecommendDeleteOnError { safe, .. } => *safe,
        Transformation::RecommendOneshell { safe, .. } => *safe,
        Transformation::DetectMissingSetE { safe, .. } => *safe,
        Transformation::DetectLoopWithoutErrorHandling { safe, .. } => *safe,
        // Sprint 83 - Portability (Day 7)
        Transformation::DetectBashism { safe, .. } => *safe,
        Transformation::DetectPlatformSpecific { safe, .. } => *safe,
        Transformation::DetectShellSpecific { safe, .. } => *safe,
        Transformation::DetectNonPortableFlags { safe, .. } => *safe,
        Transformation::DetectNonPortableEcho { safe, .. } => *safe,
    }
}

/// Generate human-readable report of transformations
pub(super) fn generate_report(transformations: &[Transformation]) -> Vec<String> {
    transformations.iter().map(format_transformation).collect()
}

fn format_transformation(t: &Transformation) -> String {
    match t {
        Transformation::WrapWithSort {
            variable_name,
            pattern,
            ..
        } => {
            format!(
                "✅ Wrapped {} in variable '{}' with $(sort ...)",
                pattern, variable_name
            )
        }
        Transformation::AddComment {
            variable_name,
            rule,
            ..
        } => {
            format!(
                "⚠️  Manual fix needed for variable '{}': {}",
                variable_name, rule
            )
        }
        // Parallel safety
        Transformation::RecommendNotParallel { reason, .. } => {
            format!("⚠️  Parallel safety: {} (.NOTPARALLEL)", reason)
        }
        Transformation::DetectRaceCondition {
            target_names,
            conflicting_file,
            ..
        } => {
            format!(
                "⚠️  Race condition detected: targets {:?} write to same file '{}'",
                target_names, conflicting_file
            )
        }
        Transformation::RecommendOrderOnlyPrereq {
            target_name,
            prereq_name,
            reason,
            ..
        } => {
            format!(
                "⚠️  Recommend order-only prerequisite for '{}': add | {} ({})",
                target_name, prereq_name, reason
            )
        }
        Transformation::DetectMissingDependency {
            target_name,
            missing_file,
            provider_target,
            ..
        } => {
            format!("⚠️  Missing dependency: '{}' uses '{}' created by '{}', but '{}' is not in prerequisites", target_name, missing_file, provider_target, provider_target)
        }
        Transformation::DetectOutputConflict { output_file, .. } => {
            format!(
                "⚠️  Output conflict: multiple targets write to same output file '{}'",
                output_file
            )
        }
        Transformation::RecommendRecursiveMakeHandling {
            target_name,
            subdirs,
            ..
        } => {
            format!(
                "⚠️  Recursive make in '{}': consider dependencies for subdirs {:?} ($(MAKE))",
                target_name, subdirs
            )
        }
        Transformation::DetectDirectoryRace {
            target_names,
            directory,
            ..
        } => {
            format!(
                "⚠️  Directory creation race: targets {:?} create directory '{}'",
                target_names, directory
            )
        }
        // Reproducible builds / performance / error handling / portability
        _ => format_analysis_transformation(t),
    }
}

fn format_analysis_transformation(t: &Transformation) -> String {
    match t {
        Transformation::DetectTimestamp {
            variable_name,
            pattern,
            ..
        } => {
            format!(
                "⚠️  Non-deterministic timestamp in '{}': {} - consider using SOURCE_DATE_EPOCH",
                variable_name, pattern
            )
        }
        Transformation::DetectRandom { variable_name, .. } => {
            format!(
                "⚠️  Non-deterministic $RANDOM in '{}' - use fixed seed or version number",
                variable_name
            )
        }
        Transformation::DetectProcessId { variable_name, .. } => {
            format!(
                "⚠️  Non-deterministic process ID ($$) in '{}' - use fixed temporary file name",
                variable_name
            )
        }
        Transformation::SuggestSourceDateEpoch { variable_name, .. } => {
            format!(
                "💡 Suggestion: Use SOURCE_DATE_EPOCH for reproducible timestamps in '{}'",
                variable_name
            )
        }
        Transformation::DetectNonDeterministicCommand {
            variable_name,
            command,
            reason,
            ..
        } => {
            format!(
                "⚠️  Non-deterministic command in '{}': {} - {}",
                variable_name, command, reason
            )
        }
        Transformation::SuggestCombineShellInvocations {
            target_name,
            recipe_count,
            ..
        } => {
            format!("💡 Performance: Combine {} shell invocations in '{}' using && or ; to reduce subshell spawns", recipe_count, target_name)
        }
        Transformation::SuggestSimpleExpansion {
            variable_name,
            reason,
            ..
        } => {
            format!(
                "💡 Performance: {} for variable '{}'",
                reason, variable_name
            )
        }
        Transformation::RecommendSuffixes { reason, .. } => {
            format!("💡 Performance: {}", reason)
        }
        Transformation::DetectSequentialRecipes {
            target_name,
            recipe_count,
            ..
        } => {
            format!("⚠️  Performance: Target '{}' has {} sequential recipe lines - consider combining with && or ;", target_name, recipe_count)
        }
        Transformation::SuggestPatternRule {
            pattern,
            target_count,
            ..
        } => {
            format!(
                "💡 Performance: {} explicit rules could use pattern rule '{}'",
                target_count, pattern
            )
        }
        Transformation::DetectMissingErrorHandling {
            target_name,
            command,
            ..
        } => {
            format!("⚠️  Error handling: Target '{}' has command without error handling: '{}' - consider adding || exit 1", target_name, command)
        }
        Transformation::DetectSilentFailure {
            target_name,
            command,
            ..
        } => {
            format!("⚠️  Error handling: Target '{}' has @ prefix that may hide errors: '{}' - consider removing @", target_name, command)
        }
        Transformation::RecommendDeleteOnError { reason, .. } => {
            format!("💡 Error handling: {}", reason)
        }
        Transformation::RecommendOneshell {
            target_name,
            reason,
            ..
        } => {
            format!("💡 Error handling: Target '{}' - {}", target_name, reason)
        }
        Transformation::DetectMissingSetE {
            target_name,
            command,
            ..
        } => {
            format!("⚠️  Error handling: Target '{}' has bash -c without set -e: '{}' - add 'set -e;' to fail on errors", target_name, command)
        }
        Transformation::DetectLoopWithoutErrorHandling {
            target_name,
            loop_command,
            ..
        } => {
            format!("⚠️  Error handling: Target '{}' has loop without error handling: '{}' - add || exit 1 inside loop", target_name, loop_command)
        }
        Transformation::DetectBashism {
            target_name,
            construct,
            posix_alternative,
            ..
        } => {
            format!(
                "⚠️  Portability: Target '{}' uses bashism '{}' - {}",
                target_name, construct, posix_alternative
            )
        }
        Transformation::DetectPlatformSpecific {
            target_name,
            command,
            reason,
            ..
        } => {
            format!(
                "⚠️  Portability: Target '{}' uses platform-specific command '{}' - {}",
                target_name, command, reason
            )
        }
        Transformation::DetectShellSpecific {
            target_name,
            feature,
            posix_alternative,
            ..
        } => {
            format!(
                "⚠️  Portability: Target '{}' uses shell-specific feature '{}' - {}",
                target_name, feature, posix_alternative
            )
        }
        Transformation::DetectNonPortableFlags {
            target_name,
            command,
            flag,
            reason,
            ..
        } => {
            format!(
                "⚠️  Portability: Target '{}' uses non-portable flag '{}' in '{}' - {}",
                target_name, flag, command, reason
            )
        }
        Transformation::DetectNonPortableEcho {
            target_name,
            command,
            ..
        } => {
            format!("⚠️  Portability: Target '{}' uses non-portable echo usage '{}' - use printf for portability", target_name, command)
        }
        // Handled by format_transformation
        _ => String::new(),
    }
}
