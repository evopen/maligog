use std::ffi::CString;
use std::sync::Arc;

use crate::{Buffer, Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

struct ShaderBindingTable {
    sbt_buffer: Buffer,
}

impl ShaderBindingTable {
    pub fn new(
        device: &Device,
        ray_gen_shaders: &[ShaderStage],
        hit_groups: &[&dyn super::hit_group::HitGroup],
        miss_shaders: &[ShaderStage],
    ) -> Self {
        for ray_gen_shader in ray_gen_shaders {
            assert!(ray_gen_shader.stage == vk::ShaderStageFlags::RAYGEN_KHR);
        }
        for miss_shader in miss_shaders {
            assert!(miss_shader.stage == vk::ShaderStageFlags::MISS_KHR);
        }

        Self {}
    }
}
