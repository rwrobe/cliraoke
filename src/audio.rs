use async_trait::async_trait;

pub(crate) mod youtube;

#[derive(Debug, Clone)]
pub struct AudioResult {
    pub id: String,
    pub title: String,
    pub artist: String,
}

#[async_trait]
pub trait AudioFetcher {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<AudioResult>>;
    async fn fetch(&self, id: &str) -> anyhow::Result<AudioResult>;
}

pub trait AudioService: Send + Sync {
    fn play(&self, id: &str);
    fn pause(&self);
}