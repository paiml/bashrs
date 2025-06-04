use std::collections::HashSet;
use serde::{Deserialize, Serialize};

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
        self.contains(&Effect::SystemModification) || 
        self.contains(&Effect::ProcessExec) ||
        self.contains(&Effect::FileWrite)
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
    
    #[test]
    fn test_pure_effect_set() {
        let pure = EffectSet::pure();
        assert!(pure.is_pure());
        assert!(!pure.contains(&Effect::FileRead));
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
    fn test_command_effect_analysis() {
        assert!(analyze_command_effects("echo").is_pure());
        assert!(analyze_command_effects("cat").contains(&Effect::FileRead));
        assert!(analyze_command_effects("curl").has_network_effects());
        assert!(analyze_command_effects("rm").has_system_effects());
    }
}