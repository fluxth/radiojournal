use chrono::{DateTime, SubsecRound, Timelike, Utc};

pub(crate) fn ziso_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}

pub fn truncate_datetime_to_minutes(dt: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Some(dt.with_second(0)?.trunc_subsecs(0))
}
