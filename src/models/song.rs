use crate::util::deserialize_u64;
use std::collections::BTreeMap;
use std::time::Duration;

// Song is the master struct that holds information composed by both lyric and audio sources.
#[derive(Debug, Clone, serde::Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    #[serde(deserialize_with = "deserialize_u64", rename = "id")]
    pub(crate) lyric_id: String,
    pub(crate) video_id: String,
    #[serde(rename = "artist_name")]
    pub(crate) title: String,
    #[serde(rename = "artist_name")]
    pub(crate) artist: String,
    _duration: Duration,
    pub(crate) duration_ms: u64,
    pub(crate) synced_lyrics: String,
    pub(crate) lyric_map: Option<LyricsMap>,
    pub message: (),
}

impl Song {
    pub fn new() -> Self {
        Song {
            lyric_id: "".to_string(),
            video_id: "".to_string(),
            title: "".to_string(),
            artist: "".to_string(),
            synced_lyrics: "".to_string(),
            lyric_map: None,
            _duration: Duration::new(0, 0),
            duration_ms: 0,
            message: (),
        }
    }
}

pub type SongList = Vec<Song>;

pub type LyricsMap = BTreeMap<u64, String>;
