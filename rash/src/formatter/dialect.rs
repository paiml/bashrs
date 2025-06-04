//! Shell dialect detection and compatibility system

use std::collections::HashMap;

/// Shell dialect variants with version information
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ShellDialect {
    #[default]
    Posix, // IEEE Std 1003.1-2017
    Bash5_2,    // GNU Bash 5.2.21
    Ksh93uPlus, // AT&T KornShell 93u+ 2012-08-01
    Zsh5_9,     // Z shell 5.9
    Dash0_5_12, // Debian Almquist Shell
    Inferred(Box<InferenceConfidence>),
}

/// Confidence scoring for dialect inference
#[derive(Debug, Clone, PartialEq)]
pub struct InferenceConfidence {
    pub dialect: Box<ShellDialect>,
    pub confidence: f32, // 0.0-1.0
    pub evidence: InferenceEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceEvidence {
    Shebang(&'static str),
    Syntax(SyntaxFeature),
    Builtins(BuiltinProfile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxFeature {
    BashArrays,       // array=( ... )
    BashProcessSubst, // <( ... )
    ZshGlobs,         // **/ recursive globs
    KshFunctions,     // function name { ... }
    PosixFunctions,   // name() { ... }
    BashConditionals, // [[ ... ]]
    BashArithmetic,   // (( ... ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinProfile {
    BashReadarray,
    ZshZparseopts,
    KshTypeset,
    DashLocal,
}

/// Core shell dialects for scoring (excludes Inferred variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreDialect {
    Posix,
    Bash5_2,
    Ksh93uPlus,
    Zsh5_9,
    Dash0_5_12,
}

impl CoreDialect {
    pub fn to_shell_dialect(self) -> ShellDialect {
        match self {
            CoreDialect::Posix => ShellDialect::Posix,
            CoreDialect::Bash5_2 => ShellDialect::Bash5_2,
            CoreDialect::Ksh93uPlus => ShellDialect::Ksh93uPlus,
            CoreDialect::Zsh5_9 => ShellDialect::Zsh5_9,
            CoreDialect::Dash0_5_12 => ShellDialect::Dash0_5_12,
        }
    }
}

/// Dialect scoring system for inference
pub struct DialectScorer {
    scores: HashMap<CoreDialect, f32>,
    evidence: Vec<(InferenceEvidence, f32)>,
}

impl DialectScorer {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        scores.insert(CoreDialect::Posix, 0.1); // Base score for POSIX compatibility
        scores.insert(CoreDialect::Bash5_2, 0.0);
        scores.insert(CoreDialect::Ksh93uPlus, 0.0);
        scores.insert(CoreDialect::Zsh5_9, 0.0);
        scores.insert(CoreDialect::Dash0_5_12, 0.0);

        Self {
            scores,
            evidence: Vec::new(),
        }
    }

    pub fn add_evidence(&mut self, evidence: InferenceEvidence, weight: f32) {
        self.evidence.push((evidence, weight));

        match evidence {
            InferenceEvidence::Shebang(shebang) => {
                self.score_shebang(shebang, weight);
            }
            InferenceEvidence::Syntax(feature) => {
                self.score_syntax_feature(feature, weight);
            }
            InferenceEvidence::Builtins(profile) => {
                self.score_builtin_profile(profile, weight);
            }
        }
    }

    pub fn compute_confidence(&self) -> InferenceConfidence {
        let (best_core_dialect, best_score) = self
            .scores
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(d, s)| (*d, *s))
            .unwrap_or((CoreDialect::Posix, 0.1));

        let best_dialect = best_core_dialect.to_shell_dialect();

        let total_evidence_weight: f32 = self.evidence.iter().map(|(_, w)| w).sum();
        let confidence = if total_evidence_weight > 0.0 {
            (best_score / total_evidence_weight).min(1.0)
        } else {
            0.1
        };

        // Find the strongest evidence
        let strongest_evidence = self
            .evidence
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(e, _)| *e)
            .unwrap_or(InferenceEvidence::Syntax(SyntaxFeature::PosixFunctions));

        InferenceConfidence {
            dialect: Box::new(best_dialect),
            confidence,
            evidence: strongest_evidence,
        }
    }

    fn score_shebang(&mut self, shebang: &str, weight: f32) {
        if shebang.contains("bash") {
            *self.scores.get_mut(&CoreDialect::Bash5_2).unwrap() += weight;
        } else if shebang.contains("zsh") {
            *self.scores.get_mut(&CoreDialect::Zsh5_9).unwrap() += weight;
        } else if shebang.contains("ksh") {
            *self.scores.get_mut(&CoreDialect::Ksh93uPlus).unwrap() += weight;
        } else if shebang.contains("dash") {
            *self.scores.get_mut(&CoreDialect::Dash0_5_12).unwrap() += weight;
        } else if shebang.contains("sh") {
            *self.scores.get_mut(&CoreDialect::Posix).unwrap() += weight;
        }
    }

