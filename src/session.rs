use crate::line_table::LineTable;
use crate::widgets::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use std::borrow::Cow;
use std::io::{Read, Seek, Write};
use std::ops::Range;
use std::{fs, io};
use tui::*;

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

    fn command(&mut self) {
        let parts: Vec<&str> = self.prompt.split('/').collect();

        match parts.as_slice() {
            &["w"] => {
                self.file
                    .set_len(self.buffer.as_bytes().len() as u64)
                    .unwrap();
                let n = self.file.write(self.buffer.as_bytes()).unwrap();

                self.prompt.clear();
                self.status = format!("{} bytes written", n);
            }
            &["s", s, r, ""] => {
                for line in &mut self.highlight {
                    line.clear();
                }

                let find = Regex::new(s).unwrap();

                let new = find.replace_all(&self.buffer, r);

                self.status = format!(
                    "difference: {} characters",
                    self.buffer.len() as isize - new.len() as isize
                );

                if let Cow::Owned(s) = new {
                    self.buffer = s;
                }

                self.line_table = LineTable::new(&self.buffer);
                self.prompt.clear();
            }
            &["s", s, ""] => {
                for line in &mut self.highlight {
                    line.clear();
                }

                let regex = Regex::new(s).unwrap();

                let old_len = self.buffer.len();
                self.buffer.remove_matches(&regex);

                self.line_table = LineTable::new(&self.buffer);
                self.prompt.clear();

                self.status = format!("{} characters less", old_len - self.buffer.len());
            }
            &["p", s, ""] => {
                for line in &mut self.highlight {
                    line.clear();
                }

                let regex = Regex::new(s).unwrap();
                let mut matches: usize = 0;
                let mut lines: usize = 0;

                for m in regex.find_iter(&self.buffer) {
                    let mut line_n = self.line_table.get_line_at(m.start());

                    loop {
                        let (line_start, line_end) = self.line_table.get_bounds(line_n);

                        let final_range = Range {
                            start: if line_start < m.start() {
                                m.start()
                            } else {
                                line_start
                            },
                            end: if line_end > m.end() {
                                m.end()
                            } else {
                                line_end
                            },
                        };

                        self.highlight[line_n].push(final_range);

                        if line_end >= m.end() {
                            break;
                        }

                        line_n += 1;
                        lines += 1;
                    }

                    matches += 1;
                }

                self.status = format!("{} matches, {} lines", matches, lines);
            }
            _ => {}
        }
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
            } => self.command(),
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
