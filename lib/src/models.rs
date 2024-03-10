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
            pk: format!("STATION#{}#TRACKS", station_id),
            sk: format!("TRACK#{}", track_id),
            gsi1pk: format!("STATION#{}#ARTIST#{}", station_id, &artist),
            gsi1sk: format!("TITLE#{}", &title),
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
    pub fn new(station_id: Ulid, track_id: Ulid) -> Self {
        let now = Utc::now();
        let play_id = Ulid::new();

        PlayInDB {
            pk: format!(
                "STATION#{}#PLAYS#{}",
                station_id,
                now.format("%Y-%m-%d").to_string()
            ),
            sk: format!("PLAY#{}", play_id),
            gsi1pk: format!("STATION#{}#TRACK#{}", station_id, track_id),
            gsi1sk: format!("PLAY#{}", play_id),
            id: play_id,
            track_id,
            created_ts: now,
            updated_ts: now,
        }
    }
}
