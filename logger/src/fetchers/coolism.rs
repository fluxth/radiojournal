use std::{borrow::Cow, collections::HashMap, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use tracing::info;

use super::{Fetcher, Play};
use radiojournal::models::station::FetcherConfig;

#[derive(Debug)]
struct CoolismToken {
    token: String,
    expiry: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct Coolism {
    client: reqwest::Client,
    token: Option<CoolismToken>,
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
            token: None,
        }
    }

    async fn fetch_token(&self) -> Result<(String, Option<DateTime<Utc>>)> {
        let response: HashMap<String, String> = self
            .client
            .post("https://api.coolism.net/api/auth/gettoken")
            .basic_auth("coolism", Some("PshjzmOQ"))
            .send()
            .await?
            .json()
            .await?;

        let token = if let Some(token) = response.get("token") {
            if token.len() > 0 {
                token.clone()
            } else {
                Err(anyhow!("empty token"))?
            }
        } else {
            Err(anyhow!("no token in response"))?
        };

        let expiry: Option<DateTime<Utc>> = response
            .get("expire")
            .map(|val| DateTime::parse_from_rfc3339(&val).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));

        info!(
            expiry = expiry.map(|val| val.to_string()),
            "New token fetched"
        );

        Ok((token, expiry))
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
    async fn fetch_play(&mut self, _config: &FetcherConfig) -> Result<Play> {
        if let Some(token) = &self.token {
            let now = Utc::now();
            let deadline = token.expiry - Duration::from_secs(600);
            if now >= deadline {
                // expiry too close, refresh
                info!(
                    now = %now,
                    expiry = %token.expiry,
                    deadline = %deadline,
                    "Token near expiry, invalidating"
                );
                self.token = None;
            }
        }

        let token = if let Some(token) = &self.token {
            Cow::Borrowed(&token.token)
        } else {
            let (token, expiry) = self.fetch_token().await?;
            if let Some(expiry) = expiry {
                self.token = Some(CoolismToken { token, expiry });
                Cow::Borrowed(&self.token.as_ref().expect("token must have value").token)
            } else {
                Cow::Owned(token)
            }
        };

        let metadata = self.fetch_metadata(token.as_str()).await?;

        Ok(Play {
            title: metadata.now_song.song,
            artist: metadata.now_song.artist,
        })
    }
}
