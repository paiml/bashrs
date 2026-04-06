
use super::*;
use crate::installer::InstallerSpec;

#[test]
fn test_plan_from_minimal_spec() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let plan = InstallerPlan::from_spec(spec).unwrap();
    assert_eq!(plan.name(), "test");
    assert_eq!(plan.version(), "1.0.0");
    assert!(plan.steps().is_empty());
}

#[test]
fn test_plan_with_dependent_steps() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "first"
name = "First Step"
action = "script"

[[step]]
id = "second"
name = "Second Step"
action = "script"
depends_on = ["first"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let plan = InstallerPlan::from_spec(spec).unwrap();
    assert_eq!(plan.steps().len(), 2);
    // First step should come before second
    let first_idx = plan.steps().iter().position(|s| s.id == "first").unwrap();
    let second_idx = plan.steps().iter().position(|s| s.id == "second").unwrap();
    assert!(first_idx < second_idx);
}

#[test]
fn test_plan_detects_cycle() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"
depends_on = ["b"]

[[step]]
id = "b"
name = "B"
action = "script"
depends_on = ["a"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let result = InstallerPlan::from_spec(spec);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("cycle"), "Expected cycle error, got: {}", err);
}

#[test]
fn test_plan_detects_unknown_dependency() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"
depends_on = ["nonexistent"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let result = InstallerPlan::from_spec(spec);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown step"),
        "Expected unknown step error, got: {}",
        err
    );
}

#[test]
fn test_plan_execution_waves() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"

[[step]]
id = "b"
name = "B"
action = "script"

[[step]]
id = "c"
name = "C"
action = "script"
depends_on = ["a", "b"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let plan = InstallerPlan::from_spec(spec).unwrap();

    // Should have 2 waves: [a, b] then [c]
    let waves = plan.waves();
    assert_eq!(waves.len(), 2);
    assert_eq!(waves[0].len(), 2); // a and b can run in parallel
    assert_eq!(waves[1].len(), 1); // c must wait
}

#[test]
fn test_plan_validates_artifact_references() {
    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"

[step.uses_artifacts]
artifacts = ["nonexistent"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let result = InstallerPlan::from_spec(spec);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown artifact"),
        "Expected unknown artifact error, got: {}",
        err
    );
}
