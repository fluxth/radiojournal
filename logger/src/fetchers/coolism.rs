use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

use super::{Fetcher, Play};

#[derive(Debug)]
pub(crate) struct Coolism {
    client: reqwest::Client,
}

impl Coolism {
    pub(crate) fn new() -> Self {
        let mut default_headers = HeaderMap::new();

        default_headers.insert(
            "User-Agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3")
        );

        default_headers.insert(
            "Origin",
            HeaderValue::from_static("https://www.coolism.net"),
        );

        default_headers.insert(
            "Referer",
            HeaderValue::from_static("https://www.coolism.net/"),
        );

        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .default_headers(default_headers)
                .build()
                .expect("successfully build reqwest client"),
        }
    }

    async fn fetch_token(&self) -> Result<String> {
        let response: HashMap<String, String> = self
            .client
            .post("https://api.coolism.net/api/auth/gettoken")
            .basic_auth("coolism", Some("PshjzmOQ"))
            .send()
            .await?
            .json()
            .await?;

        if let Some(token) = response.get("token") {
            if token.len() > 0 {
                Ok(token.clone())
            } else {
                Err(anyhow!("empty token"))
            }
        } else {
            Err(anyhow!("no token in response"))
        }
    }

    async fn fetch_metadata(&self, token: &str) -> Result<Data> {
        let response: MetadataResponse = self
            .client
            .get("https://api.coolism.net/api/song/radio/nowPlaying")
            .query(&[
                ("type", "start"),
                ("t", &Utc::now().timestamp_millis().to_string()),
            ])
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data)
    }
}

#[derive(Debug, Deserialize)]
struct MetadataResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    now_song: NowSong,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NowSong {
    song: String,
    artist: String,
}

#[async_trait]
impl Fetcher for Coolism {
    fn get_name(&self) -> &'static str {
        "Coolism"
    }

    async fn fetch_play(&self) -> Result<Play> {
        let token = self.fetch_token().await?;
        let metadata = self.fetch_metadata(&token).await?;
        Ok(Play {
            title: metadata.now_song.song,
            artist: metadata.now_song.artist,
        })
    }
}
