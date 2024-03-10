use chrono::{DateTime, Utc};

pub(crate) fn ziso_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}
