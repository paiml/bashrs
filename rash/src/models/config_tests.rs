#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::config::{Config, ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;

// ---------------------------------------------------------------------------
// 1. Config::default() — verify all default field values
// ---------------------------------------------------------------------------

#[test]
fn test_config_default_target_is_posix() {
    let cfg = Config::default();
    assert_eq!(cfg.target, ShellDialect::Posix);
}

#[test]
fn test_config_default_verify_is_strict() {
    let cfg = Config::default();
    assert_eq!(cfg.verify, VerificationLevel::Strict);
}

#[test]
fn test_config_default_emit_proof_is_false() {
    let cfg = Config::default();
    assert!(!cfg.emit_proof);
}

#[test]
fn test_config_default_optimize_is_true() {
    let cfg = Config::default();
    assert!(cfg.optimize);
}

#[test]
fn test_config_default_validation_level_is_some_minimal() {
    let cfg = Config::default();
    assert_eq!(cfg.validation_level, Some(ValidationLevel::Minimal));
}

#[test]
fn test_config_default_strict_mode_is_false() {
    let cfg = Config::default();
    assert!(!cfg.strict_mode);
}

// ---------------------------------------------------------------------------
// 2. ShellDialect variants — Debug, Clone, Copy, PartialEq, Serialize/Deserialize
// ---------------------------------------------------------------------------

#[test]
fn test_shell_dialect_debug_posix() {
    assert_eq!(format!("{:?}", ShellDialect::Posix), "Posix");
}

#[test]
fn test_shell_dialect_debug_bash() {
    assert_eq!(format!("{:?}", ShellDialect::Bash), "Bash");
}

#[test]
fn test_shell_dialect_debug_dash() {
    assert_eq!(format!("{:?}", ShellDialect::Dash), "Dash");
}

#[test]
fn test_shell_dialect_debug_ash() {
    assert_eq!(format!("{:?}", ShellDialect::Ash), "Ash");
}

#[test]
fn test_shell_dialect_clone() {
    let original = ShellDialect::Bash;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_shell_dialect_copy() {
    let original = ShellDialect::Dash;
    let copied = original; // Copy, not move
    assert_eq!(original, copied); // original still usable
}

#[test]
fn test_shell_dialect_partial_eq_same() {
    assert_eq!(ShellDialect::Posix, ShellDialect::Posix);
    assert_eq!(ShellDialect::Bash, ShellDialect::Bash);
    assert_eq!(ShellDialect::Dash, ShellDialect::Dash);
    assert_eq!(ShellDialect::Ash, ShellDialect::Ash);
}

#[test]
fn test_shell_dialect_partial_eq_different() {
    assert_ne!(ShellDialect::Posix, ShellDialect::Bash);
    assert_ne!(ShellDialect::Bash, ShellDialect::Dash);
    assert_ne!(ShellDialect::Dash, ShellDialect::Ash);
    assert_ne!(ShellDialect::Ash, ShellDialect::Posix);
}

#[test]
fn test_shell_dialect_serialize_posix() {
    let json = serde_json::to_string(&ShellDialect::Posix).unwrap();
    assert_eq!(json, "\"Posix\"");
}

#[test]
fn test_shell_dialect_serialize_bash() {
    let json = serde_json::to_string(&ShellDialect::Bash).unwrap();
    assert_eq!(json, "\"Bash\"");
}

#[test]
fn test_shell_dialect_serialize_dash() {
    let json = serde_json::to_string(&ShellDialect::Dash).unwrap();
    assert_eq!(json, "\"Dash\"");
}

#[test]
fn test_shell_dialect_serialize_ash() {
    let json = serde_json::to_string(&ShellDialect::Ash).unwrap();
    assert_eq!(json, "\"Ash\"");
}

#[test]
fn test_shell_dialect_deserialize_posix() {
    let dialect: ShellDialect = serde_json::from_str("\"Posix\"").unwrap();
    assert_eq!(dialect, ShellDialect::Posix);
}

#[test]
fn test_shell_dialect_deserialize_bash() {
    let dialect: ShellDialect = serde_json::from_str("\"Bash\"").unwrap();
    assert_eq!(dialect, ShellDialect::Bash);
}

#[test]
fn test_shell_dialect_deserialize_dash() {
    let dialect: ShellDialect = serde_json::from_str("\"Dash\"").unwrap();
    assert_eq!(dialect, ShellDialect::Dash);
}

#[test]
fn test_shell_dialect_deserialize_ash() {
    let dialect: ShellDialect = serde_json::from_str("\"Ash\"").unwrap();
    assert_eq!(dialect, ShellDialect::Ash);
}

#[test]
fn test_shell_dialect_serde_roundtrip_all_variants() {
    let variants = [
        ShellDialect::Posix,
        ShellDialect::Bash,
        ShellDialect::Dash,
        ShellDialect::Ash,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let deserialized: ShellDialect = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, deserialized);
    }
}

