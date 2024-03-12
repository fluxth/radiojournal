use std::sync::Arc;

use axum::extract::State;
use radiojournal::crud::station::CRUDStation;

use crate::models::{APIJson, Station};

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
