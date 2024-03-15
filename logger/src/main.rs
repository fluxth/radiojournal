mod fetchers;
use fetchers::Fetcher;

use std::sync::Arc;

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::info;
use ulid::Ulid;

use radiojournal::{
    crud::station::{AddPlayResult, CRUDStation},
    models::{FetcherConfig, StationInDB},
};

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

#[derive(Debug, Default)]
struct State {
    fetchers: Fetchers,
}

#[derive(Debug, Default)]
struct Fetchers {
    coolism: Option<fetchers::coolism::Coolism>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    if use_localstack() {
        tracing_subscriber::fmt().compact().init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    info!(
        "Initializing {} v{}...",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack() {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    };

    let config = config.load().await;
    let db_client = Client::new(&config);
    let table_name = std::env::var("DB_TABLE_NAME").expect("env DB_TABLE_NAME to be set");

    let crud_station = Arc::new(CRUDStation::new(db_client, &table_name));

    let state = Arc::new(Mutex::new(State::default()));

    let func = service_fn(|event| invoke(event, state.clone(), crud_station.clone()));
    info!("Initialization complete, now listening for events");

    lambda_runtime::run(func).await?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct InvokeOutput {
    stations: Vec<StationResult>,
}

#[derive(Debug, Serialize)]
struct StationResult {
    id: Ulid,
    name: String,
    logger_result: Option<AddPlayResult>,
}

async fn invoke(
    _event: LambdaEvent<Value>,
    state: Arc<Mutex<State>>,
    crud_station: Arc<CRUDStation>,
) -> Result<InvokeOutput, Error> {
    let mut join_set = JoinSet::new();

    crud_station
        .list(100)
        .await
        .expect("list stations successfully")
        .into_iter()
        .filter(|station| station.fetcher.is_some())
        .for_each(|station| {
            let crud_station = crud_station.clone();
            let state = state.clone();
            join_set.spawn(async move { process_station(state, crud_station, station) });
        });

    let mut stations = vec![];
    while let Some(res) = join_set.join_next().await {
        stations.push(res?.await?);
    }

    Ok(InvokeOutput { stations })
}

#[tracing::instrument(skip_all, fields(station.id = station.id.to_string(), station.name = station.name))]
async fn process_station(
    state: Arc<Mutex<State>>,
    crud_station: Arc<CRUDStation>,
    station: StationInDB,
) -> anyhow::Result<StationResult> {
    let mut state = state.lock().await;

    let maybe_fetcher = match station.fetcher {
        Some(FetcherConfig::Coolism) => {
            if let Some(shared) = &mut state.fetchers.coolism {
                Some(shared)
            } else {
                state.fetchers.coolism = Some(fetchers::coolism::Coolism::new());
                state.fetchers.coolism.as_mut()
            }
        }
        None => None,
    };

    let logger_result = if let Some(fetcher) = maybe_fetcher {
        info!(
            station_name = station.name,
            fetcher = fetcher.get_name(),
            "Processing station"
        );

        let play = fetcher.fetch_play().await.unwrap();
        info!(title = play.title, artist = play.artist, "Fetched play");

        let result = crud_station.add_play(&station, play).await?;
        info!(
            add_type = ?result.add_type,
            track_id = result.track_id.to_string(),
            play_id = result.play_id.to_string(),
            "Play added with type {:?}", result.add_type
        );

        Some(result)
    } else {
        info!("Fetcher not found, skipping station");

        None
    };

    Ok(StationResult {
        id: station.id,
        name: station.name,
        logger_result,
    })
}
