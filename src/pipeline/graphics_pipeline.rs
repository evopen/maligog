use std::ffi::CString;
use std::sync::Arc;

use super::PipelineLayout;
use crate::{Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

pub struct GraphicsPipelineRef {
    pub(crate) handle: vk::Pipeline,
    pub(crate) layout: PipelineLayout,
    stages: Vec<ShaderStage>,
    render_pass: RenderPass,
    device: Device,
}

pub struct GraphicsPipeline {
    pub(crate) inner: Arc<GraphicsPipelineRef>,
}

impl GraphicsPipeline {
    pub fn new(
        name: Option<&str>,
        device: &Device,
        layout: PipelineLayout,
        stages: Vec<ShaderStage>,
        render_pass: RenderPass,
        vertex_input_state: &vk::PipelineVertexInputStateCreateInfo,
        input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo,
        rasterization_state: &vk::PipelineRasterizationStateCreateInfo,
        multisample_state: &vk::PipelineMultisampleStateCreateInfo,
        depth_stencil_state: &vk::PipelineDepthStencilStateCreateInfo,
        color_blend_state: &vk::PipelineColorBlendStateCreateInfo,
        viewport_state: &vk::PipelineViewportStateCreateInfo,
        dynamic_state: &vk::PipelineDynamicStateCreateInfo,
    ) -> Self {
        let stage_create_infos = stages
            .iter()
            .map(|s| s.shader_stage_create_info())
            .collect::<Vec<_>>();
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .layout(layout.inner.handle)
            .stages(&stage_create_infos)
            .vertex_input_state(vertex_input_state)
            .input_assembly_state(input_assembly_state)
            .rasterization_state(rasterization_state)
            .multisample_state(multisample_state)
            .depth_stencil_state(depth_stencil_state)
            .color_blend_state(color_blend_state)
            .viewport_state(viewport_state)
            .dynamic_state(dynamic_state)
            .render_pass(render_pass.inner.handle)
            .build();
        unsafe {
            let handle = device
                .inner
                .handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
                .unwrap()
                .first()
                .unwrap()
                .to_owned();
            if let Some(name) = name {
                device
                    .inner
                    .pdevice
                    .instance
                    .inner
                    .debug_utils_loader
                    .as_ref()
                    .unwrap()
                    .debug_utils_set_object_name(
                        device.inner.handle.handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::PIPELINE)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            Self {
                inner: Arc::new(GraphicsPipelineRef {
                    handle,
                    device: device.clone(),
                    layout,
                    stages,
                    render_pass,
                }),
            }
        }
    }
}

impl Drop for GraphicsPipelineRef {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.handle.destroy_pipeline(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_graphics_pipeline(
        &self,
        name: Option<&str>,
        layout: PipelineLayout,
        stages: Vec<ShaderStage>,
        render_pass: RenderPass,
        vertex_input_state: &vk::PipelineVertexInputStateCreateInfo,
        input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo,
        rasterization_state: &vk::PipelineRasterizationStateCreateInfo,
        multisample_state: &vk::PipelineMultisampleStateCreateInfo,
        depth_stencil_state: &vk::PipelineDepthStencilStateCreateInfo,
        color_blend_state: &vk::PipelineColorBlendStateCreateInfo,
        viewport_state: &vk::PipelineViewportStateCreateInfo,
        dynamic_state: &vk::PipelineDynamicStateCreateInfo,
    ) -> GraphicsPipeline {
        GraphicsPipeline::new(
            name,
            &self,
            layout,
            stages,
            render_pass,
            vertex_input_state,
            input_assembly_state,
            rasterization_state,
            multisample_state,
            depth_stencil_state,
            color_blend_state,
            viewport_state,
            dynamic_state,
        )
    }
}
