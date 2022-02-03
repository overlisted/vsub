use crate::commands;
use crate::line_table::LineTable;
use crate::widgets::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use std::io::{Read, Seek, Write};
use std::ops::Range;
use std::{fs, io};
use tui::*;

pub struct CommandInterface<'s>(&'s mut Session);

impl<'s> CommandInterface<'s> {
    pub fn save_buffer(&mut self) -> io::Result<usize> {
        let bytes = self.0.buffer.as_bytes();

        self.0.file.set_len(bytes.len() as u64)?;
        let n = self.0.file.write(bytes)?;
        self.0.file.rewind()?;

        Ok(n)
    }

    // this is fucking stupid
    pub fn highlight(
        &mut self,
        mut f: impl FnMut(&mut Vec<Vec<Range<usize>>>, &String, &LineTable),
    ) {
        f(&mut self.0.highlight, &self.0.buffer, &self.0.line_table)
    }

    pub fn reset_highlight(&mut self) {
        self.0.highlight = vec![Vec::with_capacity(16); self.0.line_table.len()];
    }

    pub fn update_buffer(&mut self, f: impl FnOnce(&mut String)) {
        f(&mut self.0.buffer);

        self.0.line_table = LineTable::new(&self.0.buffer);
        self.reset_highlight();
    }

    pub fn get_buffer(&self) -> &String {
        &self.0.buffer
    }

    pub fn finalize(self, clear_prompt: bool, status: String) {
        self.0.status = status;

        if clear_prompt {
            self.0.prompt.clear();
        }
    }
}

pub struct Session {
    file: fs::File,
    buffer: String,
    line_table: LineTable,
    prompt: String,
    highlight: Vec<Vec<Range<usize>>>,
    status: String,
    scroll_y: usize,
    scroll_x: usize,
}

impl Session {
    pub fn new(mut file: fs::File) -> io::Result<Self> {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        file.rewind()?;

        let table = LineTable::new(&buffer);
        let lines = table.len();

        Ok(Session {
            status: format!("loaded {} characters, {} lines", buffer.len(), lines),
            file,
            buffer,
            line_table: table,
            prompt: String::with_capacity(128),
            highlight: vec![Vec::with_capacity(16); lines],
            scroll_y: 0,
            scroll_x: 0,
        })
    }

    fn get_command(&self) -> Option<Box<dyn commands::Command>> {
        let parts: Vec<&str> = self.prompt.split('/').collect();

        Some(match parts.as_slice() {
            &["w"] => Box::new(commands::file::Write),
            &["s", s, r, ""] => Box::new(commands::regex::Substitute(
                Regex::new(s).unwrap(),
                r.into(),
            )),
            &["s", s, ""] => Box::new(commands::regex::Remove(Regex::new(s).unwrap())),
            &["p", s, ""] => Box::new(commands::regex::Highlight(Regex::new(s).unwrap())),
            _ => return None,
        })
    }

    pub fn key(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => self.prompt.push(ch),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                if let Some(mut command) = self.get_command() {
                    let interface = CommandInterface(self);

                    if let Err(e) = command.run(interface) {
                        self.status = format!("Error: {}", e);
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => drop(self.prompt.pop()),
            KeyEvent {
                code: KeyCode::Up, ..
            } => {
                if self.scroll_y > 0 {
                    self.scroll_y -= 1
                }
            }
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => self.scroll_y += 1,
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                if self.scroll_x > 0 {
                    self.scroll_x -= 1
                }
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => self.scroll_x += 1,
            _ => {}
        }
    }

    fn file_ui<B: backend::Backend>(&self, f: &mut Frame<B>, area: layout::Rect) {
        let number_digits = self.line_table.len().to_string().len() as u16 + 1;

        let chunks = layout::Layout::default()
            .direction(layout::Direction::Horizontal)
            .constraints([
                layout::Constraint::Length(number_digits),
                layout::Constraint::Length(1),
                layout::Constraint::Min(1),
            ])
            .split(area);

        f.render_widget(
            text_view::LineNumbers {
                start_at: self.scroll_y,
                lines: self.line_table.len(),
            },
            chunks[0],
        );

        f.render_widget(
            text_view::TextView {
                buffer: &self.buffer,
                line_table: &self.line_table,
                highlight: &self.highlight,
                scroll_x: self.scroll_x,
                scroll_y: self.scroll_y,
            },
            chunks[2],
        );
    }

    pub fn ui<B: backend::Backend>(&self, f: &mut Frame<B>) {
        let chunks = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([
                layout::Constraint::Min(0),
                layout::Constraint::Length(1),
                layout::Constraint::Length(1),
            ])
            .split(f.size());

        self.file_ui(f, chunks[0]);

        f.render_widget(
            label::Label(
                &self.status,
                style::Style::default().bg(style::Color::LightRed),
            ),
            chunks[1],
        );

        f.render_widget(
            label::Label(&self.prompt, style::Style::default().bg(style::Color::Red)),
            chunks[2],
        );
    }
}
