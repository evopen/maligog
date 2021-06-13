use std::ffi::c_void;

use ash::vk;

use crate::Device;

unsafe impl Sync for InstanceGeometry {}
unsafe impl Send for InstanceGeometry {}

#[derive(Clone)]
pub struct InstanceGeometry {
    pub(crate) acceleration_structure_geometry: vk::AccelerationStructureGeometryKHR,
    pub(crate) build_range_info: vk::AccelerationStructureBuildRangeInfoKHR,
    blas_instances: Vec<super::BLASInstance>,
    array_of_pointer_buffer: crate::Buffer,
}

impl InstanceGeometry {
    pub fn new(device: &Device, blas_instances: &[super::BLASInstance]) -> Self {
        let instance_buffer_addresses: Vec<u64> = blas_instances
            .iter()
            .map(|i| i.instance_buffer.as_ref().unwrap().device_address())
            .collect();
        let array_of_pointer_buffer = device.create_buffer_init(
            Some("array of pointers"),
            bytemuck::cast_slice(&instance_buffer_addresses),
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            gpu_allocator::MemoryLocation::GpuOnly,
        );

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .flags(
                vk::GeometryFlagsKHR::OPAQUE
                    | vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION,
            )
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                    .array_of_pointers(true)
                    .data(vk::DeviceOrHostAddressConstKHR {
                        device_address: array_of_pointer_buffer.device_address(),
                    })
                    .build(),
            })
            .build();

        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(blas_instances.len() as u32)
            .primitive_offset(0) // offset to index buffer in bytes
            .build();
        Self {
            acceleration_structure_geometry: geometry,
            build_range_info,
            blas_instances: blas_instances.to_owned(),
            array_of_pointer_buffer,
        }
    }

    pub fn instance_count(&self) -> u32 {
        self.blas_instances.len() as u32
    }

    pub fn blas_instances(&self) -> &[crate::BLASInstance] {
        &self.blas_instances
    }
}
