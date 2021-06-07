use std::ffi::CString;
use std::sync::Arc;

use super::PipelineLayout;
use crate::{Buffer, Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

pub(crate) struct RayTracingPipelineRef {
    handle: vk::Pipeline,
    layout: PipelineLayout,
    ray_gen_shaders: Vec<ShaderStage>,
    miss_shaders: Vec<ShaderStage>,
    hit_groups: Vec<Box<dyn crate::HitGroup + 'static>>,
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
        ray_gen_shaders: &[ShaderStage],
        miss_shaders: &[ShaderStage],
        hit_groups: &[&dyn crate::HitGroup],
        recursion_depth: u32,
    ) -> Self {
        // validation
        for ray_gen_shader in ray_gen_shaders {
            assert!(ray_gen_shader.stage == vk::ShaderStageFlags::RAYGEN_KHR);
        }
        for miss_shader in miss_shaders {
            assert!(miss_shader.stage == vk::ShaderStageFlags::MISS_KHR);
        }

        let mut stage_create_infos = Vec::new();
        stage_create_infos.extend(
            ray_gen_shaders
                .iter()
                .map(|s| s.shader_stage_create_info())
                .collect::<Vec<_>>(),
        );
        stage_create_infos.extend(
            miss_shaders
                .iter()
                .map(|s| s.shader_stage_create_info())
                .collect::<Vec<_>>(),
        );
        stage_create_infos.extend(
            hit_groups
                .iter()
                .map(|g| g.shader_stage_create_infos())
                .flatten()
                .collect::<Vec<_>>(),
        );

        let mut group_create_infos = Vec::new();
        let mut i = 0;
        for ray_gen_shader in ray_gen_shaders {
            group_create_infos.push(
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(i)
                    .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                    .any_hit_shader(vk::SHADER_UNUSED_KHR)
                    .intersection_shader(vk::SHADER_UNUSED_KHR)
                    .build(),
            );
            i += 1;
        }
        for miss_shader in miss_shaders {
            group_create_infos.push(
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(i)
                    .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                    .any_hit_shader(vk::SHADER_UNUSED_KHR)
                    .intersection_shader(vk::SHADER_UNUSED_KHR)
                    .build(),
            );
            i += 1;
        }
        for hit_group in hit_groups {
            group_create_infos.push(
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(hit_group.shader_group_type())
                    .general_shader(vk::SHADER_UNUSED_KHR)
                    .closest_hit_shader(i)
                    .any_hit_shader(match hit_group.has_any_hit_shader() {
                        true => {
                            i += 1;
                            i
                        }
                        false => vk::SHADER_UNUSED_KHR,
                    })
                    .intersection_shader(match hit_group.has_any_hit_shader() {
                        true => {
                            i += 1;
                            i
                        }
                        false => vk::SHADER_UNUSED_KHR,
                    })
                    .build(),
            );
            i += 1;
        }
        drop(i);

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
            let a = dyn_clone::clone_box(hit_groups[0]);

            Self {
                inner: Arc::new(RayTracingPipelineRef {
                    handle,
                    layout,
                    device: device.clone(),
                    ray_gen_shaders: ray_gen_shaders.to_owned(),
                    miss_shaders: miss_shaders.to_owned(),
                    hit_groups: hit_groups
                        .iter()
                        .map(|g| dyn_clone::clone_box(*g))
                        .collect(),
                }),
            }
        }
    }
}

impl Drop for RayTracingPipelineRef {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_pipeline(self.handle, None);
        }
    }
}
