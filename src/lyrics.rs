use async_trait::async_trait;
use crate::audio::AudioResult;

pub(crate) mod lrclib;

type LyricsMap = std::collections::BTreeMap<u64, String>;

#[derive(Debug)]
struct LyricsResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub synced_lyrics: String,
    pub lyric_map: Option<LyricsMap>,
}

#[async_trait]
pub trait LyricsService {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<LyricsResult>>;
    async fn fetch(&self, id: &str) -> anyhow::Result<String>;
    fn play(&self, url: &str);
}