/// Generate alternatives for idempotency transformations
pub fn generate_idempotency_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("mkdir") && title.contains("-p") => vec![
            Alternative::new(
                "Check before creating",
                "[ -d /path ] || mkdir /path",
                "When you want explicit control over error handling",
            )
            .add_pro("Explicit about what's happening")
            .add_pro("Can add custom logic between check and creation")
            .add_con("Not atomic - race condition between check and create")
            .add_con("More verbose than mkdir -p"),
            Alternative::new(
                "Use mkdir with error suppression",
                "mkdir /path 2>/dev/null || true",
                "When you don't care about the reason for failure",
            )
            .add_pro("Simple and concise")
            .add_pro("Idempotent")
            .add_con("Hides all errors, not just 'already exists'")
            .add_con("Can mask real problems like permission issues"),
        ],

        title if title.contains("rm") && title.contains("-f") => vec![
            Alternative::new(
                "Check before removing",
                "[ -e /path ] && rm /path",
                "When you want to know if the file existed",
            )
            .add_pro("Explicit about file existence")
            .add_pro("Can add logging or side effects")
            .add_con("Not atomic - race condition")
            .add_con("More verbose"),
            Alternative::new(
                "Use rm with error check",
                "rm /path 2>/dev/null || true",
                "When you want to suppress errors but keep other rm behavior",
            )
            .add_pro("Simple")
            .add_pro("Idempotent")
            .add_con("Hides all errors")
            .add_con("May mask permission problems"),
        ],

        title if title.contains("ln") && title.contains("-sf") => vec![
            Alternative::new(
                "Remove then create",
                "rm -f /link && ln -s /target /link",
                "When you need two separate operations",
            )
            .add_pro("Very explicit")
            .add_pro("Can add logic between removal and creation")
            .add_con("Not atomic - window where link doesn't exist")
            .add_con("More verbose"),
            Alternative::new(
                "Check before creating",
                "[ -L /link ] || ln -s /target /link",
                "When you want to preserve existing links",
            )
            .add_pro("Won't overwrite existing links")
            .add_pro("Explicit check")
            .add_con("Not idempotent if link points elsewhere")
            .add_con("Race condition between check and create"),
        ],

        _ => vec![Alternative::new(
            "Add explicit idempotency check",
            "[ condition ] || operation",
            "When you want fine-grained control",
        )
        .add_pro("Explicit about preconditions")
        .add_con("Not atomic")],
    }
}

/// Generate alternatives for determinism transformations
pub fn generate_determinism_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("RANDOM") => vec![
            Alternative::new(
                "Use UUID for unique IDs",
                "id=$(uuidgen)  # or $(cat /proc/sys/kernel/random/uuid)",
                "When you need globally unique identifiers",
            )
            .add_pro("Guaranteed unique across machines")
            .add_pro("Standard format")
            .add_pro("Deterministic if you control the seed")
            .add_con("Requires uuidgen or /proc/sys/kernel")
            .add_con("Longer than simple numbers"),
            Alternative::new(
                "Use timestamp-based IDs",
                "id=$(date +%s%N)  # nanoseconds since epoch",
                "When you need time-ordered IDs",
            )
            .add_pro("Sortable by time")
            .add_pro("No external dependencies")
            .add_con("Not unique across machines")
            .add_con("Still non-deterministic (but reproducible with fixed time)"),
            Alternative::new(
                "Use hash of inputs",
                "id=$(echo \"$input\" | sha256sum | cut -d' ' -f1)",
                "When you want IDs derived from content",
            )
            .add_pro("Truly deterministic")
            .add_pro("Same input = same ID")
            .add_con("Requires sha256sum")
            .add_con("Collisions possible (though extremely rare)"),
            Alternative::new(
                "Use sequential counter",
                "id=$((++counter))  # with state file",
                "When you need simple incrementing IDs",
            )
            .add_pro("Simple and predictable")
            .add_pro("Compact")
            .add_con("Requires state management")
            .add_con("Not unique across processes without locking"),
        ],

        title if title.contains("timestamp") || title.contains("date") => vec![
            Alternative::new(
                "Use explicit version parameter",
                "version=$1  # Pass version as argument",
                "When version is known at invocation time",
            )
            .add_pro("Fully deterministic")
            .add_pro("Version controlled externally")
            .add_con("Requires coordination with caller"),
            Alternative::new(
                "Use git commit hash",
                "version=$(git rev-parse --short HEAD)",
                "When deploying from git repository",
            )
            .add_pro("Deterministic for given commit")
            .add_pro("Traceable to source code")
            .add_con("Requires git repository")
            .add_con("Not available in all environments"),
            Alternative::new(
                "Use build number from CI",
                "version=${BUILD_NUMBER:-dev}",
                "When running in CI/CD pipeline",
            )
            .add_pro("Integrates with CI/CD")
            .add_pro("Incrementing version numbers")
            .add_con("Requires CI environment")
            .add_con("May not be available locally"),
        ],

        _ => vec![Alternative::new(
            "Make value an input parameter",
            "value=$1  # Pass as argument",
            "When value should be controlled by caller",
        )
        .add_pro("Fully deterministic")
        .add_con("Requires caller to provide value")],
    }
}

// Re-export functions moved to purifier_transforms_gen.rs
pub use super::purifier_transforms_gen::{
    explain_purification_changes_detailed, format_alternatives, format_transformation_report,
    generate_safety_alternatives,
};

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "purifier_transforms_tests_inline.rs"]
// FIXME(PMAT-238): mod tests_inline;
