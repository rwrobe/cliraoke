pub(crate) mod lrclib;

type LyricsMap = std::collections::BTreeMap<u64, String>;

struct LyricsResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub synced_lyrics: String,
    pub lyric_map: Option<LyricsMap>,
}

pub trait LyricsService {
    fn search(&self, query: &str) -> Vec<LyricsResult>;
    fn fetch(&self, id: &str) -> anyhow::Result<String>;
    fn play(&self, url: &str);
}