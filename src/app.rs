use std::collections::BTreeMap;
use std::io;
use std::iter::Map;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Default, PartialEq)]
pub enum WidgetState {
    Lyrics,
    Queue,
    #[default]
    Search,
}

#[derive(Debug, Default)]
pub struct App {
    pub exit: bool,
    pub lyric: String,
    pub queue: BTreeMap<String, BTreeMap<u64, String>>,
    pub time: u64, // Time in milliseconds.
    pub widget_state: WidgetState,
}

impl App {
    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        yt_api_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while !self.exit {
            self.queue = BTreeMap::new();
            self.queue.insert("1. Test Song".to_string(), BTreeMap::new());
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.open_queue(),
            KeyCode::Right => self.advance_lyrics(),
            KeyCode::Left => self.retreat_lyrics(),
            _ => {}
        }
    }

    // todo -- "advancing the lyrics" will mean moving the current time forward
    fn advance_lyrics(&mut self) {
        //todo
    }

    // todo -- "retreating the lyrics" will mean moving the current time backward
    fn retreat_lyrics(&mut self) {
        //todo
    }

    fn open_queue(&mut self) {
        self.widget_state = WidgetState::Queue;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        let title = Line::from(" CLIraoke ".bold());
        let instructions = Line::from(vec![
            " Move Lyrics Forward ".into(),
            "<Left>".blue().bold(),
            " Move Lyrics Backward ".into(),
            "<Right>".blue().bold(),
            " Open Song Queue ".into(),
            "<Esc>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let lyric = Text::from(vec![Line::from(vec![
            self.lyric.to_string().yellow(),
        ])]);

        if self.widget_state == WidgetState::Queue {
            let mut lines = vec![Line::from("Queue".bold())];
            for (queue_title, nested_map) in self.queue.iter() {
                lines.push(Line::from(queue_title.to_string().bold()));
            }
            let queue = Text::from(lines);
            Paragraph::new(queue)
                .centered()
                .block(block)
                .render(area, buf);
        } else {
            Paragraph::new(lyric)
                .centered()
                .block(block)
                .render(area, buf);
        }
    }
}