use crate::label::Label;
use crate::line_table::LineTable;
use crate::text_view::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
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
            // &["", s, r, ""] => {
            //     let sr = Regex::new(s).unwrap();
            //     let rr = Regex::new(r).unwrap();
            //
            //     for m in sr.find_iter(&self.buffer) {
            //
            //     }
            // }
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

    pub fn ui<B: backend::Backend>(&self, f: &mut Frame<B>) {
        let size = f.size();

        let number_digits = self.line_table.len().to_string().len() as u16 + 1; // yeah my iq score is 150 how could you tell

        f.render_widget(
            LineNumbers {
                start_at: self.scroll_y,
                lines: self.line_table.len(),
            },
            layout::Rect {
                width: number_digits,
                ..size
            },
        );

        f.render_widget(
            TextView {
                buffer: &self.buffer,
                line_table: &self.line_table,
                highlight: &self.highlight,
                scroll_x: self.scroll_x,
                scroll_y: self.scroll_y,
            },
            layout::Rect {
                x: number_digits + 1,
                y: 0,
                width: size.width - number_digits - 1,
                height: size.height - 2,
            },
        );

        f.render_widget(
            Label(
                &self.status,
                style::Style::default().bg(style::Color::LightRed),
            ),
            layout::Rect::new(0, size.height - 2, size.width, 1),
        );

        f.render_widget(
            Label(&self.prompt, style::Style::default().bg(style::Color::Red)),
            layout::Rect::new(0, size.height - 1, size.width, 1),
        );
    }
}
