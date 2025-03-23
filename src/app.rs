use crate::audio::Audio;
use ratatui::widgets::ListState;
use std::process::exit;
use crate::{audio, lyrics};

#[derive(Debug, Default, PartialEq)]
pub enum WidgetState {
    Lyrics,
    Queue,
    #[default]
    SearchYT,
    SearchLyrics,
}

#[derive(Debug)]
pub struct App {
    audio: Audio,
    pub exit: bool,
    pub lyric: String,
    pub query: String,
    pub queue: SongQueue,
    pub time: u64, // Time in milliseconds.
    pub ui_mode: UIMode,
    pub widget_state: WidgetState,
}

#[derive(Debug, Default, PartialEq)]
pub enum UIMode {
    Edit,
    #[default]
    Navigation,
}

pub struct SearchState {
    pub query: String,
    pub results: Vec<Song>,
    pub stat: ListState,
}

#[derive(Debug, Default)]
pub struct SongQueue {
    pub songs: Vec<Song>,
    pub stat: ListState,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub yt_ud: String,
    pub lyric_ud: u64,
    pub title: String,
    pub artist: String,
    pub lyrics: Vec<lyrics::DisplayLyric>,
}

impl App {
    pub fn new(audio: Audio)-> App {
        App {
            audio,
            exit: false,
            lyric: String::new(),
            query: String::new(),
            queue: SongQueue::default(),
            time: 0,
            ui_mode: UIMode::Navigation,
            widget_state: WidgetState::SearchYT,
        }
    }

    pub async fn search(&mut self) -> Result<(Vec<lyrics::OptionResponse>, Vec<audio::OptionResponse>), anyhow::Error> {
        // Fetch the videos using the audio client.
        let videos = self.audio.fetch_videos(self.query.as_str()).await;

        if videos.is_err() {
            println!(
                "Error fetching videos: {}",
                videos.err().and_then(|e| Some(e.to_string())).unwrap()
            );
            exit(1);
        }

        // Get the lyrics for the query:
        let lyrs = lyrics::search_lyrics(self.query.as_str()).await;
        if lyrs.is_err() {
            println!("We found your song, but had a problem finding the lyrics: {}", lyrs.err().unwrap());
            exit(1);
        }

        Ok((lyrs?, videos?))
    }

    pub fn compose_song(&mut self, lyric: lyrics::OptionResponse, video: audio::OptionResponse) {

    }

    pub fn add_to_queue(&mut self, song: Song) {
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
