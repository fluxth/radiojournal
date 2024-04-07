use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    errors::APIError,
    models::{APIJson, ListPlaysResponse, NextToken, Play, TrackMinimal},
};
use radiojournal::crud::station::CRUDStation;

#[derive(Debug, Deserialize)]
pub(crate) struct ListPlaysQuery {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/plays",
    params(
        ("station_id" = Ulid, Path, deprecated = false),
        ("start" = Option<DateTime<Utc>>, Query, deprecated = false),
        ("end" = Option<DateTime<Utc>>, Query, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Plays listed successfully", body = ListPlaysResponse),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    )
)]
pub(crate) async fn list_plays(
    Path(station_id): Path<Ulid>,
    Query(query): Query<ListPlaysQuery>,
    State(crud_station): State<Arc<CRUDStation>>,
) -> Result<APIJson<ListPlaysResponse>, APIError> {
    let next_key = if let Some(next_token) = query.next_token {
        Some(
            Ulid::from_string(&next_token.0).or(Err(APIError::ValidationFailed {
                message: Some("Invalid next_token"),
            }))?,
        )
    } else {
        None
    };

    if let (Some(start), Some(end)) = (query.start, query.end) {
        if end <= start {
            return Err(APIError::ValidationFailed {
                message: Some("`end` must be more than `start`"),
            });
        }
    }

    let (plays, next_key) = crud_station
        .list_plays(station_id, 50, query.start, query.end, next_key)
        .await
        .unwrap();

    let track_ids: HashSet<Ulid> = HashSet::from_iter(plays.iter().map(|play| play.track_id));
    if track_ids.is_empty() {
        // FIXME actually return 404 if station id in pk not found
        return Ok(APIJson(ListPlaysResponse {
            plays: vec![],
            next_token: None,
        }));
    }

    let tracks: HashMap<Ulid, TrackMinimal> = HashMap::from_iter(
        crud_station
            .batch_get_tracks(station_id, track_ids.iter())
            .await
            .unwrap()
            .into_iter()
            .map(|track_internal| (track_internal.id, TrackMinimal::from(track_internal))),
    );

    Ok(APIJson(ListPlaysResponse {
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
