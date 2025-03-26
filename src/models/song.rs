use crate::util::deserialize_u64;
use std::collections::BTreeMap;

// Song is the master struct that holds information composed by both lyric and audio sources.
#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    #[serde(deserialize_with = "deserialize_u64", rename = "id")]
    pub(crate) lyric_id: String,
    pub(crate) video_id: String,
    #[serde(rename = "artist_name")]
    pub(crate) title: String,
    #[serde(rename = "artist_name")]
    pub(crate) artist: String,
    pub(crate) synced_lyrics: String,
    pub(crate) lyric_map: Option<LyricsMap>,
    pub message: ()
}

pub type LyricsMap = BTreeMap<u64, String>;
