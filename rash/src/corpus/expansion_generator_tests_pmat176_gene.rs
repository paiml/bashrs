#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_PMAT176_generate_bash_templates() {
        let scripts = generate_bash_templates(100, 42);
        assert_eq!(scripts.len(), 100);
        // Should have a mix of safe and unsafe
        let labels: Vec<u8> = scripts
            .iter()
            .map(|s| label_script(s, GenFormat::Bash))
            .collect();
        let safe = labels.iter().filter(|&&l| l == 0).count();
        let unsafe_count = labels.iter().filter(|&&l| l == 1).count();
        assert!(safe > 0, "should have safe entries");
        assert!(unsafe_count > 0, "should have unsafe entries");
    }

    #[test]
    fn test_PMAT176_generate_makefile_templates() {
        let scripts = generate_makefile_templates(100, 42);
        assert_eq!(scripts.len(), 100);
        let labels: Vec<u8> = scripts
            .iter()
            .map(|s| label_script(s, GenFormat::Makefile))
            .collect();
        let safe = labels.iter().filter(|&&l| l == 0).count();
        let unsafe_count = labels.iter().filter(|&&l| l == 1).count();
        assert!(safe > 0, "should have safe makefile entries");
        assert!(unsafe_count > 0, "should have unsafe makefile entries");
    }

    #[test]
    fn test_PMAT176_generate_dockerfile_templates() {
        let scripts = generate_dockerfile_templates(100, 42);
        assert_eq!(scripts.len(), 100);
        let labels: Vec<u8> = scripts
            .iter()
            .map(|s| label_script(s, GenFormat::Dockerfile))
            .collect();
        let safe = labels.iter().filter(|&&l| l == 0).count();
        let unsafe_count = labels.iter().filter(|&&l| l == 1).count();
        assert!(safe > 0, "should have safe dockerfile entries");
        assert!(unsafe_count > 0, "should have unsafe dockerfile entries");
    }

    #[test]
    fn test_PMAT176_generate_expansion_deterministic() {
        let entries1 = generate_expansion(GenFormat::Bash, 50, 42);
        let entries2 = generate_expansion(GenFormat::Bash, 50, 42);
        assert_eq!(entries1.len(), entries2.len());
        for (a, b) in entries1.iter().zip(entries2.iter()) {
            assert_eq!(a.input, b.input);
            assert_eq!(a.label, b.label);
        }
    }

    #[test]
    fn test_PMAT176_generate_expansion_different_seeds() {
        let entries1 = generate_expansion(GenFormat::Bash, 50, 42);
        let entries2 = generate_expansion(GenFormat::Bash, 50, 99);
        // Different seeds should produce different entries
        let different = entries1
            .iter()
            .zip(entries2.iter())
            .filter(|(a, b)| a.input != b.input)
            .count();
        assert!(
            different > 0,
            "different seeds should produce different entries"
        );
    }

    #[test]
    fn test_PMAT176_write_expansion_round_trip() {
        let entries = generate_expansion(GenFormat::Bash, 10, 42);
        let dir = tempfile::TempDir::new().expect("tmpdir");
        let path = dir.path().join("expansion.jsonl");
        let summary = write_expansion(&entries, &path).expect("write");
        assert_eq!(summary.total, 10);
        assert_eq!(summary.safe + summary.unsafe_count, 10);

        // Read back and verify
        let content = std::fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn test_PMAT176_label_script_safe_bash() {
        let label = label_script("echo \"hello world\"", GenFormat::Bash);
        assert_eq!(label, 0);
    }

    #[test]
    fn test_PMAT176_label_script_unsafe_eval() {
        let label = label_script("eval $USER_INPUT", GenFormat::Bash);
        assert_eq!(label, 1);
    }

    #[test]
    fn test_PMAT176_label_script_safe_dockerfile() {
        let label = label_script(
            "FROM alpine:3.18\nRUN apk add --no-cache curl\nUSER nobody",
            GenFormat::Dockerfile,
        );
        assert_eq!(label, 0);
    }
}
