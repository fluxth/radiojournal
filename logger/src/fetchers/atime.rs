use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

use super::{Fetcher, Play};
use radiojournal::models::station::{AtimeStation, FetcherConfig};

#[derive(Debug)]
pub(crate) struct Atime {
    client: reqwest::Client,
}

impl Atime {
    pub(crate) fn new() -> Self {
        let mut default_headers = HeaderMap::new();

        default_headers.insert(
            "User-Agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3")
        );

        default_headers.insert("Origin", HeaderValue::from_static("https://atime.live"));

        default_headers.insert("Referer", HeaderValue::from_static("https://atime.live/"));

        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .default_headers(default_headers)
                .build()
                .expect("successfully build reqwest client"),
        }
    }

    async fn fetch_metadata(&self) -> Result<Vec<StationData>> {
        let response: MetadataResponse = self
            .client
            .get("https://onair.atime.live/nowplaying")
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data)
    }
}

#[derive(Debug, Deserialize)]
struct MetadataResponse {
    data: Vec<StationData>,
}

#[derive(Debug, Deserialize)]
struct StationData {
    station_id: i32,
    station_name: String,
    title: String,
    artists: String,
}

#[async_trait]
impl Fetcher for Atime {
    async fn fetch_play(&mut self, config: &FetcherConfig) -> Result<Play> {
        let (station_id, station_name) = if let FetcherConfig::Atime { station } = config {
            match station {
                AtimeStation::EFM => (1, "EFM"),
                AtimeStation::Greenwave => (2, "Green Wave"),
                AtimeStation::Chill => (3, "Chill"),
            }
        } else {
            bail!("misconfigured atime station: {:?}", config);
        };

        let metadata = self.fetch_metadata().await?;

        let station_data = metadata
            .into_iter()
            .find(|station_data| {
                station_data.station_id == station_id && station_data.station_name == station_name
            })
            .ok_or(anyhow!(
                "could not find station id={}, name=\"{}\" in atime response",
                station_id,
                station_name
            ))?;

        Ok(Play {
            title: station_data.title,
            artist: station_data.artists,
        })
    }
}
