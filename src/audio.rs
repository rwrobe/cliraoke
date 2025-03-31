use async_trait::async_trait;

pub(crate) mod youtube;

#[derive(Debug, Clone)]
pub struct AudioResult {
    pub id: String,
    pub title: String,
    pub artist: String,
}

#[async_trait]
pub trait AudioService {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<AudioResult>>;
    async fn fetch(&self, id: &str) -> anyhow::Result<AudioResult>;
    fn play(&self, id: &str);
}
