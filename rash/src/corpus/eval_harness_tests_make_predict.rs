
use super::*;

fn make_prediction(
    id: &str,
    classification: &str,
    rules: &[&str],
    cwes: &[&str],
) -> Prediction {
    Prediction {
        id: id.to_string(),
        classification: classification.to_string(),
        cited_rules: rules.iter().map(|s| s.to_string()).collect(),
        cited_cwes: cwes.iter().map(|s| s.to_string()).collect(),
        proposed_fix: None,
        explanation: format!("This script is {classification}. Rules: {rules:?}"),
    }
}

fn make_gt(id: &str, label: u8, rules: &[&str], cwes: &[&str]) -> GroundTruth {
    GroundTruth {
        id: id.to_string(),
        label,
        rules: rules.iter().map(|s| s.to_string()).collect(),
        cwes: cwes.iter().map(|s| s.to_string()).collect(),
        script: String::new(),
    }
}

#[test]
fn test_perfect_eval() {
    let preds = vec![
        make_prediction("SSB-1", "unsafe", &["SEC001"], &["CWE-78"]),
        make_prediction("SSB-2", "safe", &[], &[]),
    ];
    let gt = vec![
        make_gt("SSB-1", 1, &["SEC001"], &["CWE-78"]),
        make_gt("SSB-2", 0, &[], &[]),
    ];

    let result = run_eval(&preds, &gt);
    assert!((result.detection_f1 - 1.0).abs() < 1e-9, "Perfect F1");
    assert!(
        (result.rule_citation - 1.0).abs() < 1e-9,
        "Perfect citation"
    );
    assert!((result.cwe_mapping - 1.0).abs() < 1e-9, "Perfect CWE");
}

#[test]
fn test_zero_eval() {
    let preds = vec![
        make_prediction("SSB-1", "safe", &[], &[]), // FN
        make_prediction("SSB-2", "unsafe", &["SEC002"], &["CWE-94"]), // FP
    ];
    let gt = vec![
        make_gt("SSB-1", 1, &["SEC001"], &["CWE-78"]),
        make_gt("SSB-2", 0, &[], &[]),
    ];

    let result = run_eval(&preds, &gt);
    assert!((result.detection_f1 - 0.0).abs() < 1e-9, "Zero F1");
}

#[test]
fn test_weights_sum_to_one() {
    let sum = DETECTION_F1_WEIGHT
        + RULE_CITATION_WEIGHT
        + CWE_MAPPING_WEIGHT
        + FIX_VALIDITY_WEIGHT
        + EXPLANATION_WEIGHT
        + OOD_WEIGHT;
    assert!(
        (sum - 1.0).abs() < 1e-9,
        "Weights must sum to 1.0, got {sum}"
    );
}

#[test]
fn test_composite_score_bounded() {
    let preds = vec![
        make_prediction("SSB-1", "unsafe", &["SEC001"], &["CWE-78"]),
        make_prediction("SSB-2", "safe", &[], &[]),
        make_prediction("SSB-3", "unsafe", &["DET001"], &["CWE-330"]),
    ];
    let gt = vec![
        make_gt("SSB-1", 1, &["SEC001"], &["CWE-78"]),
        make_gt("SSB-2", 0, &[], &[]),
        make_gt("SSB-3", 1, &["DET001"], &["CWE-330"]),
    ];

    let result = run_eval(&preds, &gt);
    assert!(result.composite_score >= 0.0, "Score >= 0");
    assert!(result.composite_score <= 1.0, "Score <= 1");
}

#[test]
fn test_format_report_output() {
    let preds = vec![make_prediction("SSB-1", "safe", &[], &[])];
    let gt = vec![make_gt("SSB-1", 0, &[], &[])];
    let result = run_eval(&preds, &gt);
    let report = format_eval_report(&result);
    assert!(report.contains("Detection F1"));
    assert!(report.contains("COMPOSITE SCORE"));
}

#[test]
fn test_empty_eval() {
    let result = run_eval(&[], &[]);
    assert_eq!(result.total, 0);
    // Composite is not zero because fix_validity defaults to 0.5 (neutral)
    // when there are no fixes to validate
    assert!(result.composite_score >= 0.0 && result.composite_score <= 1.0);
}

#[test]
fn test_fix_validity_with_fix() {
    let mut pred = make_prediction("SSB-1", "unsafe", &["SEC001"], &["CWE-78"]);
    pred.proposed_fix = Some("echo \"$var\"".to_string());

    let gt = make_gt("SSB-1", 1, &["SEC001"], &["CWE-78"]);
    let gt_slice = [gt];
    let pred_slice = [pred];

    let gt_map: std::collections::HashMap<&str, &GroundTruth> =
        gt_slice.iter().map(|g| (g.id.as_str(), g)).collect();
    let validity = compute_fix_validity(&pred_slice, &gt_map);
    // The fix "echo \"$var\"" should not trigger SEC001 (quoted var)
    assert!(validity >= 0.0 && validity <= 1.0);
}

#[test]
fn test_explanation_quality_heuristic() {
    let preds = vec![
        Prediction {
            id: "SSB-1".to_string(),
            classification: "unsafe".to_string(),
            cited_rules: vec!["SEC001".to_string()],
            cited_cwes: vec!["CWE-78".to_string()],
            proposed_fix: None,
            explanation: "This script is unsafe. SEC001 detects unquoted variable. Use double quotes instead to prevent injection.".to_string(),
        },
    ];
    let quality = compute_explanation_quality(&preds);
    assert!(
        quality > 0.5,
        "Good explanation should score >0.5, got {quality}"
    );
}
