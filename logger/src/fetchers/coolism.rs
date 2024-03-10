use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{Fetcher, Play};

#[derive(Debug)]
pub(crate) struct Coolism;
impl Coolism {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

async fn fetch_token() -> Result<String> {
    let client = reqwest::Client::new();
    let response: HashMap<String, String> = client
        .post("https://api.coolism.net/api/auth/gettoken")
        .basic_auth("coolism", Some("PshjzmOQ"))
        .header("Authorization", "Basic Y29vbGlzbTpQc2hqem1PUQ==")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3")
        .header("Origin", "https://www.coolism.net")
        .header("Referer", "https://www.coolism.net/")
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

async fn fetch_metadata(token: &str) -> Result<Data> {
    let client = reqwest::Client::new();
    let response: MetadataResponse = client
        .get("https://api.coolism.net/api/song/radio/nowPlaying")
        .query(&[("type", "start"), ("t", &chrono::offset::Utc::now().timestamp_millis().to_string())])
        .bearer_auth(token)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3")
        .header("Origin", "https://www.coolism.net")
        .header("Referer", "https://www.coolism.net/")
        .send()
        .await?
        .json()
        .await?;

    Ok(response.data)
}

#[async_trait]
impl Fetcher for Coolism {
    async fn fetch_play(&self) -> Result<Play> {
        let token = fetch_token().await?;
        let metadata = fetch_metadata(&token).await?;
        Ok(Play {
            title: metadata.now_song.song,
            artist: metadata.now_song.artist,
        })
    }
}
