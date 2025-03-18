use crate::audio::Audio;
use color_eyre::owo_colors::OwoColorize;
use ratatui::widgets::ListState;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        palette::tailwind::SLATE,
        Color, Stylize,
    },
    symbols,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    Frame,
};
use std::io;
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
    pub lyrics: Vec<Lyric>,
}

#[derive(Debug, Clone)]
pub struct Lyric {
    pub timestamp: u64,
    pub text: String,
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

    pub async fn search(&mut self) -> Result<(Vec<lyrics::Lyric>, Vec<audio::Video>), anyhow::Error> {
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" CLIraoke ".bold());
        let time = Line::from(format!("Time: {}", self.time).bold());
        let instructions = Line::from(vec![
            " Move Lyrics Forward ".into(),
            "<Left>".blue().bold(),
            " Move Lyrics Backward ".into(),
            "<Right>".blue().bold(),
            " Open Song Queue ".into(),
            "<Esc>".blue().bold(),
            " Search for a Song ".into(),
            "<Tab>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let lyric = Text::from(vec![Line::from(vec![self.lyric.to_string().yellow()])]);

        match self.widget_state {
            WidgetState::Lyrics => {
                let lyric = Text::from(vec![Line::from(vec![self.lyric.to_string().yellow()])]);
                Paragraph::new(lyric)
                    .centered()
                    .block(Block::new())
                    .render(area, buf);
            }
            WidgetState::Queue => {
                let mut lines = vec![Line::from("Queue".bold())];
                let block = Block::new()
                    .title(Line::from("Queue".bold()).centered())
                    .border_set(border::THICK)
                    .border_set(symbols::border::EMPTY);
                let song_list: Vec<Song> = self
                    .queue
                    .songs
                    .iter()
                    .enumerate()
                    .map(|(i, song)| {
                        let color = alternate_colors(i);
                        Song {
                            yt_ud: song.yt_ud.clone(),
                            lyric_ud: song.lyric_ud,
                            title: song.title.clone(),
                            artist: song.artist.clone(),
                            lyrics: song.lyrics.clone(),
                        }
                    })
                    .collect();

                Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
                    .centered()
                    .render(area, buf);
            }
            WidgetState::SearchYT => {
                let search = Text::from(vec![Line::from("Search YouTube for Your Song (Press <Tab> to cancel)".bold())]);
                Paragraph::new(search)
                    .centered()
                    .block(block)
                    .render(area, buf);
            }
            WidgetState::SearchLyrics => {
                let search = Text::from(vec![Line::from("Ok, now let's find the lyrics (Press <Tab> to cancel)".bold())]);
                Paragraph::new(search)
                    .centered()
                    .block(block)
                    .render(area, buf);
            }
        }
    }
}

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
