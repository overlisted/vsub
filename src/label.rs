use tui::*;

pub struct Label<'a>(pub &'a str, pub style::Style);

impl widgets::Widget for Label<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        widgets::Clear.render(area.clone(), buf);

        buf.set_style(area, self.1); // set_string only does this for the effective area :rage:
        buf.set_string(area.left(), area.top(), self.0, self.1);
    }
}
