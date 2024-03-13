use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::{Path, State};
use ulid::Ulid;

use crate::models::{APIJson, Play, TrackMinimal};
use radiojournal::crud::station::CRUDStation;

#[utoipa::path(
    get,
    path = "/station/{station_id}/plays",
    params(
        ("station_id" = Ulid, Path, deprecated = false)
    ),
    responses(
        (status = 200, description = "Plays listed successfully", body = Vec<Play>),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn list_plays(
    Path(station_id): Path<Ulid>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> APIJson<Vec<Play>> {
    let plays = crud_station.list_plays(station_id, 50).await.unwrap();

    let track_ids: HashSet<Ulid> = HashSet::from_iter(plays.iter().map(|play| play.track_id));
    let tracks: HashMap<Ulid, TrackMinimal> = HashMap::from_iter(
        crud_station
            .batch_get_tracks(station_id, track_ids.iter())
            .await
            .unwrap()
            .into_iter()
            .map(|track_internal| (track_internal.id, TrackMinimal::from(track_internal))),
    );

    APIJson(
        plays
            .into_iter()
            .map(|play_internal| {
                let track = tracks
                    .get(&play_internal.track_id)
                    .expect("track key to exist")
                    .clone();

                Play {
                    id: play_internal.id,
                    played_at: play_internal.created_ts,
                    track,
                }
            })
            .collect(),
    )
}
