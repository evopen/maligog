mod graphics_pipeline;
mod pipeline_layout;

pub use graphics_pipeline::GraphicsPipeline;
pub use pipeline_layout::PipelineLayout;

use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use crate::Device;
use crate::RenderPass;
use crate::ShaderStage;

pub trait Pipeline {
    fn layout(&self) -> PipelineLayout;
}

impl Pipeline for GraphicsPipeline {
    fn layout(&self) -> PipelineLayout {
        self.inner.layout.clone()
    }
}
