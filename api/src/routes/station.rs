use std::sync::Arc;

use axum::extract::{Path, State};
use radiojournal::models::id::StationId;

use crate::{
    errors::APIError,
    models::{APIJson, Station},
    AppState,
};

#[utoipa::path(
    get,
    path = "/stations",
    responses(
        (status = 200, description = "Stations listed successfully", body = Vec<Station>),
    ),
    tag = "station"
)]
pub(crate) async fn list_stations(State(state): State<Arc<AppState>>) -> APIJson<Vec<Station>> {
    let internal_stations = state.crud_station.list_stations(50).await.unwrap();

    APIJson(internal_stations.into_iter().map(Station::from).collect())
}

#[utoipa::path(
    get,
    path = "/station/{station_id}",
    params(
        ("station_id" = StationId, Path, deprecated = false),
    ),
    responses(
        (status = 200, description = "Station returned successfully", body = Station),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    ),
    tag = "station"
)]
pub(crate) async fn get_station(
    Path(station_id): Path<StationId>,
    State(state): State<Arc<AppState>>,
) -> Result<APIJson<Station>, APIError> {
    let maybe_station_internal = state.crud_station.get_station(station_id).await.unwrap();

    if let Some(station) = maybe_station_internal.map(Station::from) {
        Ok(APIJson(station))
    } else {
        Err(APIError::NotFound)
    }
}
