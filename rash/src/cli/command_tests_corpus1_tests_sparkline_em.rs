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
