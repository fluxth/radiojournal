use std::sync::Arc;

use axum::{routing::get, Router};
use radiojournal::crud::station::CRUDStation;

pub(crate) mod play;
pub(crate) mod station;
pub(crate) mod track;

pub(crate) fn get_router() -> Router<Arc<CRUDStation>> {
    Router::new()
        .route("/stations", get(station::list_stations))
        .route("/station/:station_id/plays", get(play::list_plays))
        .route(
            "/station/:station_id/track/:track_id",
            get(track::get_track),
        )
}
