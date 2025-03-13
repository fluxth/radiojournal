use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use moka::future::Cache;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::info;

use super::{Fetcher, Play};
use radiojournal::crud::station::models::{AtimeStation, FetcherConfig};

#[derive(Debug)]
pub(crate) struct Atime {
    client: reqwest::Client,
    cache: Mutex<Cache<usize, Arc<Vec<StationData>>>>,
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
                .timeout(Duration::from_secs(10))
                .default_headers(default_headers)
                .build()
                .expect("successfully build reqwest client"),
            cache: Mutex::new(
                Cache::builder()
                    .max_capacity(1)
                    .time_to_live(Duration::from_secs(10))
                    .build(),
            ),
        }
    }

    async fn fetch_metadata(&self) -> Result<Vec<StationData>> {
        info!("Fetching atime metadata");
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
    async fn fetch_play(&self, config: &FetcherConfig) -> Result<Play> {
        let (station_id, station_name) = if let FetcherConfig::Atime { station } = config {
            match station {
                AtimeStation::EFM => (1, "EFM"),
                AtimeStation::Greenwave => (2, "Green Wave"),
                AtimeStation::Chill => (3, "Chill"),
            }
        } else {
            bail!("misconfigured atime station: {:?}", config);
        };

        let metadata = {
            let cache_locked = self.cache.lock().await;
            match cache_locked.get(&0).await {
                Some(metadata) => metadata,
                _ => {
                    let metadata = Arc::new(self.fetch_metadata().await?);
                    cache_locked.insert(0, metadata.clone()).await;
                    metadata
                }
            }
        };

        let station_data = metadata
            .iter()
            .find(|station_data| {
                station_data.station_id == station_id && station_data.station_name == station_name
            })
            .ok_or(anyhow!(
                "could not find station id={}, name=\"{}\" in atime response",
                station_id,
                station_name
            ))?;

        Ok(Play {
            title: station_data.title.trim().to_owned(),
            artist: station_data.artists.trim().to_owned(),
        })
    }
}
