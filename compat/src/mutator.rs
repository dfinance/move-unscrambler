use std::ops::Range;

#[derive(Debug)]
pub struct Mutator {
    diff: Vec<Diff>,
}

impl Mutator {
    pub fn new() -> Mutator {
        Mutator { diff: vec![] }
    }

    pub fn make_diff(&mut self, start_offset: usize, end_offset: usize, new_value: Vec<u8>) {
        self.diff.push(Diff {
            source_range: start_offset..end_offset,
            value: new_value,
        });
    }

    pub fn mutate(mut self, buffer: &mut Vec<u8>) {
        self.diff
            .sort_by(|a, b| a.source_range.start.cmp(&b.source_range.start));

        let origin_len = buffer.len();

        let mut offset_diff = 0;
        for mutation in self.diff {
            let mutation_diff_len = mutation.offset_diff();

            if mutation.source_range.start >= origin_len {
                buffer.extend_from_slice(mutation.value.as_slice());
                continue;
            }

            match mutation_diff_len {
                0 => {
                    let dest = &mut buffer[mutation.source_range_with_diff(offset_diff)];
                    dest.copy_from_slice(&mutation.value);
                }
                len if len > 0 => {
                    let start = mutation.start_with_diff(offset_diff);
                    let mutation_diff_len = mutation_diff_len.abs() as usize;

                    for src_index in 0..mutation_diff_len {
                        buffer.insert(start + src_index, mutation.value[src_index]);
                    }

                    let mut range = mutation.source_range_with_diff(offset_diff);
                    range.start += mutation_diff_len;
                    range.end += mutation_diff_len;
                    let dest = &mut buffer[range];

                    dest.copy_from_slice(&mutation.value[mutation_diff_len..]);
                }
                _ => {
                    let start = mutation.start_with_diff(offset_diff);
                    for _ in 0..mutation_diff_len.abs() {
                        buffer.remove(start);
                    }
                    let dest = &mut buffer[start..start + mutation.len()];
                    dest.copy_from_slice(&mutation.value);
                }
            }

            offset_diff += mutation_diff_len;
        }
    }
}

#[derive(Debug)]
pub struct Diff {
    source_range: Range<usize>,
    value: Vec<u8>,
}

impl Diff {
    pub fn offset_diff(&self) -> isize {
        let origin_len = (self.source_range.end - self.source_range.start) as isize;
        (self.value.len() as isize) - origin_len
    }

    pub fn source_range_with_diff(&self, offset_diff: isize) -> Range<usize> {
        ((self.source_range.start as isize) + offset_diff) as usize
            ..((self.source_range.end as isize) + offset_diff) as usize
    }

    pub fn start_with_diff(&self, offset_diff: isize) -> usize {
        ((self.source_range.start as isize) + offset_diff) as usize
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::mutator::Mutator;

    #[test]
    fn test_extend_mutation() {
        let mut buffer = vec![0x1, 0x2, 0x3];
        let mut m = Mutator::new();

        m.make_diff(4, 4, vec![0x4, 0x5, 0x6]);
        m.make_diff(5, 5, vec![0x7, 0x8, 0x9]);

        m.mutate(&mut buffer);

        assert_eq!(buffer, vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9]);
    }

    #[test]
    fn test_equal_size_mutation() {
        let mut buffer = vec![0x1, 0x0A, 0x0B, 0x0C, 0x5, 0x6, 0x0D, 0x0E, 0x0A];
        let mut m = Mutator::new();

        m.make_diff(1, 4, vec![0x2, 0x3, 0x4]);
        m.make_diff(6, 9, vec![0x7, 0x8, 0x9]);

        m.mutate(&mut buffer);

        assert_eq!(buffer, vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9]);
    }

    #[test]
    fn test_equal_size_mutation_1() {
        let mut buffer = vec![0x0A, 0x0B, 0x0C, 0x4, 0x5, 0x6, 0x0A, 0x0B, 0x0C];
        let mut m = Mutator::new();

        m.make_diff(0, 3, vec![0x1, 0x2, 0x3]);
        m.make_diff(6, 9, vec![0x7, 0x8, 0x9]);

        m.mutate(&mut buffer);

        assert_eq!(buffer, vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9]);
    }

    #[test]
    fn test_cat_mutation() {
        let mut buffer = vec![
            0x0D, 0x0E, 0x0A, 0x1, 0x0D, 0x0E, 0x0A, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x0D, 0x0E,
            0x0A, 0x8, 0x9, 0x0D, 0x0E, 0x0A,
        ];
        let mut m = Mutator::new();

        m.make_diff(0, 3, vec![]);
        m.make_diff(4, 7, vec![]);
        m.make_diff(13, 16, vec![]);
        m.make_diff(18, 21, vec![]);

        m.mutate(&mut buffer);

        assert_eq!(buffer, vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9]);
    }

    #[test]
    fn test_cat_mutation_1() {
        let mut buffer = vec![
            0x0D, 0x0E, 0x0A, 0x1, 0x0D, 0x0E, 0x0A, 0x4, 0x5, 0x6, 0x7, 0x0D, 0x0E, 0x0A, 0x8,
            0x9, 0x0D, 0x0E, 0x0A, 0x00, 0x00,
        ];
        let mut m = Mutator::new();

        m.make_diff(0, 3, vec![]);

        m.make_diff(4, 7, vec![0x2, 0x3]);
        m.make_diff(11, 14, vec![]);
        m.make_diff(16, 21, vec![0x0A]);

        m.mutate(&mut buffer);

        assert_eq!(
            buffer,
            vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x0A]
        );
    }

    #[test]
    fn test_insert_mutation() {
        let mut buffer = vec![0x4, 0x5, 0x00, 0x00, 0x00];
        let mut m = Mutator::new();

        m.make_diff(0, 0, vec![0x1, 0x2, 0x3]);
        m.make_diff(2, 4, vec![0x6, 0x7, 0x8]);
        m.make_diff(4, 5, vec![0x9, 0x0A]);

        m.mutate(&mut buffer);

        assert_eq!(
            buffer,
            vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x0A]
        );
    }

    #[test]
    fn test_complex_mutation() {
        let mut buffer = vec![0x02, 0x00, 0x00, 0x00];
        let mut m = Mutator::new();

        m.make_diff(0, 0, vec![0x1]);
        m.make_diff(1, 3, vec![0x03, 0x04]);
        m.make_diff(3, 4, vec![]);
        m.make_diff(4, 5, vec![0x05, 0x06, 0x07, 0x08, 0x09, 0x0a]);

        m.mutate(&mut buffer);

        assert_eq!(
            buffer,
            vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x0A]
        );
    }
}
