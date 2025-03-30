pub(crate) mod youtube;

pub struct AudioResult {
    pub id: String,
    pub title: String,
    pub artist: String,
}

pub trait AudioService {
    async fn search(&self, query: &str) -> Vec<AudioResult>;
    async fn fetch(&self, id: &str) -> anyhow::Result<AudioResult>;
    fn play(&self, url: &str);
}