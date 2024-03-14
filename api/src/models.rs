use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, SubsecRound, Timelike, Utc};
use radiojournal::models::{PlayInDB, StationInDB, TrackInDB, TrackMinimalInDB};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::errors::APIError;

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
pub(crate) struct NextToken(pub(crate) String);

impl Into<NextToken> for String {
    fn into(self) -> NextToken {
        NextToken(self)
    }
}

fn truncate_datetime(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.trunc_subsecs(0)
        .with_second(0)
        .expect("set second to 0 on utc tz")
}

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
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
            created_at: truncate_datetime(track.created_ts),
            updated_at: truncate_datetime(track.updated_ts),
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
pub(crate) struct Play {
    pub(crate) id: Ulid,
    pub(crate) played_at: DateTime<Utc>,
    pub(crate) track: TrackMinimal,
}

impl Play {
    pub(crate) fn new(play: PlayInDB, track: TrackMinimal) -> Self {
        Self {
            id: play.id,
            played_at: truncate_datetime(play.created_ts),
            track,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListPlayResponse {
    pub(crate) plays: Vec<Play>,
    pub(crate) next_token: Option<NextToken>,
}
