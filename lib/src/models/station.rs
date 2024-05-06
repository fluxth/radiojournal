use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::id::{PlayId, StationId, TrackId};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StationInDB {
    pk: String,
    sk: String,
    pub id: StationId,
    pub name: String,
    pub location: Option<String>,
    pub fetcher: Option<FetcherConfig>,
    pub first_play_id: Option<PlayId>,
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

    pub(crate) fn get_sk(station_id: StationId) -> String {
        format!("STATION#{}", station_id.0)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "STATION#".to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestPlay {
    pub id: PlayId,
    pub track_id: TrackId,
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
