use std::collections::HashSet;
/// Known external variables that should suppress SC2154 warnings
pub fn known_external_vars() -> HashSet<String> {
    vec![
        "HOME",
        "PATH",
        "USER",
        "SHELL",
        "PWD",
        "OLDPWD",
        "TMPDIR",
        "EDITOR",
        "VISUAL",
        "NVM_DIR",
        "PYENV_ROOT",
        "RBENV_ROOT",
        "NODENV_ROOT",
        "GOPATH",
        "GOROOT",
        "BUN_INSTALL",
        "CARGO_HOME",
        "CARGO_BUILD_JOBS",
        "RUSTC_WRAPPER",
        "SCCACHE_CACHE_SIZE",
        "NPM_CONFIG_PREFIX",
        "PIP_USER",
        "ZSH",
        "ZSH_THEME",
        "ZSH_CUSTOM",
        "BASH_VERSION",
        "ZSH_VERSION",
        "CC",
        "CXX",
        "CFLAGS",
        "LDFLAGS",
        "MAKEFLAGS",
        "DOCKER_HOST",
        "KUBECONFIG",
        "AWS_REGION",
        "AWS_PROFILE",
        "AZURE_SUBSCRIPTION_ID",
        "GCP_PROJECT",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
/// Check if a variable name should suppress SC2154 warnings
pub fn should_suppress_sc2154(var_name: &str, context: &LintContext) -> bool {
    if known_external_vars().contains(var_name) {
        return true;
    }
    if let Some(func_name) = &context.current_function {
        if func_name.starts_with("test_") {
            return true;
        }
    }
    if var_name.chars().any(|c| c.is_uppercase())
        && var_name.chars().all(|c| c.is_uppercase() || c == '_')
    {
        return true;
    }
    false
}
/// Context for lint suppression decisions
#[derive(Debug, Clone, Default)]
pub struct LintContext {
    pub current_function: Option<String>,
    pub file_type: FileType,
    pub line_number: usize,
}
/// File type affects suppression rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Script,
    Config,
    Library,
}
impl FileType {
    ///
    pub fn from_path(path: &std::path::Path) -> Self {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename.ends_with("rc")
            || filename.ends_with("profile")
            || filename == ".bash_profile"
            || filename == ".bash_login"
            || filename == ".bash_logout"
        {
            return FileType::Config;
        }
        if path.extension().map_or(false, |e| e == "sh") {
            return FileType::Script;
        }
        FileType::Library
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn test_known_external_vars() {
        let vars = known_external_vars();
        assert!(vars.contains("HOME"));
        assert!(vars.contains("NVM_DIR"));
        assert!(vars.contains("BUN_INSTALL"));
        assert!(vars.contains("ZSH"));
    }
    #[test]
    fn test_should_suppress_sc2154_external_var() {
        let context = LintContext::default();
        assert!(should_suppress_sc2154("NVM_DIR", &context));
        assert!(should_suppress_sc2154("HOME", &context));
        assert!(should_suppress_sc2154("BUN_INSTALL", &context));
    }
    #[test]
    fn test_should_suppress_sc2154_test_function() {
        let context = LintContext {
            current_function: Some("test_example".to_string()),
            ..Default::default()
        };
        assert!(should_suppress_sc2154("model", &context));
        assert!(should_suppress_sc2154("result", &context));
    }
    #[test]
    fn test_should_suppress_sc2154_uppercase_env() {
        let context = LintContext::default();
        assert!(should_suppress_sc2154("MY_CUSTOM_VAR", &context));
        assert!(should_suppress_sc2154("BUILD_NUMBER", &context));
    }
    #[test]
    fn test_should_not_suppress_lowercase_var() {
        let context = LintContext::default();
        assert!(!should_suppress_sc2154("my_var", &context));
        assert!(!should_suppress_sc2154("result", &context));
    }
    #[test]
    fn test_file_type_detection_config() {
        assert_eq!(
            FileType::from_path(Path::new("~/.bashrc")),
            FileType::Config
        );
        assert_eq!(FileType::from_path(Path::new("~/.zshrc")), FileType::Config);
        assert_eq!(
            FileType::from_path(Path::new("~/.profile")),
            FileType::Config
        );
        assert_eq!(
            FileType::from_path(Path::new("~/.bash_profile")),
            FileType::Config
        );
    }
    #[test]
    fn test_file_type_detection_script() {
        assert_eq!(
            FileType::from_path(Path::new("deploy.sh")),
            FileType::Script
        );
        assert_eq!(
            FileType::from_path(Path::new("install.sh")),
            FileType::Script
        );
    }
    #[test]
    fn test_file_type_detection_library() {
        assert_eq!(
            FileType::from_path(Path::new("helpers.bash")),
            FileType::Library
        );
        assert_eq!(FileType::from_path(Path::new("utils")), FileType::Library);
    }
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        proptest! {
            #[test] fn prop_known_vars_always_suppressed(var_name in "[A-Z_]{3,20}") {
            let context = LintContext::default(); let vars = known_external_vars(); if
            vars.contains(& var_name) { prop_assert!(should_suppress_sc2154(& var_name, &
            context)); } } #[test] fn prop_test_functions_suppress_all_vars(var_name in
            "[a-z_]{3,20}", func_name in "test_[a-z_]{3,20}") { let context = LintContext
            { current_function : Some(func_name), ..Default::default() };
            prop_assert!(should_suppress_sc2154(& var_name, & context)); } #[test] fn
            prop_uppercase_vars_always_suppressed(var_name in "[A-Z][A-Z_]{0,29}") { let
            context = LintContext::default(); prop_assert!(should_suppress_sc2154(&
            var_name, & context)); } #[test] fn
            prop_lowercase_vars_not_suppressed_by_default(var_name in "[a-z_]{3,20}") {
            let context = LintContext::default(); if ! known_external_vars().contains(&
            var_name.to_uppercase()) { prop_assert!(! should_suppress_sc2154(& var_name,
            & context)); } } #[test] fn prop_file_type_detection_consistent(filename in
            "[a-z]{1,10}\\.(sh|bash|zsh)") { let path = Path::new(& filename); let
            file_type = FileType::from_path(path); if filename.ends_with(".sh") {
            prop_assert_eq!(file_type, FileType::Script); } } #[test] fn
            prop_config_files_detected(suffix in "rc|profile") { let filename =
            format!(".{}", suffix); let path = Path::new(& filename); let file_type =
            FileType::from_path(path); prop_assert_eq!(file_type, FileType::Config); }
        }
    }
}
