//! Differential: `../binding_yaml.rs` (the reader `rash/build.rs` uses for
//! provable-contracts `binding.yaml`) against `serde_yaml_ng`, the YAML crate
//! `build.rs` used before it (RUSTSEC-2024-0449).
//!
//! For every input the old script read, the reader must return the same
//! bindings, and every input it refuses must be a shape it documents as
//! refused. This package is its own workspace (see Cargo.toml), not a bashrs member, so
//! `serde_yaml_ng` never enters bashrs's dependency graph or Cargo.lock.
//!
//! The committed corpus is the bashrs `binding.yaml` fixture plus the inline
//! cases below. To run the same comparison over a provable-contracts
//! checkout, point `BINDING_YAML_CORPUS` at its `contracts/` directory; every
//! `*/binding.yaml` under it is compared, and the test fails if it finds none,
//! so a wrong path cannot pass by reading nothing:
//!
//! ```text
//! BINDING_YAML_CORPUS=../provable-contracts/contracts \
//!   cargo test --manifest-path rash/build_support/differential/Cargo.toml -- --nocapture
//! ```
#![cfg(test)]

#[path = "../binding_yaml.rs"]
#[allow(dead_code)]
mod binding_yaml;

use binding_yaml::{parse_bindings, Binding};

/// The shape the old `build.rs` deserialized, field for field.
#[derive(serde::Deserialize)]
struct OldFile {
    #[allow(dead_code)]
    version: String,
    bindings: Vec<OldBinding>,
}

#[derive(serde::Deserialize)]
struct OldBinding {
    contract: String,
    equation: String,
    status: String,
    #[serde(default)]
    function: Option<String>,
}

fn old_reader(src: &str) -> Option<Vec<Binding>> {
    let f: OldFile = serde_yaml_ng::from_str(src).ok()?;
    Some(
        f.bindings
            .into_iter()
            .map(|b| Binding {
                contract: b.contract,
                equation: b.equation,
                status: b.status,
                function: b.function,
            })
            .collect(),
    )
}

/// Compares one input. Returns `true` when both readers agree on the
/// bindings, `false` when the new reader refuses what the old one read.
/// Panics on any other disagreement.
fn compare(name: &str, src: &str) -> bool {
    match (parse_bindings(src), old_reader(src)) {
        (Ok(new), Some(old)) => {
            assert_eq!(new, old, "{name}: readers disagree");
            true
        }
        (Ok(new), None) => {
            panic!(
                "{name}: the reader accepted {} binding(s) serde_yaml_ng refused",
                new.len()
            )
        }
        (Err(_), Some(_)) => false,
        (Err(e), None) => panic!("{name}: neither reader read it ({e}); not a differential case"),
    }
}

const FIXTURE: &str = include_str!("../../tests/data/binding_yaml/bashrs-binding.yaml");

#[test]
fn bashrs_fixture_reads_the_same() {
    assert!(compare("bashrs-binding.yaml", FIXTURE));
    assert!(parse_bindings(FIXTURE).map(|b| b.len()).unwrap_or(0) > 10);
}

#[test]
fn accepted_shapes_read_the_same() {
    let cases = [
        (
            "quotes, comment, null function",
            "version: 1.0.0\nbindings:\n- contract: a-v1.yaml\n  equation: 'it''s'\n  function: \"x::y\" # c\n  status: implemented\n- contract: b.yaml\n  equation: e\n  status: partial\n  function: ~\n",
        ),
        (
            "unused keys with nested lines",
            "version: '1'\ntarget_crate: x\ncritical_path:\n- a\n- b\npolicy:\n  k: v\nbindings:\n  - contract: c\n    notes: >-\n      long: text\n      - more\n    signature: 'fn f(a: u32, cache:\n      &mut C)'\n    equation: e\n    status: implemented\n",
        ),
        ("empty flow list", "version: '1'\nbindings: []\n"),
    ];
    for (name, src) in cases {
        assert!(compare(name, src), "{name}: refused, serde_yaml_ng read it");
    }
}

#[test]
fn refused_shapes_are_the_documented_ones() {
    // serde_yaml_ng reads both; the reader refuses both by design (a null
    // required value, a block scalar in a key it uses).
    let item = "version: '1'\nbindings:\n- contract: c\n  equation: e\n";
    for (name, src) in [
        (
            "block scalar status",
            format!("{item}  status: |\n    implemented\n"),
        ),
        ("anchor", format!("{item}  status: &a implemented\n")),
    ] {
        assert!(!compare(name, &src), "{name}: expected a refusal");
    }
}

#[test]
fn corpus_from_env_reads_the_same() {
    let Ok(dir) = std::env::var("BINDING_YAML_CORPUS") else {
        eprintln!("BINDING_YAML_CORPUS unset: the corpus comparison did not run");
        return;
    };
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("BINDING_YAML_CORPUS={dir}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("binding.yaml"))
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "BINDING_YAML_CORPUS={dir}: no */binding.yaml found"
    );
    let (mut same, mut refused) = (0usize, Vec::new());
    for p in &files {
        let src = std::fs::read_to_string(p).expect("read binding.yaml");
        if compare(&p.display().to_string(), &src) {
            same += 1;
        } else {
            refused.push(p.display().to_string());
        }
    }
    eprintln!(
        "binding.yaml corpus: {} file(s), {same} identical, {} refused {refused:?}",
        files.len(),
        refused.len()
    );
    assert!(
        refused.is_empty(),
        "the reader refused corpus files serde_yaml_ng read: {refused:?}"
    );
}
