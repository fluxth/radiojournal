use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use radiojournal::crud::station::models::{AtimeStation, FetcherConfig, StationInDBCreate};
use radiojournal::crud::station::{CRUDStation, Play as PlayTrait};
use radiojournal::crud::Context;

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

pub(crate) async fn mock_database(context: Arc<Context>, crud_station: &CRUDStation) {
    info!("Initializing DynamoDB table");
    radiojournal::mock::table::delete_then_create_table(context)
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
