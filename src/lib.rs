use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    style::{self, style, Color},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand, Result,
};

pub struct Flick {
    border_color: Color,
    buffer: Vec<String>,
    top_row: usize,
    header: Option<Vec<String>>,
    footer: Option<Vec<String>>,
}

impl Flick {
    pub fn new() -> Self {
        Flick {
            border_color: Color::Blue,
            buffer: vec![],
            top_row: 0,
            header: None,
            footer: None,
        }
    }

    pub fn header(&mut self, message: &str) {
        self.header = Some(message.lines().map(|l| l.to_owned()).collect());
    }

    pub fn footer(&mut self, message: &str) {
        self.footer = Some(message.lines().map(|l| l.to_owned()).collect());
    }

    pub fn append(&mut self, content: &str) {
        for line in content.lines() {
            self.buffer.push(line.to_owned());
        }
    }

    fn clear_screen(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn redraw(&self) -> Result<()> {
        self.clear_screen()?;

        let mut stdout = stdout();

        let header_height = self.header_height();

        // Header
        if let Some(ref header) = self.header {
            for r in 0..header.len() {
                stdout.queue(cursor::MoveTo(0, r as u16))?;
                stdout.queue(style::PrintStyledContent(style(&header[r])))?;
            }
        }

        // Body
        let body_height = self.body_height()?;

        for r in 0..body_height {
            if let Some(ref line) = self.buffer.get(self.top_row + r as usize) {
                stdout.queue(cursor::MoveTo(0, header_height + r))?;
                stdout.queue(style::PrintStyledContent(style(line)))?;
            } else {
                break;
            }
        }

        // Footer
        stdout.queue(cursor::MoveTo(0, body_height))?;
        if let Some(ref footer) = self.footer {
            for r in 0..footer.len() {
                stdout.queue(cursor::MoveTo(0, header_height + body_height + r as u16))?;
                stdout.queue(style::PrintStyledContent(style(&footer[r])))?;
            }
        }

        stdout.flush()?;

        Ok(())
    }

    pub fn scroll_down<U: Into<usize>>(&mut self, lines: U) -> Result<()> {
        let length = self.content_length();
        let height = self.body_height()? as usize;

        let lines = lines.into();

        self.top_row += lines;

        let max_top_row = if length > height { length - height } else { 0 };
        if self.top_row > max_top_row {
            self.top_row = max_top_row;
        }

        Ok(())
    }

    pub fn scroll_up<U: Into<usize>>(&mut self, lines: U) -> Result<()> {
        let lines = lines.into();
        self.top_row = if self.top_row >= lines {
            self.top_row - lines
        } else {
            0
        };

        Ok(())
    }

    pub fn header_height(&self) -> u16 {
        self.header.as_ref().map(|h| h.len()).unwrap_or(0) as u16
    }

    pub fn footer_height(&self) -> u16 {
        self.footer.as_ref().map(|h| h.len()).unwrap_or(0) as u16
    }

    pub fn body_height(&self) -> Result<u16> {
        Ok(crossterm::terminal::size()?.1 - self.header_height() - self.footer_height())
        // TODO: handle overflows, what about size 0 terminals?
    }

    pub fn content_length(&self) -> usize {
        self.buffer.len()
    }

    pub fn run(&mut self) {
        let result = self.run_impl();

        match result {
            Ok(_) => {}
            Err(e) => {
                self.cleanup().ok();
                dbg!(&e); // TODO
            }
        }
    }

    fn run_impl(&mut self) -> Result<()> {
        let mut stdout = stdout();

        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        enable_raw_mode()?;

        self.redraw()?;

        loop {
            if poll(Duration::from_millis(20))? {
                match read()? {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Char('r') => {
                                self.border_color = Color::Red;
                            }
                            KeyCode::Char('b') => {
                                self.border_color = Color::Blue;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Enter => {
                                self.scroll_down(1usize)?;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                self.scroll_up(1usize)?;
                            }
                            KeyCode::PageDown => {
                                self.scroll_down(self.body_height()?)?;
                            }
                            KeyCode::PageUp => {
                                self.scroll_up(self.body_height()?)?;
                            }
                            KeyCode::Home => {
                                self.top_row = 0;
                            }
                            KeyCode::End => {
                                let length = self.content_length();
                                self.scroll_down(length)?;
                            }
                            KeyCode::Char('c')
                                if event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                break;
                            }
                            _ => {}
                        }
                        self.redraw()?;
                    }
                    Event::Mouse(_) => {
                        // Capturing of mouse events is not enabled in order to allow
                        // for normal text selection. Scrolling still works on terminals
                        // that send up/down arrow events.
                    }
                    Event::Resize(_width, _height) => {
                        self.redraw()?;
                    }
                }
            }
        }

        self.cleanup()?;

        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        disable_raw_mode()?;

        execute!(stdout(), cursor::Show, terminal::LeaveAlternateScreen)
    }
}
