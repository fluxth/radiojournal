use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::crud::play::models::PlayId;
use crate::crud::track::models::TrackId;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct StationId(pub Ulid);

impl From<Ulid> for StationId {
    fn from(val: Ulid) -> Self {
        Self(val)
    }
}

impl Deref for StationId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AtimeStation {
    EFM,
    Greenwave,
    Chill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "id")]
pub enum FetcherConfig {
    Coolism,
    Iheart { slug: String },
    Atime { station: AtimeStation },
}

#[derive(Debug)]
pub struct StationInDBCreate {
    pub name: String,
    pub location: Option<String>,
    pub fetcher: Option<FetcherConfig>,
}

impl From<StationInDBCreate> for StationInDB {
    fn from(value: StationInDBCreate) -> Self {
        let now = Utc::now();
        let id = Ulid::from_datetime(now.into()).into();

        Self {
            pk: Self::get_pk(),
            sk: Self::get_sk(id),
            id,
            name: value.name,
            location: value.location,
            fetcher: value.fetcher,
            first_play_id: None,
            latest_play: None,
            track_count: 0,
            play_count: 0,
            created_ts: now,
            updated_ts: now,
        }
    }
}