// ---------------------------------------------------------------------------
// 3. VerificationLevel variants — Debug, Clone, Copy, PartialEq, Serialize/Deserialize
// ---------------------------------------------------------------------------

#[test]
fn test_verification_level_debug_none() {
    assert_eq!(format!("{:?}", VerificationLevel::None), "None");
}

#[test]
fn test_verification_level_debug_basic() {
    assert_eq!(format!("{:?}", VerificationLevel::Basic), "Basic");
}

#[test]
fn test_verification_level_debug_strict() {
    assert_eq!(format!("{:?}", VerificationLevel::Strict), "Strict");
}

#[test]
fn test_verification_level_debug_paranoid() {
    assert_eq!(format!("{:?}", VerificationLevel::Paranoid), "Paranoid");
}

#[test]
fn test_verification_level_clone() {
    let original = VerificationLevel::Paranoid;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_verification_level_copy() {
    let original = VerificationLevel::Basic;
    let copied = original; // Copy, not move
    assert_eq!(original, copied); // original still usable
}

#[test]
fn test_verification_level_partial_eq_same() {
    assert_eq!(VerificationLevel::None, VerificationLevel::None);
    assert_eq!(VerificationLevel::Basic, VerificationLevel::Basic);
    assert_eq!(VerificationLevel::Strict, VerificationLevel::Strict);
    assert_eq!(VerificationLevel::Paranoid, VerificationLevel::Paranoid);
}

#[test]
fn test_verification_level_partial_eq_different() {
    assert_ne!(VerificationLevel::None, VerificationLevel::Basic);
    assert_ne!(VerificationLevel::Basic, VerificationLevel::Strict);
    assert_ne!(VerificationLevel::Strict, VerificationLevel::Paranoid);
    assert_ne!(VerificationLevel::Paranoid, VerificationLevel::None);
}

#[test]
fn test_verification_level_serialize_none() {
    let json = serde_json::to_string(&VerificationLevel::None).unwrap();
    assert_eq!(json, "\"None\"");
}

#[test]
fn test_verification_level_serialize_basic() {
    let json = serde_json::to_string(&VerificationLevel::Basic).unwrap();
    assert_eq!(json, "\"Basic\"");
}

#[test]
fn test_verification_level_serialize_strict() {
    let json = serde_json::to_string(&VerificationLevel::Strict).unwrap();
    assert_eq!(json, "\"Strict\"");
}

#[test]
fn test_verification_level_serialize_paranoid() {
    let json = serde_json::to_string(&VerificationLevel::Paranoid).unwrap();
    assert_eq!(json, "\"Paranoid\"");
}

#[test]
fn test_verification_level_deserialize_none() {
    let level: VerificationLevel = serde_json::from_str("\"None\"").unwrap();
    assert_eq!(level, VerificationLevel::None);
}

#[test]
fn test_verification_level_deserialize_basic() {
    let level: VerificationLevel = serde_json::from_str("\"Basic\"").unwrap();
    assert_eq!(level, VerificationLevel::Basic);
}

#[test]
fn test_verification_level_deserialize_strict() {
    let level: VerificationLevel = serde_json::from_str("\"Strict\"").unwrap();
    assert_eq!(level, VerificationLevel::Strict);
}

#[test]
fn test_verification_level_deserialize_paranoid() {
    let level: VerificationLevel = serde_json::from_str("\"Paranoid\"").unwrap();
    assert_eq!(level, VerificationLevel::Paranoid);
}

#[test]
fn test_verification_level_serde_roundtrip_all_variants() {
    let variants = [
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let deserialized: VerificationLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, deserialized);
    }
}

