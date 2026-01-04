use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// Pure computation (no side effects)
    Pure,

    /// Reading environment variables
    EnvRead,

    /// Reading from files
    FileRead,

    /// Writing to files
    FileWrite,

    /// Network access
    NetworkAccess,

    /// Process execution
    ProcessExec,

    /// System calls that modify state
    SystemModification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create a pure effect set (no side effects)
    pub fn pure() -> Self {
        Self {
            effects: HashSet::new(),
        }
    }

    /// Create an effect set with a single effect
    pub fn single(effect: Effect) -> Self {
        let mut effects = HashSet::new();
        effects.insert(effect);
        Self { effects }
    }

    /// Add an effect to this set
    pub fn add(&mut self, effect: Effect) {
        self.effects.insert(effect);
    }

    /// Check if this effect set is pure
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty() || (self.effects.len() == 1 && self.effects.contains(&Effect::Pure))
    }

    /// Check if this effect set contains a specific effect
    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    /// Union two effect sets
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        let mut combined = self.effects.clone();
        combined.extend(other.effects.iter().cloned());
        EffectSet { effects: combined }
    }

    /// Check if this effect set is a subset of another
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get all effects as a vector
    pub fn to_vec(&self) -> Vec<Effect> {
        self.effects.iter().cloned().collect()
    }

    /// Check if this effect set has any file system effects
    pub fn has_filesystem_effects(&self) -> bool {
        self.contains(&Effect::FileRead) || self.contains(&Effect::FileWrite)
    }

    /// Check if this effect set has any network effects
    pub fn has_network_effects(&self) -> bool {
        self.contains(&Effect::NetworkAccess)
    }

    /// Check if this effect set has any system modification effects
    pub fn has_system_effects(&self) -> bool {
        self.contains(&Effect::SystemModification)
            || self.contains(&Effect::ProcessExec)
            || self.contains(&Effect::FileWrite)
    }
}

impl Default for EffectSet {
    fn default() -> Self {
        Self::pure()
    }
}

impl From<Effect> for EffectSet {
    fn from(effect: Effect) -> Self {
        Self::single(effect)
    }
}

impl From<Vec<Effect>> for EffectSet {
    fn from(effects: Vec<Effect>) -> Self {
        let mut set = HashSet::new();
        for effect in effects {
            set.insert(effect);
        }
        Self { effects: set }
    }
}

