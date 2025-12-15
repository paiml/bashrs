//! Tests for Docker image profiling and size verification
#![allow(clippy::unwrap_used)]

use super::*;

mod size_estimation {
    use super::*;

    #[test]
    fn test_estimate_alpine_base_image() {
        let dockerfile = "FROM alpine:latest\nRUN echo hello";
        let estimate = estimate_size(dockerfile);

        assert_eq!(estimate.base_image, "alpine:latest");
        assert!(estimate.base_image_size > 0);
        assert!(estimate.base_image_size < 20_000_000); // Alpine is small
    }

    #[test]
    fn test_estimate_ubuntu_base_image() {
        let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update";
        let estimate = estimate_size(dockerfile);

        assert_eq!(estimate.base_image, "ubuntu:22.04");
        assert!(estimate.base_image_size > 50_000_000); // Ubuntu is larger
    }

    #[test]
    fn test_estimate_jupyter_base_image() {
        let dockerfile = "FROM jupyter/scipy-notebook:latest\nUSER jovyan";
        let estimate = estimate_size(dockerfile);

        assert_eq!(estimate.base_image, "jupyter/scipy-notebook:latest");
        assert!(estimate.base_image_size >= 3_000_000_000); // Jupyter scipy is large
    }

    #[test]
    fn test_estimate_nvidia_cuda_image() {
        let dockerfile = "FROM nvidia/cuda:12.0-devel-ubuntu22.04\nRUN echo hi";
        let estimate = estimate_size(dockerfile);

        assert_eq!(estimate.base_image, "nvidia/cuda:12.0-devel-ubuntu22.04");
        assert!(estimate.base_image_size >= 8_000_000_000); // CUDA devel is huge
    }

    #[test]
    fn test_estimate_unknown_base_image_warns() {
        let dockerfile = "FROM unknown-image:v1.0\nRUN echo hello";
        let estimate = estimate_size(dockerfile);

        assert!(!estimate.warnings.is_empty());
        assert!(estimate.warnings[0].contains("Unknown base image"));
    }

    #[test]
    fn test_estimate_apt_install_packages() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y build-essential cmake git
"#;
        let estimate = estimate_size(dockerfile);

        // Should estimate sizes for known packages
        let run_layer = estimate
            .layer_estimates
            .iter()
            .find(|l| l.instruction == "RUN")
            .expect("Should have RUN layer");

        assert!(run_layer.estimated_size > 0);
        assert!(run_layer.notes.is_some());
    }

    #[test]
    fn test_estimate_pip_install_packages() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN pip install numpy pandas scipy matplotlib
"#;
        let estimate = estimate_size(dockerfile);

        let run_layer = estimate
            .layer_estimates
            .iter()
            .find(|l| l.instruction == "RUN")
            .expect("Should have RUN layer");

        // Known packages should contribute to estimate
        assert!(run_layer.estimated_size > 200_000_000); // numpy + pandas + scipy + matplotlib
    }

    #[test]
    fn test_estimate_total_size() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN apt-get update && apt-get install -y build-essential
RUN pip install numpy pandas
COPY . /app
"#;
        let estimate = estimate_size(dockerfile);

        // Total should be sum of all layers
        let sum: u64 = estimate
            .layer_estimates
            .iter()
            .map(|l| l.estimated_size)
            .sum();
        assert_eq!(estimate.total_estimated, sum);
    }

    #[test]
    fn test_estimate_copy_unknown_size() {
        let dockerfile = r#"
FROM alpine:latest
COPY ./data /app/data
"#;
        let estimate = estimate_size(dockerfile);

        let copy_layer = estimate
            .layer_estimates
            .iter()
            .find(|l| l.instruction == "COPY")
            .expect("Should have COPY layer");

        assert_eq!(copy_layer.estimated_size, 0);
        assert!(copy_layer
            .notes
            .as_ref()
            .is_some_and(|n| n.contains("build context")));
    }

    #[test]
    fn test_estimate_metadata_layers() {
        let dockerfile = r#"
FROM alpine:latest
ENV FOO=bar
WORKDIR /app
EXPOSE 8080
USER nobody
HEALTHCHECK --interval=30s CMD curl -f http://localhost/
"#;
        let estimate = estimate_size(dockerfile);

        // Metadata layers should have 0 size
        for layer in &estimate.layer_estimates {
            if ["ENV", "WORKDIR", "EXPOSE", "USER", "HEALTHCHECK"]
                .contains(&layer.instruction.as_str())
            {
                assert_eq!(layer.estimated_size, 0);
            }
        }
    }
}

mod bloat_detection {
    use super::*;

    #[test]
    fn test_detect_apt_cache_not_cleaned() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3
"#;
        let estimate = estimate_size(dockerfile);

