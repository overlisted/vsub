use crate::line_table::LineTable;
use std::ops::Range;
use tui::*;

pub struct LineNumbers {
    pub start_at: usize,
    pub lines: usize,
}

impl widgets::Widget for LineNumbers {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        for h in 0..area.height {
            let y = area.y + h;
            let line = self.start_at + h as usize + 1;

            buf.set_style(
                layout::Rect::new(area.x, y, area.width, 1),
                style::Style::default().bg(style::Color::Black),
            );

            if line > self.lines {
                buf.get_mut(area.x, y).set_symbol("~");
            } else {
                buf.set_string(
                    area.x,
                    y,
                    line.to_string(),
                    style::Style::default().bg(style::Color::Black),
                );
            }
        }
    }
}

pub struct TextView<'a> {
    pub buffer: &'a str,
    pub line_table: &'a LineTable,
    pub highlight: &'a [Vec<Range<usize>>],
    pub scroll_x: usize,
    pub scroll_y: usize,
}

impl widgets::Widget for TextView<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let lines = self.line_table.iter(self.scroll_y);

        let mut h = 0;
        for (n, range) in lines {
            let y = area.y + h;

            let shifted = Range {
                start: range.start + self.scroll_x,
                end: range.end,
            };

            if n > self.scroll_y + area.height as usize {
                break;
            }

            if shifted.start < shifted.end {
                buf.set_string(
                    area.x,
                    y,
                    &self.buffer[shifted.clone()],
                    style::Style::default(),
                );

                for hl_range in &self.highlight[n] {
                    let mut x = hl_range.start - range.start;
                    let mut hl_shifted = hl_range.clone();

                    if x < self.scroll_x {
                        // todo: self.scroll_x % hl_range.len()
                        hl_shifted.start = hl_range.start + self.scroll_x;
                        x += self.scroll_x;
                    }

                    if x < area.width as usize + self.scroll_x && hl_shifted.start < hl_shifted.end
                    {
                        buf.set_string(
                            area.x + x as u16 - self.scroll_x as u16,
                            y,
                            &self.buffer[hl_shifted],
                            style::Style::default().bg(style::Color::LightYellow),
                        );
                    }
                }

                if shifted.end - shifted.start > area.width as usize {
                    let cell = buf.get_mut(area.x + area.width - 1, y);

                    cell.set_style(style::Style::default().bg(style::Color::Green));
                    cell.set_symbol(">");
                }
            }

            h += 1;
        }
    }
}
