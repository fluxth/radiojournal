use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::{Path, Query, State};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    errors::APIError,
    models::{APIJson, ListPlayResponse, NextToken, Play, TrackMinimal},
};
use radiojournal::crud::station::CRUDStation;

#[derive(Debug, Deserialize)]
pub(crate) struct ListPlayQuery {
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/plays",
    params(
        ("station_id" = Ulid, Path, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Plays listed successfully", body = ListPlayResponse),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn list_plays(
    Path(station_id): Path<Ulid>,
    Query(query): Query<ListPlayQuery>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> Result<APIJson<ListPlayResponse>, APIError> {
    let next_key = if let Some(next_token) = query.next_token {
        Some(
            Ulid::from_string(&next_token.0).or(Err(APIError::ValidationFailed {
                message: Some("Invalid next_token"),
            }))?,
        )
    } else {
        None
    };

    let (plays, next_key) = crud_station
        .list_plays(station_id, 50, next_key)
        .await
        .unwrap();

    let track_ids: HashSet<Ulid> = HashSet::from_iter(plays.iter().map(|play| play.track_id));
    let tracks: HashMap<Ulid, TrackMinimal> = HashMap::from_iter(
        crud_station
            .batch_get_tracks(station_id, track_ids.iter())
            .await
            .unwrap()
            .into_iter()
            .map(|track_internal| (track_internal.id, TrackMinimal::from(track_internal))),
    );

    Ok(APIJson(ListPlayResponse {
        plays: plays
            .into_iter()
            .map(|play_internal| {
                let track = tracks
                    .get(&play_internal.track_id)
                    .expect("track key to exist")
                    .clone();

                Play::new(play_internal, track)
            })
            .collect(),
        next_token: next_key.map(|val| val.to_string().into()),
    }))
}
