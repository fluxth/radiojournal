use std::sync::Arc;

use axum::extract::State;

use crate::{
    models::{APIJson, Station},
    AppState,
};

#[utoipa::path(
    get,
    path = "/stations",
    responses(
        (status = 200, description = "Stations listed successfully", body = Vec<Station>),
    )
)]
pub(crate) async fn list_stations(State(state): State<Arc<AppState>>) -> APIJson<Vec<Station>> {
    let stations = state.crud_station.list(50).await.unwrap();

    APIJson(stations.into_iter().map(Station::from).collect())
}
