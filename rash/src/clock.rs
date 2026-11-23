//! Wall-clock helpers (jiff). Replaces chrono (RUSTSEC-2026-0543, #537);
//! each helper keeps the text format of the chrono call it replaced.

use jiff::{Timestamp, Zoned};

/// RFC 3339 UTC, e.g. `2026-11-23T10:00:00.123456789+00:00`
/// (chrono's `Utc::now().to_rfc3339()`: offset form, not `Z`).
pub fn utc_rfc3339() -> String {
    format_rfc3339(Timestamp::now())
}

pub(crate) fn format_rfc3339(ts: Timestamp) -> String {
    let base = ts.strftime("%Y-%m-%dT%H:%M:%S").to_string();
    let nanos = ts.subsec_nanosecond();
    let frac = if nanos == 0 {
        String::new()
    } else if nanos % 1_000_000 == 0 {
        format!(".{:03}", nanos / 1_000_000)
    } else if nanos % 1_000 == 0 {
        format!(".{:06}", nanos / 1_000)
    } else {
        format!(".{nanos:09}")
    };
    format!("{base}{frac}+00:00")
}

/// Seconds since the Unix epoch (chrono's `Utc::now().timestamp()`).
pub fn unix_timestamp() -> i64 {
    Timestamp::now().as_second()
}

/// UTC time formatted with a strftime pattern (`Utc::now().format(..)`).
pub fn utc_format(pattern: &str) -> String {
    Timestamp::now().strftime(pattern).to_string()
}

/// Local time formatted with a strftime pattern (`Local::now().format(..)`).
pub fn local_format(pattern: &str) -> String {
    Zoned::now().strftime(pattern).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rfc3339_matches_chrono_shapes() {
        let ts = |s, n| Timestamp::new(s, n).expect("ts");
        assert_eq!(
            format_rfc3339(ts(1_795_428_000, 0)),
            "2026-11-23T10:00:00+00:00"
        );
        assert_eq!(
            format_rfc3339(ts(1_795_428_000, 123_000_000)),
            "2026-11-23T10:00:00.123+00:00"
        );
        assert_eq!(
            format_rfc3339(ts(1_795_428_000, 123_456_000)),
            "2026-11-23T10:00:00.123456+00:00"
        );
        assert_eq!(
            format_rfc3339(ts(1_795_428_000, 123_456_789)),
            "2026-11-23T10:00:00.123456789+00:00"
        );
    }

    #[test]
    fn test_utc_rfc3339_parses_back_and_is_utc_offset() {
        let s = utc_rfc3339();
        assert!(s.ends_with("+00:00"), "{s}");
        let parsed: Timestamp = s.parse().expect("jiff parses own output");
        assert!((Timestamp::now().as_second() - parsed.as_second()).abs() < 5);
    }

    #[test]
    fn test_unix_timestamp_close_to_now() {
        assert!((unix_timestamp() - Timestamp::now().as_second()).abs() < 5);
    }

    #[test]
    fn test_format_patterns_have_chrono_shapes() {
        assert_eq!(utc_format("%Y-%m-%d %H:%M:%S").len(), 19);
        assert_eq!(local_format("%Y-%m-%d_%H-%M-%S").len(), 19);
        assert_eq!(local_format("%H:%M:%S").len(), 8);
    }
}
