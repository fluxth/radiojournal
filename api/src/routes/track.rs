use std::sync::Arc;

use axum::extract::{Path, State};
use ulid::Ulid;

use crate::{
    errors::APIError,
    models::{APIJson, Track},
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

#[utoipa::path(
    get,
    path = "/station/{station_id}/tracks",
    params(
        ("station_id" = Ulid, Path, deprecated = false),
    ),
    responses(
        (status = 200, description = "Tracks listed successfully", body = Vec<Track>),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn list_tracks(
    Path(station_id): Path<Ulid>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> APIJson<Vec<Track>> {
    let tracks_internal = crud_station.list_tracks(station_id, 50).await.unwrap();

    APIJson(
        tracks_internal
            .into_iter()
            .map(|track_internal| Track::from(track_internal))
            .collect(),
    )
}
