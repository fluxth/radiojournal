use std::sync::Arc;

use anyhow::Result;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;
use tracing::info;

use radiojournal::{
    crud::{
        station::{CRUDStation, Play as PlayTrait},
        Context,
    },
    mock,
    models::station::{AtimeStation, FetcherConfig, StationInDBCreate},
};

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

#[derive(Clone)]
struct Play {
    title: String,
    artist: String,
    is_song: bool,
}

impl PlayTrait for Play {
    fn get_title(&self) -> &str {
        self.title.as_str()
    }

    fn get_artist(&self) -> &str {
        self.artist.as_str()
    }

    fn is_song(&self) -> bool {
        self.is_song
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();

    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack() {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    };

    let config = config.load().await;
    let db_client = Client::new(&config);
    let table_name = std::env::var("DB_TABLE_NAME").expect("env DB_TABLE_NAME to be set");

    let context = Arc::new(Context::new(db_client, table_name));

    let crud_station = CRUDStation::new(context.clone());

    mock(context, &crud_station).await;
}

async fn mock(context: Arc<Context>, crud_station: &CRUDStation) {
    info!("Initializing DynamoDB table");
    mock::table::delete_then_create_table(context)
        .await
        .unwrap();

    let coolism = StationInDBCreate {
        name: "coolism".to_string(),
        location: None,
        fetcher: Some(FetcherConfig::Coolism),
    };

    mock_station(crud_station, coolism).await.unwrap();

    let efm = StationInDBCreate {
        name: "efm".to_string(),
        location: None,
        fetcher: Some(FetcherConfig::Atime {
            station: AtimeStation::EFM,
        }),
    };

    mock_station(crud_station, efm).await.unwrap();

    let greenwave = StationInDBCreate {
        name: "greenwave".to_string(),
        location: None,
        fetcher: Some(FetcherConfig::Atime {
            station: AtimeStation::Greenwave,
        }),
    };

    mock_station(crud_station, greenwave).await.unwrap();

    let chill = StationInDBCreate {
        name: "chill".to_string(),
        location: None,
        fetcher: Some(FetcherConfig::Atime {
            station: AtimeStation::Chill,
        }),
    };

    mock_station(crud_station, chill).await.unwrap();

    let z100 = StationInDBCreate {
        name: "z100".to_string(),
        location: Some("usa".to_string()),
        fetcher: Some(FetcherConfig::Iheart {
            slug: "whtz-fm".to_string(),
        }),
    };

    mock_station(crud_station, z100).await.unwrap();

    let kiis = StationInDBCreate {
        name: "kiis".to_string(),
        location: Some("usa".to_string()),
        fetcher: Some(FetcherConfig::Iheart {
            slug: "kiis-fm".to_string(),
        }),
    };

    mock_station(crud_station, kiis).await.unwrap();
}

async fn mock_station(crud_station: &CRUDStation, station_create: StationInDBCreate) -> Result<()> {
    info!(station_name = station_create.name, "Creating station");
    let mut station = crud_station.create_station(station_create).await?;

    info!(
        station_name = station.name,
        "Populating mock tracks and plays"
    );

    let play1 = Play {
        title: "test song".to_string(),
        artist: "some artist".to_string(),
        is_song: true,
    };

    let play2 = Play {
        title: "jingle".to_string(),
        artist: format!("{} station id", station.name),
        is_song: false,
    };

    let play3 = Play {
        title: "another song".to_string(),
        artist: "other artist".to_string(),
        is_song: true,
    };

    crud_station.add_play(&mut station, play1).await?;
    crud_station.add_play(&mut station, play2.clone()).await?;
    crud_station.add_play(&mut station, play3).await?;
    crud_station.add_play(&mut station, play2).await?;

    Ok(())
}
