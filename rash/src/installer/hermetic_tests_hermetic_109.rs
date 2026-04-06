
use super::*;
use tempfile::TempDir;

// =========================================================================
// EXTREME TDD Tests for Hermetic Builds (#109)
// =========================================================================

#[test]
fn test_HERMETIC_109_lockfile_create() {
    let lockfile = Lockfile::new();
    assert!(lockfile.artifacts.is_empty());
    assert!(!lockfile.generator.is_empty());
}

#[test]
fn test_HERMETIC_109_lockfile_add_artifact() {
    let mut lockfile = Lockfile::new();
    let artifact = LockedArtifact::new(
        "docker-ce",
        "24.0.7",
        "https://example.com/docker.deb",
        "abc123",
        12345678,
    );

    lockfile.add_artifact(artifact);
    lockfile.finalize();

    assert_eq!(lockfile.artifacts.len(), 1);
    assert!(!lockfile.content_hash.is_empty());
}

#[test]
fn test_HERMETIC_109_lockfile_get_artifact() {
    let mut lockfile = Lockfile::new();
    lockfile.add_artifact(LockedArtifact::new("test", "1.0", "url", "hash", 100));

    assert!(lockfile.get_artifact("test").is_some());
    assert!(lockfile.get_artifact("nonexistent").is_none());
}

#[test]
fn test_HERMETIC_109_lockfile_toml_roundtrip() {
    let mut lockfile = Lockfile::new();
    lockfile.add_artifact(LockedArtifact::new(
        "pkg-1",
        "1.0.0",
        "https://example.com/pkg-1.tar.gz",
        "sha256:abc123",
        1024,
    ));
    lockfile.add_artifact(LockedArtifact::new(
        "pkg-2",
        "2.0.0",
        "https://example.com/pkg-2.tar.gz",
        "sha256:def456",
        2048,
    ));
    lockfile.finalize();

    let toml = lockfile.to_toml();
    let parsed = Lockfile::from_toml(&toml).unwrap();

    assert_eq!(parsed.artifacts.len(), 2);
    assert_eq!(parsed.artifacts[0].id, "pkg-1");
    assert_eq!(parsed.artifacts[1].id, "pkg-2");
    assert_eq!(parsed.content_hash, lockfile.content_hash);
}

#[test]
fn test_HERMETIC_109_lockfile_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let lockfile_path = temp_dir.path().join("installer.lock");

    // Create and save
    {
        let mut lockfile = Lockfile::new();
        lockfile.add_artifact(LockedArtifact::new("test", "1.0", "url", "hash", 100));
        lockfile.finalize();
        lockfile.save(&lockfile_path).unwrap();
    }

    // Load and verify
    {
        let lockfile = Lockfile::load(&lockfile_path).unwrap();
        assert_eq!(lockfile.artifacts.len(), 1);
        assert_eq!(lockfile.artifacts[0].id, "test");
    }
}

#[test]
fn test_HERMETIC_109_lockfile_verify_integrity() {
    let mut lockfile = Lockfile::new();
    lockfile.add_artifact(LockedArtifact::new("test", "1.0", "url", "hash", 100));
    lockfile.finalize();

    // Valid lockfile should verify
    assert!(lockfile.verify().is_ok());

    // Tampered lockfile should fail
    lockfile.artifacts[0].sha256 = "tampered".to_string();
    assert!(lockfile.verify().is_err());
}

#[test]
fn test_HERMETIC_109_context_create() {
    let mut lockfile = Lockfile::new();
    lockfile.add_artifact(LockedArtifact::new("test", "1.0", "url", "hash123", 100));
    lockfile.finalize();

    let context = HermeticContext::from_lockfile(lockfile).unwrap();
    assert!(context.has_artifact("test"));
    assert!(!context.has_artifact("nonexistent"));
}

#[test]
fn test_HERMETIC_109_context_verify_artifact() {
    let mut lockfile = Lockfile::new();
    lockfile.add_artifact(LockedArtifact::new(
        "test",
        "1.0",
        "url",
        "correct-hash",
        100,
    ));
    lockfile.finalize();

    let context = HermeticContext::from_lockfile(lockfile).unwrap();

    // Correct hash should pass
    assert!(context.verify_artifact("test", "correct-hash").is_ok());

    // Wrong hash should fail
    assert!(context.verify_artifact("test", "wrong-hash").is_err());

    // Unknown artifact should fail
    assert!(context.verify_artifact("unknown", "any").is_err());
}

#[test]
fn test_HERMETIC_109_environment_capture() {
    let env = LockfileEnvironment::capture();
    assert!(!env.lc_all.is_empty());
    assert!(!env.tz.is_empty());
}

#[test]
fn test_HERMETIC_109_environment_deterministic() {
    let env = LockfileEnvironment::deterministic(1234567890);
    assert_eq!(env.source_date_epoch, 1234567890);
    assert_eq!(env.lc_all, "C.UTF-8");
    assert_eq!(env.tz, "UTC");
}

#[test]
fn test_HERMETIC_109_source_date_epoch() {
    let mut lockfile = Lockfile::new();
    lockfile.environment = LockfileEnvironment::deterministic(1700000000);
    lockfile.finalize();

    let context = HermeticContext::from_lockfile(lockfile).unwrap();
    assert_eq!(context.source_date_epoch(), 1700000000);
}
