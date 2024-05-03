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
    let internal_stations = state.crud_station.list_stations(50).await.unwrap();

    APIJson(internal_stations.into_iter().map(Station::from).collect())
}
