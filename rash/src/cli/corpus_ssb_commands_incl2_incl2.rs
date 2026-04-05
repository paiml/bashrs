/// Parse model output to extract classification, rule IDs, and CWE mappings.
///
/// Looks for "Classification: safe" or "Classification: unsafe" in the output.
/// Extracts SEC/DET/IDEM/SC rule patterns and maps them to CWEs.
#[cfg_attr(not(feature = "ml"), allow(dead_code))]
fn parse_batch_eval_output(output: &str) -> (String, Vec<String>, Vec<String>) {
    use crate::corpus::cwe_mapping;

    // Parse classification
    let lower = output.to_lowercase();
    let classification =
        if lower.contains("classification: unsafe") || lower.contains("classification:unsafe") {
            "unsafe".to_string()
        } else if lower.contains("classification: safe") || lower.contains("classification:safe") {
            "safe".to_string()
        } else if lower.contains("unsafe") {
            // Fallback: look for the word "unsafe" anywhere
            "unsafe".to_string()
        } else {
            "safe".to_string()
        };

    // Extract rule IDs (SEC001, DET002, IDEM003, SC2039, etc.)
    let mut rules: Vec<String> = Vec::new();
    for word in output.split(|c: char| !c.is_alphanumeric()) {
        let is_rule = match word.len() {
            6 if word.starts_with("SEC") && word[3..].chars().all(|c| c.is_ascii_digit()) => true,
            6 if word.starts_with("DET") && word[3..].chars().all(|c| c.is_ascii_digit()) => true,
            7 if word.starts_with("IDEM") && word[4..].chars().all(|c| c.is_ascii_digit()) => true,
            6 if word.starts_with("SC") && word[2..].chars().all(|c| c.is_ascii_digit()) => true,
            _ => false,
        };
        if is_rule && !rules.contains(&word.to_string()) {
            rules.push(word.to_string());
        }
    }

    // Map rules to CWEs
    let mut cwes: Vec<String> = Vec::new();
    for rule in &rules {
        if let Some(mapping) = cwe_mapping::lookup_rule(rule) {
            let cwe = mapping.cwe.to_string();
            if !cwes.contains(&cwe) {
                cwes.push(cwe);
            }
        }
    }

    (classification, rules, cwes)
}

/// Extract shell code from markdown code block in SSB input text.
///
/// Handles formats like:
/// ```bash
/// some code here
/// ```
fn extract_shell_from_markdown(input: &str) -> String {
    // Find ```bash or ```sh or ``` block
    let start_markers = ["```bash\n", "```sh\n", "```shell\n", "```\n"];
    for marker in &start_markers {
        if let Some(start_idx) = input.find(marker) {
            let code_start = start_idx + marker.len();
            if let Some(end_idx) = input[code_start..].find("```") {
                return input[code_start..code_start + end_idx].to_string();
            }
            // No closing ```, take rest of input
            return input[code_start..].to_string();
        }
    }
    // No markdown block found — return entire input as-is
    input.to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_batch_eval_output_classification_unsafe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("Classification: unsafe\nThis script uses eval.");
        assert_eq!(cls, "unsafe");
    }

    #[test]
    fn test_parse_batch_eval_output_classification_safe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("Classification: safe\nNo issues found.");
        assert_eq!(cls, "safe");
    }

    #[test]
    fn test_parse_batch_eval_output_fallback_unsafe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("This script is unsafe because it uses eval.");
        assert_eq!(cls, "unsafe");
    }

    #[test]
    fn test_parse_batch_eval_output_fallback_safe() {
        let (cls, _rules, _cwes) = parse_batch_eval_output("The script looks fine, no problems.");
        assert_eq!(cls, "safe");
    }

    #[test]
    fn test_parse_batch_eval_output_extracts_rules() {
        let (_cls, rules, _cwes) = parse_batch_eval_output(
            "Classification: unsafe\nViolations: SEC001 (command injection), SEC002 (unquoted var).",
        );
        assert!(rules.contains(&"SEC001".to_string()));
        assert!(rules.contains(&"SEC002".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_extracts_det_idem() {
        let (_cls, rules, _cwes) =
            parse_batch_eval_output("DET001 non-determinism, IDEM001 not idempotent.");
        assert!(rules.contains(&"DET001".to_string()));
        assert!(rules.contains(&"IDEM001".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_maps_cwes() {
        let (_cls, rules, cwes) =
            parse_batch_eval_output("Classification: unsafe\nSEC001 command injection detected.");
        assert!(rules.contains(&"SEC001".to_string()));
        // SEC001 maps to CWE-78
        assert!(cwes.contains(&"CWE-78".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_no_rules() {
        let (_cls, rules, cwes) = parse_batch_eval_output("Classification: safe\nAll good.");
        assert!(rules.is_empty());
        assert!(cwes.is_empty());
    }

    #[test]
    fn test_parse_batch_eval_output_deduplicates_rules() {
        let (_cls, rules, _cwes) =
            parse_batch_eval_output("SEC001 found here. SEC001 also found there.");
        assert_eq!(
            rules.iter().filter(|r| *r == "SEC001").count(),
            1,
            "SEC001 should appear exactly once"
        );
    }
}
