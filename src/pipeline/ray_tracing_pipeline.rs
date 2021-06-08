use std::ffi::CString;
use std::sync::Arc;

use super::PipelineLayout;
use crate::{Buffer, Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

pub(crate) struct RayTracingPipelineRef {
    handle: vk::Pipeline,
    layout: PipelineLayout,
    pub(crate) ray_gen_shader: ShaderStage,
    pub(crate) miss_shaders: Vec<ShaderStage>,
    pub(crate) hit_groups: Vec<Box<dyn crate::HitGroup + 'static>>,
    pub(crate) device: Device,
    pub(crate) shader_group_handles: Vec<u8>,
}

#[derive(Clone)]
pub struct RayTracingPipeline {
    pub(crate) inner: Arc<RayTracingPipelineRef>,
}

impl RayTracingPipeline {
    pub fn new(
        name: Option<&str>,
        device: &Device,
        layout: PipelineLayout,
        ray_gen_shader: &ShaderStage,
        miss_shaders: &[ShaderStage],
        hit_groups: &[&dyn crate::HitGroup],
        recursion_depth: u32,
    ) -> Self {
        // validation
        assert!(ray_gen_shader.stage == vk::ShaderStageFlags::RAYGEN_KHR);
        for miss_shader in miss_shaders {
            assert!(miss_shader.stage == vk::ShaderStageFlags::MISS_KHR);
        }

        let mut stage_create_infos = Vec::new();
        stage_create_infos.push(ray_gen_shader.shader_stage_create_info());
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

            let rt_p = &device.inner.pdevice.ray_tracing_pipeline_properties;
            let shader_group_handles = device
                .ray_tracing_pipeline_loader()
                .get_ray_tracing_shader_group_handles(
                    handle,
                    0,
                    group_create_infos.len() as u32,
                    rt_p.shader_group_handle_size as usize * group_create_infos.len(),
                )
                .unwrap();

            Self {
                inner: Arc::new(RayTracingPipelineRef {
                    handle,
                    layout,
                    device: device.clone(),
                    ray_gen_shader: ray_gen_shader.to_owned(),
                    miss_shaders: miss_shaders.to_owned(),
                    hit_groups: hit_groups
                        .iter()
                        .map(|g| dyn_clone::clone_box(*g))
                        .collect(),
                    shader_group_handles,
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

impl Device {
    pub fn create_ray_tracing_pipeline(
        &self,
        name: Option<&str>,
        layout: PipelineLayout,
        ray_gen_shader: &ShaderStage,
        miss_shaders: &[ShaderStage],
        hit_groups: &[&dyn crate::HitGroup],
        recursion_depth: u32,
    ) -> RayTracingPipeline {
        RayTracingPipeline::new(
            name,
            self,
            layout,
            ray_gen_shader,
            miss_shaders,
            hit_groups,
            recursion_depth,
        )
    }
}
