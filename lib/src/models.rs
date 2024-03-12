use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "id")]
pub enum FetcherConfig {
    Coolism,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StationInDB {
    pk: String,
    sk: String,
    pub id: Ulid,
    pub name: String,
    pub fetcher: Option<FetcherConfig>,
    pub first_play_id: Option<Ulid>, // TODO
    pub latest_play_id: Option<Ulid>,
    pub latest_play_track_id: Option<Ulid>,
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
pub struct TrackInDB {
    pk: String,
    sk: String,
    gsi1pk: String,
    gsi1sk: String,
    pub id: Ulid,
    pub title: String,
    pub artist: String,
    pub is_song: bool,
    pub latest_play_id: Option<Ulid>,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl TrackInDB {
    pub(crate) fn get_pk(station_id: Ulid) -> String {
        format!("STATION#{}#TRACKS", station_id)
    }

    pub(crate) fn get_sk(track_id: Ulid) -> String {
        format!("TRACK#{}", track_id)
    }

    pub(crate) fn get_gsi1pk(station_id: Ulid, artist: &str) -> String {
        format!("STATION#{}#ARTIST#{}", station_id, artist)
    }

    pub(crate) fn get_gsi1sk(title: &str) -> String {
        format!("TITLE#{}", title)
    }

    pub fn new(
        station_id: Ulid,
        artist: impl Into<String>,
        title: impl Into<String>,
        is_song: bool,
        latest_play_id: Option<Ulid>,
    ) -> Self {
        let now = Utc::now();
        let track_id = Ulid::new();

        let title = title.into();
        let artist = artist.into();

        Self {
            pk: Self::get_pk(station_id),
            sk: Self::get_sk(track_id),
            gsi1pk: Self::get_gsi1pk(station_id, &artist),
            gsi1sk: Self::get_gsi1sk(&title),
            id: track_id,
            title,
            artist,
            is_song,
            latest_play_id,
            created_ts: now,
            updated_ts: now,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayInDB {
    pk: String,
    sk: String,
    gsi1pk: String,
    gsi1sk: String,
    pub id: Ulid,
    pub track_id: Ulid,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: DateTime<Utc>,
}

impl PlayInDB {
    pub(crate) fn get_pk(station_id: Ulid, play_partition: &str) -> String {
        format!("STATION#{}#PLAYS#{}", station_id, play_partition)
    }

    pub(crate) fn get_sk(play_id: Ulid) -> String {
        format!("PLAY#{}", play_id)
    }

    pub(crate) fn get_sk_prefix() -> String {
        "PLAY#".to_owned()
    }

    pub(crate) fn get_gsi1pk(station_id: Ulid, track_id: Ulid) -> String {
        format!("STATION#{}#TRACK#{}", station_id, track_id)
    }

    pub(crate) fn get_gsi1sk(play_id: Ulid) -> String {
        format!("PLAY#{}", play_id)
    }

    pub fn new(station_id: Ulid, track_id: Ulid) -> Self {
        let now = Utc::now();
        let play_id = Ulid::new();

        PlayInDB {
            pk: Self::get_pk(station_id, &now.format("%Y-%m-%d").to_string()),
            sk: Self::get_sk(play_id),
            gsi1pk: Self::get_gsi1pk(station_id, track_id),
            gsi1sk: Self::get_gsi1sk(play_id),
            id: play_id,
            track_id,
            created_ts: now,
            updated_ts: now,
        }
    }
}
