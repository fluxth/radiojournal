use std::sync::Arc;

use axum::extract::State;

use crate::models::{APIJson, Station};
use radiojournal::crud::station::CRUDStation;

#[utoipa::path(
    get,
    path = "/stations",
    responses(
        (status = 200, description = "Stations listed successfully", body = Vec<Station>),
    )
)]
pub(crate) async fn list_stations(
    State(crud_station): State<Arc<CRUDStation>>,
) -> APIJson<Vec<Station>> {
    let stations = crud_station.list().await.unwrap();

    APIJson(
        stations
            .into_iter()
            .map(|station_internal| Station::from(station_internal))
            .collect(),
    )
}
