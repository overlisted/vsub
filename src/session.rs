use crate::label::Label;
use crate::line_table::LineTable;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use std::fs;
use std::ops::Range;
use tui::*;

pub struct Session {
    buffer: String,
    line_table: LineTable,
    prompt: String,
    highlight: Vec<Vec<Range<usize>>>,
}

impl Session {
    pub fn new(buffer: String) -> Self {
        let table = LineTable::new(&buffer);
        let lines = table.len();

        Session {
            line_table: table,
            buffer,
            prompt: String::with_capacity(128),
            highlight: vec![Vec::with_capacity(16); lines],
        }
    }

    fn command(&mut self) {
        let parts: Vec<&str> = self.prompt.split('/').collect();

        match parts.as_slice() {
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

                self.buffer.remove_matches(&regex);

                self.line_table = LineTable::new(&self.buffer);
                self.prompt.clear();
            }
            &["p", s, ""] => {
                for line in &mut self.highlight {
                    line.clear();
                }

                let regex = Regex::new(s).unwrap();

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
                    }
                }
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
            _ => {}
        }
    }

    pub fn ui<B: backend::Backend>(&self, f: &mut Frame<B>) {
        let size = f.size();

        let mut y = 0;
        for (n, range) in self.line_table.iter() {
            f.render_widget(
                Label(&self.buffer[range.clone()], style::Style::default()),
                layout::Rect::new(1, y, size.width - 1, 1),
            );

            for hl_range in &self.highlight[n] {
                let x = hl_range.start as u16 - range.start as u16;
                let width = hl_range.end as u16 - hl_range.start as u16;

                f.render_widget(
                    Label(
                        &self.buffer[hl_range.clone()],
                        style::Style::default().bg(style::Color::LightYellow),
                    ),
                    layout::Rect::new(x, y, width, 1),
                );
            }

            y += 1;
        }

        f.render_widget(
            Label(&self.prompt, style::Style::default().bg(style::Color::Red)),
            layout::Rect::new(0, size.height - 1, size.width, 1),
        );
    }
}
