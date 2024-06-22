pub(crate) mod atime;
pub(crate) mod coolism;
pub(crate) mod iheart;

use anyhow::Result;
use async_trait::async_trait;

use radiojournal::crud::station::models::FetcherConfig;
use radiojournal::crud::station::Play as PlayTrait;

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
    async fn fetch_play(&self, config: &FetcherConfig) -> Result<Play>;
}
