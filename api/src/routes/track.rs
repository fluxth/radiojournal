use std::sync::Arc;

use axum::extract::{Path, State};
use radiojournal::crud::station::CRUDStation;
use ulid::Ulid;

use crate::models::{APIJson, Track};

pub(crate) async fn get_track(
    Path((station_id, track_id)): Path<(Ulid, Ulid)>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> APIJson<Option<Track>> {
    let maybe_track_internal = crud_station.get_track(station_id, track_id).await.unwrap();

    APIJson(maybe_track_internal.map(|track_internal| Track::from(track_internal)))
}
