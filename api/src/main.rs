use std::collections::{HashMap, HashSet};

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use lambda_http::{run, Error};
use radiojournal::{
    crud::station::CRUDStation,
    models::{PlayInDB, StationInDB, TrackInDB, TrackMinimalInDB},
};
use serde::Serialize;
use serde_json::json;
use ulid::Ulid;

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

const TABLE_NAME: &str = "radiojournal-local";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    std::env::set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack() {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    };

    let config = config.load().await;
    let db_client = Client::new(&config);

    let crud_station = CRUDStation::new(db_client, TABLE_NAME);

    let api = Router::new()
        .route("/stations", get(list_stations))
        .route("/station/:station_id/plays", get(list_plays))
        .route("/station/:station_id/track/:track_id", get(get_track))
        .with_state(crud_station);

    let app = Router::new().nest("/v1", api);

    run(app).await
}

#[derive(Debug, Serialize)]
struct Station {
    id: Ulid,
    name: String,
}

impl From<StationInDB> for Station {
    fn from(station: StationInDB) -> Self {
        Self {
            id: station.id,
            name: station.name,
        }
    }
}

async fn list_stations(State(crud_station): State<CRUDStation>) -> Json<Vec<Station>> {
    let stations = crud_station.list().await.unwrap();

    Json(
        stations
            .into_iter()
            .map(|station_internal| Station::from(station_internal))
            .collect(),
    )
}

#[derive(Debug, Serialize)]
struct Track {
    id: Ulid,
    title: String,
    artist: String,
    is_song: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TrackInDB> for Track {
    fn from(track: TrackInDB) -> Self {
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            is_song: track.is_song,
            created_at: track.created_ts,
            updated_at: track.updated_ts,
        }
    }
}

async fn get_track(
    Path((station_id, track_id)): Path<(Ulid, Ulid)>,
    State(crud_station): State<CRUDStation>,
) -> Json<Option<Track>> {
    let maybe_track_internal = crud_station.get_track(station_id, track_id).await.unwrap();

    Json(maybe_track_internal.map(|track_internal| Track::from(track_internal)))
}

#[derive(Debug, Clone, Serialize)]
struct TrackMinimal {
    id: Ulid,
    title: String,
    artist: String,
    is_song: bool,
}

impl From<TrackMinimalInDB> for TrackMinimal {
    fn from(track: TrackMinimalInDB) -> Self {
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            is_song: track.is_song,
        }
    }
}

#[derive(Debug, Serialize)]
struct Play {
    id: Ulid,
    played_at: DateTime<Utc>,
    track: TrackMinimal,
}

async fn list_plays(
    Path(station_id): Path<Ulid>,
    State(crud_station): State<CRUDStation>,
) -> Json<Vec<Play>> {
    let plays = crud_station.list_plays(station_id).await.unwrap();

    let track_ids: HashSet<Ulid> = HashSet::from_iter(plays.iter().map(|play| play.track_id));
    let tracks: HashMap<Ulid, TrackMinimal> = HashMap::from_iter(
        crud_station
            .batch_get_tracks(station_id, track_ids.iter())
            .await
            .unwrap()
            .into_iter()
            .map(|track_internal| (track_internal.id, TrackMinimal::from(track_internal))),
    );

    Json(
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
