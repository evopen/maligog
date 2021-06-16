use std::ffi::CString;
use std::sync::Arc;

use crate::{Buffer, Device, RenderPass, ShaderStage};
use ash::vk::{self, Handle};

pub trait ShaderBindingTables {
    fn ray_gen_table(&self) -> ShaderBindingTable;
    fn miss_table(&self) -> ShaderBindingTable;
    fn hit_table(&self) -> ShaderBindingTable;
    fn callable_table(&self) -> ShaderBindingTable;
}

pub struct ShaderBindingTable {
    parent: Box<dyn ShaderBindingTables>,
    pub(crate) region: vk::StridedDeviceAddressRegionKHR,
}

pub(crate) struct PipelineShaderBindingTablesRef {
    rt_pipeline: crate::RayTracingPipeline,
    sbt_buffer: Buffer,
    ray_tracing_pipeline: crate::RayTracingPipeline,
    raygen_table: vk::StridedDeviceAddressRegionKHR,
    miss_table: vk::StridedDeviceAddressRegionKHR,
    hit_table: vk::StridedDeviceAddressRegionKHR,
    callable_table: vk::StridedDeviceAddressRegionKHR,
}

#[derive(Clone)]
pub struct PipelineShaderBindingTables {
    inner: Arc<PipelineShaderBindingTablesRef>,
}

impl PipelineShaderBindingTables {
    pub fn new(device: &Device, pipeline: &crate::RayTracingPipeline, hit_groups: &[u32]) -> Self {
        let rt_p = &device.inner.pdevice.ray_tracing_pipeline_properties;
        let sbt_base_alignment = rt_p.shader_group_base_alignment as usize;
        let handle_size = rt_p.shader_group_handle_size as usize;
        let sbt_buffer_size = sbt_base_alignment * 2 + hit_groups.len() * handle_size;
        let mut sbt_buffer_data = vec![0; sbt_buffer_size];
        //raygen
        sbt_buffer_data[0..handle_size]
            .copy_from_slice(&pipeline.inner.shader_group_handles[0..handle_size]);
        //miss
        let miss_group_count = pipeline.inner.miss_shaders.len();
        sbt_buffer_data[sbt_base_alignment..sbt_base_alignment + handle_size * miss_group_count]
            .copy_from_slice(
                &pipeline.inner.shader_group_handles
                    [handle_size..handle_size + handle_size * miss_group_count],
            );
        // hit group
        let hit_group_count = hit_groups.len();
        for (i, hit_group) in hit_groups.iter().enumerate() {
            sbt_buffer_data[(2 * sbt_base_alignment + i * handle_size)
                ..(2 * sbt_base_alignment + (i + 1) * handle_size)]
                .copy_from_slice(
                    &pipeline.inner.shader_group_handles[(2 + *hit_group as usize) * handle_size
                        ..(2 + *hit_group as usize + 1) * handle_size],
                )
        }

        let sbt_buffer = device.create_buffer_init(
            Some("sbt buffer"),
            sbt_buffer_data,
            vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            crate::MemoryLocation::GpuOnly,
        );

        let ray_gen_table = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_buffer.device_address())
            .stride(handle_size as u64)
            .size(handle_size as u64)
            .build();
        let miss_table = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_buffer.device_address() + sbt_base_alignment as u64)
            .stride(handle_size as u64)
            .size((handle_size * miss_group_count) as u64)
            .build();
        let hit_table = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_buffer.device_address() + 2 * sbt_base_alignment as u64)
            .stride(handle_size as u64)
            .size((handle_size * hit_group_count) as u64)
            .build();
        let callable_table = vk::StridedDeviceAddressRegionKHR::default();
        Self {
            inner: Arc::new(PipelineShaderBindingTablesRef {
                rt_pipeline: pipeline.clone(),
                sbt_buffer,
                ray_tracing_pipeline: pipeline.to_owned(),
                raygen_table: ray_gen_table,
                miss_table,
                hit_table,
                callable_table,
            }),
        }
    }
}

impl ShaderBindingTables for PipelineShaderBindingTables {
    fn ray_gen_table(&self) -> ShaderBindingTable {
        ShaderBindingTable {
            parent: Box::new(self.clone()),
            region: self.inner.raygen_table,
        }
    }

    fn miss_table(&self) -> ShaderBindingTable {
        ShaderBindingTable {
            parent: Box::new(self.clone()),
            region: self.inner.miss_table,
        }
    }

    fn hit_table(&self) -> ShaderBindingTable {
        ShaderBindingTable {
            parent: Box::new(self.clone()),
            region: self.inner.hit_table,
        }
    }

    fn callable_table(&self) -> ShaderBindingTable {
        ShaderBindingTable {
            parent: Box::new(self.clone()),
            region: self.inner.callable_table,
        }
    }
}

impl crate::RayTracingPipeline {
    pub fn create_shader_binding_tables(&self, hit_groups: &[u32]) -> PipelineShaderBindingTables {
        PipelineShaderBindingTables::new(&self.inner.device, self, hit_groups)
    }
}
