use std::ops::Deref;

use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::errors::APIError;
use radiojournal::models::{
    id::TrackId,
    play::PlayInDB,
    station::StationInDB,
    track::{TrackInDB, TrackMinimalInDB, TrackPlayInDB},
};
use radiojournal::{helpers::truncate_datetime_to_minutes, models::id::StationId};

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(APIError))]
pub(crate) struct APIJson<T>(pub(crate) T);

impl<T> IntoResponse for APIJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[repr(transparent)]
pub(crate) struct NextToken(String);

impl From<String> for NextToken {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl Deref for NextToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct Station {
    id: StationId,
    name: String,
    location: Option<String>,
    track_count: usize,
    play_count: usize,
}

impl From<StationInDB> for Station {
    fn from(station: StationInDB) -> Self {
        Self {
            id: station.id,
            name: station.name,
            location: station.location,
            track_count: station.track_count,
            play_count: station.play_count,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct Track {
    id: TrackId,
    title: String,
    artist: String,
    is_song: bool,
    play_count: usize,
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
            play_count: track.play_count,
            created_at: truncate_datetime_to_minutes(track.created_ts)
                .expect("truncate to minutes on utc datetime"),
            updated_at: truncate_datetime_to_minutes(track.updated_ts)
                .expect("truncate to minutes on utc datetime"),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct PlayMinimal {
    pub(crate) id: Ulid,
    pub(crate) played_at: DateTime<Utc>,
}

impl From<TrackPlayInDB> for PlayMinimal {
    fn from(track_play: TrackPlayInDB) -> Self {
        Self {
            id: track_play.id,
            played_at: truncate_datetime_to_minutes(track_play.id.datetime().into())
                .expect("truncate to minutes on utc datetime"),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct Play {
    pub(crate) id: Ulid,
    pub(crate) played_at: DateTime<Utc>,
    pub(crate) track: TrackMinimal,
}

impl Play {
    pub(crate) fn new(play: PlayInDB, track: TrackMinimal) -> Self {
        Self {
            id: play.id,
            played_at: truncate_datetime_to_minutes(play.created_ts)
                .expect("truncate to minutes on utc datetime"),
            track,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListPlaysResponse {
    pub(crate) plays: Vec<Play>,
    pub(crate) next_token: Option<NextToken>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListTracksResponse {
    pub(crate) tracks: Vec<Track>,
    pub(crate) next_token: Option<NextToken>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListTrackPlaysResponse {
    pub(crate) plays: Vec<PlayMinimal>,
    pub(crate) next_token: Option<NextToken>,
}
