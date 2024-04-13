use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StationInDB {
    pk: String,
    sk: String,
    pub id: Ulid,
    pub name: String,
    pub fetcher: Option<FetcherConfig>,
    pub first_play_id: Option<Ulid>,
    pub latest_play: Option<LatestPlay>,
    pub track_count: usize,
    pub play_count: usize,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl StationInDB {
    pub(crate) fn get_pk() -> String {
        "STATIONS".to_owned()
    }

    pub(crate) fn get_sk(station_id: Ulid) -> String {
        format!("STATION#{}", station_id)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "STATION#".to_owned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestPlay {
    pub id: Ulid,
    pub track_id: Ulid,
    pub artist: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AtimeStation {
    EFM,
    Greenwave,
    Chill,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "id")]
pub enum FetcherConfig {
    Coolism,
    Iheart { slug: String },
    Atime { station: AtimeStation },
}
