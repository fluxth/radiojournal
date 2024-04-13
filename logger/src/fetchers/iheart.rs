use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use super::{Fetcher, Play};
use radiojournal::models::station::FetcherConfig;

#[derive(Debug)]
pub(crate) struct Iheart {
    client: reqwest::Client,
}

impl Iheart {
    pub(crate) fn new() -> Self {
        let mut default_headers = HeaderMap::new();

        default_headers.insert(
            "User-Agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3")
        );

        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .default_headers(default_headers)
                .build()
                .expect("successfully build reqwest client"),
        }
    }

    async fn fetch_metadata(&self, slug: &str) -> Result<CurrentlyPlaying> {
        let mut url = reqwest::Url::parse("https://webapi.radioedit.iheart.com/graphql")
            .expect("url parse successfully");

        url.query_pairs_mut()
            .append_pair("operationName", "GetCurrentlyPlayingSongs")
            .append_pair(
                "variables",
                json!({
                    "slug": slug,
                    "paging": {
                        "take": 1
                    }
                })
                .to_string()
                .as_str(),
            )
            .append_pair("extensions", json!({
                "persistedQuery":{
                    "version": 1,
                    "sha256Hash": "386763c17145056713327cddec890cd9d4fea7558efc56d09b7cd4167eef6060"
                }
            }).to_string().as_str());

        let response: Response = self.client.get(url).send().await?.json().await?;
        Ok(response.data.sites.find.stream.amp.currently_playing)
    }
}

#[derive(Deserialize)]
struct Response {
    data: Data,
}
#[derive(Deserialize)]
struct Data {
    sites: Sites,
}
#[derive(Deserialize)]
struct Sites {
    find: Find,
}
#[derive(Deserialize)]
struct Find {
    stream: Stream,
}
#[derive(Deserialize)]
struct Stream {
    amp: Amp,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Amp {
    currently_playing: CurrentlyPlaying,
}

#[derive(Debug, Deserialize)]
struct CurrentlyPlaying {
    tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
struct Track {
    title: String,
    artist: Artist,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Artist {
    artist_name: String,
}

#[async_trait]
impl Fetcher for Iheart {
    async fn fetch_play(&self, config: &FetcherConfig) -> Result<Play> {
        let slug = if let FetcherConfig::Iheart { slug } = config {
            slug
        } else {
            bail!("misconfigured iheart station: {:?}", config)
        };

        let result = self.fetch_metadata(slug).await?;
        let current_track = result
            .tracks
            .into_iter()
            .next()
            .ok_or(anyhow!("cannot find track!"))?;

        Ok(Play {
            artist: current_track.artist.artist_name,
            title: current_track.title,
        })
    }
}
