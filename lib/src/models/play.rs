use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::id::{PlayId, StationId, TrackId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayInDB {
    pk: String,
    sk: String,
    gsi1pk: String,
    pub id: PlayId,
    pub track_id: TrackId,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl PlayInDB {
    pub(crate) fn get_pk(station_id: StationId, datetime: &DateTime<Utc>) -> String {
        let play_partition = datetime.format("%Y-%m-%d").to_string();
        format!("STATION#{}#PLAYS#{}", station_id.0, play_partition)
    }

    pub(crate) fn get_pk_station_prefix(station_id: StationId) -> String {
        format!("STATION#{}", station_id.0)
    }

    pub(crate) fn get_sk(play_id: PlayId) -> String {
        format!("PLAY#{}", play_id.0)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "PLAY#".to_owned()
    }

    pub(crate) fn get_gsi1pk(track_id: TrackId, datetime: &DateTime<Utc>) -> String {
        let track_partition = datetime.format("%Y-%m").to_string();
        format!("TRACK#{}#{}", track_id.0, track_partition)
    }

    pub fn new(station_id: StationId, track_id: TrackId) -> Self {
        let now = Utc::now();
        let play_id = Ulid::new().into();

        PlayInDB {
            pk: Self::get_pk(station_id, &now),
            sk: Self::get_sk(play_id),
            gsi1pk: Self::get_gsi1pk(track_id, &now),
            id: play_id,
            track_id,
            created_ts: now,
            updated_ts: now,
        }
    }
}
