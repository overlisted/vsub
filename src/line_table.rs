use std::ops::Range;

pub struct LineTable {
    indices: Vec<usize>,
    buffer_len: usize,
}

impl LineTable {
    pub fn new(buffer: &str) -> Self {
        LineTable {
            indices: buffer
                .char_indices()
                .filter(|(_, ch)| *ch == '\n')
                .map(|(i, _)| i)
                .collect(),
            buffer_len: buffer.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, Range<usize>)> + '_ {
        LineIter {
            indices: &self.indices,
            step: 0,
            buffer_len: self.buffer_len,
        }
    }
}

struct LineIter<'a> {
    indices: &'a Vec<usize>,
    step: usize,
    buffer_len: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = (usize, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let step = self.step;

        if step < self.indices.len() {
            self.step += 1;

            let start = self.indices[step];
            let next_start = if step == self.indices.len() - 1 {
                self.buffer_len
            } else {
                self.indices[step + 1]
            };

            Some((step, start..next_start))
        } else {
            None
        }
    }
}
