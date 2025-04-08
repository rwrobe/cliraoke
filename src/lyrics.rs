use crate::util::deserialize_u64;
use async_trait::async_trait;

pub(crate) mod lrclib;

type LyricsMap = std::collections::BTreeMap<u64, String>;

#[derive(Debug, Clone)]
pub struct LyricsResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub synced_lyrics: String,
    pub lyric_map: Option<LyricsMap>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricResponse {
    #[serde(deserialize_with = "deserialize_u64")]
    pub(crate) id: String,
    pub(crate) track_name: String,
    pub(crate) artist_name: String,
    _album_name: String,
    _instrumental: bool,
    _plain_lyrics: Option<String>,
    pub(crate) synced_lyrics: Option<String>,
    // TODO: Is this even possible?
    pub(crate) message: Option<Message>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub(crate) synced_lyrics: Option<String>,
}

// This is for print purposes only, perhaps the abstraction should not be here but
// in the future print to std layer?
#[derive(Debug)]
pub struct Lyric {
    pub(crate) id: String,
    pub(crate) artist: String,
    pub(crate) title: String,
    pub(crate) synced_lyrics: String,
}

#[async_trait]
pub trait LyricsFetcher {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<LyricsResult>>;
    async fn parse(&self, synced: String) -> anyhow::Result<Option<crate::models::song::LyricsMap>>;
}

pub trait LyricsService: Send + Sync {
    fn play(&self, elapsed_time_ms: u64, lyrics_map: LyricsMap) -> anyhow::Result<String>;
}