use tui::*;

pub struct Label<'a>(pub &'a str, pub style::Style);

impl widgets::Widget for Label<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        widgets::Clear.render(area.clone(), buf);

        let width = area.width as usize;

        buf.set_style(area, self.1); // set_string only does this for the effective area :rage:
        buf.set_string(area.left(), area.top(), &self.0, self.1);

        if self.0.len() > width {
            let cell = buf.get_mut(area.x + area.width - 1, area.y);

            cell.set_style(style::Style::default().bg(style::Color::Green));
            cell.set_symbol(">");
        }
    }
}
