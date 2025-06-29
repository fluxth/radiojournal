use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::crud::play::models::PlayId;
use crate::crud::station::models::StationId;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct TrackId(pub Ulid);

impl From<Ulid> for TrackId {
    fn from(val: Ulid) -> Self {
        Self(val)
    }
}

impl Deref for TrackId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackInDB {
    pk: String,
    sk: String,
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub is_song: bool,
    pub play_count: usize,
    pub latest_play_id: Option<PlayId>,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl TrackInDB {
    pub(crate) fn get_pk(station_id: StationId) -> String {
        format!("STATION#{}#TRACKS", station_id.0)
    }

    pub(crate) fn get_sk(track_id: TrackId) -> String {
        format!("TRACK#{}", track_id.0)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "TRACK#".to_owned()
    }

    fn station_id(&self) -> StationId {
        Ulid::from_string(
            self.pk
                .trim_start_matches("STATION#")
                .trim_end_matches("#TRACKS"),
        )
        .expect("track station id must be ulid")
        .into()
    }

    pub fn new(
        station_id: StationId,
        artist: impl Into<String>,
        title: impl Into<String>,
        is_song: bool,
    ) -> Self {
        let now = Utc::now();
        let track_id = Ulid::new().into();

        let title = title.into();
        let artist = artist.into();

        Self {
            pk: Self::get_pk(station_id),
            sk: Self::get_sk(track_id),
            id: track_id,
            title,
            artist,
            is_song,
            play_count: 0,
            latest_play_id: None,
            created_ts: now,
            updated_ts: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TrackPlayInDB {
    // pk: String,
    // sk: String,
    // gsi1pk: String,
    pub id: Ulid,
    pub track_id: Ulid,
}

impl TrackPlayInDB {
    pub(crate) fn get_gsi1pk(track_id: TrackId, datetime: &DateTime<Utc>) -> String {
        let track_partition = datetime.format("%Y-%m").to_string();
        format!("TRACK#{}#{}", track_id.0, track_partition)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "PLAY#".to_owned()
    }
}

pub(crate) trait TrackMetadataKeys {
    fn get_pk(station_id: StationId, artist: &str) -> String {
        format!("STATION#{}#ARTIST#{}", station_id.0, artist)
    }

    fn get_sk(title: &str) -> String {
        format!("TITLE#{title}")
    }

    fn get_sk_prefix() -> String {
        "TITLE#".to_owned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Query variant of track item used for lookup using metadata
pub struct TrackMetadataInDB {
    pub track_id: TrackId,
}

impl TrackMetadataKeys for TrackMetadataInDB {}

#[derive(Debug, Serialize, Deserialize)]
/// Insert variant of track item used for lookup using metadata
pub struct TrackMetadataCreateInDB {
    pk: String,
    sk: String,
    pub track_id: TrackId,
}

impl TrackMetadataKeys for TrackMetadataCreateInDB {}

impl From<&TrackInDB> for TrackMetadataCreateInDB {
    fn from(track: &TrackInDB) -> Self {
        Self {
            pk: Self::get_pk(track.station_id(), &track.artist),
            sk: Self::get_sk(&track.title),
            track_id: track.id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackMinimalInDB {
    pub id: Ulid,
    pub title: String,
    pub artist: String,
    pub is_song: bool,
}
