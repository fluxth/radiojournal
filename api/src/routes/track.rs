use std::sync::Arc;

use axum::extract::{Path, State};
use ulid::Ulid;

use crate::models::{APIJson, Track};
use radiojournal::crud::station::CRUDStation;

#[utoipa::path(
    get,
    path = "/station/{station_id}/track/{track_id}",
    params(
        ("station_id" = Ulid, Path, description = "ID of station"),
        ("track_id" = Ulid, Path, description = "ID of track"),
    ),
    responses(
        (status = 200, description = "Track returned successfully", body = Vec<Play>),
        (status = 404, description = "Station or track not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn get_track(
    Path((station_id, track_id)): Path<(Ulid, Ulid)>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> APIJson<Option<Track>> {
    let maybe_track_internal = crud_station.get_track(station_id, track_id).await.unwrap();

    APIJson(maybe_track_internal.map(|track_internal| Track::from(track_internal)))
}
