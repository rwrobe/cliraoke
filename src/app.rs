use std::io;

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

#[derive(Debug, Default)]
pub struct App {
    pub time: u64, // Time in milliseconds.
    pub lyric: String,
    pub exit: bool,
}

impl App {
    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        yt_api_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while !self.exit {
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
            KeyCode::Enter => self.open_queue(),
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
        //todo
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
        let title = Line::from("CLIraoke".bold());
        let instructions = Line::from(vec![
            " Move Lyrics Forward ".into(),
            "<Left>".blue().bold(),
            " Move Lyrics Backward ".into(),
            "<Right>".blue().bold(),
            " Open Song Queue ".into(),
            "<Space>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            self.lyric.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}