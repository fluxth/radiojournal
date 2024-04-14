use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayInDB {
    pk: String,
    sk: String,
    gsi1pk: String,
    pub id: Ulid,
    pub track_id: Ulid,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl PlayInDB {
    pub(crate) fn get_pk(station_id: Ulid, datetime: &DateTime<Utc>) -> String {
        let play_partition = datetime.format("%Y-%m-%d").to_string();
        format!("STATION#{}#PLAYS#{}", station_id, play_partition)
    }

    pub(crate) fn get_sk(play_id: Ulid) -> String {
        format!("PLAY#{}", play_id)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "PLAY#".to_owned()
    }

    pub(crate) fn get_gsi1pk(track_id: Ulid) -> String {
        format!("TRACK#{}", track_id)
    }

    pub fn new(station_id: Ulid, track_id: Ulid) -> Self {
        let now = Utc::now();
        let play_id = Ulid::new();

        PlayInDB {
            pk: Self::get_pk(station_id, &now),
            sk: Self::get_sk(play_id),
            gsi1pk: Self::get_gsi1pk(track_id),
            id: play_id,
            track_id,
            created_ts: now,
            updated_ts: now,
        }
    }
}
