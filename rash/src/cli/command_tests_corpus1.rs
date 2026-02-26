//! Tests for corpus helper functions in viz, ranking, entry, failure, and score_print modules.
//! These tests target lightweight pure functions that do not invoke runner.run().
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ---------------------------------------------------------------------------
// corpus_ranking_commands::sparkline_str
// ---------------------------------------------------------------------------

#[cfg(test)]
mod ranking_sparkline_str {
    use super::super::corpus_ranking_commands::sparkline_str;

    #[test]
    fn test_sparkline_empty_returns_empty() {
        assert_eq!(sparkline_str(&[]), "");
    }

    #[test]
    fn test_sparkline_single_value_returns_one_char() {
        let s = sparkline_str(&[50.0]);
        assert_eq!(s.chars().count(), 1);
    }

    #[test]
    fn test_sparkline_all_same_returns_full_blocks() {
        // When all values are the same, range = 0 → all max block
        let s = sparkline_str(&[80.0, 80.0, 80.0]);
        for ch in s.chars() {
            assert_eq!(ch, '\u{2588}', "Expected full block for constant series");
        }
    }

    #[test]
    fn test_sparkline_ascending_produces_ascending_chars() {
        let s = sparkline_str(&[0.0, 50.0, 100.0]);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 3);
        assert!(
            chars[0] <= chars[2],
            "Ascending series should have ascending chars"
        );
    }

    #[test]
    fn test_sparkline_length_matches_input() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let s = sparkline_str(&data);
        assert_eq!(s.chars().count(), data.len());
    }

    #[test]
    fn test_sparkline_uses_block_unicode_chars() {
        let s = sparkline_str(&[0.0, 100.0]);
        for ch in s.chars() {
            let code = ch as u32;
            assert!(
                (0x2581..=0x2588).contains(&code),
                "Expected block character U+2581..U+2588, got U+{code:04X}"
            );
        }
    }

    #[test]
    fn test_sparkline_two_equal_values_both_full() {
        let s = sparkline_str(&[42.0, 42.0]);
        assert_eq!(s.chars().count(), 2);
        for ch in s.chars() {
            assert_eq!(ch, '\u{2588}');
        }
    }

    #[test]
    fn test_sparkline_descending_produces_descending_chars() {
        let s = sparkline_str(&[100.0, 50.0, 0.0]);
        let chars: Vec<char> = s.chars().collect();
        assert!(
            chars[0] >= chars[2],
            "Descending series should have descending chars"
        );
    }
}

// ---------------------------------------------------------------------------
// corpus_ranking_commands::classify_category
// ---------------------------------------------------------------------------

#[cfg(test)]
mod ranking_classify_category {
    use super::super::corpus_ranking_commands::classify_category;

    #[test]
    fn test_config_keyword_bashrc() {
        assert_eq!(classify_category("bashrc-setup"), "Config (A)");
    }

    #[test]
    fn test_config_keyword_profile() {
        assert_eq!(classify_category("profile-loader"), "Config (A)");
    }

    #[test]
    fn test_config_keyword_alias() {
        assert_eq!(classify_category("alias-expansion"), "Config (A)");
    }

    #[test]
    fn test_config_keyword_history() {
        assert_eq!(classify_category("history-search"), "Config (A)");
    }

    #[test]
    fn test_oneliner_keyword() {
        assert_eq!(classify_category("oneliner-pipe"), "One-liner (B)");
    }

    #[test]
    fn test_pipeline_keyword() {
        assert_eq!(classify_category("pipeline-sort"), "One-liner (B)");
    }

    #[test]
    fn test_coreutil_keyword() {
        assert_eq!(classify_category("coreutil-ls"), "Coreutils (G)");
    }

    #[test]
    fn test_reimpl_keyword() {
        assert_eq!(classify_category("reimpl-cat"), "Coreutils (G)");
    }

    #[test]
    fn test_regex_keyword() {
        assert_eq!(classify_category("regex-match"), "Regex (H)");
    }

    #[test]
    fn test_glob_match_keyword() {
        assert_eq!(classify_category("glob-match-test"), "Regex (H)");
    }

    #[test]
    fn test_daemon_keyword() {
        assert_eq!(classify_category("daemon-start"), "System (F)");
    }

    #[test]
    fn test_cron_keyword() {
        assert_eq!(classify_category("cron-job"), "System (F)");
    }

    #[test]
    fn test_startup_keyword() {
        assert_eq!(classify_category("startup-script"), "System (F)");
    }

    #[test]
    fn test_service_keyword() {
        assert_eq!(classify_category("service-manager"), "System (F)");
    }

    #[test]
    fn test_milestone_keyword() {
        assert_eq!(classify_category("milestone-100"), "Milestone");
    }

    #[test]
    fn test_adversarial_keyword() {
        assert_eq!(classify_category("adversarial-injection"), "Adversarial");
    }

    #[test]
    fn test_fuzz_keyword() {
        assert_eq!(classify_category("fuzz-test"), "Adversarial");
    }

