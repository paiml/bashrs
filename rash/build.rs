//! Build script for bashrs crate.
//!
//! This build script configures cargo to recognize the `kani` cfg for verification.
//! It reads provable-contracts `binding.yaml` with `build_support/binding_yaml.rs`,
//! a strict reader of that one shape, so no YAML crate is a build-dependency.

#[path = "build_support/binding_yaml.rs"]
mod binding_yaml;

use std::collections::HashSet;
use std::path::Path;

fn main() {
    // Allow kani cfg for verification
    println!("cargo::rustc-check-cfg=cfg(kani)");

    // ── provable-contracts binding enforcement (AllImplemented) ──
    let binding_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .parent()
        .unwrap_or(Path::new("."))
        .join("provable-contracts/contracts/bashrs/binding.yaml");

    println!("cargo:rerun-if-changed={}", binding_path.display());

    if !binding_path.exists() {
        return;
    }
    let Ok(yaml) = std::fs::read_to_string(&binding_path) else {
        return;
    };
    match binding_yaml::parse_bindings(&yaml) {
        Ok(bindings) => enforce_all_implemented(&bindings),
        Err(e) => println!("cargo:warning=[contract] binding.yaml not read: {e}"),
    }
}

/// One `rustc-env` per binding; any binding not `implemented` fails the build.
fn enforce_all_implemented(bindings: &[binding_yaml::Binding]) {
    let mut gaps = Vec::new();
    for b in bindings {
        let var = binding_yaml::env_var_name(b);
        println!("cargo:rustc-env={var}={}", b.status);
        if b.status != "implemented" {
            gaps.push(var);
        }
    }
    let imp = bindings.len() - gaps.len();
    println!(
        "cargo:warning=[contract] AllImplemented: {imp}/{} implemented, {} gaps",
        bindings.len(),
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
    check_bound_functions(bindings);
}

/// Layer 2: soft warning when a bound function is not declared in source.
fn check_bound_functions(bindings: &[binding_yaml::Binding]) {
    let expected: HashSet<String> = bindings
        .iter()
        .filter(|b| b.status == "implemented")
        .filter_map(|b| b.function.as_deref())
        .map(|f| f.rsplit("::").next().unwrap_or(f).to_lowercase())
        .collect();
    if expected.is_empty() {
        return;
    }
    let mut found = HashSet::new();
    scan_rs(Path::new("src"), &mut found);
    scan_rs(Path::new("crates"), &mut found);
    let missing = expected.iter().filter(|n| !found.contains(*n)).count();
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
                found.extend(c.lines().filter_map(pub_fn_name));
            }
        }
    }
}

/// The lowercased name declared by a `pub fn`, `pub async fn` or `pub(crate) fn` line.
fn pub_fn_name(line: &str) -> Option<String> {
    let t = line.trim();
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
