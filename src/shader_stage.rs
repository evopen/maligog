use std::ffi::CString;

use ash::vk;

use crate::ShaderModule;

pub struct ShaderStage {
    module: ShaderModule,
    stage: vk::ShaderStageFlags,
    entry_point: String,
    entry_point_cstr: CString,
}

impl ShaderStage {
    pub fn new(module: ShaderModule, stage: vk::ShaderStageFlags, entry_point: &str) -> Self {
        let entry_point_cstr = CString::new(entry_point).unwrap();
        Self {
            module,
            stage,
            entry_point: entry_point.to_string(),
            entry_point_cstr,
        }
    }

    pub(crate) fn shader_stage_create_info(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .module(self.module.inner.handle)
            .stage(self.stage)
            .name(&self.entry_point_cstr)
            .build()
    }
}
