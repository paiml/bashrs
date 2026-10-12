//! Build script for bashrs crate.
//!
//! This build script configures cargo to recognize the `kani` cfg for verification.

use std::collections::HashSet;
use std::path::Path;

#[derive(serde::Deserialize)]
struct BF {
    #[allow(dead_code)]
    version: String,
    bindings: Vec<B>,
}

#[derive(serde::Deserialize)]
struct B {
    contract: String,
    equation: String,
    status: String,
    #[serde(default)]
    function: Option<String>,
}

fn main() {
    // Allow kani cfg for verification
    println!("cargo::rustc-check-cfg=cfg(kani)");

    // ── provable-contracts binding enforcement (AllImplemented) ──
    enforce_bindings();
}

/// `<workspace parent>/provable-contracts/contracts/bashrs/binding.yaml`.
fn binding_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .parent()
        .unwrap_or(Path::new("."))
        .join("provable-contracts/contracts/bashrs/binding.yaml")
}

fn enforce_bindings() {
    let binding_path = binding_path();
    println!("cargo:rerun-if-changed={}", binding_path.display());
    if !binding_path.exists() {
        return;
    }
    let Ok(yaml) = std::fs::read_to_string(&binding_path) else {
        return;
    };
    let Some(bf) = parse_bindings(&yaml) else {
        return;
    };
    check_all_implemented(&bf);
    // ── Layer 2: Verify bound functions exist in source ──
    check_bound_functions(&bf);
}

fn parse_bindings(yaml: &str) -> Option<BF> {
    // RUSTSEC-2024-0449 (PMAT-450): serde-saphyr, held to what serde_yaml_ng read:
    // YAML 1.2 booleans only, `<<` is an ordinary key, a duplicate key is an error.
    let opts = serde_saphyr::options! {
        strict_booleans: true,
        duplicate_keys: serde_saphyr::options::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::options::MergeKeyPolicy::AsOrdinary,
    };
    serde_saphyr::from_str_with_options::<BF>(yaml, opts).ok()
}

fn binding_env_var(b: &B) -> String {
    format!(
        "CONTRACT_{}_{}",
        b.contract
            .trim_end_matches(".yaml")
            .to_uppercase()
            .replace('-', "_"),
        b.equation.to_uppercase().replace('-', "_")
    )
}

fn check_all_implemented(bf: &BF) {
    let (mut imp, mut gaps) = (0u32, Vec::new());
    for b in &bf.bindings {
        let var = binding_env_var(b);
        println!("cargo:rustc-env={var}={}", b.status);
        if b.status == "implemented" {
            imp += 1;
        } else {
            gaps.push(var);
        }
    }
    let total = bf.bindings.len() as u32;
    println!(
        "cargo:warning=[contract] AllImplemented: {imp}/{total} implemented, {} gaps",
        gaps.len()
    );
    if !gaps.is_empty() {
        for g in &gaps {
            println!("cargo:warning=[contract] UNALLOWED GAP: {g}");
        }
        panic!(
            "[contract] AllImplemented: {} gap(s). Fix bindings or update status.",
            gaps.len()
        );
    }
}

fn check_bound_functions(bf: &BF) {
    let expected: HashSet<String> = bf
        .bindings
        .iter()
        .filter(|b| b.status == "implemented")
        .filter_map(|b| b.function.as_deref())
        .map(|func| func.rsplit("::").next().unwrap_or(func).to_lowercase())
        .collect();
    if expected.is_empty() {
        return;
    }
    let mut found = HashSet::new();
    scan_rs(Path::new("src"), &mut found);
    scan_rs(Path::new("crates"), &mut found);
    let missing = expected
        .iter()
        .filter(|n| !found.contains(n.as_str()))
        .count();
    if missing > 0 {
        println!("cargo:warning=[contract] L2: {missing} bound function(s) not found in source (soft warning)");
    }
}

fn scan_rs(dir: &Path, found: &mut HashSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if n != "target" && n != ".git" {
                scan_rs(&p, found);
            }
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(c) = std::fs::read_to_string(&p) {
                collect_pub_fns(&c, found);
            }
        }
    }
}

fn collect_pub_fns(source: &str, found: &mut HashSet<String>) {
    for line in source.lines() {
        if let Some(name) = pub_fn_name(line.trim()) {
            found.insert(name);
        }
    }
}

/// Lower-cased name of a `pub fn` / `pub async fn` / `pub(crate) fn` line, if any.
fn pub_fn_name(t: &str) -> Option<String> {
    if !(t.starts_with("pub fn ")
        || t.starts_with("pub async fn ")
        || t.starts_with("pub(crate) fn "))
    {
        return None;
    }
    let part = t
        .trim_start_matches("pub async fn ")
        .trim_start_matches("pub(crate) fn ")
        .trim_start_matches("pub fn ");
    let name = part
        .split('(')
        .next()
        .unwrap_or("")
        .split('<')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    (!name.is_empty()).then_some(name)
}
