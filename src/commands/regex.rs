use crate::session::CommandInterface;
use regex::Regex;
use std::borrow::Cow;
use std::io;
use std::ops::Range;

pub struct Substitute(pub Regex, pub String);

impl super::Command for Substitute {
    fn run(&mut self, mut interface: CommandInterface) -> io::Result<()> {
        let buffer = interface.get_buffer();
        let old_len = buffer.len();

        let new = self.0.replace_all(&buffer, &self.1);
        let new_len = new.len();

        if let Cow::Owned(s) = new {
            interface.update_buffer(|buffer| *buffer = s);
        }

        interface.finalize(
            true,
            format!(
                "difference: {} characters",
                new_len as isize - old_len as isize
            ),
        );

        Ok(())
    }
}

pub struct Remove(pub Regex);

impl super::Command for Remove {
    fn run(&mut self, mut interface: CommandInterface) -> io::Result<()> {
        let old_len = interface.get_buffer().len();

        interface.update_buffer(|buffer| buffer.remove_matches(&self.0));

        let new_len = interface.get_buffer().len();

        interface.finalize(
            true,
            format!(
                "difference: {} characters",
                new_len as isize - old_len as isize
            ),
        );

        Ok(())
    }
}

pub struct Highlight(pub Regex);

impl super::Command for Highlight {
    fn run(&mut self, mut interface: CommandInterface) -> io::Result<()> {
        interface.reset_highlight();

        let mut matches: usize = 0;
        let mut lines: usize = 0;

        interface.highlight(|highlight, buffer, line_table| {
            for m in self.0.find_iter(buffer) {
                let mut line_n = line_table.get_line_at(m.start());

                loop {
                    let (line_start, line_end) = line_table.get_bounds(line_n);

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

                    highlight[line_n].push(final_range);

                    if line_end >= m.end() {
                        break;
                    }

                    line_n += 1;
                    lines += 1;
                }

                matches += 1;
            }
        });

        interface.finalize(false, format!("{} matches, {} lines", matches, lines));

        Ok(())
    }
}
