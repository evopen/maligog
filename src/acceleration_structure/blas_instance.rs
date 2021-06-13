use std::convert::TryInto;

use ash::vk;

use crate::Device;

#[derive(Clone)]
pub struct BLASInstance {
    pub(crate) vk_instance: vk::AccelerationStructureInstanceKHR,
    blas: super::BottomAccelerationStructure,
    transform: glam::Mat4,
    device: Device,
    pub(crate) instance_buffer: Option<crate::Buffer>,
}

impl BLASInstance {
    pub fn new(
        device: &Device,
        blas: &super::BottomAccelerationStructure,
        transform: &glam::Mat4,
        sbt_record_offset: u32,
    ) -> Self {
        let vk_instance = vk::AccelerationStructureInstanceKHR {
            transform: vk::TransformMatrixKHR {
                matrix: transform.transpose().as_ref()[..12].try_into().unwrap(),
            },
            instance_custom_index_and_mask: 0 | (0xFF << 24),
            instance_shader_binding_table_record_offset_and_flags: sbt_record_offset
                | (vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() << 24),
            acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                device_handle: blas.inner.device_address,
            },
        };

        Self {
            device: device.clone(),
            vk_instance,
            blas: blas.clone(),
            transform: transform.clone(),
            instance_buffer: None,
        }
    }

    pub fn set_transform(&mut self, transform: &glam::Mat4) {
        self.transform.clone_from(transform)
    }

    pub fn transform(&self) -> &glam::Mat4 {
        &self.transform
    }

    pub fn build(&mut self) {
        let data = unsafe {
            std::slice::from_raw_parts(
                std::mem::transmute(&self.vk_instance),
                std::mem::size_of::<vk::AccelerationStructureInstanceKHR>(),
            )
        };
        let instance_buffer = self.device.create_buffer_init(
            Some("instance buffer"),
            data,
            crate::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            gpu_allocator::MemoryLocation::GpuOnly,
        );
        self.instance_buffer = Some(instance_buffer);
    }

    pub fn blas(&self) -> &crate::BottomAccelerationStructure {
        &self.blas
    }
}
