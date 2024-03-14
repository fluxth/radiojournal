use std::sync::Arc;

use axum::extract::{Path, Query, State};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    errors::APIError,
    models::{APIJson, ListTracksResponse, NextToken, Track},
};
use radiojournal::crud::station::CRUDStation;

#[utoipa::path(
    get,
    path = "/station/{station_id}/track/{track_id}",
    params(
        ("station_id" = Ulid, Path, deprecated = false),
        ("track_id" = Ulid, Path, deprecated = false),
    ),
    responses(
        (status = 200, description = "Track returned successfully", body = Track),
        (status = 404, description = "Station or track not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn get_track(
    Path((station_id, track_id)): Path<(Ulid, Ulid)>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> Result<APIJson<Track>, APIError> {
    let maybe_track_internal = crud_station.get_track(station_id, track_id).await.unwrap();

    if let Some(track) = maybe_track_internal.map(|track_internal| Track::from(track_internal)) {
        Ok(APIJson(track))
    } else {
        Err(APIError::NotFound)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListTracksQuery {
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/tracks",
    params(
        ("station_id" = Ulid, Path, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Tracks listed successfully", body = Vec<Track>),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn list_tracks(
    Path(station_id): Path<Ulid>,
    Query(query): Query<ListTracksQuery>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> Result<APIJson<ListTracksResponse>, APIError> {
    let next_key = if let Some(next_token) = query.next_token {
        Some(
            Ulid::from_string(&next_token.0).or(Err(APIError::ValidationFailed {
                message: Some("Invalid next_token"),
            }))?,
        )
    } else {
        None
    };

    let (tracks_internal, next_key) = crud_station
        .list_tracks(station_id, 50, next_key)
        .await
        .unwrap();

    Ok(APIJson(ListTracksResponse {
        tracks: tracks_internal
            .into_iter()
            .map(|track_internal| Track::from(track_internal))
            .collect(),
        next_token: next_key.map(|val| val.to_string().into()),
    }))
}
