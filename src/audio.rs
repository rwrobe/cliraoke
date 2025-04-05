use std::error::Error;
use async_trait::async_trait;
use std::time::Duration;

pub(crate) mod youtube;
mod platform;

#[derive(Debug, Clone)]
pub struct AudioResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration: Duration,
}

#[async_trait]
pub trait AudioFetcher {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<AudioResult>>;
    async fn fetch(&self, id: &str) -> anyhow::Result<AudioResult>;
}

#[async_trait]
pub trait AudioService: Send + Sync {
    async fn play(&self, id: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn pause(&self);
}