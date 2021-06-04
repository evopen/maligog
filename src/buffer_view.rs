use ash::vk;

use crate::Buffer;

#[derive(Clone)]
pub struct BufferView {
    pub buffer: Buffer,
    pub offset: u64,
}

#[derive(Clone)]
pub struct IndexBufferView {
    pub buffer_view: BufferView,
    pub index_type: vk::IndexType,
    pub count: u32,
}

#[derive(Clone)]
pub struct VertexBufferView {
    pub buffer_view: BufferView,
    pub format: vk::Format,
    pub stride: u64,
    pub count: u32,
}
