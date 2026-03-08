//! Demonstrates the CWE taxonomy mapping for bashrs linter rules (SSC v12 S14.2).
//!
//! Run: `cargo run --example cwe_mapping_demo`

fn main() {
    println!("ShellSafetyBench CWE Taxonomy Mapping");
    println!("=====================================\n");

    // Show all linter rule → CWE mappings
    for m in bashrs::corpus::cwe_mapping::CWE_MAPPINGS {
        println!(
            "  {} → {} (CVSS {:.1} {}) — {}",
            m.rule, m.cwe, m.cvss_score, m.cvss_severity, m.owasp
        );
    }

    println!("\nOOD CWEs (eval-only, not in linter):");
    for o in bashrs::corpus::cwe_mapping::OOD_CWES {
        println!("  {} — {} (CVSS {:.1})", o.cwe, o.name, o.cvss_score);
    }

    // Demonstrate lookup
    println!("\nLookup SEC006:");
    if let Some(m) = bashrs::corpus::cwe_mapping::lookup_rule("SEC006") {
        println!(
            "  {} → {} ({:.1} {}) — {}",
            m.rule, m.cwe, m.cvss_score, m.cvss_severity, m.owasp
        );
    }

    // Summary
    println!("\n{}", bashrs::corpus::cwe_mapping::summary());

    // Verify OOD disjoint
    assert!(
        bashrs::corpus::cwe_mapping::verify_ood_disjoint(),
        "OOD CWEs must not overlap with linter CWEs"
    );
    println!("OOD disjoint check: PASSED");
}
