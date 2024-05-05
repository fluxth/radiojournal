use std::sync::Arc;

use axum::extract::State;
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    errors::APIError,
    extractors::{Path, Query},
    models::{APIJson, ListTrackPlaysResponse, ListTracksResponse, NextToken, PlayMinimal, Track},
    AppState,
};
use radiojournal::models::id::{StationId, TrackId};

#[utoipa::path(
    get,
    path = "/station/{station_id}/track/{track_id}",
    params(
        ("station_id" = StationId, Path, deprecated = false),
        ("track_id" = TrackId, Path, deprecated = false),
    ),
    responses(
        (status = 200, description = "Track returned successfully", body = Track),
        (status = 404, description = "Station or track not found", body = APIErrorResponse),
    ),
    tag = "track"
)]
pub(crate) async fn get_track(
    Path((station_id, track_id)): Path<(StationId, TrackId)>,
    State(state): State<Arc<AppState>>,
) -> Result<APIJson<Track>, APIError> {
    let maybe_track_internal = state
        .crud_track
        .get_track(station_id, track_id)
        .await
        .unwrap();

    if let Some(track) = maybe_track_internal.map(Track::from) {
        Ok(APIJson(track))
    } else {
        Err(APIError::NotFound)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListTrackPlaysQuery {
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/track/{track_id}/plays",
    params(
        ("station_id" = StationId, Path, deprecated = false),
        ("track_id" = TrackId, Path, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Plays of track returned successfully", body = ListTrackPlaysResponse),
        (status = 404, description = "Station or track not found", body = APIErrorResponse),
    ),
    tag = "track"
)]
pub(crate) async fn list_plays_of_track(
    Path((station_id, track_id)): Path<(StationId, TrackId)>,
    Query(query): Query<ListTrackPlaysQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<APIJson<ListTrackPlaysResponse>, APIError> {
    let next_key = if let Some(next_token) = query.next_token {
        Some(
            Ulid::from_string(&next_token).or(Err(APIError::ValidationFailed {
                message: Some("Invalid next_token"),
            }))?,
        )
    } else {
        None
    };

    let (track_plays_internal, next_key) = state
        .crud_track
        .list_plays_of_track(station_id, track_id, 50, next_key)
        .await
        .unwrap();

    Ok(APIJson(ListTrackPlaysResponse {
        plays: track_plays_internal
            .into_iter()
            .map(PlayMinimal::from)
            .collect(),
        next_token: next_key.map(NextToken::from),
    }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListTracksQuery {
    artist: Option<String>,
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/tracks",
    params(
        ("station_id" = StationId, Path, deprecated = false),
        ("artist" = Option<String>, Query, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Tracks listed successfully", body = ListTracksResponse),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    ),
    tag = "track"
)]
pub(crate) async fn list_tracks(
    Path(station_id): Path<StationId>,
    Query(query): Query<ListTracksQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<APIJson<ListTracksResponse>, APIError> {
    let (tracks_internal, next_key) = if let Some(artist) = query.artist {
        state
            .crud_track
            .list_tracks_by_artist(station_id, &artist, 50, query.next_token.as_deref())
            .await
            .unwrap()
    } else {
        let next_key = if let Some(next_token) = query.next_token {
            Some(
                Ulid::from_string(&next_token).or(Err(APIError::ValidationFailed {
                    message: Some("Invalid next_token"),
                }))?,
            )
        } else {
            None
        };

        let (tracks_internal, next_key) = state
            .crud_track
            .list_tracks(station_id, 50, next_key)
            .await
            .unwrap();

        (tracks_internal, next_key.map(String::from))
    };

    Ok(APIJson(ListTracksResponse {
        tracks: tracks_internal.into_iter().map(Track::from).collect(),
        next_token: next_key.map(NextToken::from),
    }))
}
