pub(crate) mod play;
pub(crate) mod station;
pub(crate) mod track;

use std::sync::Arc;

use axum::{Router, routing::get};
use utoipa::{Modify, OpenApi, openapi::Server};
use utoipauto::utoipauto;

use crate::AppState;

#[utoipauto(paths = "api/src, lib/src/crud from radiojournal")]
#[derive(OpenApi)]
#[openapi(modifiers(&ServerAddon))]
pub(crate) struct APIDoc;

struct ServerAddon;
impl Modify for ServerAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/v1")])
    }
}

pub(crate) fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/station/{station_id}", get(station::get_station))
        .route("/stations", get(station::list_stations))
        .route("/station/{station_id}/plays", get(play::list_plays))
        .route(
            "/station/{station_id}/track/{track_id}",
            get(track::get_track),
        )
        .route(
            "/station/{station_id}/track/{track_id}/plays",
            get(track::list_plays_of_track),
        )
        .route("/station/{station_id}/tracks", get(track::list_tracks))
}