// ---------------------------------------------------------------------------
// 4. Config with all custom field combinations
// ---------------------------------------------------------------------------

#[test]
fn test_config_all_shell_dialects() {
    for dialect in &[
        ShellDialect::Posix,
        ShellDialect::Bash,
        ShellDialect::Dash,
        ShellDialect::Ash,
    ] {
        let cfg = Config {
            target: *dialect,
            ..Config::default()
        };
        assert_eq!(cfg.target, *dialect);
    }
}

#[test]
fn test_config_all_verification_levels() {
    for level in &[
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ] {
        let cfg = Config {
            verify: *level,
            ..Config::default()
        };
        assert_eq!(cfg.verify, *level);
    }
}

#[test]
fn test_config_emit_proof_true() {
    let cfg = Config {
        emit_proof: true,
        ..Config::default()
    };
    assert!(cfg.emit_proof);
}

#[test]
fn test_config_optimize_false() {
    let cfg = Config {
        optimize: false,
        ..Config::default()
    };
    assert!(!cfg.optimize);
}

#[test]
fn test_config_validation_level_none() {
    let cfg = Config {
        validation_level: Option::None,
        ..Config::default()
    };
    assert_eq!(cfg.validation_level, Option::None);
}

#[test]
fn test_config_validation_level_strict() {
    let cfg = Config {
        validation_level: Some(ValidationLevel::Strict),
        ..Config::default()
    };
    assert_eq!(cfg.validation_level, Some(ValidationLevel::Strict));
}

#[test]
fn test_config_validation_level_paranoid() {
    let cfg = Config {
        validation_level: Some(ValidationLevel::Paranoid),
        ..Config::default()
    };
    assert_eq!(cfg.validation_level, Some(ValidationLevel::Paranoid));
}

#[test]
fn test_config_validation_level_val_none() {
    let cfg = Config {
        validation_level: Some(ValidationLevel::None),
        ..Config::default()
    };
    assert_eq!(cfg.validation_level, Some(ValidationLevel::None));
}

#[test]
fn test_config_strict_mode_true() {
    let cfg = Config {
        strict_mode: true,
        ..Config::default()
    };
    assert!(cfg.strict_mode);
}

#[test]
fn test_config_fully_custom() {
    let cfg = Config {
        target: ShellDialect::Ash,
        verify: VerificationLevel::Paranoid,
        emit_proof: true,
        optimize: false,
        validation_level: Some(ValidationLevel::Paranoid),
        strict_mode: true,
    };
    assert_eq!(cfg.target, ShellDialect::Ash);
    assert_eq!(cfg.verify, VerificationLevel::Paranoid);
    assert!(cfg.emit_proof);
    assert!(!cfg.optimize);
    assert_eq!(cfg.validation_level, Some(ValidationLevel::Paranoid));
    assert!(cfg.strict_mode);
}

#[test]
fn test_config_all_booleans_true() {
    let cfg = Config {
        emit_proof: true,
        optimize: true,
        strict_mode: true,
        ..Config::default()
    };
    assert!(cfg.emit_proof);
    assert!(cfg.optimize);
    assert!(cfg.strict_mode);
}

#[test]
fn test_config_all_booleans_false() {
    let cfg = Config {
        emit_proof: false,
        optimize: false,
        strict_mode: false,
        ..Config::default()
    };
    assert!(!cfg.emit_proof);
    assert!(!cfg.optimize);
    assert!(!cfg.strict_mode);
}

// ---------------------------------------------------------------------------
// 5. Serde round-trip: serialize Config to JSON and deserialize back
// ---------------------------------------------------------------------------

#[test]
fn test_config_serde_roundtrip_default() {
    let original = Config::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.target, original.target);
    assert_eq!(restored.verify, original.verify);
    assert_eq!(restored.emit_proof, original.emit_proof);
    assert_eq!(restored.optimize, original.optimize);
    assert_eq!(restored.validation_level, original.validation_level);
    assert_eq!(restored.strict_mode, original.strict_mode);
}

