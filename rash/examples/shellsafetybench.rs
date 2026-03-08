//! Example: ShellSafetyBench pipeline components (SSC v12 S14).
//!
//! Demonstrates: CWE mapping, benchmark export, eval harness.
//!
//! Run: cargo run -p bashrs --example shellsafetybench

#![allow(clippy::unwrap_used)]

use bashrs::corpus::cwe_mapping;
use bashrs::corpus::eval_harness;

fn main() {
    println!("=== ShellSafetyBench Pipeline Components ===\n");

    // 1. CWE Mapping
    println!("1. CWE Taxonomy Mapping");
    println!(
        "   {} linter rules mapped to CWE IDs",
        cwe_mapping::CWE_MAPPINGS.len()
    );
    println!("   {} OOD CWEs for eval", cwe_mapping::OOD_CWES.len());
    println!("   Disjoint: {}", cwe_mapping::verify_ood_disjoint());
    println!();

    for m in cwe_mapping::CWE_MAPPINGS.iter().take(5) {
        println!(
            "   {} → {} (CVSS {:.1} {:?})",
            m.rule, m.cwe, m.cvss_score, m.cvss_severity
        );
    }
    println!("   ... ({} more)", cwe_mapping::CWE_MAPPINGS.len() - 5);
    println!();

    // 2. Eval Harness Demo
    println!("2. Eval Harness (6-metric weighted)");
    println!(
        "   Detection F1:      {:.0}%",
        eval_harness::DETECTION_F1_WEIGHT * 100.0
    );
    println!(
        "   Rule Citation:     {:.0}%",
        eval_harness::RULE_CITATION_WEIGHT * 100.0
    );
    println!(
        "   CWE Mapping:       {:.0}%",
        eval_harness::CWE_MAPPING_WEIGHT * 100.0
    );
    println!(
        "   Fix Validity:      {:.0}%",
        eval_harness::FIX_VALIDITY_WEIGHT * 100.0
    );
    println!(
        "   Explanation:       {:.0}%",
        eval_harness::EXPLANATION_WEIGHT * 100.0
    );
    println!(
        "   OOD Generalize:    {:.0}%",
        eval_harness::OOD_WEIGHT * 100.0
    );
    println!();

    // 3. Demo eval with synthetic predictions
    let predictions = vec![
        eval_harness::Prediction {
            id: "SSB-00001".to_string(),
            classification: "unsafe".to_string(),
            cited_rules: vec!["SEC001".to_string()],
            cited_cwes: vec!["CWE-78".to_string()],
            proposed_fix: Some("echo \"$var\"".to_string()),
            explanation: "Unquoted variable expansion. Use double quotes.".to_string(),
        },
        eval_harness::Prediction {
            id: "SSB-00002".to_string(),
            classification: "safe".to_string(),
            cited_rules: vec![],
            cited_cwes: vec![],
            proposed_fix: None,
            explanation: "No issues found.".to_string(),
        },
    ];

    let ground_truth = vec![
        eval_harness::GroundTruth {
            id: "SSB-00001".to_string(),
            label: 1,
            rules: vec!["SEC001".to_string()],
            cwes: vec!["CWE-78".to_string()],
            script: "echo $var".to_string(),
        },
        eval_harness::GroundTruth {
            id: "SSB-00002".to_string(),
            label: 0,
            rules: vec![],
            cwes: vec![],
            script: "echo 'hello'".to_string(),
        },
    ];

    let result = eval_harness::run_eval(&predictions, &ground_truth);
    print!("{}", eval_harness::format_eval_report(&result));

    // 4. Summary
    println!("\n3. Summary");
    println!(
        "   {} → {}",
        cwe_mapping::summary(),
        "CWE taxonomy verified"
    );
    println!("   Pipeline config: configs/pipeline/ssc.yaml");
    println!("   QA gate: configs/qa/ssc-release-v1.yaml");
    println!("   Training: configs/train/ssc-qwen3-4b-qlora.yaml");
    println!("   Contract: provable-contracts/contracts/shellsafetybench-v1.yaml");
}
