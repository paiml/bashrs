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
