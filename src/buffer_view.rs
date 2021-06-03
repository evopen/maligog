use crate::Buffer;

#[derive(Clone)]
pub struct BufferView {
    pub buffer: Buffer,
    pub offset: u64,
}
