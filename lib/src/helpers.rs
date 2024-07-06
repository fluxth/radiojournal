use chrono::{DateTime, Datelike, SubsecRound, Timelike, Utc};

pub(crate) fn ziso_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}

pub fn truncate_datetime_to_minutes(dt: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Some(dt.with_second(0)?.trunc_subsecs(0))
}

pub fn truncate_datetime_to_days(dt: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Some(
        dt.with_hour(0)?
            .with_minute(0)?
            .with_second(0)?
            .trunc_subsecs(0),
    )
}

pub fn truncate_datetime_to_months(dt: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Some(
        dt.with_day0(0)?
            .with_hour(0)?
            .with_minute(0)?
            .with_second(0)?
            .trunc_subsecs(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(981173100000000000, "2001-02-03T04:05:00Z")] // minutes
    #[case(981173106000000000, "2001-02-03T04:05:06Z")] // seconds
    #[case(981173106700000000, "2001-02-03T04:05:06.700Z")] // milliseconds
    #[case(981173106780000000, "2001-02-03T04:05:06.780Z")]
    #[case(981173106789000000, "2001-02-03T04:05:06.789Z")]
    #[case(981173106789100000, "2001-02-03T04:05:06.789100Z")] // microseconds
    #[case(981173106789120000, "2001-02-03T04:05:06.789120Z")]
    #[case(981173106789123000, "2001-02-03T04:05:06.789123Z")]
    #[case(981173106789123400, "2001-02-03T04:05:06.789123400Z")] // nanoseconds
    #[case(981173106789123450, "2001-02-03T04:05:06.789123450Z")]
    #[case(981173106789123456, "2001-02-03T04:05:06.789123456Z")]
    fn test_ziso_timestamp_success(#[case] input: i64, #[case] expected: &str) {
        let input_dt = DateTime::from_timestamp_nanos(input);
        assert_eq!(ziso_timestamp(&input_dt), expected);
    }

    #[test]
    fn test_truncate_datetime_to_minutes_success() {
        let input_dt = DateTime::from_timestamp_nanos(981173106789012345);
        let truncated_dt = truncate_datetime_to_minutes(input_dt).unwrap();
        assert_eq!(
            truncated_dt,
            DateTime::parse_from_rfc3339("2001-02-03T04:05:00Z").unwrap()
        );
    }

    #[test]
    fn test_truncate_datetime_to_days_months() {
        let input_dt = DateTime::from_timestamp_nanos(981173106789012345);
        let truncated_dt = truncate_datetime_to_days(input_dt).unwrap();
        assert_eq!(
            truncated_dt,
            DateTime::parse_from_rfc3339("2001-02-03T00:00:00Z").unwrap()
        );
    }

    #[test]
    fn test_truncate_datetime_to_months_success() {
        let input_dt = DateTime::from_timestamp_nanos(981173106789012345);
        let truncated_dt = truncate_datetime_to_months(input_dt).unwrap();
        assert_eq!(
            truncated_dt,
            DateTime::parse_from_rfc3339("2001-02-01T00:00:00Z").unwrap()
        );
    }
}