/// Analyze the effects of a shell command based on its name
pub fn analyze_command_effects(command: &str) -> EffectSet {
    match command {
        // Pure commands
        "echo" | "printf" | "test" | "[" => EffectSet::pure(),

        // File system reads
        "cat" | "ls" | "find" | "grep" | "head" | "tail" | "wc" => {
            EffectSet::single(Effect::FileRead)
        }

        // File system writes
        "cp" | "mv" | "rm" | "mkdir" | "rmdir" | "touch" | "chmod" | "chown" => {
            vec![Effect::FileWrite, Effect::SystemModification].into()
        }

        // Network commands
        "curl" | "wget" | "ssh" | "scp" | "rsync" => {
            vec![Effect::NetworkAccess, Effect::FileWrite].into()
        }

        // Archive commands
        "tar" | "gzip" | "gunzip" | "zip" | "unzip" => {
            vec![Effect::FileRead, Effect::FileWrite].into()
        }

        // System modification
        "sudo" | "su" | "systemctl" | "service" => {
            vec![Effect::SystemModification, Effect::ProcessExec].into()
        }

        // Process execution
        _ => vec![Effect::ProcessExec].into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== EffectSet basic tests =====

    #[test]
    fn test_pure_effect_set() {
        let pure = EffectSet::pure();
        assert!(pure.is_pure());
        assert!(!pure.contains(&Effect::FileRead));
    }

    #[test]
    fn test_single_effect_set() {
        let set = EffectSet::single(Effect::FileRead);
        assert!(set.contains(&Effect::FileRead));
        assert!(!set.is_pure());
    }

    #[test]
    fn test_effect_set_add() {
        let mut set = EffectSet::pure();
        set.add(Effect::FileRead);
        assert!(set.contains(&Effect::FileRead));
        assert!(!set.is_pure());

        set.add(Effect::FileWrite);
        assert!(set.contains(&Effect::FileWrite));
    }

    #[test]
    fn test_effect_set_union() {
        let set1 = EffectSet::single(Effect::FileRead);
        let set2 = EffectSet::single(Effect::FileWrite);
        let combined = set1.union(&set2);

        assert!(combined.contains(&Effect::FileRead));
        assert!(combined.contains(&Effect::FileWrite));
        assert!(!combined.is_pure());
    }

    #[test]
    fn test_effect_set_is_subset_of() {
        let set1 = EffectSet::single(Effect::FileRead);
        let mut set2 = EffectSet::single(Effect::FileRead);
        set2.add(Effect::FileWrite);

        assert!(set1.is_subset_of(&set2));
        assert!(!set2.is_subset_of(&set1));
    }

    #[test]
    fn test_effect_set_to_vec() {
        let mut set = EffectSet::pure();
        set.add(Effect::FileRead);
        set.add(Effect::FileWrite);

        let vec = set.to_vec();
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_effect_set_has_filesystem_effects() {
        let read_set = EffectSet::single(Effect::FileRead);
        let write_set = EffectSet::single(Effect::FileWrite);
        let pure_set = EffectSet::pure();

        assert!(read_set.has_filesystem_effects());
        assert!(write_set.has_filesystem_effects());
        assert!(!pure_set.has_filesystem_effects());
    }

    #[test]
    fn test_effect_set_has_network_effects() {
        let network_set = EffectSet::single(Effect::NetworkAccess);
        let file_set = EffectSet::single(Effect::FileRead);

        assert!(network_set.has_network_effects());
        assert!(!file_set.has_network_effects());
    }

    #[test]
    fn test_effect_set_has_system_effects() {
        assert!(EffectSet::single(Effect::SystemModification).has_system_effects());
        assert!(EffectSet::single(Effect::ProcessExec).has_system_effects());
        assert!(EffectSet::single(Effect::FileWrite).has_system_effects());
        assert!(!EffectSet::single(Effect::FileRead).has_system_effects());
        assert!(!EffectSet::single(Effect::EnvRead).has_system_effects());
    }

    #[test]
    fn test_effect_set_default() {
        let default = EffectSet::default();
        assert!(default.is_pure());
    }

    #[test]
    fn test_effect_set_from_effect() {
        let set: EffectSet = Effect::FileRead.into();
        assert!(set.contains(&Effect::FileRead));
    }

    #[test]
    fn test_effect_set_from_vec() {
        let set: EffectSet = vec![Effect::FileRead, Effect::FileWrite].into();
        assert!(set.contains(&Effect::FileRead));
        assert!(set.contains(&Effect::FileWrite));
    }

    #[test]
    fn test_is_pure_with_pure_effect() {
        let mut set = EffectSet::pure();
        set.add(Effect::Pure);
        assert!(set.is_pure());
    }

    // ===== Effect enum tests =====

    #[test]
    fn test_effect_eq() {
        assert_eq!(Effect::Pure, Effect::Pure);
        assert_ne!(Effect::Pure, Effect::FileRead);
    }

    #[test]
    fn test_effect_hash() {
        let mut set = HashSet::new();
        set.insert(Effect::Pure);
        set.insert(Effect::FileRead);
        assert!(set.contains(&Effect::Pure));
        assert!(set.contains(&Effect::FileRead));
    }

    #[test]
    fn test_effect_clone() {
        let effects = vec![
            Effect::Pure,
            Effect::EnvRead,
            Effect::FileRead,
            Effect::FileWrite,
            Effect::NetworkAccess,
            Effect::ProcessExec,
            Effect::SystemModification,
        ];
        for effect in effects {
            let _ = effect.clone();
        }
    }

    // ===== analyze_command_effects tests =====

    #[test]
    fn test_command_effect_analysis() {
        assert!(analyze_command_effects("echo").is_pure());
        assert!(analyze_command_effects("cat").contains(&Effect::FileRead));
        assert!(analyze_command_effects("curl").has_network_effects());
        assert!(analyze_command_effects("rm").has_system_effects());
    }

    #[test]
    fn test_pure_commands() {
        for cmd in ["echo", "printf", "test", "["] {
            assert!(
                analyze_command_effects(cmd).is_pure(),
                "Command '{}' should be pure",
                cmd
            );
        }
    }

    #[test]
    fn test_file_read_commands() {
        for cmd in ["cat", "ls", "find", "grep", "head", "tail", "wc"] {
            assert!(
                analyze_command_effects(cmd).contains(&Effect::FileRead),
                "Command '{}' should have FileRead effect",
                cmd
            );
        }
    }

    #[test]
    fn test_file_write_commands() {
        for cmd in [
            "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown",
        ] {
            let effects = analyze_command_effects(cmd);
            assert!(
                effects.contains(&Effect::FileWrite),
                "Command '{}' should have FileWrite effect",
                cmd
            );
            assert!(
                effects.contains(&Effect::SystemModification),
                "Command '{}' should have SystemModification effect",
                cmd
            );
        }
    }

    #[test]
    fn test_network_commands() {
        for cmd in ["curl", "wget", "ssh", "scp", "rsync"] {
            let effects = analyze_command_effects(cmd);
            assert!(
                effects.contains(&Effect::NetworkAccess),
                "Command '{}' should have NetworkAccess effect",
                cmd
            );
        }
    }

    #[test]
    fn test_archive_commands() {
        for cmd in ["tar", "gzip", "gunzip", "zip", "unzip"] {
            let effects = analyze_command_effects(cmd);
            assert!(
                effects.contains(&Effect::FileRead),
                "Command '{}' should have FileRead effect",
                cmd
            );
            assert!(
                effects.contains(&Effect::FileWrite),
                "Command '{}' should have FileWrite effect",
                cmd
            );
        }
    }

    #[test]
    fn test_system_commands() {
        for cmd in ["sudo", "su", "systemctl", "service"] {
            let effects = analyze_command_effects(cmd);
            assert!(
                effects.contains(&Effect::SystemModification),
                "Command '{}' should have SystemModification effect",
                cmd
            );
            assert!(
                effects.contains(&Effect::ProcessExec),
                "Command '{}' should have ProcessExec effect",
                cmd
            );
        }
    }

    #[test]
    fn test_unknown_command() {
        let effects = analyze_command_effects("my_custom_script");
        assert!(effects.contains(&Effect::ProcessExec));
    }

    // ===== EffectSet edge cases =====

    #[test]
    fn test_union_with_self() {
        let set = EffectSet::single(Effect::FileRead);
        let combined = set.union(&set);
        assert!(combined.contains(&Effect::FileRead));
        assert_eq!(combined.to_vec().len(), 1);
    }

    #[test]
    fn test_union_pure_with_non_pure() {
        let pure = EffectSet::pure();
        let non_pure = EffectSet::single(Effect::FileRead);
        let combined = pure.union(&non_pure);
        assert!(combined.contains(&Effect::FileRead));
    }

    #[test]
    fn test_empty_vec_to_effect_set() {
        let set: EffectSet = vec![].into();
        assert!(set.is_pure());
    }
}
