use std::ffi::CString;
use std::sync::Arc;

use super::PipelineLayout;
use crate::{Buffer, Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

pub(crate) struct RayTracingPipelineRef {
    handle: vk::Pipeline,
    layout: PipelineLayout,
    stages: Vec<ShaderStage>,
    sbt_buffer: Buffer,
    sbt_stride: u32,
    device: Device,
}

pub struct RayTracingPipeline {
    inner: Arc<RayTracingPipelineRef>,
}

impl RayTracingPipeline {
    pub fn new(
        name: Option<&str>,
        device: &Device,
        layout: PipelineLayout,
        stages: Vec<ShaderStage>,
        recursion_depth: u32,
    ) -> Self {
        let stage_create_infos = stages
            .iter()
            .map(|s| s.shader_stage_create_info())
            .collect::<Vec<_>>();
        let group_create_infos = stage_create_infos
            .iter()
            .enumerate()
            .map(|(i, info)| {
                match info.stage {
                    vk::ShaderStageFlags::RAYGEN_KHR => {
                        vk::RayTracingShaderGroupCreateInfoKHR::builder()
                            .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                            .general_shader(i as u32)
                            .any_hit_shader(vk::SHADER_UNUSED_KHR)
                            .intersection_shader(vk::SHADER_UNUSED_KHR)
                            .build()
                    }
                    vk::ShaderStageFlags::CLOSEST_HIT_KHR => {
                        vk::RayTracingShaderGroupCreateInfoKHR::builder()
                            .ty(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                            .closest_hit_shader(i as u32)
                            .general_shader(vk::SHADER_UNUSED_KHR)
                            .any_hit_shader(vk::SHADER_UNUSED_KHR)
                            .intersection_shader(vk::SHADER_UNUSED_KHR)
                            .build()
                    }
                    vk::ShaderStageFlags::MISS_KHR => {
                        vk::RayTracingShaderGroupCreateInfoKHR::builder()
                            .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                            .general_shader(i as u32)
                            .any_hit_shader(vk::SHADER_UNUSED_KHR)
                            .intersection_shader(vk::SHADER_UNUSED_KHR)
                            .build()
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            })
            .collect::<Vec<_>>();
        unsafe {
            let handle = device
                .ray_tracing_pipeline_loader()
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    &[vk::RayTracingPipelineCreateInfoKHR::builder()
                        .layout(layout.inner.handle)
                        .stages(stage_create_infos.as_slice())
                        .groups(group_create_infos.as_slice())
                        .max_pipeline_ray_recursion_depth(recursion_depth)
                        .build()],
                    None,
                )
                .unwrap()
                .first()
                .unwrap()
                .to_owned();

            if let Some(name) = name {
                device.debug_set_object_name(name, handle.as_raw(), vk::ObjectType::PIPELINE);
            }

            let rt_p = &device.inner.pdevice.ray_tracing_pipeline_properties;

            let shader_handle_storage = device
                .ray_tracing_pipeline_loader()
                .get_ray_tracing_shader_group_handles(
                    handle,
                    0,
                    group_create_infos.len() as u32,
                    rt_p.shader_group_handle_size as usize * group_create_infos.len(),
                )
                .unwrap();
            assert!(rt_p.shader_group_base_alignment % rt_p.shader_group_handle_alignment == 0);
            let sbt_stride = rt_p.shader_group_base_alignment
                * ((rt_p.shader_group_handle_size + rt_p.shader_group_base_alignment - 1)
                    / rt_p.shader_group_base_alignment);
            assert!(sbt_stride <= rt_p.max_shader_group_stride);
            assert!(sbt_stride == 64);

            let sbt_size = sbt_stride * group_create_infos.len() as u32;

            let mut temp: Vec<u8> = vec![0; sbt_size as usize];
            for group_index in 0..group_create_infos.len() {
                std::ptr::copy_nonoverlapping(
                    shader_handle_storage
                        .as_ptr()
                        .add(group_index * rt_p.shader_group_handle_size as usize),
                    temp.as_mut_ptr().add(group_index * sbt_stride as usize),
                    rt_p.shader_group_handle_size as usize,
                );
            }
            let sbt_buffer = device.create_buffer_init(
                Some("sbt buffer"),
                temp,
                vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                gpu_allocator::MemoryLocation::GpuOnly,
            );

            Self {
                inner: Arc::new(RayTracingPipelineRef {
                    handle,
                    layout,
                    stages,
                    sbt_buffer,
                    sbt_stride,
                    device: device.clone(),
                }),
            }
        }
    }

    pub fn sbt_buffer(&self) -> &Buffer {
        &self.inner.sbt_buffer
    }

    pub fn sbt_stride(&self) -> u32 {
        self.inner.sbt_stride
    }
}

impl Drop for RayTracingPipelineRef {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_pipeline(self.handle, None);
        }
    }
}
