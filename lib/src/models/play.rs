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
        format!(
            "STATION#{}#PLAYS#{}",
            station_id.0,
            Self::get_partition(datetime)
        )
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

    pub(crate) fn get_partition(datetime: &DateTime<Utc>) -> String {
        datetime.format("%Y-%m-%d").to_string()
    }

    pub(crate) fn is_same_partition(left: &DateTime<Utc>, right: &DateTime<Utc>) -> bool {
        let left_partition = Self::get_partition(left);
        let right_partition = Self::get_partition(right);

        left_partition == right_partition
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
