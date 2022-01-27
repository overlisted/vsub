use crate::label::Label;
use crate::line_table::LineTable;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use tui::*;

pub struct Session {
    buffer: String,
    line_table: LineTable,
    prompt: String,
    highlight: Vec<(usize, usize)>,
}

impl Session {
    pub fn new(buffer: String) -> Self {
        Session {
            line_table: LineTable::new(&buffer),
            buffer,
            prompt: String::with_capacity(128),
            highlight: Vec::new(),
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
                self.highlight.clear();

                let regex = Regex::new(s).unwrap();

                self.buffer.remove_matches(&regex);

                self.line_table = LineTable::new(&self.buffer);
                self.prompt.clear();
            }
            &["p", s, ""] => {
                self.highlight.clear();

                let regex = Regex::new(s).unwrap();

                for m in regex.find_iter(&self.buffer) {
                    self.highlight.push((m.start(), m.end()))
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
        for (_, range) in self.line_table.iter() {
            let line = &self.buffer[range];

            let widget = Label(&line, style::Style::default());

            f.render_widget(widget, layout::Rect::new(0, y, size.width, 1));

            y += 1;
        }

        f.render_widget(
            Label(&self.prompt, style::Style::default().bg(style::Color::Red)),
            layout::Rect::new(0, size.height - 1, size.width, 1),
        );
    }
}
