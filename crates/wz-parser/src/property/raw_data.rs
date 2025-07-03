use crate::WzReader;
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct WzRawData {
    pub reader: Arc<WzReader>,
    offset: usize,
    length: usize,
}

impl WzRawData {
    pub fn new(reader: &Arc<WzReader>, offset: usize, length: usize) -> Self {
        Self {
            reader: Arc::clone(reader),
            offset,
            length,
        }
    }
    fn get_buffer_range(&self) -> Range<usize> {
        self.offset..self.offset + self.length
    }
    pub fn get_buffer(&self) -> &[u8] {
        let range = self.get_buffer_range();
        self.reader.get_slice(range)
    }
}
