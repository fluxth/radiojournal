pub(crate) mod atime;
pub(crate) mod coolism;

use anyhow::Result;
use async_trait::async_trait;

use radiojournal::{crud::station::Play as PlayTrait, models::station::FetcherConfig};

#[derive(Debug)]
pub(crate) struct Play {
    pub(crate) title: String,
    pub(crate) artist: String,
}

impl PlayTrait for Play {
    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_artist(&self) -> &str {
        &self.artist
    }

    fn is_song(&self) -> bool {
        true
    }
}

#[async_trait]
pub(crate) trait Fetcher {
    async fn fetch_play(&mut self, config: &FetcherConfig) -> Result<Play>;
}
