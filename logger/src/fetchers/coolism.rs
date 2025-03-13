use std::{collections::HashMap, time::Duration};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::{Fetcher, Play};
use radiojournal::crud::station::models::FetcherConfig;

#[derive(Debug)]
struct CoolismToken {
    token: String,
    expiry: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct Coolism {
    client: reqwest::Client,
    token: Mutex<Option<CoolismToken>>,
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
            token: Mutex::new(None),
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
            if !token.is_empty() {
                token.clone()
            } else {
                Err(anyhow!("empty token"))?
            }
        } else {
            Err(anyhow!("no token in response"))?
        };

        let expiry: Option<DateTime<Utc>> = response
            .get("expire")
            .and_then(|val| DateTime::parse_from_rfc3339(val).ok())
            .map(|dt| dt.with_timezone(&Utc));

        info!(
            expiry = expiry.map(|val| val.to_string()),
            "New token fetched"
        );

        Ok((token, expiry))
    }

    async fn fetch_token_cached(&self) -> Result<String> {
        let mut token_locked = self.token.lock().await;
        if let Some(token) = token_locked.as_ref() {
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
                *token_locked = None;
            }
        }

        if let Some(token) = token_locked.as_ref() {
            Ok(token.token.clone())
        } else {
            let (token, maybe_expiry) = self.fetch_token().await?;
            if let Some(expiry) = maybe_expiry {
                *token_locked = Some(CoolismToken {
                    token: token.clone(),
                    expiry,
                });

                Ok(token)
            } else {
                warn!("No expiry detected, will not save token");
                Ok(token)
            }
        }
    }

    async fn fetch_metadata(&self) -> Result<Data> {
        let token = self.fetch_token_cached().await?;
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
    async fn fetch_play(&self, _config: &FetcherConfig) -> Result<Play> {
        let metadata = self.fetch_metadata().await?;
        Ok(Play {
            title: metadata.now_song.song.trim().to_owned(),
            artist: metadata.now_song.artist.trim().to_owned(),
        })
    }
}