    fn score_syntax_feature(&mut self, feature: SyntaxFeature, weight: f32) {
        match feature {
            SyntaxFeature::BashArrays
            | SyntaxFeature::BashProcessSubst
            | SyntaxFeature::BashConditionals
            | SyntaxFeature::BashArithmetic => {
                *self.scores.get_mut(&CoreDialect::Bash5_2).unwrap() += weight;
            }
            SyntaxFeature::ZshGlobs => {
                *self.scores.get_mut(&CoreDialect::Zsh5_9).unwrap() += weight;
            }
            SyntaxFeature::KshFunctions => {
                *self.scores.get_mut(&CoreDialect::Ksh93uPlus).unwrap() += weight;
            }
            SyntaxFeature::PosixFunctions => {
                *self.scores.get_mut(&CoreDialect::Posix).unwrap() += weight;
            }
        }
    }

    fn score_builtin_profile(&mut self, profile: BuiltinProfile, weight: f32) {
        match profile {
            BuiltinProfile::BashReadarray => {
                *self.scores.get_mut(&CoreDialect::Bash5_2).unwrap() += weight;
            }
            BuiltinProfile::ZshZparseopts => {
                *self.scores.get_mut(&CoreDialect::Zsh5_9).unwrap() += weight;
            }
            BuiltinProfile::KshTypeset => {
                *self.scores.get_mut(&CoreDialect::Ksh93uPlus).unwrap() += weight;
            }
            BuiltinProfile::DashLocal => {
                *self.scores.get_mut(&CoreDialect::Dash0_5_12).unwrap() += weight;
            }
        }
    }
}

impl Default for DialectScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellDialect {
    /// Infer dialect with confidence scoring
    pub fn infer(source: &[u8]) -> InferenceConfidence {
        let mut scorer = DialectScorer::new();

        // Convert to string for analysis
        let source_str = String::from_utf8_lossy(source);

        // Shebang provides strongest signal (weight: 0.7)
        if let Some(shebang) = Self::parse_shebang(&source_str) {
            scorer.add_evidence(InferenceEvidence::Shebang(shebang), 0.7);
        }

        // Syntactic constructs (weight: 0.2)
        let syntax_features = Self::extract_syntax_features(&source_str);
        for feature in syntax_features {
            scorer.add_evidence(InferenceEvidence::Syntax(feature), 0.2);
        }

        // Builtin usage patterns (weight: 0.1)
        let builtin_profile = Self::profile_builtins(&source_str);
        if let Some(profile) = builtin_profile {
            scorer.add_evidence(InferenceEvidence::Builtins(profile), 0.1);
        }

        scorer.compute_confidence()
    }

    fn parse_shebang(source: &str) -> Option<&'static str> {
        let first_line = source.lines().next()?;
        if first_line.starts_with("#!") {
            if first_line.contains("bash") {
                Some("bash")
            } else if first_line.contains("zsh") {
                Some("zsh")
            } else if first_line.contains("ksh") {
                Some("ksh")
            } else if first_line.contains("dash") {
                Some("dash")
            } else if first_line.contains("/sh") || first_line.ends_with("sh") {
                Some("sh")
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_syntax_features(source: &str) -> Vec<SyntaxFeature> {
        let mut features = Vec::new();

        // Check for bash arrays
        if source.contains("=(") && source.contains(")") {
            features.push(SyntaxFeature::BashArrays);
        }

        // Check for process substitution
        if source.contains("<(") || source.contains(">(") {
            features.push(SyntaxFeature::BashProcessSubst);
        }

        // Check for bash conditionals
        if source.contains("[[") && source.contains("]]") {
            features.push(SyntaxFeature::BashConditionals);
        }

        // Check for bash arithmetic
        if source.contains("((") && source.contains("))") {
            features.push(SyntaxFeature::BashArithmetic);
        }

        // Check for function definitions
        if source.contains("function ") {
            features.push(SyntaxFeature::KshFunctions);
        }
        if source.contains("()") && source.contains("{") {
            features.push(SyntaxFeature::PosixFunctions);
        }

        // Check for zsh globs
        if source.contains("**/") {
            features.push(SyntaxFeature::ZshGlobs);
        }

        features
    }

    fn profile_builtins(source: &str) -> Option<BuiltinProfile> {
        if source.contains("readarray") || source.contains("mapfile") {
            Some(BuiltinProfile::BashReadarray)
        } else if source.contains("zparseopts") {
            Some(BuiltinProfile::ZshZparseopts)
        } else if source.contains("typeset") {
            Some(BuiltinProfile::KshTypeset)
        } else if source.contains("local ") {
            Some(BuiltinProfile::DashLocal)
        } else {
            None
        }
    }

    /// Get display name for the dialect
    pub fn display_name(&self) -> &'static str {
        match self {
            ShellDialect::Posix => "POSIX",
            ShellDialect::Bash5_2 => "Bash 5.2",
            ShellDialect::Ksh93uPlus => "KornShell 93u+",
            ShellDialect::Zsh5_9 => "Z shell 5.9",
            ShellDialect::Dash0_5_12 => "Dash 0.5.12",
            ShellDialect::Inferred(conf) => conf.dialect.display_name(),
        }
    }

    /// Check if this dialect supports a given feature
    pub fn supports_feature(&self, feature: SyntaxFeature) -> bool {
        match (self, feature) {
            (ShellDialect::Bash5_2, SyntaxFeature::BashArrays) => true,
            (ShellDialect::Bash5_2, SyntaxFeature::BashProcessSubst) => true,
            (ShellDialect::Bash5_2, SyntaxFeature::BashConditionals) => true,
            (ShellDialect::Bash5_2, SyntaxFeature::BashArithmetic) => true,
            (ShellDialect::Zsh5_9, SyntaxFeature::ZshGlobs) => true,
            (ShellDialect::Ksh93uPlus, SyntaxFeature::KshFunctions) => true,
            (_, SyntaxFeature::PosixFunctions) => true, // All shells support POSIX functions
            _ => false,
        }
    }
}

/// Feature compatibility between dialects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    /// Feature is directly compatible
    Direct,
    /// Feature is incompatible
    Incompatible,
}

