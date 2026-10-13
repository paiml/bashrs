//! RUSTSEC-2024-0449: `unsafe-libyaml` must not be in the graph bashrs
//! ships (normal and build dependencies, default features). It used to come
//! in through a `serde_yaml_ng` build-dependency of `rash/build.rs`, so every
//! crate depending on bashrs inherited the advisory.
//!
//! The test asks cargo itself (`cargo tree`), not a hand-kept list, and
//! asserts it actually read a graph before it trusts an empty match.

use std::process::Command;

#[test]
fn bashrs_shipped_graph_has_no_unsafe_libyaml() {
    let out = Command::new(env!("CARGO"))
        .args([
            "tree",
            "--locked",
            "--offline",
            "-p",
            "bashrs",
            "-e",
            "normal,build",
            "--prefix",
            "none",
            "--format",
            "{p}",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo tree");
    assert!(
        out.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);

    // A tree that does not contain bashrs itself, or is near-empty, measured nothing.
    assert!(
        tree.lines().any(|l| l.starts_with("bashrs v")),
        "cargo tree output does not contain bashrs:\n{tree}"
    );
    assert!(
        tree.lines().count() > 20,
        "cargo tree output is implausibly small ({} lines)",
        tree.lines().count()
    );

    let hits: Vec<&str> = tree
        .lines()
        .filter(|l| l.starts_with("unsafe-libyaml ") || l.starts_with("serde_yaml_ng "))
        .collect();
    assert!(
        hits.is_empty(),
        "bashrs (normal+build, default features) still resolves: {hits:?}"
    );
}
