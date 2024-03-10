mod fetchers;
use std::sync::Arc;

use fetchers::Fetcher;

use radiojournal::{
    crud::station::{AddPlayResult, CRUDStation},
    models::{FetcherConfig, StationInDB},
};

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tokio::task::JoinSet;
use tracing::{debug, info};
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
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

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

    let crud_station = Arc::new(CRUDStation::new(db_client, TABLE_NAME));

    let func = service_fn(|event| invoke(event, crud_station.clone()));
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
    crud_station: Arc<CRUDStation>,
) -> Result<InvokeOutput, Error> {
    let mut join_set = JoinSet::new();

    crud_station
        .list()
        .await
        .expect("list stations successfully")
        .into_iter()
        .filter(|station| station.fetcher.is_some())
        .for_each(|station| {
            let crud_station = crud_station.clone();
            join_set.spawn(async move { process_station(crud_station, station) });
        });

    let mut stations = vec![];
    while let Some(res) = join_set.join_next().await {
        stations.push(res?.await?);
    }

    Ok(InvokeOutput { stations })
}

fn get_fetcher(station: &StationInDB) -> Option<impl fetchers::Fetcher + std::fmt::Debug> {
    match station.fetcher {
        Some(FetcherConfig::Coolism) => Some(fetchers::coolism::Coolism::new()),
        None => None,
    }
}

#[tracing::instrument(skip_all, fields(station.id = station.id.to_string()))]
async fn process_station(
    crud_station: Arc<CRUDStation>,
    station: StationInDB,
) -> anyhow::Result<StationResult> {
    let logger_result = if let Some(fetcher) = get_fetcher(&station) {
        info!(
            station_name = station.name,
            fetcher = ?fetcher,
            "Processing station"
        );

        let play = fetcher.fetch_play().await.unwrap();
        debug!(title = play.title, artist = play.artist, "Fetched play");

        Some(crud_station.add_play(&station, play).await?)
    } else {
        None
    };

    let result = StationResult {
        id: station.id,
        name: station.name,
        logger_result,
    };

    info!(result = ?result, "Processed station");

    Ok(result)
}
