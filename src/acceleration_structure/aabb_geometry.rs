use std::convert::TryInto;
use std::ffi::c_void;

use ash::vk;

unsafe impl Sync for AABBGeometry {}
unsafe impl Send for AABBGeometry {}

#[derive(Clone)]
pub struct AABBGeometry {
    pub(crate) acceleration_structure_geometry: vk::AccelerationStructureGeometryKHR,
    pub(crate) build_range_info: vk::AccelerationStructureBuildRangeInfoKHR,
}

impl AABBGeometry {
    pub fn new(positions_buffer_view: crate::BufferView, count: u32) -> Self {
        let mut aabbs_data = vk::AccelerationStructureGeometryAabbsDataKHR::builder()
            .data(vk::DeviceOrHostAddressConstKHR {
                device_address: positions_buffer_view.buffer.device_address(),
            })
            .stride(16)
            .build();
        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .flags(
                vk::GeometryFlagsKHR::OPAQUE
                    | vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION,
            )
            .geometry(vk::AccelerationStructureGeometryDataKHR { aabbs: aabbs_data })
            .build();
        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(count)
            .primitive_offset(0) // offset to index buffer in bytes
            .build();
        Self {
            acceleration_structure_geometry: geometry,
            build_range_info,
        }
    }
}
