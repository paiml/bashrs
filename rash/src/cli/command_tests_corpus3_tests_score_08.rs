#[cfg(test)]
mod decision_score_impact_color {
    use super::super::corpus_decision_commands::score_impact_color;

    #[test]
    fn test_score_08_is_high() {
        let (impact, _color) = score_impact_color(0.8);
        assert!(
            impact.contains("HIGH"),
            "Score 0.8 should be HIGH: {impact}"
        );
    }

    #[test]
    fn test_score_1_0_is_high() {
        let (impact, _color) = score_impact_color(1.0);
        assert!(
            impact.contains("HIGH"),
            "Score 1.0 should be HIGH: {impact}"
        );
    }

    #[test]
    fn test_score_0_5_is_medium() {
        let (impact, _color) = score_impact_color(0.5);
        assert!(
            impact.contains("MEDIUM"),
            "Score 0.5 should be MEDIUM: {impact}"
        );
    }

    #[test]
    fn test_score_0_7_is_medium() {
        let (impact, _color) = score_impact_color(0.7);
        assert!(
            impact.contains("MEDIUM"),
            "Score 0.7 should be MEDIUM: {impact}"
        );
    }

    #[test]
    fn test_score_0_0_is_low() {
        let (impact, _color) = score_impact_color(0.0);
        assert!(impact.contains("LOW"), "Score 0.0 should be LOW: {impact}");
    }

    #[test]
    fn test_score_0_49_is_low() {
        let (impact, _color) = score_impact_color(0.49);
        assert!(impact.contains("LOW"), "Score 0.49 should be LOW: {impact}");
    }

    #[test]
    fn test_returns_color_str() {
        let (_impact, color) = score_impact_color(0.9);
        // Color should be a non-empty ANSI escape or similar string reference
        assert!(!color.is_empty(), "Color should not be empty");
    }
}
