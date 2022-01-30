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
                .filter_map(|(i, ch)| {
                    if ch == '\n' {
                        Some(i + 1)
                    } else if i == 0 {
                        Some(0)
                    } else {
                        None
                    }
                })
                .collect(),
            buffer_len: buffer.len(),
        }
    }

    pub fn get_bounds(&self, line: usize) -> (usize, usize) {
        let start = self.indices[line];
        let next_start = if line == self.indices.len() - 1 {
            self.buffer_len
        } else {
            self.indices[line + 1]
        };

        (start, next_start)
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn get_line_at(&self, char: usize) -> usize {
        for i in 0..self.indices.len() - 1 {
            if self.indices[i + 1] >= char {
                return i;
            }
        }

        self.indices[self.indices.len() - 1]
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, Range<usize>)> + '_ {
        LineIter {
            table: &self,
            step: 0,
        }
    }
}

struct LineIter<'a> {
    table: &'a LineTable,
    step: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = (usize, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let step = self.step;

        if step < self.table.indices.len() {
            self.step += 1;

            let (start, end) = self.table.get_bounds(step);

            Some((step, start..end))
        } else {
            None
        }
    }
}
