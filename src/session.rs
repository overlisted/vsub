use crate::label::Label;
use crate::line_table::LineTable;
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

        for y in 0..size.height {
            f.render_widget(
                Label("~", style::Style::default().bg(style::Color::Black)),
                layout::Rect::new(0, y, number_digits, 1),
            );
        }

        let mut y = 0;
        let lines = self.line_table.iter(self.scroll_y);
        for (n, range) in lines {
            let shifted = Range {
                start: range.start + self.scroll_x,
                end: range.end,
            };

            if n > self.scroll_y + size.height as usize - 1 {
                break;
            }

            let line_number = (n + 1).to_string();

            f.render_widget(
                Label(
                    &line_number,
                    style::Style::default().bg(style::Color::Black),
                ),
                layout::Rect::new(0, y, number_digits, 1),
            );

            let file_start = number_digits + 1;

            if shifted.start < shifted.end {
                f.render_widget(
                    Label(&self.buffer[shifted.clone()], style::Style::default()),
                    layout::Rect::new(file_start, y, size.width - file_start, 1),
                );

                for hl_range in &self.highlight[n] {
                    let hl_shifted = Range {
                        start: hl_range.start + self.scroll_x,
                        end: hl_range.end,
                    };

                    let x = hl_range.start as u16 - range.start as u16 - self.scroll_x as u16;
                    let width = hl_range.end as u16 - hl_range.start as u16;

                    f.render_widget(
                        Label(
                            &self.buffer[hl_shifted],
                            style::Style::default().bg(style::Color::LightYellow),
                        ),
                        layout::Rect::new(file_start + x, y, width, 1),
                    );
                }
            }

            y += 1;
        }

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
