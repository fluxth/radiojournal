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
    models::station::{FetcherConfig, StationInDB},
};

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

#[derive(Debug)]
struct State {
    fetchers: Fetchers,
}

impl State {
    fn new() -> Self {
        Self {
            fetchers: Fetchers::new(),
        }
    }
}

#[derive(Debug)]
struct Fetchers {
    coolism: Mutex<fetchers::coolism::Coolism>,
    atime: Mutex<fetchers::atime::Atime>,
}

impl Fetchers {
    fn new() -> Self {
        Self {
            coolism: Mutex::new(fetchers::coolism::Coolism::new()),
            atime: Mutex::new(fetchers::atime::Atime::new()),
        }
    }
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

    let state = Arc::new(State::new());

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
    state: Arc<State>,
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
            join_set.spawn(async move { process_station(state, crud_station, station).await });
        });

    let mut stations = vec![];
    while let Some(res) = join_set.join_next().await {
        stations.push(res??);
    }

    Ok(InvokeOutput { stations })
}

async fn get_fetcher<'a, 'b>(
    state: &'a State,
    station: &'b StationInDB,
) -> Option<(&'a Mutex<dyn Fetcher + Send + Sync>, &'b FetcherConfig)> {
    station.fetcher.as_ref().map(
        |fetcher_config| -> (&'a Mutex<dyn Fetcher + Send + Sync>, &'b FetcherConfig) {
            (
                match fetcher_config {
                    FetcherConfig::Coolism => &state.fetchers.coolism,
                    FetcherConfig::Atime { station: _ } => &state.fetchers.atime,
                },
                fetcher_config,
            )
        },
    )
}

#[tracing::instrument(skip_all, fields(station.id = station.id.to_string(), station.name = station.name))]
async fn process_station(
    state: Arc<State>,
    crud_station: Arc<CRUDStation>,
    station: StationInDB,
) -> anyhow::Result<StationResult> {
    let maybe_fetcher = get_fetcher(&state, &station).await;

    let logger_result = if let Some((fetcher, config)) = maybe_fetcher {
        info!(
            station_name = station.name,
            fetcher = ?config,
            "Processing station"
        );

        let play = {
            let mut fetcher = fetcher.lock().await;
            fetcher.fetch_play(config).await.unwrap()
        };

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
