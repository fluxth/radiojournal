pub(crate) mod play;
pub(crate) mod station;
pub(crate) mod track;

use std::sync::Arc;

use axum::{routing::get, Router};
use utoipa::{openapi::Server, Modify, OpenApi};

use crate::errors::{APIErrorDetail, APIErrorResponse};
use crate::models::{
    ListPlaysResponse, ListTracksResponse, NextToken, Play, Station, Track, TrackMinimal,
};
use radiojournal::crud::station::CRUDStation;

#[derive(OpenApi)]
#[openapi(
    paths(
        station::list_stations,
        play::list_plays,
        track::get_track,
        track::list_tracks,
    ),
    components(
        schemas(
            Station,
            Play,
            Track,
            TrackMinimal,
            NextToken,
            ListPlaysResponse,
            ListTracksResponse,
            APIErrorDetail,
            APIErrorResponse
        ),
    ),
    modifiers(&ServerAddon),
)]
pub(crate) struct APIDoc;

struct ServerAddon;
impl Modify for ServerAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/v1")])
    }
}

pub(crate) fn get_router() -> Router<Arc<CRUDStation>> {
    Router::new()
        .route("/stations", get(station::list_stations))
        .route("/station/:station_id/plays", get(play::list_plays))
        .route(
            "/station/:station_id/track/:track_id",
            get(track::get_track),
        )
        .route("/station/:station_id/tracks", get(track::list_tracks))
}
