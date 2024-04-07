use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

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
    pub play_count: usize,
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

    pub(crate) fn get_sk_prefix() -> String {
        "TRACK#".to_owned()
    }

    pub(crate) fn get_gsi1pk(station_id: Ulid, artist: &str) -> String {
        format!("STATION#{}#ARTIST#{}", station_id, artist)
    }

    pub(crate) fn get_gsi1sk(title: &str) -> String {
        format!("TITLE#{}", title)
    }

    fn station_id(&self) -> Ulid {
        Ulid::from_string(
            self.pk
                .trim_start_matches("STATION#")
                .trim_end_matches("#TRACKS"),
        )
        .expect("track station id must be ulid")
    }

    pub fn new(
        station_id: Ulid,
        artist: impl Into<String>,
        title: impl Into<String>,
        is_song: bool,
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
            play_count: 0,
            latest_play_id: None,
            created_ts: now,
            updated_ts: now,
        }
    }
}

pub(crate) trait TrackMetadataKeys {
    fn get_pk(station_id: Ulid, artist: &str) -> String {
        format!("STATION#{}#ARTIST#{}", station_id, artist)
    }

    fn get_sk(title: &str) -> String {
        format!("TITLE#{}", title)
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Query variant of track item used for lookup using metadata
pub struct TrackMetadataInDB {
    pub track_id: Ulid,
}

impl TrackMetadataKeys for TrackMetadataInDB {}

#[derive(Debug, Serialize, Deserialize)]
/// Insert variant of track item used for lookup using metadata
pub struct TrackMetadataCreateInDB {
    pk: String,
    sk: String,
    pub track_id: Ulid,
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