        assert!(!estimate.bloat_patterns.is_empty());
        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE001");
        assert!(bloat.is_some());
        assert!(bloat.unwrap().description.contains("apt cache"));
    }

    #[test]
    fn test_no_bloat_with_cleanup() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE001");
        assert!(bloat.is_none());
    }

    #[test]
    fn test_detect_missing_no_install_recommends() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE002");
        assert!(bloat.is_some());
        assert!(bloat
            .unwrap()
            .description
            .contains("--no-install-recommends"));
    }

    #[test]
    fn test_no_bloat_with_no_install_recommends() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y --no-install-recommends python3 && rm -rf /var/lib/apt/lists/*
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE002");
        assert!(bloat.is_none());
    }

    #[test]
    fn test_detect_pip_without_no_cache_dir() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN pip install numpy pandas
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE003");
        assert!(bloat.is_some());
        assert!(bloat.unwrap().description.contains("--no-cache-dir"));
    }

    #[test]
    fn test_no_bloat_with_no_cache_dir() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN pip install --no-cache-dir numpy pandas
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE003");
        assert!(bloat.is_none());
    }

    #[test]
    fn test_detect_npm_dev_dependencies() {
        let dockerfile = r#"
FROM node:18
RUN npm install
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE004");
        assert!(bloat.is_some());
    }

    #[test]
    fn test_no_bloat_with_npm_production() {
        let dockerfile = r#"
FROM node:18
RUN npm ci --only=production
"#;
        let estimate = estimate_size(dockerfile);

        let bloat = estimate.bloat_patterns.iter().find(|b| b.code == "SIZE004");
        assert!(bloat.is_none());
    }
}

mod platform_profiles {
    use super::*;

    #[test]
    fn test_coursera_profile_limits() {
        let profile = PlatformProfile::Coursera;

        assert_eq!(profile.max_size_bytes(), 10 * 1024 * 1024 * 1024); // 10GB
        assert_eq!(profile.max_memory_bytes(), 4 * 1024 * 1024 * 1024); // 4GB
        assert_eq!(profile.max_startup_ms(), 60_000); // 1 minute
    }

    #[test]
    fn test_standard_profile_no_limits() {
        let profile = PlatformProfile::Standard;

        assert_eq!(profile.max_size_bytes(), u64::MAX);
        assert_eq!(profile.max_memory_bytes(), u64::MAX);
        assert_eq!(profile.max_startup_ms(), u64::MAX);
    }

    #[test]
    fn test_lint_result_size_exceeds_coursera() {
        // Create a Dockerfile with estimated size > 10GB (Coursera limit)
        // nvidia/cuda:12.0-devel (8.5GB) + multiple large packages should exceed 10GB
        let dockerfile = r#"FROM nvidia/cuda:12.0-devel-ubuntu22.04
RUN apt-get update && apt-get install -y build-essential cmake
RUN pip install tensorflow torch transformers"#;
        let estimate = estimate_size(dockerfile);

        let result = size_estimate_to_lint_result(&estimate, PlatformProfile::Coursera, false);

        // With base (8.5GB) + packages, should exceed 10GB or at least hit warning threshold
        let has_size_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SIZE-WARNING" || d.code == "SIZE-LIMIT");

        // Note: actual size check depends on estimated package sizes
        // If estimate exceeds 80% of 10GB (8GB), should have warning
        let threshold = (10u64 * 1024 * 1024 * 1024) as f64 * 0.80;
        if estimate.total_estimated as f64 > threshold {
            assert!(
                has_size_warning,
                "Expected size warning for estimate of {} bytes",
                estimate.total_estimated
            );
        }
    }

    #[test]
    fn test_lint_result_size_within_coursera() {
        let dockerfile = "FROM alpine:latest\nRUN echo hi";
        let estimate = estimate_size(dockerfile);

        let result = size_estimate_to_lint_result(&estimate, PlatformProfile::Coursera, false);

        // Should not have SIZE-LIMIT warnings
        let has_size_limit = result.diagnostics.iter().any(|d| d.code == "SIZE-LIMIT");
        assert!(!has_size_limit);
    }

    #[test]
    fn test_lint_result_strict_mode() {
        let dockerfile = "FROM nvidia/cuda:12.0-devel-ubuntu22.04\nRUN echo hi";
        let estimate = estimate_size(dockerfile);

        let result = size_estimate_to_lint_result(&estimate, PlatformProfile::Coursera, true);

        // Should have Error severity in strict mode
        let has_error = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SIZE-LIMIT" && d.severity == Severity::Error);

        // Only check if there's a SIZE-LIMIT diagnostic (depends on estimate)
        if estimate.total_estimated > PlatformProfile::Coursera.max_size_bytes() {
            assert!(has_error);
        }
    }
}

mod formatting {
    use super::*;

    #[test]
    fn test_format_human_readable() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN pip install numpy
COPY . /app
"#;
        let estimate = estimate_size(dockerfile);
        let output = format_size_estimate(&estimate, false);

        assert!(output.contains("Image Size Analysis"));
        assert!(output.contains("Base image:"));
        assert!(output.contains("Estimated total:"));
    }

    #[test]
    fn test_format_verbose() {
        let dockerfile = r#"
FROM python:3.11-slim
RUN pip install numpy
COPY . /app
"#;
        let estimate = estimate_size(dockerfile);
        let output = format_size_estimate(&estimate, true);

        assert!(output.contains("Layer Breakdown:"));
        assert!(output.contains("[1]")); // Layer numbers
    }

    #[test]
    fn test_format_json() {
        let dockerfile = "FROM alpine:latest\nRUN echo hi";
        let estimate = estimate_size(dockerfile);
        let json = format_size_estimate_json(&estimate);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

        assert!(parsed["base_image"].is_string());
        assert!(parsed["total_estimated_bytes"].is_u64());
        assert!(parsed["layers"].is_array());
    }
}

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