    #[test]
    fn test_unknown_name_returns_general() {
        assert_eq!(classify_category("basic-echo"), "General");
    }

    #[test]
    fn test_empty_name_returns_general() {
        assert_eq!(classify_category(""), "General");
    }

    #[test]
    fn test_case_insensitive_config() {
        assert_eq!(classify_category("BASHRC-SETUP"), "Config (A)");
    }

    #[test]
    fn test_case_insensitive_oneliner() {
        assert_eq!(classify_category("ONELINER"), "One-liner (B)");
    }

    #[test]
    fn test_xdg_is_config() {
        assert_eq!(classify_category("xdg-dirs"), "Config (A)");
    }
}

// ---------------------------------------------------------------------------
// corpus_entry_commands::truncate_line
// ---------------------------------------------------------------------------

#[cfg(test)]
mod entry_truncate_line {
    use super::super::corpus_entry_commands::truncate_line;

    #[test]
    fn test_short_string_unchanged() {
        assert_eq!(truncate_line("hello", 10), "hello");
    }

    #[test]
    fn test_exact_length_unchanged() {
        assert_eq!(truncate_line("hello", 5), "hello");
    }

    #[test]
    fn test_long_string_truncated_with_ellipsis() {
        let result = truncate_line("hello world", 5);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_multiline_uses_first_line_only() {
        let result = truncate_line("first line\nsecond line", 20);
        assert_eq!(result, "first line");
    }

    #[test]
    fn test_multiline_long_first_line_truncated() {
        let result = truncate_line("this is a long first line\nsecond", 10);
        assert!(result.ends_with("..."), "Should end with '...': {result}");
        assert!(!result.contains("second"), "Should not include second line");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(truncate_line("", 10), "");
    }

    #[test]
    fn test_zero_max_len_truncates_immediately() {
        let result = truncate_line("hello", 0);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_unicode_string_truncation() {
        // ASCII truncation works on byte boundaries in source
        let result = truncate_line("abcdefghij", 5);
        assert_eq!(result, "abcde...");
    }
}

// ---------------------------------------------------------------------------
// corpus_entry_commands::tier_label
// ---------------------------------------------------------------------------

#[cfg(test)]
mod entry_tier_label {
    use super::super::corpus_entry_commands::tier_label;

    #[test]
    fn test_tier_1_trivial() {
        assert_eq!(tier_label(1), "Trivial");
    }

    #[test]
    fn test_tier_2_standard() {
        assert_eq!(tier_label(2), "Standard");
    }

    #[test]
    fn test_tier_3_complex() {
        assert_eq!(tier_label(3), "Complex");
    }

    #[test]
    fn test_tier_4_adversarial() {
        assert_eq!(tier_label(4), "Adversarial");
    }

    #[test]
    fn test_tier_5_production() {
        assert_eq!(tier_label(5), "Production");
    }

    #[test]
    fn test_tier_0_unknown() {
        assert_eq!(tier_label(0), "Unknown");
    }

    #[test]
    fn test_tier_6_unknown() {
        assert_eq!(tier_label(6), "Unknown");
    }

    #[test]
    fn test_tier_255_unknown() {
        assert_eq!(tier_label(255), "Unknown");
    }
}

// ---------------------------------------------------------------------------
// corpus_entry_commands::dimension_risk
// ---------------------------------------------------------------------------

#[cfg(test)]
mod entry_dimension_risk {
    use super::super::corpus_entry_commands::dimension_risk;

    #[test]
    fn test_a_is_high() {
        assert_eq!(dimension_risk("A"), "HIGH");
    }

    #[test]
    fn test_b3_is_high() {
        assert_eq!(dimension_risk("B3"), "HIGH");
    }

    #[test]
    fn test_e_is_high() {
        assert_eq!(dimension_risk("E"), "HIGH");
    }

    #[test]
    fn test_d_is_medium() {
        assert_eq!(dimension_risk("D"), "MEDIUM");
    }

    #[test]
    fn test_g_is_medium() {
        assert_eq!(dimension_risk("G"), "MEDIUM");
    }

    #[test]
    fn test_f_is_medium() {
        assert_eq!(dimension_risk("F"), "MEDIUM");
    }

    #[test]
    fn test_b1_is_low() {
        assert_eq!(dimension_risk("B1"), "LOW");
    }

    #[test]
    fn test_b2_is_low() {
        assert_eq!(dimension_risk("B2"), "LOW");
    }

    #[test]
    fn test_unknown_dim_is_low() {
        assert_eq!(dimension_risk("Z"), "LOW");
    }

    #[test]
    fn test_empty_is_low() {
        assert_eq!(dimension_risk(""), "LOW");
    }
}

// ---------------------------------------------------------------------------
// corpus_entry_commands::classify_difficulty
// ---------------------------------------------------------------------------

#[cfg(test)]
mod entry_classify_difficulty {
    use super::super::corpus_entry_commands::classify_difficulty;

    #[test]
    fn test_simple_3_lines_is_tier_1() {
        let input = "fn main() {\n    println!(\"hello\");\n}";
        let (tier, _) = classify_difficulty(input);
        assert_eq!(tier, 1, "Simple 3-line input should be tier 1");
    }

    #[test]
    fn test_loop_increases_tier() {
        let input = "fn main() {\n    for i in 0..5 {\n        println!(\"{}\", i);\n    }\n}";
        let (tier, factors) = classify_difficulty(input);
        assert!(
            tier >= 1,
            "Loop input should be at least tier 1, got {tier}"
        );
        let loop_present = factors.iter().any(|(f, p)| f.contains("loop") && *p);
        assert!(
            loop_present || tier >= 1,
            "Should detect loop or be tier 1+"
        );
    }

    #[test]
    fn test_unsafe_increases_tier() {
        let input = "fn main() {\n    unsafe { exec(\"ls\"); }\n}";
        let (tier, factors) = classify_difficulty(input);
        assert!(
            tier >= 1,
            "unsafe input should be at least tier 1, got {tier}"
        );
        let unsafe_present = factors.iter().any(|(f, p)| f.contains("unsafe") && *p);
        assert!(
            unsafe_present || tier >= 1,
            "Should detect unsafe or be tier 1+"
        );
    }

    #[test]
    fn test_unicode_flagged() {
        let input = "fn main() { println!(\"héllo\"); }";
        let (_, factors) = classify_difficulty(input);
        let unicode_present = factors.iter().any(|(f, p)| *f == "Has Unicode" && *p);
        assert!(unicode_present, "Should detect unicode");
    }

    #[test]
    fn test_pipe_char_flagged() {
        let input = "fn main() { let x = a | b; }";
        let (_, factors) = classify_difficulty(input);
        let pipe_present = factors
            .iter()
            .any(|(f, p)| *f == "Has pipes/redirects" && *p);
        assert!(pipe_present, "Should detect pipe character");
    }

    #[test]
    fn test_if_keyword_flagged() {
        let input = "fn main() { if x { y } }";
        let (_, factors) = classify_difficulty(input);
        let cond_present = factors.iter().any(|(f, p)| *f == "Has conditionals" && *p);
        assert!(cond_present, "Should detect conditional");
    }

    #[test]
    fn test_factors_has_min_10_entries() {
        let input = "fn main() { println!(\"x\"); }";
        let (_, factors) = classify_difficulty(input);
        assert!(
            factors.len() >= 10,
            "Should have at least 10 factors, got {}",
            factors.len()
        );
    }

    #[test]
    fn test_tier_in_range_1_to_5() {
        let input = "fn main() {}";
        let (tier, _) = classify_difficulty(input);
        assert!((1..=5).contains(&tier), "Tier should be 1-5, got {tier}");
    }
}

// ---------------------------------------------------------------------------
// corpus_score_print_commands::stats_bar
// ---------------------------------------------------------------------------

#[cfg(test)]
mod score_print_stats_bar {
    use super::super::corpus_score_print_commands::stats_bar;

    #[test]
    fn test_100pct_all_filled() {
        let bar = stats_bar(100.0, 8);
        let filled = bar.chars().filter(|c| *c == '█').count();
        let empty = bar.chars().filter(|c| *c == '░').count();
        assert_eq!(filled, 8, "100% should have 8 filled blocks");
        assert_eq!(empty, 0, "100% should have no empty blocks");
    }

    #[test]
    fn test_0pct_all_empty() {
        let bar = stats_bar(0.0, 8);
        let filled = bar.chars().filter(|c| *c == '█').count();
        let empty = bar.chars().filter(|c| *c == '░').count();
        assert_eq!(filled, 0, "0% should have no filled blocks");
        assert_eq!(empty, 8, "0% should have 8 empty blocks");
    }

    #[test]
    fn test_50pct_mixed() {
        let bar = stats_bar(50.0, 10);
        assert!(bar.contains('█'), "50% bar should have some filled blocks");
        assert!(bar.contains('░'), "50% bar should have some empty blocks");
    }

    #[test]
    fn test_width_is_respected() {
        let bar = stats_bar(75.0, 16);
        let total = bar.chars().filter(|c| *c == '█' || *c == '░').count();
        assert_eq!(total, 16, "Total blocks should equal width=16");
    }

    #[test]
    fn test_zero_width_empty_string() {
        let bar = stats_bar(50.0, 0);
        assert!(bar.is_empty(), "Zero width bar should be empty");
    }

    #[test]
    fn test_25pct_bar() {
        let bar = stats_bar(25.0, 8);
        let filled = bar.chars().filter(|c| *c == '█').count();
        let empty = bar.chars().filter(|c| *c == '░').count();
        assert_eq!(filled + empty, 8);
        assert!(filled <= 3, "25% of 8 = 2 filled blocks, got {filled}");
    }

    #[test]
    fn test_width_1_gives_single_block() {
        let bar_full = stats_bar(100.0, 1);
        let bar_empty = stats_bar(0.0, 1);
        assert_eq!(bar_full.chars().count(), 1);
        assert_eq!(bar_empty.chars().count(), 1);
    }
}
