/// Lint profiles for specialized validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintProfile {
    /// Standard Dockerfile linting (default)
    #[default]
    Standard,
    /// Coursera Labs image validation
    Coursera,
    /// Dev Container validation (devcontainer.json + Dockerfile)
    DevContainer,
}

impl std::str::FromStr for LintProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" | "default" => Ok(LintProfile::Standard),
            "coursera" | "coursera-labs" => Ok(LintProfile::Coursera),
            "devcontainer" | "dev-container" => Ok(LintProfile::DevContainer),
            _ => Err(format!(
                "Unknown profile: {}. Valid profiles: standard, coursera, devcontainer",
                s
            )),
        }
    }
}

impl std::fmt::Display for LintProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintProfile::Standard => write!(f, "standard"),
            LintProfile::Coursera => write!(f, "coursera"),
            LintProfile::DevContainer => write!(f, "devcontainer"),
        }
    }
}

/// Lint a Dockerfile and return all diagnostics
pub fn lint_dockerfile(source: &str) -> LintResult {
    lint_dockerfile_with_profile(source, LintProfile::Standard)
}

/// Lint a Dockerfile with a specific profile
pub fn lint_dockerfile_with_profile(source: &str, profile: LintProfile) -> LintResult {
    let mut result = LintResult::new();

    // Run standard Dockerfile rules (inspired by hadolint)
    result.merge(docker001::check(source)); // Missing USER directive (DL3002)
    result.merge(docker002::check(source)); // Unpinned base images (DL3006, DL3007)
    result.merge(docker003::check(source)); // Missing apt cleanup (DL3009)
    result.merge(docker004::check(source)); // Invalid COPY --from (DL3022)
    result.merge(docker005::check(source)); // Missing --no-install-recommends (DL3015)
    result.merge(docker006::check(source)); // Use COPY not ADD (DL3020)

    // Run profile-specific rules
    match profile {
        LintProfile::Standard => {
            // Standard rules only (already added above)
        }
        LintProfile::Coursera => {
            // Coursera Labs-specific rules
            result.merge(coursera::lint_dockerfile_coursera(source));
        }
        LintProfile::DevContainer => {
            // Dev Container rules (to be implemented)
            // result.merge(devcontainer::lint_dockerfile_devcontainer(source));
        }
    }

    result
}

/// Lint a Makefile and return all diagnostics
pub fn lint_makefile(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Preprocess Makefile to convert $$ → $ in recipes for shell linting
    use crate::linter::make_preprocess::preprocess_for_linting;
    let preprocessed = preprocess_for_linting(source);

    // Run Makefile-specific rules on original source
    result.merge(make001::check(source));
    result.merge(make002::check(source));
    result.merge(make003::check(source));
    result.merge(make004::check(source));
    result.merge(make005::check(source));
    result.merge(make006::check(source));
    result.merge(make007::check(source));
    result.merge(make008::check(source)); // CRITICAL: Tab vs spaces
    result.merge(make009::check(source));
    result.merge(make010::check(source));
    result.merge(make011::check(source));
    result.merge(make012::check(source));
    result.merge(make013::check(source));
    result.merge(make014::check(source));
    result.merge(make015::check(source));
    result.merge(make016::check(source));
    result.merge(make017::check(source));
    result.merge(make018::check(source));
    result.merge(make019::check(source));
    result.merge(make020::check(source));

    // Run shell linting rules on preprocessed source
    // This prevents false positives from Make's $$ escaping
    result.merge(sc2133::check(&preprocessed));
    result.merge(sc2168::check(&preprocessed));
    result.merge(sc2299::check(&preprocessed));

    // For DET002, we want to allow timestamps in Makefiles
    // (they're used for build tracking), so we don't run it

    result
}

#[cfg(test)]
#[path = "docker003_tests.rs"]
mod docker003_tests;

#[cfg(test)]
#[path = "docker004_tests.rs"]
mod docker004_tests;

#[cfg(test)]
#[path = "mod_tests_lint_profile.rs"]
mod tests_extracted;
