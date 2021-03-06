pub use libra::vm::file_format_common::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct TableContext<'a, 'b> {
    pub cursor: &'a mut Cursor<&'b [u8]>,
    old_pos: u64,
    pub len: u32,
}

impl<'a, 'b> TableContext<'a, 'b> {
    pub fn new(cursor: &'a mut Cursor<&'b [u8]>, offset: u32, len: u32) -> TableContext<'a, 'b> {
        let old_pos = cursor.position();
        cursor.set_position(offset as u64);

        TableContext {
            cursor,
            old_pos,
            len,
        }
    }

    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }
}

impl<'a, 'b> Drop for TableContext<'a, 'b> {
    fn drop(&mut self) {
        self.cursor.set_position(self.old_pos);
    }
}
