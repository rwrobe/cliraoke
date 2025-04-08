use crate::audio::AudioResult;
use crate::lyrics::LyricsResult;
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
    #[serde(rename = "track_name")]
    pub(crate) title: String,
    #[serde(rename = "artist_name")]
    pub(crate) artist: String,
    duration: Duration,
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
            duration: Duration::new(0, 0),
            duration_ms: 0,
            message: (),
        }
    }
}

impl Song {
    pub fn with_lr(&self, lr: LyricsResult, map: Option<LyricsMap>) -> Self {
        Self {
            lyric_id: lr.id.to_string(),
            video_id: self.video_id.clone(),
            title: lr.title.clone(),
            artist: lr.artist.clone(),
            duration: self.duration.clone(),
            duration_ms: self.duration_ms.clone(),
            synced_lyrics: lr.synced_lyrics.clone(),
            lyric_map: map,
            message: (),
        }
    }

    pub fn with_ar(&self, ar: AudioResult) -> Self {
        Self {
            lyric_id: self.lyric_id.clone(),
            video_id: ar.id.clone(),
            title: self.title.clone(),
            artist: self.artist.clone(),
            duration: ar.duration,
            duration_ms: ar.duration.as_millis() as u64,
            synced_lyrics: self.synced_lyrics.clone(),
            lyric_map: self.lyric_map.clone(),
            message: (),
        }
    }
}

pub type SongList = Vec<Song>;

pub type LyricsMap = BTreeMap<u64, String>;
