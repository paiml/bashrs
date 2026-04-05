mod helper_functions {
    use super::*;

    #[test]
    fn test_lookup_base_image_size_exact() {
        assert!(lookup_base_image_size("alpine:latest") > 0);
        assert!(lookup_base_image_size("ubuntu:22.04") > 0);
        assert!(lookup_base_image_size("python:3.11-slim") > 0);
    }

    #[test]
    fn test_lookup_base_image_size_prefix() {
        // Should match prefix for tagged versions
        let alpine_size = lookup_base_image_size("alpine:3.19");
        assert!(alpine_size > 0);
    }

    #[test]
    fn test_lookup_package_size() {
        assert!(lookup_package_size("numpy") > 0);
        assert!(lookup_package_size("tensorflow") > 0);
        assert!(lookup_package_size("build-essential") > 0);
    }

    #[test]
    fn test_extract_apt_packages() {
        let packages = extract_apt_packages("apt-get install -y python3 python3-pip git");
        assert!(packages.contains(&"python3".to_string()));
        assert!(packages.contains(&"python3-pip".to_string()));
        assert!(packages.contains(&"git".to_string()));
    }

    #[test]
    fn test_extract_pip_packages() {
        let packages = extract_pip_packages("pip install numpy pandas scipy");
        assert!(packages.contains(&"numpy".to_string()));
        assert!(packages.contains(&"pandas".to_string()));
        assert!(packages.contains(&"scipy".to_string()));
    }

    #[test]
    fn test_parse_docker_size() {
        assert_eq!(parse_docker_size("1.5GB"), Some(1_500_000_000));
        assert_eq!(parse_docker_size("500MB"), Some(500_000_000));
        assert_eq!(parse_docker_size("100KB"), Some(100_000));
        assert_eq!(parse_docker_size("1024B"), Some(1024));
    }

    #[test]
    fn test_parse_docker_size_lowercase() {
        assert_eq!(parse_docker_size("1.5gb"), Some(1_500_000_000));
        assert_eq!(parse_docker_size("500mb"), Some(500_000_000));
    }
}

mod list_rules {
    use super::*;

    #[test]
    fn test_list_size_rules() {
        let rules = list_size_rules();

        assert!(rules.iter().any(|(code, _)| *code == "SIZE001"));
        assert!(rules.iter().any(|(code, _)| *code == "SIZE002"));
        assert!(rules.iter().any(|(code, _)| *code == "SIZE003"));
        assert!(rules.iter().any(|(code, _)| *code == "SIZE004"));
        assert!(rules.iter().any(|(code, _)| *code == "SIZE-LIMIT"));
    }
}

// Property-based tests
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_estimate_never_panics(
            base_image in "FROM [a-z]{3,10}:[a-z0-9.]{1,10}",
            run_cmd in "RUN [a-z -]{0,50}"
        ) {
            let dockerfile = format!("{}\n{}", base_image, run_cmd);
            let _ = estimate_size(&dockerfile);
        }

        #[test]
        fn prop_total_is_sum_of_layers(
            base_image in "FROM alpine:latest|FROM ubuntu:22.04|FROM python:3.11",
            num_runs in 0usize..5
        ) {
            let mut dockerfile = base_image;
            for _ in 0..num_runs {
                dockerfile.push_str("\nRUN echo hello");
            }

            let estimate = estimate_size(&dockerfile);
            let sum: u64 = estimate.layer_estimates.iter().map(|l| l.estimated_size).sum();
            prop_assert_eq!(estimate.total_estimated, sum);
        }

        #[test]
        fn prop_format_json_valid(dockerfile in "FROM [a-z]+:[a-z0-9]+\nRUN [a-z ]{0,30}") {
            let estimate = estimate_size(&dockerfile);
            let json = format_size_estimate_json(&estimate);
            let parsed: std::result::Result<serde_json::Value, _> = serde_json::from_str(&json);
            prop_assert!(parsed.is_ok());
        }
    }
}
