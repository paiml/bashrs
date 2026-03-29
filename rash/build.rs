//! Build script for bashrs crate.
//!
//! This build script configures cargo to recognize the `kani` cfg for verification.

fn main() {
    // Allow kani cfg for verification
    println!("cargo::rustc-check-cfg=cfg(kani)");

    // ── provable-contracts binding enforcement (AllImplemented) ──
    {
        let binding_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("provable-contracts/contracts/bashrs/binding.yaml");

        println!("cargo:rerun-if-changed={}", binding_path.display());

        if binding_path.exists() {
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

            if let Ok(yaml) = std::fs::read_to_string(&binding_path) {
                if let Ok(bf) = serde_yaml_ng::from_str::<BF>(&yaml) {
                    let (mut imp, mut gaps) = (0u32, Vec::new());
                    for b in &bf.bindings {
                        let var = format!(
                            "CONTRACT_{}_{}",
                            b.contract
                                .trim_end_matches(".yaml")
                                .to_uppercase()
                                .replace('-', "_"),
                            b.equation.to_uppercase().replace('-', "_")
                        );
                        println!("cargo:rustc-env={var}={}", b.status);
                        if b.status == "implemented" {
                            imp += 1;
                        } else {
                            gaps.push(var.clone());
                        }
                    }
                    let total = bf.bindings.len() as u32;
                    println!("cargo:warning=[contract] AllImplemented: {imp}/{total} implemented, {} gaps", gaps.len());
                    if !gaps.is_empty() {
                        for g in &gaps {
                            println!("cargo:warning=[contract] UNALLOWED GAP: {g}");
                        }
                        panic!(
                            "[contract] AllImplemented: {} gap(s). Fix bindings or update status.",
                            gaps.len()
                        );
                    }

                    // ── Layer 2: Verify bound functions exist in source ──
                    {
                        let mut expected: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        for b in &bf.bindings {
                            if b.status == "implemented" {
                                if let Some(ref func) = b.function {
                                    let short =
                                        func.rsplit("::").next().unwrap_or(func).to_lowercase();
                                    expected.insert(short);
                                }
                            }
                        }
                        if !expected.is_empty() {
                            let mut found: std::collections::HashSet<String> =
                                std::collections::HashSet::new();
                            fn scan_rs(
                                dir: &std::path::Path,
                                found: &mut std::collections::HashSet<String>,
                            ) {
                                let Ok(entries) = std::fs::read_dir(dir) else {
                                    return;
                                };
                                for e in entries.flatten() {
                                    let p = e.path();
                                    if p.is_dir() {
                                        let n =
                                            p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                        if n != "target" && n != ".git" {
                                            scan_rs(&p, found);
                                        }
                                    } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                                        if let Ok(c) = std::fs::read_to_string(&p) {
                                            for line in c.lines() {
                                                let t = line.trim();
                                                if t.starts_with("pub fn ")
                                                    || t.starts_with("pub async fn ")
                                                    || t.starts_with("pub(crate) fn ")
                                                {
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
                                                    if !name.is_empty() {
                                                        found.insert(name);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            scan_rs(std::path::Path::new("src"), &mut found);
                            scan_rs(std::path::Path::new("crates"), &mut found);
                            let missing: Vec<_> = expected
                                .iter()
                                .filter(|n| !found.contains(n.as_str()))
                                .collect();
                            if !missing.is_empty() {
                                println!("cargo:warning=[contract] L2: {} bound function(s) not found in source (soft warning)", missing.len());
                            }
                        }
                    }
                }
            }
        }
    }
}
