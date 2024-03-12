use std::sync::Arc;

use axum::{extract::State, Json};
use radiojournal::crud::station::CRUDStation;

use crate::models::v1::Station;

pub(crate) async fn list_stations(
    State(crud_station): State<Arc<CRUDStation>>,
) -> Json<Vec<Station>> {
    let stations = crud_station.list().await.unwrap();

    Json(
        stations
            .into_iter()
            .map(|station_internal| Station::from(station_internal))
            .collect(),
    )
}