/// Check compatibility between source and target dialects for a feature
pub fn check_compatibility(
    _source: ShellDialect,
    target: ShellDialect,
    feature: SyntaxFeature,
) -> Compatibility {
    // If target supports the feature, it's compatible
    if target.supports_feature(feature) {
        Compatibility::Direct
    } else {
        Compatibility::Incompatible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_default() {
        assert_eq!(ShellDialect::default(), ShellDialect::Posix);
    }

    #[test]
    fn test_parse_shebang() {
        assert_eq!(ShellDialect::parse_shebang("#!/bin/bash"), Some("bash"));
        assert_eq!(ShellDialect::parse_shebang("#!/bin/zsh"), Some("zsh"));
        assert_eq!(ShellDialect::parse_shebang("#!/bin/sh"), Some("sh"));
        assert_eq!(ShellDialect::parse_shebang("echo hello"), None);
    }

    #[test]
    fn test_extract_syntax_features() {
        let bash_script = "array=(a b c)\nif [[ $var == 'test' ]]; then\n  echo $var\nfi";
        let features = ShellDialect::extract_syntax_features(bash_script);

        assert!(features.contains(&SyntaxFeature::BashArrays));
        assert!(features.contains(&SyntaxFeature::BashConditionals));
    }

    #[test]
    fn test_dialect_inference() {
        let bash_script = "#!/bin/bash\narray=(a b c)\nif [[ $var == 'test' ]]; then\n  readarray lines < file.txt\nfi";
        let confidence = ShellDialect::infer(bash_script.as_bytes());

        assert_eq!(*confidence.dialect, ShellDialect::Bash5_2);
        assert!(confidence.confidence > 0.5);
    }

    #[test]
    fn test_posix_inference() {
        let posix_script = "#!/bin/sh\ntest_func() {\n  echo 'Hello'\n}";
        let confidence = ShellDialect::infer(posix_script.as_bytes());

        assert_eq!(*confidence.dialect, ShellDialect::Posix);
    }

    #[test]
    fn test_dialect_scorer() {
        let mut scorer = DialectScorer::new();
        scorer.add_evidence(InferenceEvidence::Shebang("bash"), 0.7);
        scorer.add_evidence(InferenceEvidence::Syntax(SyntaxFeature::BashArrays), 0.2);

        let confidence = scorer.compute_confidence();
        assert_eq!(*confidence.dialect, ShellDialect::Bash5_2);
        assert!(confidence.confidence > 0.8);
    }

    #[test]
    fn test_feature_support() {
        assert!(ShellDialect::Bash5_2.supports_feature(SyntaxFeature::BashArrays));
        assert!(!ShellDialect::Posix.supports_feature(SyntaxFeature::BashArrays));
        assert!(ShellDialect::Posix.supports_feature(SyntaxFeature::PosixFunctions));
        assert!(ShellDialect::Zsh5_9.supports_feature(SyntaxFeature::ZshGlobs));
    }

    #[test]
    fn test_compatibility_check() {
        assert_eq!(
            check_compatibility(
                ShellDialect::Bash5_2,
                ShellDialect::Bash5_2,
                SyntaxFeature::BashArrays
            ),
            Compatibility::Direct
        );

        assert_eq!(
            check_compatibility(
                ShellDialect::Bash5_2,
                ShellDialect::Posix,
                SyntaxFeature::BashArrays
            ),
            Compatibility::Incompatible
        );
    }

    #[test]
    fn test_dialect_display_names() {
        assert_eq!(ShellDialect::Posix.display_name(), "POSIX");
        assert_eq!(ShellDialect::Bash5_2.display_name(), "Bash 5.2");
        assert_eq!(ShellDialect::Zsh5_9.display_name(), "Z shell 5.9");
    }
}
