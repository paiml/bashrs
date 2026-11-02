//! Tests for `build_support/binding_yaml.rs`, the reader `build.rs` uses for
//! provable-contracts `binding.yaml` in place of a YAML crate.
//!
//! The golden (`data/binding_yaml/bashrs-binding.directives`) was captured
//! from the cargo directives the previous, `serde_yaml_ng`-based build script
//! printed for `data/binding_yaml/bashrs-binding.yaml`; the reader must
//! reproduce them byte for byte.

#[path = "../build_support/binding_yaml.rs"]
mod binding_yaml;

use binding_yaml::{env_var_name, parse_bindings, Binding};

const FIXTURE: &str = include_str!("data/binding_yaml/bashrs-binding.yaml");
const GOLDEN: &str = include_str!("data/binding_yaml/bashrs-binding.directives");

/// The directive lines build.rs prints for a successfully read file.
fn directives(bindings: &[Binding]) -> String {
    let mut out = String::new();
    let mut imp = 0u32;
    let mut gaps = 0usize;
    for b in bindings {
        out.push_str(&format!(
            "cargo:rustc-env={}={}\n",
            env_var_name(b),
            b.status
        ));
        if b.status == "implemented" {
            imp += 1;
        } else {
            gaps += 1;
        }
    }
    out.push_str(&format!(
        "cargo:warning=[contract] AllImplemented: {imp}/{} implemented, {gaps} gaps\n",
        bindings.len()
    ));
    out
}

#[test]
fn golden_directives_match_the_previous_build_script() {
    let bindings = parse_bindings(FIXTURE).expect("fixture parses");
    assert!(
        bindings.len() > 10,
        "fixture read {} bindings",
        bindings.len()
    );
    assert_eq!(directives(&bindings), GOLDEN);
}

#[test]
fn reads_function_and_quoted_values() {
    let src = "version: 1.0.0\nbindings:\n- contract: a-v1.yaml\n  equation: 'it''s'\n  function: \"x::y\" # c\n  status: implemented\n- contract: b.yaml\n  equation: e\n  status: partial\n  function: ~\n";
    let b = parse_bindings(src).expect("parses");
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].equation, "it's");
    assert_eq!(b[0].function.as_deref(), Some("x::y"));
    assert_eq!(b[1].function, None);
    assert_eq!(env_var_name(&b[0]), "CONTRACT_A_V1_IT'S");
}

#[test]
fn skips_unused_keys_with_nested_lines() {
    let src = "version: 1\ntarget_crate: x\ncritical_path:\n- a\n- b\npolicy:\n  k: v\nbindings:\n  - contract: c\n    notes: >-\n      long: text\n      - more\n    signature: 'fn f(a: u32, cache:\n      &mut C)'\n    equation: e\n    status: implemented\n";
    let b = parse_bindings(src).expect("parses");
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].contract, "c");
}

#[test]
fn empty_flow_bindings_is_an_empty_list() {
    assert_eq!(parse_bindings("version: 1\nbindings: []\n"), Ok(Vec::new()));
}

#[test]
fn refuses_what_it_does_not_fully_read() {
    let item = "version: 1\nbindings:\n- contract: c\n  equation: e\n";
    let cases = [
        ("missing version", "bindings: []\n".to_string()),
        ("missing bindings", "version: 1\n".to_string()),
        ("missing status", item.to_string()),
        (
            "block scalar",
            format!("{item}  status: |\n    implemented\n"),
        ),
        (
            "multi-line plain",
            format!("{item}  status: imple\n    mented\n"),
        ),
        (
            "flow item",
            "version: 1\nbindings:\n- {contract: c}\n".to_string(),
        ),
        ("anchor", format!("{item}  status: &a implemented\n")),
        ("escape", format!("{item}  status: \"impl\\u0065mented\"\n")),
        ("duplicate key", format!("{item}  status: x\n  status: y\n")),
        ("null required", format!("{item}  status: ~\n")),
        ("tab", format!("{item}\tstatus: x\n")),
        ("unterminated", format!("{item}  status: 'x\n")),
        ("bad indent", format!("{item} status: x\n")),
    ];
    for (name, src) in cases {
        assert!(parse_bindings(&src).is_err(), "{name} was accepted:\n{src}");
    }
}
