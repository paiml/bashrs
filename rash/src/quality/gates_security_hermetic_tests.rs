//! #542: the security gate's tests never start the real `cargo audit`.
//! A stub `cargo` stands in, so no network or advisory database is read.

use super::*;

fn stub_cargo(dir: &std::path::Path, body: &str) -> String {
    let path = dir.join("cargo");
    crate::testing::exec_stub::write_exec_stub(&path, body);
    path.to_string_lossy().into_owned()
}

fn gate() -> QualityGate {
    QualityGate::with_defaults()
}

#[test]
fn test_security_gate_stub_clean_passes_and_stub_ran() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let marker = dir.path().join("ran");
    let cargo = stub_cargo(
        dir.path(),
        &format!("#!/bin/sh\ntouch '{}'\nexit 0\n", marker.display()),
    );
    let (passed, msg, _, _) = gate().run_security_gate_with(&cargo);
    assert!(passed);
    assert!(msg.contains("No security vulnerabilities"));
    assert!(marker.exists(), "the stub must be what ran");
}

#[test]
fn test_security_gate_stub_vulnerable_fails() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cargo = stub_cargo(dir.path(), "#!/bin/sh\nexit 1\n");
    let (passed, msg, _, _) = gate().run_security_gate_with(&cargo);
    assert!(!passed);
    assert!(msg.contains("vulnerabilities detected"));
}

#[test]
fn test_security_gate_missing_cargo_skips() {
    let (passed, msg, _, _) = gate().run_security_gate_with("/nonexistent/no-such-cargo");
    assert!(passed);
    assert!(msg.contains("skipped"));
}

#[test]
fn test_security_gate_default_path_never_spawns_real_cargo() {
    // Default config has security enabled; under cfg(test) the seam resolves
    // to a nonexistent program, so the gate skips instead of auditing.
    assert_eq!(security_cargo(), "/nonexistent/hermetic-cargo-542");
    let result = gate().run_gate("security");
    assert!(result.passed);
    assert!(result.message.contains("skipped"), "{}", result.message);
}

#[test]
fn test_security_gate_disabled_does_not_spawn() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let marker = dir.path().join("ran");
    let cargo = stub_cargo(
        dir.path(),
        &format!("#!/bin/sh\ntouch '{}'\nexit 1\n", marker.display()),
    );
    let mut config = GateConfig::default();
    config.gates.security.enabled = false;
    let (passed, msg, _, _) = QualityGate::new(config).run_security_gate_with(&cargo);
    assert!(passed && msg.contains("disabled"));
    assert!(!marker.exists());
}
