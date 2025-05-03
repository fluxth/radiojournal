pub(crate) mod atime;
pub(crate) mod coolism;
pub(crate) mod iheart;

use anyhow::Result;
use async_trait::async_trait;

use radiojournal::crud::logger::models::Play as PlayTrait;
use radiojournal::crud::station::models::FetcherConfig;

pub(crate) const DEFAULT_USER_AGENT: &str = include_str!("./default_user_agent.txt");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_user_agent_trimmed() {
        assert_eq!(DEFAULT_USER_AGENT, DEFAULT_USER_AGENT.trim());
        assert!(DEFAULT_USER_AGENT.len() > 0);
    }
}