#[test]
fn test_config_serde_roundtrip_fully_custom() {
    let original = Config {
        target: ShellDialect::Bash,
        verify: VerificationLevel::Paranoid,
        emit_proof: true,
        optimize: false,
        validation_level: Some(ValidationLevel::Strict),
        strict_mode: true,
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.target, original.target);
    assert_eq!(restored.verify, original.verify);
    assert_eq!(restored.emit_proof, original.emit_proof);
    assert_eq!(restored.optimize, original.optimize);
    assert_eq!(restored.validation_level, original.validation_level);
    assert_eq!(restored.strict_mode, original.strict_mode);
}

#[test]
fn test_config_serde_roundtrip_validation_level_none() {
    let original = Config {
        validation_level: Option::None,
        ..Config::default()
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.validation_level, Option::None);
}

#[test]
fn test_config_serde_roundtrip_all_dialects() {
    for dialect in &[
        ShellDialect::Posix,
        ShellDialect::Bash,
        ShellDialect::Dash,
        ShellDialect::Ash,
    ] {
        let original = Config {
            target: *dialect,
            ..Config::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.target, *dialect);
    }
}

#[test]
fn test_config_serde_roundtrip_all_verification_levels() {
    for level in &[
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ] {
        let original = Config {
            verify: *level,
            ..Config::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.verify, *level);
    }
}

#[test]
fn test_config_serde_json_field_names() {
    let cfg = Config::default();
    let json = serde_json::to_string(&cfg).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert!(obj.contains_key("target"));
    assert!(obj.contains_key("verify"));
    assert!(obj.contains_key("emit_proof"));
    assert!(obj.contains_key("optimize"));
    assert!(obj.contains_key("validation_level"));
    assert!(obj.contains_key("strict_mode"));
    assert_eq!(obj.len(), 6);
}

#[test]
fn test_config_serde_pretty_roundtrip() {
    let original = Config {
        target: ShellDialect::Dash,
        verify: VerificationLevel::Basic,
        emit_proof: true,
        optimize: true,
        validation_level: Some(ValidationLevel::Minimal),
        strict_mode: false,
    };
    let pretty_json = serde_json::to_string_pretty(&original).unwrap();
    let restored: Config = serde_json::from_str(&pretty_json).unwrap();

    assert_eq!(restored.target, original.target);
    assert_eq!(restored.verify, original.verify);
    assert_eq!(restored.emit_proof, original.emit_proof);
    assert_eq!(restored.optimize, original.optimize);
    assert_eq!(restored.validation_level, original.validation_level);
    assert_eq!(restored.strict_mode, original.strict_mode);
}

#[test]
fn test_config_deserialize_from_known_json() {
    let json = r#"{
        "target": "Bash",
        "verify": "Paranoid",
        "emit_proof": true,
        "optimize": false,
        "validation_level": "Strict",
        "strict_mode": true
    }"#;
    let cfg: Config = serde_json::from_str(json).unwrap();

    assert_eq!(cfg.target, ShellDialect::Bash);
    assert_eq!(cfg.verify, VerificationLevel::Paranoid);
    assert!(cfg.emit_proof);
    assert!(!cfg.optimize);
    assert_eq!(cfg.validation_level, Some(ValidationLevel::Strict));
    assert!(cfg.strict_mode);
}

#[test]
fn test_config_deserialize_validation_level_null() {
    let json = r#"{
        "target": "Posix",
        "verify": "Strict",
        "emit_proof": false,
        "optimize": true,
        "validation_level": null,
        "strict_mode": false
    }"#;
    let cfg: Config = serde_json::from_str(json).unwrap();

    assert_eq!(cfg.validation_level, Option::None);
}

#[test]
fn test_config_debug_impl() {
    let cfg = Config::default();
    let debug_str = format!("{:?}", cfg);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("Posix"));
    assert!(debug_str.contains("Strict"));
}

#[test]
fn test_config_clone() {
    let original = Config {
        target: ShellDialect::Bash,
        verify: VerificationLevel::Paranoid,
        emit_proof: true,
        optimize: false,
        validation_level: Some(ValidationLevel::Paranoid),
        strict_mode: true,
    };
    let cloned = original.clone();

    assert_eq!(cloned.target, ShellDialect::Bash);
    assert_eq!(cloned.verify, VerificationLevel::Paranoid);
    assert!(cloned.emit_proof);
    assert!(!cloned.optimize);
    assert_eq!(cloned.validation_level, Some(ValidationLevel::Paranoid));
    assert!(cloned.strict_mode);
}
