use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    style::{self, style, Attribute, Color},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand, Result,
};

pub struct Flick {
    border_color: Color,
    buffer: Vec<String>,
    top_row: usize,
    footer: Option<String>,
}

impl Flick {
    pub fn new() -> Self {
        Flick {
            border_color: Color::Blue,
            buffer: vec![],
            top_row: 0,
            footer: None,
        }
    }

    pub fn footer(&mut self, message: &str) {
        self.footer = Some(message.to_owned());
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

        let body_height = self.body_height()?;

        for r in 0..body_height {
            if let Some(ref line) = self.buffer.get(self.top_row + r as usize) {
                stdout.queue(cursor::MoveTo(0, r))?;
                stdout.queue(style::PrintStyledContent(style(line)))?;
            } else {
                break;
            }
        }

        // Draw footer

        stdout.queue(cursor::MoveTo(0, body_height))?;
        stdout.queue(style::PrintStyledContent(
            style(self.footer.as_deref().unwrap_or("flick"))
                .attribute(Attribute::Bold)
                .with(Color::Rgb {
                    r: 26,
                    g: 26,
                    b: 26,
                })
                .on(Color::Rgb {
                    r: 201,
                    g: 64,
                    b: 114,
                }),
        ))?;

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

    pub fn body_height(&self) -> Result<u16> {
        Ok(crossterm::terminal::size()?.1 - 1) // TODO: can the terminal height be 0? then this would overflow
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
                          // eprintln!("Error: {}", e);
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
