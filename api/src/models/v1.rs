use chrono::{DateTime, Utc};
use radiojournal::models::{StationInDB, TrackInDB, TrackMinimalInDB};
use serde::Serialize;
use ulid::Ulid;

#[derive(Debug, Serialize)]
pub(crate) struct Station {
    id: Ulid,
    name: String,
    track_count: usize,
    play_count: usize,
}

impl From<StationInDB> for Station {
    fn from(station: StationInDB) -> Self {
        Self {
            id: station.id,
            name: station.name,
            track_count: station.track_count,
            play_count: station.play_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Track {
    id: Ulid,
    title: String,
    artist: String,
    is_song: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TrackInDB> for Track {
    fn from(track: TrackInDB) -> Self {
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            is_song: track.is_song,
            created_at: track.created_ts,
            updated_at: track.updated_ts,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TrackMinimal {
    id: Ulid,
    title: String,
    artist: String,
    is_song: bool,
}

impl From<TrackMinimalInDB> for TrackMinimal {
    fn from(track: TrackMinimalInDB) -> Self {
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            is_song: track.is_song,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Play {
    pub(crate) id: Ulid,
    pub(crate) played_at: DateTime<Utc>,
    pub(crate) track: TrackMinimal,
}
