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
                }
            }
        }
    }
}
