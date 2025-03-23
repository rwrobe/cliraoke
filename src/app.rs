use crate::ui::{UIMode, UIState};
use crate::{audio, lyrics};
use ratatui::widgets::ListState;
use std::process::exit;
use serde::{Deserialize, Deserializer};
use serde_json::Number;
use std::collections::BTreeMap;
use std::u64;

fn deserialize_u64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Number::deserialize(deserializer)?;
    Ok(s.to_string())
}

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub lyric: String,
    pub query: String,
    pub audio_results: Vec<Song>,
    pub lyric_results: Vec<Song>,
    pub queue: SongQueue,
    pub time: u64, // Time in milliseconds.
    pub ui_mode: UIMode,
    pub ui_state: UIState,
}

#[derive(Debug, Default)]
pub struct SongQueue {
    pub songs: Vec<Song>,
    pub stat: ListState,
}

pub type LyricsMap = BTreeMap<u64, String>;

// Song is the master struct that holds information composed by both lyric and audio sources.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    #[serde(deserialize_with = "deserialize_u64", rename = "id")]
    pub(crate) lyric_id: String,
    pub(crate) video_id: String,
    pub(crate) track_name: String,
    pub(crate) artist_name: String,
    pub(crate) synced_lyrics: String,
    pub(crate) lyric_map: Option<LyricsMap>,
    pub message: ()
}

impl App {
    pub fn new() -> App {
        App {
            exit: false,
            lyric: String::new(),
            query: String::new(),
            audio_results: Vec::new(),
            lyric_results: Vec::new(),
            queue: SongQueue::default(),
            time: 0,
            ui_mode: UIMode::Navigation,
            ui_state: UIState::Search,
        }
    }

    // search returns two vectors of songs: one from the audio search and one from the lyrics search. These become options for the user.
    pub async fn
    search(&mut self, api_ky: &str) -> Result<(Vec<Song>, Vec<Song>), anyhow::Error> {
        // Search for videos from YouTube.
        let videos = audio::search(api_ky, self.query.as_str()).await;

        if videos.is_err() {
            println!(
                "Error fetching videos: {}",
                videos.err().and_then(|e| Some(e.to_string())).unwrap()
            );
            exit(1);
        }

        // Get the lyrics for the query:
        let lyrs = lyrics::search(self.query.as_str()).await;
        if lyrs.is_err() {
            println!(
                "We found your song, but had a problem finding the lyrics: {}",
                lyrs.err().unwrap()
            );
            exit(1);
        }

        Ok((lyrs?, videos?))
    }

    // add_to_queue will compose a Song from the audio and lyric selections and add it to the queue.
    pub fn add_to_queue(&mut self, audio_selection: Song, lyric_selection: Song) {
        let mut song = lyric_selection.clone();
        song.video_id = audio_selection.video_id;

        self.queue.songs.push(song);
    }

    // todo -- "advancing the lyrics" will mean moving the current time forward
    pub fn advance_lyrics(&mut self) {
        self.time += 500;
    }

    // todo -- "retreating the lyrics" will mean moving the current time backward
    pub fn retreat_lyrics(&mut self) {
        self.time -= 500;
    }
}

