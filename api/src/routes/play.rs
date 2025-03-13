use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::State;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use ulid::Ulid;

use crate::AppState;
use crate::errors::{APIError, APIErrorResponse};
use crate::extractors::{Path, Query};
use crate::models::{APIJson, ListPlaysResponse, NextToken, Play, TrackMinimal};
use radiojournal::crud::station::models::StationId;
use radiojournal::crud::track::models::TrackId;

#[derive(Debug, Deserialize)]
pub(crate) struct ListPlaysQuery {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    next_token: Option<NextToken>,
}

#[utoipa::path(
    get,
    path = "/station/{station_id}/plays",
    params(
        ("station_id" = StationId, Path, deprecated = false),
        ("start" = DateTime<Utc>, Query, deprecated = false),
        ("end" = DateTime<Utc>, Query, deprecated = false),
        ("next_token" = Option<String>, Query, deprecated = false),
    ),
    responses(
        (status = 200, description = "Plays listed successfully", body = ListPlaysResponse),
        (status = 404, description = "Station not found", body = APIErrorResponse),
    ),
    tag = "play"
)]
pub(crate) async fn list_plays(
    Path(station_id): Path<StationId>,
    Query(query): Query<ListPlaysQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<APIJson<ListPlaysResponse>, APIError> {
    let next_key = if let Some(next_token) = query.next_token {
        Some(
            Ulid::from_string(&next_token).or(Err(APIError::ValidationFailed {
                message: Some("Invalid next_token"),
            }))?,
        )
    } else {
        None
    };

    if query.end <= query.start {
        return Err(APIError::ValidationFailed {
            message: Some("`end` must be later than `start`"),
        });
    }

    let (plays_internal, next_key) = state
        .crud_play
        .list_plays(station_id, 50, query.start, query.end, next_key)
        .await
        .unwrap();

    let track_ids: HashSet<TrackId> = plays_internal.iter().map(|play| play.track_id).collect();
    if track_ids.is_empty() {
        // FIXME actually return 404 if station id in pk not found
        return Ok(APIJson(ListPlaysResponse {
            plays: vec![],
            next_token: None,
        }));
    }

    let tracks: HashMap<TrackId, TrackMinimal> = state
        .crud_track
        .batch_get_tracks_minimal(station_id, track_ids.iter())
        .await
        .unwrap()
        .into_iter()
        .map(|track_internal| (track_internal.id.into(), TrackMinimal::from(track_internal)))
        .collect();

    Ok(APIJson(ListPlaysResponse {
        plays: plays_internal
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
